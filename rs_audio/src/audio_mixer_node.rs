use crate::audio_node::{AudioNode, ID};
use rs_core_audio::{audio_format::AudioFormat, audio_pcmbuffer::AudioPcmbuffer};
use rs_foundation::new::MultipleThreadMutType;
use std::collections::HashMap;

pub struct AudioMixerNode {
    nodes: HashMap<String, MultipleThreadMutType<dyn AudioNode>>,
    name: String,
}

impl AudioMixerNode {
    pub fn new(name: String) -> AudioMixerNode {
        AudioMixerNode {
            nodes: HashMap::new(),
            name,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn connect(&mut self, node: MultipleThreadMutType<dyn AudioNode>) {
        let id = {
            let node = node.lock().unwrap();
            node.get_id()
        };
        self.nodes.insert(id, node.clone());
    }

    pub fn disconnect(&mut self, node: MultipleThreadMutType<dyn AudioNode>) {
        let id = {
            let node = node.lock().unwrap();
            node.get_id()
        };
        self.nodes.remove(&id);
    }
}

impl AudioNode for AudioMixerNode {
    fn next_buffer(
        &mut self,
        expect_samples_per_channel: usize,
        expect_audio_format: AudioFormat,
    ) -> Option<AudioPcmbuffer> {
        let mut buffers: Vec<AudioPcmbuffer> = vec![];
        for (_, node) in self.nodes.clone() {
            let mut node = node.lock().unwrap();
            let node_buffer = node.next_buffer(expect_samples_per_channel, expect_audio_format);
            if let Some(node_buffer) = node_buffer {
                buffers.push(node_buffer);
            }
        }

        if buffers.is_empty() {
            return None;
        } else {
            let mut mix_buffer =
                AudioPcmbuffer::from(expect_audio_format, expect_samples_per_channel);

            for channel in 0..mix_buffer.get_channel_data().len() {
                let datas: &mut [f32] = mix_buffer.get_mut_channel_data_view(channel);
                for buffer in &buffers {
                    let pending_mix_datas: &[f32] = buffer.get_channel_data_view(channel);
                    for (a, b) in std::iter::zip(&mut *datas, pending_mix_datas) {
                        *a = *a + b;
                    }
                }
            }
            return Some(mix_buffer);
        }
    }
}
