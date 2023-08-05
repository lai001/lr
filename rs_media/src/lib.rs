pub mod audio_device;
pub mod audio_format;
pub mod audio_format_converter;
pub mod audio_format_flag;
pub mod audio_pcmbuffer;
pub mod audio_player_item;
pub mod dsp;
pub mod hw;
pub mod sw;
pub mod time_range;
pub mod video_player_item;

static START: std::sync::Once = std::sync::Once::new();

pub fn init() {
    START.call_once(|| match ffmpeg_next::init() {
        Ok(_) => {}
        Err(error) => panic!("{}", error),
    });
}
