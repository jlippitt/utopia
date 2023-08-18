use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::Sdl;
use std::error::Error;
use std::thread;
use std::time::{Duration, Instant};
use utopia::AudioQueue;

pub struct Audio {
    device: AudioDevice<Callback>,
    sample_rate: u64,
    total_samples: u64,
    start_time: Instant,
}

struct Callback {
    queue: AudioQueue,
}

impl Audio {
    pub fn new(sdl: &Sdl, sample_rate: u64) -> Result<Self, Box<dyn Error>> {
        let audio = sdl.audio()?;

        let spec = AudioSpecDesired {
            freq: Some(sample_rate.try_into()?),
            channels: Some(2),
            samples: None,
        };

        let device = audio.open_playback(None, &spec, |_spec| Callback {
            queue: AudioQueue::new(),
        })?;

        device.resume();

        let total_samples = 0;
        let start_time = Instant::now();

        Ok(Self {
            device,
            sample_rate,
            total_samples,
            start_time,
        })
    }

    pub fn pause(&mut self) {
        self.device.pause();
        self.device.lock().queue.clear();
        self.total_samples = 0;
    }

    pub fn resume(&mut self) {
        self.device.resume();
        self.start_time = Instant::now();
    }

    pub fn sync(&mut self, audio_queue: &mut AudioQueue) {
        self.total_samples += audio_queue.len() as u64;

        self.device.lock().queue.append(audio_queue);

        let expected_duration = (self.total_samples * 1000) / self.sample_rate;
        let expected_time = self.start_time + Duration::from_millis(expected_duration);
        let actual_time = Instant::now();

        let duration = if actual_time < expected_time {
            expected_time - actual_time
        } else {
            Duration::ZERO
        };

        thread::sleep(duration);
    }
}

impl AudioCallback for Callback {
    type Channel = f32;

    fn callback(&mut self, output: &mut [f32]) {
        let mut prev_sample = (0.0, 0.0);

        for sample_out in output.chunks_exact_mut(2) {
            if let Some(sample_in) = self.queue.pop_front() {
                sample_out[0] = sample_in.0;
                sample_out[1] = sample_in.1;
                prev_sample = sample_in;
            } else {
                sample_out[0] = prev_sample.0;
                sample_out[1] = prev_sample.1;
            }
        }
    }
}
