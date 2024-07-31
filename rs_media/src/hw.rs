use std::collections::HashMap;
extern crate ffmpeg_next as ffmpeg;
use ffmpeg::{
    ffi::{
        av_buffer_ref, av_buffer_unref, av_hwdevice_ctx_create, av_hwdevice_iterate_types,
        av_hwframe_transfer_data, avcodec_get_hw_config, AVBufferRef, AVCodec, AVCodecContext,
        AVHWDeviceType, AVPixelFormat, AV_CODEC_HW_CONFIG_METHOD_HW_DEVICE_CTX,
    },
    format::{input, Pixel},
    media::Type,
    software::scaling::{context::Context, flag::Flags},
    util::frame::video::Video,
};

unsafe fn get_available_hwdevice_types_unsafe() -> Vec<AVHWDeviceType> {
    let mut types: Vec<AVHWDeviceType> = vec![];
    let mut av_hwdevice_type = AVHWDeviceType::AV_HWDEVICE_TYPE_NONE;
    loop {
        av_hwdevice_type = av_hwdevice_iterate_types(av_hwdevice_type);
        if av_hwdevice_type == AVHWDeviceType::AV_HWDEVICE_TYPE_NONE {
            break;
        } else {
            types.push(av_hwdevice_type);
        }
    }
    types
}

pub fn get_available_hwdevice_types() -> Vec<AVHWDeviceType> {
    unsafe { get_available_hwdevice_types_unsafe() }
}

pub(crate) unsafe fn find_hw_pix_fmt(
    codec: *const AVCodec,
    device_type: AVHWDeviceType,
) -> Vec<AVPixelFormat> {
    let mut index = 0;
    let mut support_formats = vec![];
    loop {
        let codec_hw_config = avcodec_get_hw_config(codec, index);
        if codec_hw_config.is_null() {
            return support_formats;
        }
        let is_hw_methods =
            (*codec_hw_config).methods & AV_CODEC_HW_CONFIG_METHOD_HW_DEVICE_CTX as i32 != 0;
        if is_hw_methods && (*codec_hw_config).device_type == device_type {
            support_formats.push((*codec_hw_config).pix_fmt);
        }
        index += 1;
    }
}

#[repr(C)]
pub(crate) struct MyUserData {
    pub(crate) hw_type: AVHWDeviceType,
    pub(crate) hw_pix_fmt: AVPixelFormat,
    pub(crate) fallback_pix_fmt: AVPixelFormat,
}

pub(crate) unsafe extern "C" fn get_hw_format(
    ctx: *mut AVCodecContext,
    pix_fmts: *const AVPixelFormat,
) -> AVPixelFormat {
    let user_data: *mut MyUserData = std::mem::transmute((*ctx).opaque);
    if user_data.is_null() {
        panic!()
    }

    let mut p: *const AVPixelFormat = pix_fmts;
    let mut count = 1;

    while *p != AVPixelFormat::AV_PIX_FMT_NONE {
        if *p == (*user_data).hw_pix_fmt {
            log::trace!("get_hw_format: {:?}", *p);
            return *p;
        }
        p = p.offset(count);
        count += 1;
    }

    log::warn!("Failed to get HW surface format.");
    return (*user_data).fallback_pix_fmt;
}

pub(crate) unsafe fn hw_decoder_init(
    ctx: *mut AVCodecContext,
    device_type: AVHWDeviceType,
) -> crate::error::Result<impl FnMut() -> ()> {
    let mut hw_device_ctx: *mut AVBufferRef = std::ptr::null_mut();
    let state = av_hwdevice_ctx_create(
        &mut hw_device_ctx,
        device_type,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        0,
    );
    if state < 0 {
        return Err(crate::error::Error::FFMpeg(ffmpeg_next::Error::from(state)));
    }
    (*ctx).hw_device_ctx = av_buffer_ref(hw_device_ctx);
    Ok(move || {
        av_buffer_unref(&mut hw_device_ctx);
    })
}

