use crate::audio_format_flag::AudioFormatFlag;

const BIT_PER_BYTE: u32 = 8;

#[derive(Debug, Clone, Copy)]
pub enum EAudioFormatIdentifiersType {
    Pcm,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EAudioSampleType {
    Float64,
    Float32,
    SignedInteger16,
    SignedInteger32,
    UnsignedInteger16,
    UnsignedInteger32,
}

impl EAudioSampleType {
    pub fn get_bits(&self) -> u32 {
        match self {
            EAudioSampleType::Float64 => BIT_PER_BYTE * std::mem::size_of::<f64>() as u32,
            EAudioSampleType::Float32 => BIT_PER_BYTE * std::mem::size_of::<f32>() as u32,
            EAudioSampleType::SignedInteger16 => BIT_PER_BYTE * std::mem::size_of::<i16>() as u32,
            EAudioSampleType::SignedInteger32 => BIT_PER_BYTE * std::mem::size_of::<i32>() as u32,
            EAudioSampleType::UnsignedInteger16 => BIT_PER_BYTE * std::mem::size_of::<u16>() as u32,
            EAudioSampleType::UnsignedInteger32 => BIT_PER_BYTE * std::mem::size_of::<u32>() as u32,
        }
    }

    pub fn get_bytes(&self) -> u32 {
        self.get_bits() / BIT_PER_BYTE
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AudioFormat {
    pub sample_rate: u32,
    pub format_type: EAudioFormatIdentifiersType,
    pub format_flags: AudioFormatFlag,
    pub bytes_per_packet: u32,
    pub frames_per_packet: u32,
    pub bytes_per_frame: u32,
    pub channels_per_frame: u32,
    pub bits_per_channel: u32,
}

impl AudioFormat {
    pub fn from(
        sample_rate: u32,
        channels: u32,
        sample_type: EAudioSampleType,
        is_non_interleaved: bool,
    ) -> AudioFormat {
        let mut format_flags = AudioFormatFlag::empty();
        if is_non_interleaved {
            format_flags.insert(AudioFormatFlag::isNonInterleaved)
        }
        if sample_type == EAudioSampleType::Float32 || sample_type == EAudioSampleType::Float64 {
            format_flags.insert(AudioFormatFlag::isFloat)
        }
        if sample_type == EAudioSampleType::SignedInteger16
            || sample_type == EAudioSampleType::SignedInteger32
        {
            format_flags.insert(AudioFormatFlag::isSignedInteger)
        }
        let mut audio_buffer = AudioFormat {
            sample_rate,
            format_type: EAudioFormatIdentifiersType::Pcm,
            format_flags,
            bytes_per_packet: 0,
            frames_per_packet: 1,
            bytes_per_frame: 0,
            channels_per_frame: channels,
            bits_per_channel: sample_type.get_bits(),
        };
        audio_buffer.auto_fill_bytes_per_frame();
        audio_buffer.bytes_per_packet = audio_buffer.bytes_per_frame;
        audio_buffer
    }

    pub fn get_bytes_per_frame(
        channels_per_frame: u32,
        bits_per_channel: u32,
        format_flags: AudioFormatFlag,
    ) -> u32 {
        let bytes = Self::get_bytes_per_channel(bits_per_channel);
        if format_flags.contains(AudioFormatFlag::isNonInterleaved) {
            return bytes;
        } else {
            return bytes * channels_per_frame;
        }
    }

    pub fn auto_fill_bytes_per_frame(&mut self) {
        self.bytes_per_frame = Self::get_bytes_per_frame(
            self.channels_per_frame,
            self.bits_per_channel,
            self.format_flags,
        );
    }

    pub fn get_bytes_per_channel(bits_per_channel: u32) -> u32 {
        assert_eq!(bits_per_channel % BIT_PER_BYTE, 0);
        let bytes = bits_per_channel / BIT_PER_BYTE;
        bytes
    }

    pub fn is_validated(&self) -> bool {
        if self.format_flags.contains(AudioFormatFlag::isFloat)
            && self.format_flags.contains(AudioFormatFlag::isSignedInteger)
        {
            return false;
        }
        if self.bits_per_channel % BIT_PER_BYTE != 0 {
            return false;
        }
        return true;
    }

    pub fn get_sample_type(&self) -> EAudioSampleType {
        assert!(self.is_validated());
        match self.bits_per_channel {
            16 => {
                if self.format_flags.contains(AudioFormatFlag::isFloat) {
                    panic!()
                }
                if self.format_flags.contains(AudioFormatFlag::isSignedInteger) {
                    EAudioSampleType::SignedInteger16
                } else {
                    EAudioSampleType::UnsignedInteger16
                }
            }
            32 => {
                if self.format_flags.contains(AudioFormatFlag::isFloat) {
                    return EAudioSampleType::Float32;
                } else if self.format_flags.contains(AudioFormatFlag::isSignedInteger) {
                    return EAudioSampleType::SignedInteger32;
                } else {
                    return EAudioSampleType::UnsignedInteger32;
                }
            }
            64 => {
                if self.format_flags.contains(AudioFormatFlag::isFloat) {
                    return EAudioSampleType::Float64;
                } else {
                    panic!()
                }
            }
            _ => panic!(),
        }
    }

    pub fn is_non_interleaved(&self) -> bool {
        self.format_flags
            .contains(AudioFormatFlag::isNonInterleaved)
    }
}
