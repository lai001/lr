use crate::{
    audio_format::{AudioFormat, EAudioSampleType},
    audio_pcmbuffer::AudioPcmbuffer,
};

pub struct AudioFormatConverter {}

impl AudioFormatConverter {
    pub fn convert(source_buffer: &AudioPcmbuffer, to_format: &AudioFormat) -> AudioPcmbuffer {
        assert_eq!(
            source_buffer.get_audio_format().channels_per_frame,
            to_format.channels_per_frame
        );
        assert!(source_buffer.get_audio_format().channels_per_frame > 0);
        assert!(to_format.channels_per_frame > 0);

        let to_is_non_interleaved = to_format.is_non_interleaved();
        let source_is_non_interleaved = source_buffer.get_audio_format().is_non_interleaved();

        let mut tmp_buffer: Vec<Vec<f64>> = if source_is_non_interleaved {
            vec![
                vec![0.0_f64; source_buffer.frame_capacity as usize];
                to_format.channels_per_frame as usize
            ]
        } else {
            vec![vec![
                0.0_f64;
                source_buffer.frame_capacity
                    * to_format.channels_per_frame as usize
            ]]
        };

        let source_sample_type = source_buffer.get_audio_format().get_sample_type();
        match source_sample_type {
            EAudioSampleType::Float64 => {
                for i in 0..source_buffer.channel_data.len() {
                    let buffer: &[f64] = source_buffer.get_channel_data_view(i as usize);
                    let to = &mut tmp_buffer[i];
                    to.copy_from_slice(buffer);
                }
            }
            EAudioSampleType::Float32 => {
                for i in 0..source_buffer.channel_data.len() {
                    let buffer: Vec<f64> = source_buffer
                        .get_channel_data_view(i as usize)
                        .iter()
                        .map(|x: &f32| *x as f64)
                        .collect();
                    let to = &mut tmp_buffer[i];
                    to.copy_from_slice(&buffer);
                }
            }
            EAudioSampleType::SignedInteger16 => {
                for i in 0..source_buffer.channel_data.len() {
                    let buffer: Vec<_> = source_buffer
                        .get_channel_data_view(i as usize)
                        .iter()
                        .map(|x: &i16| *x as f64 / std::i16::MAX as f64)
                        .collect();

                    let to = &mut tmp_buffer[i];
                    to.copy_from_slice(&buffer);
                }
            }
            EAudioSampleType::SignedInteger32 => {
                for i in 0..source_buffer.channel_data.len() {
                    let buffer: Vec<_> = source_buffer
                        .get_channel_data_view(i as usize)
                        .iter()
                        .map(|x: &i32| *x as f64 / std::i32::MAX as f64)
                        .collect();
                    let to = &mut tmp_buffer[i];
                    to.copy_from_slice(&buffer);
                }
            }
            EAudioSampleType::UnsignedInteger16 => {
                for i in 0..source_buffer.channel_data.len() {
                    let buffer: Vec<_> = source_buffer
                        .get_channel_data_view(i as usize)
                        .iter()
                        .map(|x: &u16| *x as f64 / std::u16::MAX as f64)
                        .collect();
                    let to = &mut tmp_buffer[i];
                    to.copy_from_slice(&buffer);
                }
            }
            EAudioSampleType::UnsignedInteger32 => {
                for i in 0..source_buffer.channel_data.len() {
                    let buffer: Vec<_> = source_buffer
                        .get_channel_data_view(i as usize)
                        .iter()
                        .map(|x: &u32| *x as f64 / std::u32::MAX as f64)
                        .collect();
                    let to = &mut tmp_buffer[i];
                    to.copy_from_slice(&buffer);
                }
            }
        }
        let mut to_audio_pcmbuffer_capacity = source_buffer.frame_capacity;

        if !source_is_non_interleaved {
            let buffer = to_deinterleaved_data(
                &tmp_buffer[0],
                source_buffer.get_audio_format().channels_per_frame as usize,
            );
            tmp_buffer = buffer;
        }

        if source_buffer.get_audio_format().sample_rate != to_format.sample_rate {
            for samples in &mut tmp_buffer {
                let mut source = dasp::signal::from_iter(samples.iter().cloned());
                // let frames = ring_buffer::Fixed::from(vec![0.0; 1024]);
                // let interp = Sinc::new(frames);
                let a = dasp::Signal::next(&mut source);
                let b = dasp::Signal::next(&mut source);
                let interp = dasp::interpolate::linear::Linear::new(a, b);
                let resampled = dasp::Signal::from_hz_to_hz(
                    source,
                    interp,
                    source_buffer.get_audio_format().sample_rate as f64,
                    to_format.sample_rate as f64,
                );
                let mut resampled_frames: Vec<f64> = vec![];
                for frame in dasp::Signal::until_exhausted(resampled) {
                    resampled_frames.push(frame);
                }

                *samples = resampled_frames;
            }
            to_audio_pcmbuffer_capacity = tmp_buffer[0].len();
        }

        if !to_is_non_interleaved {
            let source_data: Vec<&[f64]> = tmp_buffer.iter().map(|x| x.as_slice()).collect();
            tmp_buffer[0] = to_interleaved_data(&source_data);
            if tmp_buffer.len() > 1 {
                tmp_buffer.drain(1..);
            }
        }

        let mut to_audio_pcmbuffer = AudioPcmbuffer::from(*to_format, to_audio_pcmbuffer_capacity);
        let to_sample_type = to_format.get_sample_type();

        match to_sample_type {
            EAudioSampleType::Float64 => {
                for i in 0..to_audio_pcmbuffer.channel_data.len() {
                    let buffer: &mut [f64] =
                        to_audio_pcmbuffer.get_mut_channel_data_view(i as usize);
                    let from = &tmp_buffer[i as usize];
                    buffer.copy_from_slice(from);
                }
            }
            EAudioSampleType::Float32 => {
                for i in 0..to_audio_pcmbuffer.channel_data.len() {
                    let buffer: &mut [f32] =
                        to_audio_pcmbuffer.get_mut_channel_data_view(i as usize);
                    let from: Vec<f32> = tmp_buffer[i as usize].iter().map(|x| *x as f32).collect();
                    buffer.copy_from_slice(&from);
                }
            }
            EAudioSampleType::SignedInteger16 => {
                for i in 0..to_audio_pcmbuffer.channel_data.len() {
                    let buffer: &mut [i16] =
                        to_audio_pcmbuffer.get_mut_channel_data_view(i as usize);

                    let from: Vec<i16> = tmp_buffer[i as usize]
                        .iter()
                        .map(|x| (*x * std::i16::MAX as f64) as i16)
                        .collect();
                    buffer.copy_from_slice(&from);
                }
            }
            EAudioSampleType::SignedInteger32 => {
                for i in 0..to_audio_pcmbuffer.channel_data.len() {
                    let buffer: &mut [i32] =
                        to_audio_pcmbuffer.get_mut_channel_data_view(i as usize);

                    let from: Vec<i32> = tmp_buffer[i as usize]
                        .iter()
                        .map(|x| (*x * std::i32::MAX as f64) as i32)
                        .collect();
                    buffer.copy_from_slice(&from);
                }
            }
            EAudioSampleType::UnsignedInteger16 => {
                for i in 0..to_audio_pcmbuffer.channel_data.len() {
                    let buffer: &mut [u16] =
                        to_audio_pcmbuffer.get_mut_channel_data_view(i as usize);

                    let from: Vec<u16> = tmp_buffer[i as usize]
                        .iter()
                        .map(|x| (*x * std::u16::MAX as f64) as u16)
                        .collect();
                    buffer.copy_from_slice(&from);
                }
            }
            EAudioSampleType::UnsignedInteger32 => {
                for i in 0..to_audio_pcmbuffer.channel_data.len() {
                    let buffer: &mut [u32] =
                        to_audio_pcmbuffer.get_mut_channel_data_view(i as usize);

                    let from: Vec<u32> = tmp_buffer[i as usize]
                        .iter()
                        .map(|x| (*x * std::u32::MAX as f64) as u32)
                        .collect();
                    buffer.copy_from_slice(&from);
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

pub fn to_deinterleaved_data<T: Copy + Default>(source_data: &[T], channel: usize) -> Vec<Vec<T>> {
    if source_data.is_empty() {
        return vec![];
    }
    let mut output: Vec<Vec<T>> = Vec::new();
    output.resize(channel, vec![T::default(); source_data.len() / channel]);
    for (i, iter) in source_data.chunks_exact(channel).enumerate() {
        for (idx, data) in iter.iter().enumerate() {
            output[idx][i] = *data;
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{to_interleaved_data, AudioFormatConverter};
    use crate::{
        audio_format::{AudioFormat, EAudioFormatIdentifiersType},
        audio_format_converter::to_deinterleaved_data,
        audio_format_flag::AudioFormatFlag,
        audio_pcmbuffer::AudioPcmbuffer,
    };

    #[test]
    pub fn test() {
        let mut builder = env_logger::Builder::new();
        builder.filter_level(log::LevelFilter::Trace);
        builder.init();
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

    #[test]
    fn test_to_interleaved_data() {
        let source_data = vec![vec![0.0, 1.0], vec![2.0, 3.0]];
        let data: Vec<&[f32]> = source_data
            .as_slice()
            .iter()
            .map(|x| x.as_slice())
            .collect();
        let interleaved_data = to_interleaved_data(&data);
        assert_eq!(interleaved_data[0], 0.0);
        assert_eq!(interleaved_data[1], 2.0);
        assert_eq!(interleaved_data[2], 1.0);
        assert_eq!(interleaved_data[3], 3.0);
    }

    #[test]
    fn test_to_deinterleaved_data() {
        let source_data = vec![0.0, 1.0, 2.0, 3.0];
        let deinterleaved_data = to_deinterleaved_data(&source_data, 2);
        assert_eq!(deinterleaved_data[0][0], 0.0);
        assert_eq!(deinterleaved_data[0][1], 2.0);
        assert_eq!(deinterleaved_data[1][0], 1.0);
        assert_eq!(deinterleaved_data[1][1], 3.0);
    }
}
