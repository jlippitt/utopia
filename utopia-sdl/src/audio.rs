use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::Sdl;
use std::error::Error;

pub struct Audio {
    device: AudioDevice<Callback>,
}

struct Callback;

impl Audio {
    pub fn new(sdl: &Sdl, sample_rate: i32) -> Result<Self, Box<dyn Error>> {
        let audio = sdl.audio()?;

        let spec = AudioSpecDesired {
            freq: Some(sample_rate),
            channels: Some(2),
            samples: None,
        };

        let device = audio.open_playback(None, &spec, |_spec| Callback)?;

        Ok(Self { device })
    }

    pub fn resume(&mut self) {
        self.device.resume();
    }
}

impl AudioCallback for Callback {
    type Channel = i16;

    fn callback(&mut self, output: &mut [i16]) {
        let half_len = output.len() / 4;

        for (index, sample) in output.chunks_exact_mut(2).enumerate() {
            let (left, right) = if index >= half_len {
                (i16::MAX, i16::MIN)
            } else {
                (i16::MIN, i16::MAX)
            };

            sample[0] = left;
            sample[1] = right;
        }
    }
}
