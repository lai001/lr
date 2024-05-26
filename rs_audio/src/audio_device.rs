use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

pub struct Opt {
    pub device: String,
}

pub struct AudioDevice {
    _host: cpal::platform::Host,
    _device: cpal::Device,
    config: cpal::SupportedStreamConfig,
    stream: cpal::Stream,
    buffer: Arc<Mutex<VecDeque<f32>>>,
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
                .find(|x| x.name().map(|y| y == opt.device).unwrap_or(false))
        }
        .ok_or(crate::error::Error::Other(format!("No audio device")))?;
        log::trace!(
            "Output device: {}",
            device
                .name()
                .map_err(|err| crate::error::Error::DeviceNameError(err))?
        );

        let config = device
            .default_output_config()
            .map_err(|err| crate::error::Error::DefaultStreamConfigError(err))?;
        log::trace!("Default output config: {:?}", config);

        let err_fn = |err| log::error!("an error occurred on stream: {}", err);

        let buffer: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::with_capacity(
            (config.sample_rate().0 * 5) as usize,
        )));
        let buffer_clone = buffer.clone();

        let stream = device
            .build_output_stream(
                &config.config(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut buffer = buffer_clone.lock().unwrap();
                    let need = data.len().min(buffer.len());
                    if need > 0 {
                        let write_data = &buffer.make_contiguous()[0..need];
                        data.copy_from_slice(write_data);
                        buffer.drain(0..need);
                    }
                },
                err_fn,
                None,
            )
            .map_err(|err| crate::error::Error::BuildStreamError(err))?;

        Ok(AudioDevice {
            _host: host,
            _device: device,
            config,
            stream,
            buffer,
        })
    }

    pub fn play(&mut self) -> crate::error::Result<()> {
        self.stream
            .play()
            .map_err(|err| crate::error::Error::PlayStreamError(err))
    }

    pub fn get_config(&self) -> cpal::StreamConfig {
        self.config.config()
    }

    pub fn get_buffer_len(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }

    pub fn push_buffer(&self, data: &[f32]) {
        let mut new_data = VecDeque::from(Vec::from(data));
        let mut buffer = self.buffer.lock().unwrap();
        buffer.append(&mut new_data);
    }
}