unsafe fn hw_test_unsafe(filename: &str) {
    let _ = std::fs::remove_dir_all("./hw_examples");
    let _ = std::fs::create_dir("./hw_examples");
    ffmpeg::init().unwrap();

    let mut hw_pixel_formats = HashMap::<AVHWDeviceType, AVPixelFormat>::new();
    let expect_hw_type = AVHWDeviceType::AV_HWDEVICE_TYPE_CUDA;

    let mut av_format_input = input(&filename.to_owned()).unwrap();
    let input_stream = av_format_input.streams().best(Type::Video).unwrap();
    let video_stream_index = input_stream.index();
    let context_decoder =
        ffmpeg::codec::context::Context::from_parameters(input_stream.parameters()).unwrap();

    let decoder = context_decoder.decoder();
    let mut video_decoder = decoder.video().unwrap();
    for device_type in get_available_hwdevice_types() {
        match find_hw_pix_fmt(video_decoder.codec().unwrap().as_ptr(), device_type).first() {
            Some(pix_fmt) => {
                log::trace!(
                    "Decoder {:?} does support device type {:?}, pix_fmt: {:?}",
                    video_decoder.codec().unwrap().name(),
                    device_type,
                    pix_fmt
                );
                hw_pixel_formats.insert(device_type, *pix_fmt);
            }
            None => log::trace!(
                "Decoder {:?} does not support device type {:?}",
                video_decoder.codec().unwrap().name(),
                device_type
            ),
        }
    }
    log::trace!("hw_pixel_formats: {:#?}", hw_pixel_formats);
    assert!(hw_pixel_formats.contains_key(&expect_hw_type));
    let user_data = MyUserData {
        hw_type: expect_hw_type,
        hw_pix_fmt: *hw_pixel_formats.get_key_value(&expect_hw_type).unwrap().1,
        fallback_pix_fmt: video_decoder.format().into(),
    };
    let user_data = Box::new(user_data);
    let raw = Box::into_raw(user_data);
    (*video_decoder.as_mut_ptr()).opaque = std::mem::transmute(raw);
    (*video_decoder.as_mut_ptr()).get_format = Some(get_hw_format);

    let mut release_hw_device_ctx =
        hw_decoder_init(video_decoder.as_mut_ptr(), expect_hw_type).unwrap();

    let mut scaler = Context::get(
        Pixel::NV12,
        video_decoder.width(),
        video_decoder.height(),
        Pixel::RGB24,
        video_decoder.width(),
        video_decoder.height(),
        Flags::BILINEAR,
    )
    .unwrap();
    let mut frame_index = 0;
    let mut packet_index = 0;
    let mut rgb_frame = Video::empty();

    let mut receive_and_process_decoded_frames =
        |packet_index: usize, decoder: &mut ffmpeg::decoder::Video| {
            let mut frame = Video::empty();
            let mut sw_frame = Video::empty();
            while decoder.receive_frame(&mut frame).is_ok() {
                // if frame.kind() == picture::Type::I {
                let state = av_hwframe_transfer_data(sw_frame.as_mut_ptr(), frame.as_mut_ptr(), 0);
                if state < 0 {
                    log::warn!("Error transferring the data to system memory");
                }
                assert_eq!(sw_frame.format(), Pixel::NV12);
                scaler.run(&sw_frame, &mut rgb_frame).unwrap();
                save_file(&rgb_frame, packet_index, frame_index);
                frame_index += 1;
                // }
            }
        };

    for (stream, packet) in av_format_input.packets() {
        if stream.index() == video_stream_index {
            // if packet.is_key() {
            video_decoder.send_packet(&packet).unwrap();
            receive_and_process_decoded_frames(packet_index, &mut video_decoder);
            // }
            packet_index += 1;
        }
    }
    // av_buffer_unref(&mut (*video_decoder.as_mut_ptr()).hw_device_ctx);
    release_hw_device_ctx();
    let _ = Box::from(raw);
}

fn save_file(frame: &Video, packet_index: usize, frame_index: usize) {
    let image =
        image::RgbImage::from_raw(frame.width(), frame.height(), frame.data(0).to_vec()).unwrap();
    let filename = format!("./hw_examples/frame_{}_{}.jpg", packet_index, frame_index);
    let dyimage = image::DynamicImage::ImageRgb8(image);
    let _ = dyimage.save(filename);
}

pub fn hw_test(filename: &str) {
    unsafe { hw_test_unsafe(filename) }
}
