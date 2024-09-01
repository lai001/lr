use rs_core_audio::{audio_format::AudioFormat, audio_pcmbuffer::AudioPcmbuffer};

pub(crate) trait ID {
    fn get_id(&self) -> String;
}

pub trait AudioNode: Send {
    fn next_buffer(
        &mut self,
        expect_samples_per_channel: usize,
        expect_audio_format: AudioFormat,
    ) -> Option<AudioPcmbuffer>;
}

impl ID for dyn AudioNode {
    fn get_id(&self) -> String {
        let raw = unsafe { std::mem::transmute::<_, (usize, usize)>(self) };
        std::format!("{:?}", raw)
    }
}
