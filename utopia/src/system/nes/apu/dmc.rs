#[rustfmt::skip]
const RATES: [u32; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214,
    190, 160, 142, 128, 106,  84,  72,  54,
];

pub struct Dmc {
    rate: u32,
    output: u8,
    sample_address: u16,
    sample_length: u16,
    bytes_remaining: u16,
    loop_flag: bool,
}

impl Dmc {
    pub fn new() -> Self {
        Self {
            rate: RATES[0],
            output: 0,
            sample_address: 0,
            sample_length: 0,
            bytes_remaining: 0,
            loop_flag: false,
        }
    }

    pub fn enabled(&self) -> bool {
        self.bytes_remaining > 0
    }

    pub fn sample(&self) -> u8 {
        self.output
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address & 3 {
            0 => {
                // TODO: IRQ
                self.loop_flag = (value & 0x40) != 0;
                self.rate = RATES[(value & 0x0f) as usize];
            }
            1 => self.output = value & 0x7f,
            2 => self.sample_address = 0xc000 + ((value as u16) << 6),
            3 => self.sample_length = ((value as u16) << 4) + 1,
            _ => unimplemented!(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled {
            // TODO: Restart sample
        } else {
            self.bytes_remaining = 0;
        }
    }

    pub fn step(&mut self) {
        // TODO
    }
}
