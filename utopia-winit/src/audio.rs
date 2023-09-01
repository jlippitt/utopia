use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    BufferSize, OutputCallbackInfo, PlayStreamError, Sample, SampleRate, Stream, StreamConfig,
};
use std::error::Error;
use std::sync::{Arc, Mutex};
use tracing::warn;
use utopia::AudioQueue;

#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

pub struct AudioController {
    stream: Stream,
    send_queue: Arc<Mutex<AudioQueue>>,
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

        let send_queue = Arc::new(Mutex::new(AudioQueue::new()));
        let receive_queue = send_queue.clone();

        let stream = device.build_output_stream(
            &config,
            move |output: &mut [f32], _: &OutputCallbackInfo| {
                let mut input = receive_queue.lock().unwrap();
                let mut prev_sample = (Sample::EQUILIBRIUM, Sample::EQUILIBRIUM);

                for sample_out in output.chunks_exact_mut(2) {
                    if let Some(sample_in) = input.pop_front() {
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
            send_queue,
            total_samples: 0,
            sample_rate,
            start_time: Instant::now(),
            sync_time: calculate_sync_time(start_time, 0, sample_rate),
        })
    }

    pub fn sync_time(&self) -> Instant {
        self.sync_time
    }

    pub fn resync(&mut self) {
        self.send_queue.lock().unwrap().clear();
        self.total_samples = 0;
        self.start_time = Instant::now();
        self.sync_time = calculate_sync_time(self.start_time, self.total_samples, self.sample_rate);
    }

    pub fn resume(&mut self) -> Result<(), PlayStreamError> {
        self.stream.play()?;
        self.resync();
        Ok(())
    }

    pub fn queue_samples(&mut self, source_queue: &mut AudioQueue) {
        self.total_samples += source_queue.len() as u64;

        self.send_queue.lock().unwrap().append(source_queue);
        source_queue.clear();

        self.sync_time = calculate_sync_time(self.start_time, self.total_samples, self.sample_rate);
    }
}

fn calculate_sync_time(start_time: Instant, total_samples: u64, sample_rate: u64) -> Instant {
    let expected_duration = (total_samples * 1000) / sample_rate;
    start_time + Duration::from_millis(expected_duration)
}
