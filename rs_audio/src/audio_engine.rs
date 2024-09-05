use crate::{audio_device::get_global_output_node, audio_node::AudioNode};
use rs_foundation::new::MultipleThreadMutType;

pub struct AudioEngine {
    nodes: Vec<MultipleThreadMutType<dyn AudioNode>>,
}

impl AudioEngine {
    pub fn new() -> AudioEngine {
        AudioEngine { nodes: vec![] }
    }

    pub fn connect(&mut self, node: MultipleThreadMutType<dyn AudioNode>) {
        let mixer_node = get_global_output_node();
        let mut mixer_node = mixer_node.lock().unwrap();
        mixer_node.connect(node.clone());
        self.nodes.push(node);
    }

    pub fn disconnect(&mut self, node: MultipleThreadMutType<dyn AudioNode>) {
        let mixer_node = get_global_output_node();
        let mut mixer_node = mixer_node.lock().unwrap();
        mixer_node.disconnect(node);
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        let mixer_node = get_global_output_node();
        let mut mixer_node = mixer_node.lock().unwrap();
        for node in self.nodes.clone() {
            mixer_node.disconnect(node);
        }
    }
}

#[cfg(test)]
mod test {
    use super::AudioEngine;
    use crate::{audio_device::AudioDevice, audio_player_node::AudioPlayerNode};
    use rs_core_minimal::file_manager;
    use rs_foundation::new::MultipleThreadMut;
    use std::{thread::sleep, time::Duration};

    #[test]
    fn test() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .init();
        let mut audio_device = AudioDevice::new().unwrap();
        audio_device.play().unwrap();

        let path = file_manager::get_engine_resource("Remote/sample-15s.mp3");

        let mut audio_engine = AudioEngine::new();
        let audio_player_node =
            MultipleThreadMut::new(AudioPlayerNode::from_path(path.clone(), false));
        audio_engine.connect(audio_player_node.clone());
        audio_player_node.lock().unwrap().start();
        sleep(Duration::from_secs_f32(10.0));
    }

    #[test]
    fn test1() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .init();
        let mut audio_device = AudioDevice::new().unwrap();
        audio_device.play().unwrap();

        let mut audio_engine = AudioEngine::new();

        let path = file_manager::get_engine_resource("Remote/sample-15s_8000.mp3");
        let audio_player_node =
            MultipleThreadMut::new(AudioPlayerNode::from_path(path.clone(), false));
        audio_engine.connect(audio_player_node.clone());
        audio_player_node.lock().unwrap().start();

        let path = file_manager::get_engine_resource("Remote/bgm_48000.mp3");
        let audio_player_node =
            MultipleThreadMut::new(AudioPlayerNode::from_path(path.clone(), false));
        audio_engine.connect(audio_player_node.clone());
        audio_player_node.lock().unwrap().start();

        sleep(Duration::from_secs_f32(10.0));
    }

    #[test]
    fn test2() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .init();
        let mut audio_device = AudioDevice::new().unwrap();
        audio_device.play().unwrap();

        let mut audio_engine = AudioEngine::new();

        let path = file_manager::get_engine_resource("Remote/sample-15s_8000.mp3");
        let data = { std::fs::read(path).unwrap() };
        let audio_player_node = MultipleThreadMut::new(AudioPlayerNode::from_data(data, false));
        audio_engine.connect(audio_player_node.clone());
        audio_player_node.lock().unwrap().start();

        let path = file_manager::get_engine_resource("Remote/bgm_48000.mp3");
        let data = { std::fs::read(path).unwrap() };
        let audio_player_node = MultipleThreadMut::new(AudioPlayerNode::from_data(data, false));
        audio_engine.connect(audio_player_node.clone());
        audio_player_node.lock().unwrap().start();

        sleep(Duration::from_secs_f32(10.0));
    }
}
