use rs_core_audio::{
    audio_format::{AudioFormat, EAudioSampleType},
    audio_pcmbuffer::AudioPcmbuffer,
};
use rs_foundation::new::MultipleThreadMutType;
use rs_media::audio_player_item::AudioPlayerItem;
use std::{collections::VecDeque, path::Path};

pub trait AudioNode: Send {
    fn next_buffer(
        &mut self,
        expect_samples_per_channel: usize,
        channels: usize,
    ) -> Option<AudioPcmbuffer>;
}

pub struct AudioOutputNode {
    audio_format: AudioFormat,
    node: Option<MultipleThreadMutType<dyn AudioNode>>,
    name: String,
}

impl AudioNode for AudioOutputNode {
    fn next_buffer(
        &mut self,
        expect_samples_per_channel: usize,
        channels: usize,
    ) -> Option<AudioPcmbuffer> {
        match &self.node {
            Some(node) => node
                .lock()
                .unwrap()
                .next_buffer(expect_samples_per_channel, channels),
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

pub struct AudioMixerNode {}

impl AudioNode for AudioMixerNode {
    fn next_buffer(&mut self, _: usize, _: usize) -> Option<AudioPcmbuffer> {
        todo!()
    }
}

pub struct AudioFilePlayerNode {
    audio_player_item: Option<AudioPlayerItem>,
    channel_data: Vec<VecDeque<u8>>,
    is_playing: bool,
}

impl AudioNode for AudioFilePlayerNode {
    fn next_buffer(
        &mut self,
        expect_samples_per_channel: usize,
        channels: usize,
    ) -> Option<AudioPcmbuffer> {
        if !self.is_playing {
            return None;
        }
        match self.audio_player_item.as_mut() {
            Some(audio_player_item) => {
                let audio_frame = audio_player_item.try_recv().ok();
                if let Some(audio_frame) = audio_frame {
                    let audio_format = audio_frame.pcm_buffer.get_audio_format();
                    if self.channel_data.len() != channels {
                        self.channel_data
                            .resize(audio_format.channels_per_frame as usize, VecDeque::new());
                    }
                    for i in 0..audio_format.channels_per_frame {
                        let data_view = audio_frame
                            .pcm_buffer
                            .get_channel_data_view::<f32>(i as usize);
                        let raw_buffer = rs_foundation::cast_to_raw_buffer(data_view);
                        self.channel_data[i as usize]
                            .append(&mut VecDeque::from(raw_buffer.to_vec()));
                    }
                }
            }
            None => {}
        }

        let mut next_buffer: Option<AudioPcmbuffer> = None;

        if !self.channel_data.is_empty() {
            let mut buffer = AudioPcmbuffer::from(
                AudioFormat::from(
                    44100,
                    self.channel_data.len() as u32,
                    EAudioSampleType::Float32,
                    true,
                ),
                expect_samples_per_channel,
            );

            for i in 0..self.channel_data.len() {
                let drain = {
                    let ab = &self.channel_data[i].make_contiguous();
                    let available_samples = ab.len() / 4;
                    if available_samples < expect_samples_per_channel {
                        break;
                    }
                    let data = buffer.get_mut_channel_data_view::<f32>(i as usize);

                    let abf32: &[f32] =
                        rs_foundation::cast_to_type_buffer(&ab[0..expect_samples_per_channel * 4]);
                    data.copy_from_slice(abf32);
                    abf32.len()
                };
                self.channel_data[i].drain(0..drain * 4);
            }
            next_buffer = Some(buffer);
        }

        next_buffer
    }
}

impl AudioFilePlayerNode {
    pub fn new(path: impl AsRef<Path>) -> AudioFilePlayerNode {
        let audio_player_item = AudioPlayerItem::new(path.as_ref().to_path_buf()).ok();
        AudioFilePlayerNode {
            audio_player_item,
            channel_data: vec![],
            is_playing: false,
        }
    }

    pub fn start(&mut self) {
        self.is_playing = true;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
    }

    pub fn seek(&mut self, time: f32) {
        for channel_data in self.channel_data.iter_mut() {
            channel_data.clear();
        }
        if let Some(audio_player_item) = self.audio_player_item.as_mut() {
            audio_player_item.seek(time);
        }
    }
}
