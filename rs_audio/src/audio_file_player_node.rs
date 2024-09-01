use crate::audio_node::AudioNode;
use rs_core_audio::{
    audio_format::{AudioFormat, EAudioSampleType},
    audio_format_converter::AudioFormatConverter,
    audio_pcmbuffer::AudioPcmbuffer,
};
use rs_media::audio_player_item::AudioPlayerItem;
use std::{collections::VecDeque, path::Path};

pub struct AudioFilePlayerNode {
    audio_player_item: Option<AudioPlayerItem>,
    channel_data: Vec<VecDeque<u8>>,
    is_playing: bool,
    audio_format: AudioFormat,
}

fn calculate_read_frames(
    source_len: usize,
    source_sample_rate: u32,
    target_sample_rate: u32,
) -> usize {
    (source_len as f32 / source_sample_rate as f32 * target_sample_rate as f32) as usize
}

impl AudioNode for AudioFilePlayerNode {
    fn next_buffer(
        &mut self,
        expect_samples_per_channel: usize,
        expect_audio_format: AudioFormat,
    ) -> Option<AudioPcmbuffer> {
        if !self.is_playing {
            return None;
        }
        self.fill_buffer_samples();
        let mut next_buffer: Option<AudioPcmbuffer> = None;
        if !self.channel_data.is_empty() {
            let read_frames = calculate_read_frames(
                expect_samples_per_channel,
                expect_audio_format.sample_rate,
                self.audio_format.sample_rate,
            );
            let drain_buffer = self.drain_buffer(read_frames);
            if let Some(drain_buffer) = drain_buffer {
                let converted_buffer =
                    AudioFormatConverter::convert(&drain_buffer, &expect_audio_format);

                next_buffer = Some(converted_buffer);
            }
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
            audio_format: AudioFormat::from(44100, 2, EAudioSampleType::Float32, true),
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

    fn fill_buffer_samples(&mut self) {
        let Some(audio_player_item) = self.audio_player_item.as_mut() else {
            return;
        };
        let Ok(audio_frame) = audio_player_item.try_recv() else {
            return;
        };
        self.audio_format = *audio_frame.pcm_buffer.get_audio_format();

        if self.channel_data.len() != audio_frame.pcm_buffer.get_channel_data().len() {
            self.channel_data.resize(
                audio_frame.pcm_buffer.get_channel_data().len(),
                VecDeque::new(),
            );
        }

        for i in 0..audio_frame.pcm_buffer.get_channel_data().len() {
            let data_view = audio_frame
                .pcm_buffer
                .get_channel_data_view::<f32>(i as usize);
            let raw_buffer = rs_foundation::cast_to_raw_buffer(data_view);
            self.channel_data[i as usize].append(&mut VecDeque::from(raw_buffer.to_vec()));
        }
    }

    fn drain_buffer(&mut self, read_frames: usize) -> Option<AudioPcmbuffer> {
        type ReadType = f32;
        let bytes_size = std::mem::size_of::<ReadType>();
        let mut source_buffer = AudioPcmbuffer::from(self.audio_format, read_frames);
        let channels = if self.audio_format.is_non_interleaved() {
            1
        } else {
            self.audio_format.channels_per_frame
        };
        for i in 0..source_buffer.get_channel_data().len() {
            let data = source_buffer.get_mut_channel_data_view::<ReadType>(i);

            let channel_data = &mut self.channel_data[i];
            let channel_data = channel_data.make_contiguous();

            let range = 0..read_frames * bytes_size * channels as usize;
            if channel_data.len() < range.end {
                return None;
            }

            let channel_data = rs_foundation::cast_to_type_buffer(
                &channel_data[0..read_frames * bytes_size * channels as usize],
            );

            data.copy_from_slice(channel_data);
            self.channel_data[i].drain(0..read_frames * bytes_size * channels as usize);
        }
        Some(source_buffer)
    }
}

#[cfg(test)]
mod test {
    use super::calculate_read_frames;

    #[test]
    fn test() {
        assert_eq!(calculate_read_frames(44100, 44100, 22050), 22050);
        assert_eq!(calculate_read_frames(44100 * 2, 44100, 22050), 44100);
    }
}
