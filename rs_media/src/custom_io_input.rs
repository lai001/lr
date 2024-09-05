use crate::media_stream::{EWhenceType, StreamIO};
use ffmpeg_next::{
    ffi::{
        av_malloc, avformat_alloc_context, avformat_close_input, avformat_find_stream_info,
        avformat_open_input, avio_alloc_context, AVFMT_FLAG_CUSTOM_IO,
    },
    format::context,
    Error,
};
use std::sync::Arc;

pub(crate) struct CleanClosure {
    pub(crate) clean: Arc<dyn Fn() -> ()>,
}

unsafe impl Send for CleanClosure {}

pub(crate) struct CreateCustomIOResult {
    pub(crate) input: ffmpeg_next::format::context::Input,
    pub(crate) clean: CleanClosure,
}

pub(crate) fn input_with_custom_read_io(
    io: Box<dyn StreamIO>,
) -> Result<CreateCustomIOResult, ffmpeg_next::Error> {
    unsafe {
        let buffer_size = 4096;
        let buffer = av_malloc(buffer_size) as *mut u8;
        assert_ne!(buffer, std::ptr::null_mut());
        let opaque = Box::into_raw(Box::new(io));
        let is_writable = false;
        let write_flag = if is_writable { 1 } else { 0 };
        let avio_context = avio_alloc_context(
            buffer,
            buffer_size as i32,
            write_flag,
            std::mem::transmute(opaque),
            Some(read_packet),
            None,
            Some(seek),
        );
        assert_ne!(avio_context, std::ptr::null_mut());

        let avformat_context = avformat_alloc_context();
        assert_ne!(avformat_context, std::ptr::null_mut());

        avformat_context.as_mut().unwrap().pb = avio_context;
        avformat_context.as_mut().unwrap().flags =
            avformat_context.as_mut().unwrap().flags | AVFMT_FLAG_CUSTOM_IO;

        let mut ps = avformat_context;
        match avformat_open_input(
            &mut ps,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ) {
            0 => match avformat_find_stream_info(ps, std::ptr::null_mut()) {
                r if r >= 0 => Ok(CreateCustomIOResult {
                    input: context::Input::wrap(ps),
                    clean: CleanClosure {
                        clean: Arc::new(Box::new(move || {
                            // avio_close(avio_context);
                            let _ = Box::from_raw(opaque);
                        })),
                    },
                }),
                e => {
                    avformat_close_input(&mut ps);
                    Err(Error::from(e))
                }
            },

            e => Err(Error::from(e)),
        }
    }
}

unsafe extern "C" fn read_packet(
    opaque: *mut std::ffi::c_void,
    buf: *mut u8,
    buf_size: std::ffi::c_int,
) -> std::ffi::c_int {
    assert_ne!(opaque, std::ptr::null_mut());
    let mut stream_io = Box::from_raw(opaque as *mut Box<dyn StreamIO>);
    let buffer = std::slice::from_raw_parts_mut(buf, buf_size as usize);
    let read = stream_io.read_packet(buffer);
    Box::into_raw(stream_io);
    read
}

unsafe extern "C" fn seek(
    opaque: *mut std::ffi::c_void,
    offset: i64,
    whence: std::ffi::c_int,
) -> i64 {
    assert_ne!(opaque, std::ptr::null_mut());
    let mut stream_io = Box::from_raw(opaque as *mut Box<dyn StreamIO>);
    let pos = {
        let whence = EWhenceType::try_from(whence).expect("Should be a vaild value");
        stream_io.seek(offset, whence)
    };
    Box::into_raw(stream_io);
    pos
}
