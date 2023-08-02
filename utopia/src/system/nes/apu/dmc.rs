use super::super::DmaRequest;
use super::component::Timer;
use tracing::debug;

#[rustfmt::skip]
const PERIODS: [u32; 16] = [
    427, 379, 339, 319, 285, 253, 225, 213,
    189, 159, 141, 127, 105,  83,  71,  53,
];

pub struct Dmc {
    output: u8,
    timer: Timer,
    silence_flag: bool,
    shift: u8,
    bits_remaining: u8,
    sample_buffer: Option<u8>,
    current_address: u16,
    bytes_remaining: u16,
    sample_address: u16,
    sample_length: u16,
    loop_flag: bool,
}

impl Dmc {
    pub fn new() -> Self {
        Self {
            output: 0,
            timer: Timer::new(PERIODS[0], 0),
            silence_flag: false,
            shift: 0,
            bits_remaining: 0,
            sample_buffer: None,
            current_address: 0,
            bytes_remaining: 0,
            sample_address: 0,
            sample_length: 0,
            loop_flag: false,
        }
    }

    pub fn enabled(&self) -> bool {
        self.bytes_remaining > 0
    }

    pub fn output(&self) -> u8 {
        self.output
    }

    pub fn sample_address(&self) -> u16 {
        self.current_address
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address & 3 {
            0 => {
                // TODO: IRQ
                self.loop_flag = (value & 0x40) != 0;
                self.timer.set_period(PERIODS[(value & 0x0f) as usize]);
            }
            1 => self.output = value & 0x7f,
            2 => self.sample_address = 0xc000 + ((value as u16) << 6),
            3 => self.sample_length = (value as u16) << 4,
            _ => unimplemented!(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled {
            if self.bytes_remaining == 0 {
                self.restart();
            }
        } else {
            self.bytes_remaining = 0;
        }
    }

    pub fn write_sample(&mut self, value: u8) {
        self.sample_buffer = Some(value);
        self.current_address = 0x8000 | (self.current_address.wrapping_add(1) & 0x7fff);

        // This means sample length is one greater than what is written to register $4013
        if self.bytes_remaining == 0 {
            if self.loop_flag {
                self.restart();
            } else {
                // TODO: IRQ
            }
        } else {
            self.bytes_remaining -= 1;
        }
    }

    pub fn step(&mut self, dma_request: &mut DmaRequest) {
        if !self.timer.step() {
            return;
        }

        if !self.silence_flag {
            if (self.shift & 1) != 0 {
                if self.output <= 125 {
                    self.output += 2;
                }
            } else if self.output >= 2 {
                self.output -= 2;
            }
        }

        self.shift >>= 1;

        if self.bits_remaining == 0 {
            // One less than total bits in sample, due to way this is implemented
            self.bits_remaining = 7;

            if let Some(sample) = self.sample_buffer {
                self.shift = sample;
                self.sample_buffer = None;
                self.silence_flag = false;
            } else {
                self.silence_flag = true;
            }

            if self.bytes_remaining != 0 {
                dma_request.insert(DmaRequest::DMC);
            }
        } else {
            self.bits_remaining -= 1;
        }
    }

    fn restart(&mut self) {
        self.current_address = self.sample_address;
        self.bytes_remaining = self.sample_length;

        debug!(
            "DMC Restart: Address = {:04X}, Length = {}",
            self.current_address,
            self.bytes_remaining + 1
        );
    }
}
