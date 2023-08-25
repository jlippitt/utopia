use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    BufferSize, OutputCallbackInfo, PlayStreamError, Sample, SampleRate, Stream, StreamConfig,
};
use std::error::Error;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tracing::warn;
use utopia::AudioQueue;

pub struct AudioController {
    stream: Stream,
    sender: mpsc::Sender<(f32, f32)>,
    total_samples: u64,
    sample_rate: u64,
    start_time: Instant,
    sync_time: Instant,
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

        let (sender, receiver) = mpsc::channel();

        let stream = device.build_output_stream(
            &config,
            move |output: &mut [f32], _: &OutputCallbackInfo| {
                let mut input: mpsc::TryIter<'_, (f32, f32)> = receiver.try_iter();
                let mut prev_sample = (Sample::EQUILIBRIUM, Sample::EQUILIBRIUM);

                for sample_out in output.chunks_exact_mut(2) {
                    if let Some(sample_in) = input.next() {
                        sample_out[0] = sample_in.0;
                        sample_out[1] = sample_in.1;
                        prev_sample = sample_in;
                    } else {
                        sample_out[0] = prev_sample.0;
                        sample_out[1] = prev_sample.1;
                    }
                }
            },
            move |err| warn!("{}", err),
            None,
        )?;

        let start_time = Instant::now();

        Ok(Self {
            stream,
            sender,
            total_samples: 0,
            sample_rate,
            start_time: Instant::now(),
            sync_time: calculate_sync_time(start_time, 0, sample_rate),
        })
    }

    pub fn sync_time(&self) -> Instant {
        self.sync_time
    }

    pub fn resume(&mut self) -> Result<(), PlayStreamError> {
        self.stream.play()?;
        self.start_time = Instant::now();
        self.sync_time = calculate_sync_time(self.start_time, self.total_samples, self.sample_rate);
        Ok(())
    }

    pub fn drain(&mut self, queue: &mut AudioQueue) -> Result<(), mpsc::SendError<(f32, f32)>> {
        self.total_samples += queue.len() as u64;
        self.sync_time = calculate_sync_time(self.start_time, self.total_samples, self.sample_rate);

        for sample in queue.drain(..) {
            self.sender.send(sample)?;
        }

        Ok(())
    }
}

fn calculate_sync_time(start_time: Instant, total_samples: u64, sample_rate: u64) -> Instant {
    let expected_duration = (total_samples * 1000) / sample_rate;
    start_time + Duration::from_millis(expected_duration)
}
