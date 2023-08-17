use crate::util::facade::{DataReader, DataWriter};
use tracing::debug;

const DAC_FREQUENCY: i64 = 48681812;

pub struct AudioInterface {
    dram_addr: u32,
    length: u32,
    dma_enabled: bool,
    dacrate: u32,
    bitrate: u32,
    dma_count: u32,
    samples: [i64; 2],
    counter: i64,
}

impl AudioInterface {
    pub fn new() -> Self {
        Self {
            dram_addr: 0,
            length: 0,
            dma_enabled: false,
            dacrate: 0,
            bitrate: 0,
            dma_count: 0,
            samples: [0; 2],
            counter: 0,
        }
    }

    pub fn step(&mut self, cycles: u64) {
        self.counter -= cycles as i64;

        if self.counter < 0 && self.dma_count > 0 {
            self.dma_count -= 1;
            debug!("AI DMA Count: {}", self.dma_count);

            if self.dma_count > 0 {
                self.samples[0] = self.samples[1];
                self.start_dma();
            }
        }
    }

    fn start_dma(&mut self) {
        let frequency = DAC_FREQUENCY / (self.dacrate as i64 + 1);
        self.counter = (125000000 * self.samples[0]) / frequency;
        debug!("AI Counter: {}", self.counter);
    }
}

impl DataReader for AudioInterface {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: u32) -> u32 {
        match address & 0x0f {
            0x0c => {
                // AI_STATUS
                let mut value = 0x0110_0000;

                if self.dma_enabled {
                    value |= 0x0200_0000;
                }

                if self.dma_count >= 1 {
                    value |= 0x4000_0000;

                    if self.dma_count >= 2 {
                        value |= 0x8000_0000;
                    }
                }

                value
            }
            _ => unimplemented!("Audio Interface Read: {:08X}", address),
        }
    }
}

impl DataWriter for AudioInterface {
    fn write(&mut self, address: u32, value: u32) {
        match address {
            0x00 => {
                self.dram_addr = value & 0x00ff_fff8;
                debug!("AI_DRAM_ADDR: {:08X}", self.dram_addr);
            }
            0x04 => {
                self.length = value & 0x0003_fff8;
                debug!("AI_LENGTH: {}", self.length);

                if self.dma_count < 2 {
                    self.samples[self.dma_count as usize] = self.length as i64 / 4;

                    self.dma_count += 1;
                    debug!("AI DMA Count: {}", self.dma_count);

                    if self.dma_count < 2 {
                        self.start_dma();
                    }
                }
            }
            0x08 => {
                self.dma_enabled = (value & 1) != 0;
                debug!("AI DMA Enabled: {}", self.dma_enabled);
            }
            0x0c => {
                // AI_STATUS
                // TODO: Acknowledge AI interrupt
            }
            0x10 => {
                self.dacrate = value & 0x3fff;
                debug!("AI_DACRATE: {}", self.dacrate);
            }
            0x14 => {
                self.bitrate = value & 15;
                debug!("AI_BITRATE: {}", self.bitrate);
            }
            _ => panic!("Audio Interface Write: {:08X} <= {:08X}", address, value),
        }
    }
}
