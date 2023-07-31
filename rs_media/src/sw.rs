extern crate ffmpeg_next as ffmpeg;
use ffmpeg::{
    format::{input, Pixel},
    media::Type,
    software::scaling::{context::Context, flag::Flags},
    util::frame::video::Video,
};

unsafe fn sw_test_unsafe(filename: &str) {
    let _ = std::fs::remove_dir_all("./sw_examples");
    let _ = std::fs::create_dir("./sw_examples");

    ffmpeg::init().unwrap();
    let mut av_format_input = input(&filename.to_owned()).unwrap();
    let input_stream = av_format_input.streams().best(Type::Video).unwrap();
    let video_stream_index = input_stream.index();

    let context_decoder =
        ffmpeg::codec::context::Context::from_parameters(input_stream.parameters()).unwrap();
    let mut decoder = context_decoder.decoder().video().unwrap();

    let mut scaler = Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGB24,
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    )
    .unwrap();
    let mut frame_index = 0;
    let mut packet_index = 0;
    let mut rgb_frame = Video::empty();

    let mut receive_and_process_decoded_frames =
        |packet_index: usize, decoder: &mut ffmpeg::decoder::Video| {
            let mut decoded = Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                // if decoded.kind() == picture::Type::I {
                scaler.run(&decoded, &mut rgb_frame).unwrap();
                save_file(&rgb_frame, packet_index, frame_index);
                frame_index += 1;
                // }
            }
        };

    for (stream, packet) in av_format_input.packets() {
        if stream.index() == video_stream_index {
            // if packet.is_key() {
            decoder.send_packet(&packet).unwrap();
            receive_and_process_decoded_frames(packet_index, &mut decoder);
            // }
            packet_index += 1;
        }
    }
    decoder.send_eof().unwrap();
    receive_and_process_decoded_frames(packet_index, &mut decoder);
}

fn save_file(frame: &Video, packet_index: usize, frame_index: usize) {
    let image =
        image::RgbImage::from_raw(frame.width(), frame.height(), frame.data(0).to_vec()).unwrap();
    let filename = format!("./sw_examples/frame_{}_{}.jpg", packet_index, frame_index);
    let dyimage = image::DynamicImage::ImageRgb8(image);
    let _ = dyimage.save(filename);
}

pub fn sw_test(filename: &str) {
    unsafe { sw_test_unsafe(filename) }
}
