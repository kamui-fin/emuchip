use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SizedSample};

pub struct Sound {
    device: cpal::Device,
    config: cpal::StreamConfig,
    format: cpal::SampleFormat,
}

impl Sound {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");
        let mut supported_configs_range = device
            .supported_output_configs()
            .expect("error while querying configs");
        let supported_config = supported_configs_range
            .next()
            .expect("no supported config?!")
            .with_max_sample_rate();
        let sample_format = supported_config.sample_format();
        Self {
            device,
            config: supported_config.into(),
            format: sample_format,
        }
    }

    pub fn beep(&self) {
        match self.format {
            cpal::SampleFormat::I8 => self.run::<i8>(),
            cpal::SampleFormat::I16 => self.run::<i16>(),
            // cpal::SampleFormat::I24 => self.run::<I24>(),
            cpal::SampleFormat::I32 => self.run::<i32>(),
            // cpal::SampleFormat::I48 => self.run::<I48>(),
            cpal::SampleFormat::I64 => self.run::<i64>(),
            cpal::SampleFormat::U8 => self.run::<u8>(),
            cpal::SampleFormat::U16 => self.run::<u16>(),
            // cpal::SampleFormat::U24 => self.run::<U24>(),
            cpal::SampleFormat::U32 => self.run::<u32>(),
            // cpal::SampleFormat::U48 => self.run::<U48>(),
            cpal::SampleFormat::U64 => self.run::<u64>(),
            cpal::SampleFormat::F32 => self.run::<f32>(),
            cpal::SampleFormat::F64 => self.run::<f64>(),
            sample_format => panic!("Unsupported sample format '{sample_format}'"),
        };
    }

    fn run<T>(&self)
    where
        T: SizedSample + FromSample<f32>,
    {
        let sample_rate = self.config.sample_rate.0 as f32;
        let channels = self.config.channels as usize;

        // Produce a sinusoid of maximum amplitude.
        let mut sample_clock = 0f32;
        let mut next_value = move || {
            sample_clock = (sample_clock + 1.0) % sample_rate;
            (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
        };

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = self
            .device
            .build_output_stream(
                &self.config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    Self::write_data(data, channels, &mut next_value)
                },
                err_fn,
                None,
            )
            .unwrap();

        stream.play().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
    where
        T: Sample + FromSample<f32>,
    {
        for frame in output.chunks_mut(channels) {
            let value: T = T::from_sample(next_sample());
            for sample in frame.iter_mut() {
                *sample = value;
            }
        }
    }
}
