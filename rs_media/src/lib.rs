pub mod audio_frame_extractor;
pub mod audio_player_item;
pub mod composition;
pub mod custom_io_input;
pub mod dsp;
pub mod error;
pub mod hw;
pub mod media_stream;
pub mod sw;
pub mod time_range;
pub mod video_frame_extractor;
pub mod video_frame_player;
pub mod video_player_item;

static START: std::sync::Once = std::sync::Once::new();

pub fn init() {
    START.call_once(|| match ffmpeg_next::init() {
        Ok(_) => {}
        Err(error) => panic!("{}", error),
    });
}
