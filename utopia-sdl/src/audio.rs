use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::Sdl;
use std::error::Error;
use utopia::AudioQueue;

pub struct Audio {
    device: AudioDevice<Callback>,
}

struct Callback {
    queue: AudioQueue,
}

impl Audio {
    pub fn new(sdl: &Sdl, sample_rate: i32) -> Result<Self, Box<dyn Error>> {
        let audio = sdl.audio()?;

        let spec = AudioSpecDesired {
            freq: Some(sample_rate),
            channels: Some(2),
            samples: None,
        };

        let device = audio.open_playback(None, &spec, |_spec| Callback {
            queue: AudioQueue::new(),
        })?;

        Ok(Self { device })
    }

    pub fn append_queue(&mut self, other: &mut AudioQueue) {
        self.device.lock().queue.append(other);
    }

    pub fn resume(&mut self) {
        self.device.resume();
    }
}

impl AudioCallback for Callback {
    type Channel = i16;

    fn callback(&mut self, output: &mut [i16]) {
        for (sample_out, sample_in) in output.chunks_exact_mut(2).zip(self.queue.iter()) {
            sample_out[0] = sample_in.0;
            sample_out[1] = sample_in.1;
        }
    }
}
