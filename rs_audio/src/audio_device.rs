use crate::audio_mixer_node::AudioMixerNode;
use crate::audio_node::AudioNode;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use lazy_static::lazy_static;
use rs_core_audio::audio_format::{AudioFormat, EAudioSampleType};
use rs_core_audio::audio_format_converter::to_interleaved_data;
use rs_foundation::new::{MultipleThreadMut, MultipleThreadMutType};

pub struct Opt {
    pub device: String,
}

pub struct AudioDevice {
    _host: cpal::platform::Host,
    _device: cpal::Device,
    config: cpal::SupportedStreamConfig,
    stream: cpal::Stream,
}

impl AudioDevice {
    pub fn new() -> crate::error::Result<AudioDevice> {
        let opt = Opt {
            device: "default".to_owned(),
        };
        let host = cpal::default_host();

        let device = if opt.device == "default" {
            host.default_output_device()
        } else {
            host.output_devices()
                .map_err(|err| crate::error::Error::DevicesError(err))?
                .find(|x| {
                    x.description()
                        .map(|y| y.name() == opt.device)
                        .unwrap_or(false)
                })
        }
        .ok_or(crate::error::Error::Other(format!("No audio device")))?;
        log::trace!(
            "Output device: {}",
            device
                .description()
                .map_err(|err| crate::error::Error::DeviceNameError(err))?
        );
        let supported_output_configs = device
            .supported_output_configs()
            .map_err(|err| crate::error::Error::SupportedStreamConfigsError(err))?;
        for supported_output_config in supported_output_configs {
            log::trace!("Supported output config: {:?}", supported_output_config);
        }
        let config = device
            .default_output_config()
            .map_err(|err| crate::error::Error::DefaultStreamConfigError(err))?;
        let sample_format = config.sample_format();

        log::trace!("Default output config: {:?}", config);

        let err_fn = |err| log::error!("an error occurred on stream: {}", err);

        let sample_type = match sample_format {
            cpal::SampleFormat::I8 => unimplemented!(),
            cpal::SampleFormat::I16 => todo!(),
            cpal::SampleFormat::I32 => todo!(),
            cpal::SampleFormat::I64 => unimplemented!(),
            cpal::SampleFormat::U8 => unimplemented!(),
            cpal::SampleFormat::U16 => todo!(),
            cpal::SampleFormat::U32 => todo!(),
            cpal::SampleFormat::U64 => unimplemented!(),
            cpal::SampleFormat::F32 => EAudioSampleType::Float32,
            cpal::SampleFormat::F64 => todo!(),
            _ => unimplemented!(),
        };
        let expect_audio_format: AudioFormat = AudioFormat::from(
            config.sample_rate(),
            config.channels() as u32,
            sample_type,
            false,
        );
        set_global_output_node(AudioMixerNode::new("MainOutput".to_string()));

        let stream = match sample_format {
            cpal::SampleFormat::I8 => unimplemented!(),
            cpal::SampleFormat::I16 => todo!(),
            cpal::SampleFormat::I32 => todo!(),
            cpal::SampleFormat::I64 => unimplemented!(),
            cpal::SampleFormat::U8 => unimplemented!(),
            cpal::SampleFormat::U16 => todo!(),
            cpal::SampleFormat::U32 => todo!(),
            cpal::SampleFormat::U64 => unimplemented!(),
            cpal::SampleFormat::F32 => device
                .build_output_stream(
                    &config.config(),
                    {
                        let output_node = get_global_output_node().clone();
                        let stream_config = config.config().clone();
                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                            data.fill(0.0);
                            let channels = stream_config.channels as usize;
                            let samples = data.len() / channels;
                            let mut output_node = output_node.lock().unwrap();
                            match output_node.next_buffer(samples, expect_audio_format) {
                                Some(next_buffer) => {
                                    let mut source_data: Vec<&[f32]> = vec![];
                                    for i in 0..next_buffer.get_channel_data().len() {
                                        let data =
                                            next_buffer.get_channel_data_view::<f32>(i as usize);
                                        source_data.push(data);
                                    }
                                    let interleaved_data = to_interleaved_data(&source_data);
                                    data.copy_from_slice(&interleaved_data);
                                }
                                None => {}
                            }
                        }
                    },
                    err_fn,
                    None,
                )
                .map_err(|err| crate::error::Error::BuildStreamError(err))?,
            cpal::SampleFormat::F64 => todo!(),
            _ => unimplemented!(),
        };

        Ok(AudioDevice {
            _host: host,
            _device: device,
            config,
            stream,
        })
    }

    pub fn play(&mut self) -> crate::error::Result<()> {
        let result = self
            .stream
            .play()
            .map_err(|err| crate::error::Error::PlayStreamError(err));
        result
    }

    pub fn get_config(&self) -> cpal::StreamConfig {
        self.config.config()
    }
}

lazy_static! {
    static ref GLOBAL_OUTPUT_NODE: MultipleThreadMutType<AudioMixerNode> =
        MultipleThreadMut::new(AudioMixerNode::new(String::from("None"),));
}

pub(crate) fn get_global_output_node() -> MultipleThreadMutType<AudioMixerNode> {
    GLOBAL_OUTPUT_NODE.clone()
}

fn set_global_output_node(node: AudioMixerNode) {
    *GLOBAL_OUTPUT_NODE.lock().unwrap() = node;
}
