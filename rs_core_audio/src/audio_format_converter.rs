use crate::{
    audio_format::{AudioFormat, EAudioSampleType},
    audio_format_flag::AudioFormatFlag,
    audio_pcmbuffer::AudioPcmbuffer,
};

pub struct AudioFormatConverter {}

impl AudioFormatConverter {
    pub fn convert(source_buffer: &AudioPcmbuffer, to_format: &AudioFormat) -> AudioPcmbuffer {
        assert_eq!(
            source_buffer.get_audio_format().sample_rate,
            to_format.sample_rate
        );
        assert_eq!(
            source_buffer.get_audio_format().channels_per_frame,
            to_format.channels_per_frame
        );

        let source_sample_type = source_buffer.get_audio_format().get_sample_type();
        let mut tmp_buffer: Vec<f64> =
            vec![0.0_f64; source_buffer.frame_capacity * to_format.channels_per_frame as usize];

        let mut offset: usize = 0;

        match source_sample_type {
            EAudioSampleType::Float64 => {
                for i in 0..source_buffer.channel_data.len() {
                    let buffer: &[f64] = source_buffer.get_channel_data_view(i as usize);
                    for data in buffer {
                        tmp_buffer[offset] = *data;
                        offset += 1;
                    }
                }
            }
            EAudioSampleType::Float32 => {
                for i in 0..source_buffer.channel_data.len() {
                    let buffer: &[f32] = source_buffer.get_channel_data_view(i as usize);
                    for data in buffer {
                        tmp_buffer[offset] = *data as f64;
                        offset += 1;
                    }
                }
            }
            EAudioSampleType::SignedInteger16 => {
                for i in 0..source_buffer.channel_data.len() {
                    let buffer: &[i16] = source_buffer.get_channel_data_view(i as usize);
                    for data in buffer {
                        tmp_buffer[offset] = *data as f64 / std::i16::MAX as f64;
                        offset += 1;
                    }
                }
            }
            EAudioSampleType::SignedInteger32 => {
                for i in 0..source_buffer.channel_data.len() {
                    let buffer: &[i32] = source_buffer.get_channel_data_view(i as usize);
                    for data in buffer {
                        tmp_buffer[offset] = *data as f64 / std::i32::MAX as f64;
                        offset += 1;
                    }
                }
            }
            EAudioSampleType::UnsignedInteger16 => {
                for i in 0..source_buffer.channel_data.len() {
                    let buffer: &[u16] = source_buffer.get_channel_data_view(i as usize);
                    for data in buffer {
                        tmp_buffer[offset] = *data as f64 / std::u16::MAX as f64;
                        offset += 1;
                    }
                }
            }
            EAudioSampleType::UnsignedInteger32 => {
                for i in 0..source_buffer.channel_data.len() {
                    let buffer: &[u32] = source_buffer.get_channel_data_view(i as usize);
                    for data in buffer {
                        tmp_buffer[offset] = *data as f64 / std::u32::MAX as f64;
                        offset += 1;
                    }
                }
            }
        }

        let mut tmp_buffer2: Vec<Vec<f64>> = vec![];

        let to_is_non_interleaved = to_format
            .format_flags
            .contains(AudioFormatFlag::isNonInterleaved);

        let source_is_non_interleaved = source_buffer
            .get_audio_format()
            .format_flags
            .contains(AudioFormatFlag::isNonInterleaved);

        match (source_is_non_interleaved, to_is_non_interleaved) {
            (true, true) => {
                for i in 0..to_format.channels_per_frame {
                    let start = i as usize * source_buffer.frame_capacity;
                    let end = start + source_buffer.frame_capacity;
                    tmp_buffer2.push(tmp_buffer[start..end].to_vec());
                }
            }
            (true, false) => {
                tmp_buffer2.push(vec![
                    0.0_f64;
                    source_buffer.frame_capacity
                        * to_format.channels_per_frame as usize
                ]);
                let source_channel = source_buffer.get_audio_format().channels_per_frame as usize;
                for i in 0..source_buffer.get_audio_format().channels_per_frame {
                    let start = i as usize * source_buffer.frame_capacity;
                    let end = start + source_buffer.frame_capacity;
                    let tmp_buffer = &tmp_buffer[start..end];
                    for j in 0..tmp_buffer.len() {
                        tmp_buffer2[0][j * source_channel + i as usize] = tmp_buffer[j];
                    }
                }
            }
            (false, true) => {
                for i in 0..to_format.channels_per_frame {
                    tmp_buffer2.push(vec![0.0_f64; source_buffer.frame_capacity as usize]);
                    for j in 0..tmp_buffer2[i as usize].len() {
                        tmp_buffer2[i as usize][j as usize] =
                            tmp_buffer[j * to_format.channels_per_frame as usize + i as usize];
                    }
                }
            }
            (false, false) => tmp_buffer2.push(tmp_buffer),
        }

        let mut to_audio_pcmbuffer = AudioPcmbuffer::from(*to_format, source_buffer.frame_capacity);
        let to_sample_type = to_format.get_sample_type();

        match to_sample_type {
            EAudioSampleType::Float64 => {
                for i in 0..to_audio_pcmbuffer.get_audio_format().channels_per_frame {
                    let buffer: &mut [f64] =
                        to_audio_pcmbuffer.get_mut_channel_data_view(i as usize);
                    let tmp_buffer = tmp_buffer2.get(i as usize).unwrap();
                    for j in 0..tmp_buffer.len() {
                        buffer[j] = tmp_buffer[j];
                    }
                }
            }
            EAudioSampleType::Float32 => {
                for i in 0..to_audio_pcmbuffer.get_audio_format().channels_per_frame {
                    let buffer: &mut [f32] =
                        to_audio_pcmbuffer.get_mut_channel_data_view(i as usize);
                    let tmp_buffer = tmp_buffer2.get(i as usize).unwrap();
                    for j in 0..tmp_buffer.len() {
                        buffer[j] = tmp_buffer[j] as f32;
                    }
                }
            }
            EAudioSampleType::SignedInteger16 => {
                for i in 0..to_audio_pcmbuffer.get_audio_format().channels_per_frame {
                    let buffer: &mut [i16] =
                        to_audio_pcmbuffer.get_mut_channel_data_view(i as usize);
                    let tmp_buffer = tmp_buffer2.get(i as usize).unwrap();
                    for j in 0..tmp_buffer.len() {
                        buffer[j] = (tmp_buffer[j] * std::i16::MAX as f64) as i16;
                    }
                }
            }
            EAudioSampleType::SignedInteger32 => {
                for i in 0..to_audio_pcmbuffer.get_audio_format().channels_per_frame {
                    let buffer: &mut [i32] =
                        to_audio_pcmbuffer.get_mut_channel_data_view(i as usize);
                    let tmp_buffer = tmp_buffer2.get(i as usize).unwrap();
                    for j in 0..tmp_buffer.len() {
                        buffer[j] = (tmp_buffer[j] * std::i32::MAX as f64) as i32;
                    }
                }
            }
            EAudioSampleType::UnsignedInteger16 => {
                for i in 0..to_audio_pcmbuffer.get_audio_format().channels_per_frame {
                    let buffer: &mut [u16] =
                        to_audio_pcmbuffer.get_mut_channel_data_view(i as usize);
                    let tmp_buffer = tmp_buffer2.get(i as usize).unwrap();
                    for j in 0..tmp_buffer.len() {
                        buffer[j] = (tmp_buffer[j] * std::u16::MAX as f64) as u16;
                    }
                }
            }
            EAudioSampleType::UnsignedInteger32 => {
                for i in 0..to_audio_pcmbuffer.get_audio_format().channels_per_frame {
                    let buffer: &mut [u32] =
                        to_audio_pcmbuffer.get_mut_channel_data_view(i as usize);
                    let tmp_buffer = tmp_buffer2.get(i as usize).unwrap();
                    for j in 0..tmp_buffer.len() {
                        buffer[j] = (tmp_buffer[j] * std::u32::MAX as f64) as u32;
                    }
                }
            }
        }

        to_audio_pcmbuffer
    }
}

