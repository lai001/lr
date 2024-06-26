use crate::audio_node::AudioOutputNode;
use rs_foundation::new::MultipleThreadMutType;

pub struct AudioEngine {
    default_output_node: MultipleThreadMutType<AudioOutputNode>,
}

impl AudioEngine {
    pub(crate) fn new(default_output_node: MultipleThreadMutType<AudioOutputNode>) -> AudioEngine {
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
    use crate::{
        audio_device::AudioDevice,
        audio_node::{AudioFilePlayerNode, AudioNode},
    };
    use rs_core_minimal::file_manager;
    use rs_foundation::new::{MultipleThreadMut, MultipleThreadMutType};
    use std::{thread::sleep, time::Duration};

    #[test]
    fn test() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .init();

        let path = file_manager::get_engine_resource("Remote/sample-15s.mp3");

        let mut audio_device = AudioDevice::new().unwrap();
        audio_device.play().unwrap();

        let audio_engine = audio_device.create_audio_engien();
        let default_output_node = audio_engine.get_default_output_node();
        let audio_player_node: MultipleThreadMutType<Box<dyn AudioNode>> =
            MultipleThreadMut::new(Box::new(AudioFilePlayerNode::new(path.clone())));
        default_output_node
            .lock()
            .unwrap()
            .connect(audio_player_node);

        sleep(Duration::from_secs_f32(10.0));
    }
}
