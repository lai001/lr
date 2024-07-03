use crate::{audio_device::get_global_output_node, audio_node::AudioOutputNode};
use rs_foundation::new::MultipleThreadMutType;

pub struct AudioEngine {
    default_output_node: MultipleThreadMutType<AudioOutputNode>,
}

impl AudioEngine {
    pub fn new() -> AudioEngine {
        let default_output_node: MultipleThreadMutType<AudioOutputNode> = get_global_output_node();
        AudioEngine {
            default_output_node,
        }
    }

    pub fn get_default_output_node(&self) -> MultipleThreadMutType<AudioOutputNode> {
        self.default_output_node.clone()
    }
}

#[cfg(test)]
mod test {
    use super::AudioEngine;
    use crate::{audio_device::AudioDevice, audio_node::AudioFilePlayerNode};
    use rs_core_minimal::file_manager;
    use rs_foundation::new::MultipleThreadMut;
    use std::{thread::sleep, time::Duration};

    #[test]
    fn test() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .init();

        let path = file_manager::get_engine_resource("Remote/sample-15s.mp3");

        let mut audio_device = AudioDevice::new().unwrap();
        audio_device.play().unwrap();

        let audio_engine = AudioEngine::new();
        let default_output_node = audio_engine.get_default_output_node();
        let audio_player_node = MultipleThreadMut::new(AudioFilePlayerNode::new(path.clone()));
        audio_player_node.lock().unwrap().start();
        default_output_node
            .lock()
            .unwrap()
            .connect(audio_player_node);

        sleep(Duration::from_secs_f32(10.0));
    }
}
