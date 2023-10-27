use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    StreamInstant,
};
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
    sync::{Arc, Mutex},
};

pub struct Opt {
    pub device: String,
}

pub struct AudioDevice {
    host: cpal::platform::Host,
    device: cpal::Device,
    config: cpal::SupportedStreamConfig,
    stream: cpal::Stream,
    buffer: Arc<Mutex<VecDeque<f32>>>,
    device_first_callback_time: Arc<Mutex<Option<StreamInstant>>>,
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

        let buffer: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::with_capacity(
            (config.sample_rate().0 * 5) as usize,
        )));
        let buffer_clone = buffer.clone();

        let device_first_callback_time: Arc<Mutex<Option<StreamInstant>>> =
            Arc::new(Mutex::new(None));
        let device_first_callback_time_clone = device_first_callback_time.clone();

        let last_callback_time: Arc<Mutex<Option<StreamInstant>>> = Arc::new(Mutex::new(None));
        let last_callback_time_clone = last_callback_time.clone();

        let stream = device
            .build_output_stream(
                &config.config(),
                move |data: &mut [f32], output_callback_info: &cpal::OutputCallbackInfo| {
                    {
                        let mut last_callback_time = last_callback_time_clone.lock().unwrap();
                        if let Some(last_callback_time) = last_callback_time.as_ref() {
                            let tick_duration = output_callback_info
                                .timestamp()
                                .callback
                                .duration_since(&last_callback_time)
                                .unwrap();
                        }
                        *last_callback_time = Some(output_callback_info.timestamp().callback);
                    }

                    {
                        let mut device_first_callback_time =
                            device_first_callback_time_clone.lock().unwrap();
                        if device_first_callback_time.is_none() {
                            *device_first_callback_time =
                                Some(output_callback_info.timestamp().callback);
                        }
                        let current_audio_device_time = output_callback_info
                            .timestamp()
                            .callback
                            .duration_since(&device_first_callback_time.unwrap())
                            .unwrap();
                    }

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
            .unwrap();

        AudioDevice {
            host,
            device,
            config,
            stream,
            buffer,
            device_first_callback_time,
        }
    }

    pub fn play(&mut self) {
        self.stream.play().unwrap();
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
