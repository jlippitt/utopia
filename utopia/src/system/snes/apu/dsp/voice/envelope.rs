use tracing::{debug, warn};

const RELEASE_RATE: usize = 31;
const MAX_LEVEL: i32 = 0x07ff;

#[rustfmt::skip]
pub const RATE: [Option<u32>; 32] = [
    None,       Some(2048), Some(1536), Some(1280),
    Some(1024), Some(768),  Some(640),  Some(512),
    Some(384),  Some(320),  Some(256),  Some(192),
    Some(160),  Some(128),  Some(96),   Some(80),
    Some(64),   Some(48),   Some(40),   Some(32),
    Some(24),   Some(20),   Some(16),   Some(12),
    Some(10),   Some(8),    Some(6),    Some(5),
    Some(4),    Some(3),    Some(2),    Some(1),
];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Mode {
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug)]
enum Gain {
    Direct(i32),
    LinearDecrease(usize),
    ExpDecrease(usize),
    LinearIncrease(usize),
    BentIncrease(usize),
}

struct Adsr {
    enabled: bool,
    attack_rate: usize,
    decay_rate: usize,
    sustain_rate: usize,
    sustain_level: i32,
}

pub struct Envelope {
    id: u32,
    mode: Mode,
    adsr: Adsr,
    gain: Gain,
    counter: Option<u32>,
    divider: Option<u32>,
    level: i32,
}

impl Envelope {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            mode: Mode::Release,
            adsr: Adsr {
                enabled: false,
                attack_rate: 0,
                decay_rate: 0,
                sustain_rate: 0,
                sustain_level: 0,
            },
            gain: Gain::Direct(0),
            counter: None,
            divider: None,
            level: 0,
        }
    }

    pub fn level(&self) -> i32 {
        self.level
    }

    pub fn set_adsr_low(&mut self, value: u8) {
        self.adsr.enabled = (value & 0x80) != 0;
        self.adsr.attack_rate = ((value as usize & 15) << 1) + 1;
        self.adsr.decay_rate = (((value >> 4) as usize & 7) << 1) + 16;
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
        self.adsr.sustain_rate = (value & 31) as usize;
        self.adsr.sustain_level = ((((value >> 5) & 7) as i32) + 1) << 8;
        debug!(
            "Voice {} ADSR Sustain Rate: {}",
            self.id, self.adsr.sustain_rate
        );
        debug!(
            "Voice {} ADSR Sustain Level: {}",
            self.id, self.adsr.sustain_level
        );

        if self.adsr.sustain_rate == 0 {
            warn!("Voice {} Sustain Rate is 0", self.id);
        }
    }

    pub fn set_gain(&mut self, value: u8) {
        self.gain = if (value & 0x80) != 0 {
            let rate = value as usize & 31;

            if rate == 0 {
                warn!("Voice {} Gain Rate is 0", self.id);
            }

            match (value >> 5) & 3 {
                0 => Gain::LinearDecrease(rate),
                1 => Gain::ExpDecrease(rate),
                2 => Gain::LinearIncrease(rate),
                3 => Gain::BentIncrease(rate),
                _ => unreachable!(),
            }
        } else {
            Gain::Direct((value as i32 & 127) << 4)
        };

        debug!("Voice {} Gain: {:?}", self.id, self.gain);
    }

    pub fn restart(&mut self) {
        self.mode = Mode::Attack;

        (self.divider, self.level) = if self.adsr.enabled {
            (RATE[self.adsr.attack_rate], 0)
        } else {
            match self.gain {
                Gain::Direct(level) => (None, level),
                Gain::LinearDecrease(rate) => (RATE[rate], MAX_LEVEL),
                Gain::ExpDecrease(rate) => (RATE[rate], MAX_LEVEL),
                Gain::LinearIncrease(rate) => (RATE[rate], 0),
                Gain::BentIncrease(rate) => (RATE[rate], 0),
            }
        };

        self.counter = self.divider;

        debug!("Voice {} Mode: {:?}", self.id, self.mode);
    }

    pub fn release(&mut self) {
        self.mode = Mode::Release;
        self.divider = RATE[RELEASE_RATE];
        self.counter = self.divider;
        debug!("Voice {} Mode: {:?}", self.id, self.mode);
    }

    pub fn mute(&mut self) {
        self.mode = Mode::Release;
        self.divider = None;
        self.counter = None;
        self.level = 0;
        debug!("Voice {} Mode: {:?}", self.id, self.mode);
    }

    pub fn step(&mut self) {
        self.counter = self.counter.map(|counter| counter - 1);

        if !self.counter.is_some_and(|counter| counter == 0) {
            return;
        }

        match self.mode {
            Mode::Attack => {
                if self.adsr.enabled {
                    self.level += if self.adsr.attack_rate == 31 {
                        1024
                    } else {
                        32
                    };
                } else {
                    self.apply_gain();
                }

                if self.level >= 0x07e0 {
                    self.mode = Mode::Decay;
                    self.divider = RATE[self.adsr.decay_rate];
                }
            }
            Mode::Decay => {
                if self.adsr.enabled {
                    self.level = exp_decrease(self.level);
                } else {
                    self.apply_gain();
                }

                if self.level <= self.adsr.sustain_level {
                    self.mode = Mode::Sustain;
                    self.divider = RATE[self.adsr.sustain_rate];
                }
            }
            Mode::Sustain => {
                if self.adsr.enabled {
                    self.level = exp_decrease(self.level);
                } else {
                    self.apply_gain();
                }
            }
            Mode::Release => {
                self.level = (self.level - 8).max(0);
            }
        }

        self.counter = self.divider;
        self.level = self.level.clamp(0, MAX_LEVEL);
    }

    fn apply_gain(&mut self) {
        self.level = match self.gain {
            Gain::Direct(level) => level,
            Gain::LinearDecrease(_) => self.level - 32,
            Gain::ExpDecrease(_) => exp_decrease(self.level),
            Gain::LinearIncrease(_) => self.level + 32,
            Gain::BentIncrease(_) => {
                if self.level < 0x0600 {
                    self.level + 32
                } else {
                    self.level + 8
                }
            }
        };
    }
}

fn exp_decrease(level: i32) -> i32 {
    level - (((level - 1) >> 8) + 1)
}
