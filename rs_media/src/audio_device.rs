use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample, SizedSample,
};

pub struct Opt {
    pub device: String,
}

pub struct AudioDevice {
    host: cpal::platform::Host,
    device: cpal::Device,
    config: cpal::SupportedStreamConfig,
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
        AudioDevice {
            host,
            device,
            config,
        }
    }

    pub fn run(&mut self) {
        let device = &self.device;
        let config = self.config.clone();
        match self.config.sample_format() {
            cpal::SampleFormat::I8 => Self::run_inner::<i8>(&device, &config.into()),
            cpal::SampleFormat::I16 => Self::run_inner::<i16>(&device, &config.into()),
            cpal::SampleFormat::I32 => Self::run_inner::<i32>(&device, &config.into()),
            cpal::SampleFormat::I64 => Self::run_inner::<i64>(&device, &config.into()),
            cpal::SampleFormat::U8 => Self::run_inner::<u8>(&device, &config.into()),
            cpal::SampleFormat::U16 => Self::run_inner::<u16>(&device, &config.into()),
            cpal::SampleFormat::U32 => Self::run_inner::<u32>(&device, &config.into()),
            cpal::SampleFormat::U64 => Self::run_inner::<u64>(&device, &config.into()),
            cpal::SampleFormat::F32 => Self::run_inner::<f32>(&device, &config.into()),
            cpal::SampleFormat::F64 => Self::run_inner::<f64>(&device, &config.into()),
            sample_format => panic!("Unsupported sample format '{sample_format}'"),
        }
    }

    pub fn run_inner<T>(device: &cpal::Device, config: &cpal::StreamConfig)
    where
        T: SizedSample + FromSample<f32> + std::fmt::Debug,
    {
        let sample_rate = config.sample_rate.0 as f32;
        let channels = config.channels as usize;

        // Produce a sinusoid of maximum amplitude.
        let mut sample_clock = 0f32;
        let mut next_value = move || {
            sample_clock = (sample_clock + 1.0) % sample_rate;
            (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
        };

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    Self::write_data(data, channels, &mut next_value)
                },
                err_fn,
                None,
            )
            .unwrap();
        stream.play().unwrap();

        std::thread::sleep(std::time::Duration::from_secs(1000000));
    }

    fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
    where
        T: Sample + FromSample<f32> + std::fmt::Debug,
    {
        for frame in output.chunks_mut(channels) {
            let value: T = T::from_sample(next_sample());
            for (channel, sample) in frame.iter_mut().enumerate() {
                *sample = value;
            }
        }
    }
}