pub fn to_interleaved_data<T: Copy + Default>(source_data: &[&[T]]) -> Vec<T> {
    if source_data.is_empty() {
        return vec![];
    }
    {
        let len = source_data[0].len();
        for item in source_data.iter().skip(1) {
            if item.len() != len {
                panic!();
            }
        }
    }
    let mut output: Vec<T> = Vec::new();
    output.resize(source_data.len() * source_data[0].len(), T::default());
    for channel in 0..source_data.len() {
        for (i, sample) in output
            .iter_mut()
            .skip(channel)
            .step_by(source_data.len())
            .enumerate()
        {
            *sample = source_data[channel][i];
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use crate::{
        audio_format::{AudioFormat, EAudioFormatIdentifiersType},
        audio_format_flag::AudioFormatFlag,
        audio_pcmbuffer::AudioPcmbuffer,
    };

    use super::AudioFormatConverter;

    #[test]
    pub fn test() {
        let source_format = AudioFormat {
            sample_rate: 44100,
            format_type: EAudioFormatIdentifiersType::Pcm,
            format_flags: AudioFormatFlag::isSignedInteger,
            frames_per_packet: 1,
            bytes_per_packet: 8,
            bytes_per_frame: 8,
            channels_per_frame: 2,
            bits_per_channel: 32,
        };
        let mut source_buffer = AudioPcmbuffer::from(source_format, 1);
        let data: &mut [i32] = source_buffer.get_mut_channel_data_view(0);
        data[0] = std::i32::MAX;
        let data: &mut [i32] = source_buffer.get_mut_channel_data_view(0);
        data[1] = std::i32::MIN;

        let to_format = AudioFormat {
            sample_rate: 44100,
            format_type: EAudioFormatIdentifiersType::Pcm,
            format_flags: AudioFormatFlag::isFloat | AudioFormatFlag::isNonInterleaved,
            frames_per_packet: 1,
            bytes_per_packet: 8,
            bytes_per_frame: 8,
            channels_per_frame: 2,
            bits_per_channel: 64,
        };

        let new_buffer = AudioFormatConverter::convert(&source_buffer, &to_format);
        let data1: &[f64] = new_buffer.get_channel_data_view(0);
        assert!((data1[0] - 1.0).abs() < 0.000000001);
        let data1: &[f64] = new_buffer.get_channel_data_view(1);
        assert!((data1[0] - -1.0).abs() < 0.000000001);
    }
}
