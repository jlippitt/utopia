use tracing::debug;

const RELEASE_RATE: u8 = 31;

#[derive(Debug)]
enum Gain {
    Direct(i32),
    LinearDecrease(u8),
    ExpDecrease(u8),
    LinearIncrease(u8),
    BentIncrease(u8),
}

struct Adsr {
    enabled: bool,
    attack_rate: u8,
    decay_rate: u8,
    sustain_rate: u8,
    sustain_level: i32,
}

pub struct Envelope {
    id: u32,
    adsr: Adsr,
    gain: Gain,
}

impl Envelope {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            adsr: Adsr {
                enabled: false,
                attack_rate: 0,
                decay_rate: 0,
                sustain_rate: 0,
                sustain_level: 0,
            },
            gain: Gain::Direct(0),
        }
    }

    pub fn set_adsr_low(&mut self, value: u8) {
        self.adsr.enabled = (value & 0x80) != 0;
        self.adsr.attack_rate = ((value & 15) << 1) + 1;
        self.adsr.decay_rate = (((value >> 4) & 7) << 1) + 16;
        debug!("Voice {} ADSR Enabled: {}", self.id, self.adsr.enabled);
        debug!(
            "Voice {} ADSR Attack Rate: {}",
            self.id, self.adsr.attack_rate
        );
        debug!(
            "Voice {} ADSR Decay Rate: {}",
            self.id, self.adsr.decay_rate
        );
    }

    pub fn set_adsr_high(&mut self, value: u8) {
        self.adsr.sustain_rate = value & 31;
        self.adsr.sustain_level = ((((value >> 5) & 7) as i32) + 1) << 8;
        debug!(
            "Voice {} ADSR Sustain Rate: {}",
            self.id, self.adsr.sustain_rate
        );
        debug!(
            "Voice {} ADSR Sustain Level: {}",
            self.id, self.adsr.sustain_level
        );
    }

    pub fn set_gain(&mut self, value: u8) {
        self.gain = if (value & 0x80) != 0 {
            Gain::Direct((value as i32 & 127) << 4)
        } else {
            let rate = value & 31;

            match (value >> 5) & 3 {
                0 => Gain::LinearDecrease(rate),
                1 => Gain::ExpDecrease(rate),
                2 => Gain::LinearIncrease(rate),
                3 => Gain::BentIncrease(rate),
                _ => unreachable!(),
            }
        };

        debug!("Voice {} Gain: {:?}", self.id, self.gain);
    }
}
