use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, PlayStreamError, Sample, SampleRate, Stream, StreamConfig};
use std::error::Error;

pub struct AudioController {
    stream: Stream,
}

impl AudioController {
    pub fn new(sample_rate: u64) -> Result<Self, Box<dyn Error>> {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();

        let config = StreamConfig {
            channels: 2,
            sample_rate: SampleRate(sample_rate.try_into()?),
            buffer_size: BufferSize::Default,
        };

        let stream = device.build_output_stream(
            &config,
            data_callback,
            move |err| {
                eprintln!("{}", err);
            },
            None,
        )?;

        Ok(Self { stream })
    }

    pub fn resume(&mut self) -> Result<(), PlayStreamError> {
        self.stream.play()
    }
}

fn data_callback(data: &mut [f32], _: &cpal::OutputCallbackInfo) {
    for sample in data.iter_mut() {
        *sample = Sample::EQUILIBRIUM;
    }
}
