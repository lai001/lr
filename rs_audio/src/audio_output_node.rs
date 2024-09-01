use crate::audio_node::AudioNode;
use rs_core_audio::{audio_format::AudioFormat, audio_pcmbuffer::AudioPcmbuffer};
use rs_foundation::new::MultipleThreadMutType;

pub struct AudioOutputNode {
    audio_format: AudioFormat,
    node: Option<MultipleThreadMutType<dyn AudioNode>>,
    name: String,
}

impl AudioNode for AudioOutputNode {
    fn next_buffer(
        &mut self,
        expect_samples_per_channel: usize,
        expect_audio_format: AudioFormat,
    ) -> Option<AudioPcmbuffer> {
        match &self.node {
            Some(node) => node
                .lock()
                .unwrap()
                .next_buffer(expect_samples_per_channel, expect_audio_format),
            None => None,
        }
    }
}

impl AudioOutputNode {
    pub fn new(name: String, audio_format: AudioFormat) -> AudioOutputNode {
        AudioOutputNode {
            audio_format,
            node: None,
            name,
        }
    }

    pub fn set_output_format(&mut self, audio_format: AudioFormat) {
        self.audio_format = audio_format;
    }

    pub fn connect(&mut self, node: MultipleThreadMutType<dyn AudioNode>) {
        self.node = Some(node);
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn disconnect(&mut self) {
        self.node = None;
    }
}
