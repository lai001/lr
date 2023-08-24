use crate::{audio_format::AudioFormat, audio_format_flag::AudioFormatFlag};

#[derive(Debug)]
pub struct AudioPcmbuffer {
    pub(crate) channel_data: Vec<Box<Vec<u8>>>,
    pub(crate) audio_format: AudioFormat,
    pub(crate) frame_capacity: usize,
}

impl AudioPcmbuffer {
    pub fn from(format: AudioFormat, frame_capacity: usize) -> AudioPcmbuffer {
        let mut channel_data: Vec<Box<Vec<u8>>> = vec![];

        if format
            .format_flags
            .contains(AudioFormatFlag::isNonInterleaved)
        {
            for _ in 0..format.channels_per_frame {
                let data: Box<Vec<u8>> =
                    Box::new(vec![0; frame_capacity * format.bytes_per_frame as usize]);
                channel_data.push(data);
            }
        } else {
            let data: Box<Vec<u8>> =
                Box::new(vec![0; frame_capacity * format.bytes_per_frame as usize]);
            channel_data.push(data);
        }

        AudioPcmbuffer {
            channel_data,
            audio_format: format,
            frame_capacity,
        }
    }

    pub fn get_audio_format(&self) -> &AudioFormat {
        &self.audio_format
    }

    pub fn get_channel_data_view<T>(&self, channel: usize) -> &[T] {
        let channel_data: &Box<Vec<u8>>;
        let len: usize;
        if self
            .audio_format
            .format_flags
            .contains(AudioFormatFlag::isNonInterleaved)
        {
            channel_data = &self.channel_data[channel];
            len = channel_data.len() / std::mem::size_of::<T>();
        } else {
            assert_eq!(channel, 0);
            channel_data = &self.channel_data[channel];
            len = channel_data.len() / std::mem::size_of::<T>();
        }
        unsafe { std::slice::from_raw_parts(channel_data.as_ptr() as *const T, len) }
    }

    pub fn get_mut_channel_data_view<T>(&mut self, channel: usize) -> &mut [T] {
        let channel_data: &mut Box<Vec<u8>>;
        let len: usize;
        if self
            .audio_format
            .format_flags
            .contains(AudioFormatFlag::isNonInterleaved)
        {
            channel_data = &mut self.channel_data[channel];
            len = channel_data.len() / std::mem::size_of::<T>();
        } else {
            assert_eq!(channel, 0);
            channel_data = &mut self.channel_data[channel];
            len = channel_data.len() / std::mem::size_of::<T>();
        }
        unsafe { std::slice::from_raw_parts_mut(channel_data.as_mut_ptr() as *mut T, len) }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        audio_format::{AudioFormat, EAudioFormatIdentifiersType, EAudioSampleType},
        audio_format_flag::AudioFormatFlag,
        audio_pcmbuffer::AudioPcmbuffer,
    };

    #[test]
    pub fn test() {
        let source_format = AudioFormat::from(44100, 2, EAudioSampleType::Float32, true);
        let mut source_buffer = AudioPcmbuffer::from(source_format, 1);
        assert_eq!(source_buffer.channel_data.len(), 2);
        assert_eq!(source_buffer.get_mut_channel_data_view::<f32>(0).len(), 1);
        assert_eq!(source_buffer.get_mut_channel_data_view::<f32>(1).len(), 1);
    }

    #[test]
    pub fn test1() {
        let source_format = AudioFormat::from(44100, 2, EAudioSampleType::SignedInteger32, false);
        let mut source_buffer = AudioPcmbuffer::from(source_format, 100);
        assert_eq!(source_buffer.channel_data.len(), 1);
        assert_eq!(source_buffer.get_mut_channel_data_view::<i32>(0).len(), 200);
    }
}
