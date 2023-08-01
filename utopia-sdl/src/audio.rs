use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::Sdl;
use std::error::Error;

pub struct Audio {
    device: AudioDevice<Callback>,
}

struct Callback;

impl Audio {
    pub fn new(sdl: &Sdl, sample_rate: u32) -> Result<Self, Box<dyn Error>> {
        let audio = sdl.audio()?;

        let spec = AudioSpecDesired {
            freq: Some(sample_rate.try_into()?),
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
        let half_len = output.len() / 2;

        for (index, sample) in output.iter_mut().enumerate() {
            *sample = if index >= half_len {
                i16::MAX
            } else {
                i16::MIN
            };
        }
    }
}
