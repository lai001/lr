use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub struct Opt {
    pub device: String,
}

pub struct AudioDevice {
    host: cpal::platform::Host,
    device: cpal::Device,
    config: cpal::SupportedStreamConfig,
    stream: cpal::Stream,
    buffer: Arc<Mutex<Vec<f32>>>,
}

impl AudioDevice {
    pub fn new() -> AudioDevice {
        let opt = Opt {
            device: "default".to_owned(),
        };
        let host = cpal::default_host();

        let device = if opt.device == "default" {
            host.default_output_device()
        } else {
            host.output_devices()
                .unwrap()
                .find(|x| x.name().map(|y| y == opt.device).unwrap_or(false))
        }
        .expect("failed to find output device");
        log::trace!("Output device: {}", device.name().unwrap());

        let config = device.default_output_config().unwrap();
        log::trace!("Default output config: {:?}", config);

        let err_fn = |err| log::error!("an error occurred on stream: {}", err);
        let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(vec![]));

        let buffer_clone = buffer.clone();
        let stream = device
            .build_output_stream(
                &config.config(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut buffer = buffer_clone.lock().unwrap();
                    let (left, right) = buffer.split_at(data.len().min(buffer.len()));
                    let mut write_data = left.to_vec();
                    write_data.resize(data.len(), 0.0);
                    data.copy_from_slice(&write_data);
                    *buffer = right.to_vec();
                },
                err_fn,
                None,
            )
            .unwrap();

        AudioDevice {
            host,
            device,
            config,
            stream,
            buffer,
        }
    }

    pub fn play(&mut self) {
        self.stream.play().unwrap();
    }

    pub fn get_config(&self) -> cpal::StreamConfig {
        self.config.config()
    }

    pub fn get_buffer_mut(&self) -> Arc<Mutex<Vec<f32>>> {
        self.buffer.clone()
    }
}
