use super::interrupt::{RcpIntType, RcpInterrupt};
use crate::util::memory::{Masked, Reader, Writer};
use tracing::trace;

const DAC_FREQUENCY: i64 = 48681812;

pub struct AudioInterface {
    dram_addr: u32,
    length: u32,
    dma_enabled: bool,
    dac_rate: u32,
    bit_rate: u32,
    counter: i64,
    dma_count: u32,
    samples: [i64; 2],
    rcp_int: RcpInterrupt,
}

impl AudioInterface {
    pub fn new(rcp_int: RcpInterrupt) -> Self {
        Self {
            dram_addr: 0,
            length: 0,
            dma_enabled: false,
            dac_rate: 0,
            bit_rate: 0,
            counter: 0,
            dma_count: 0,
            samples: [0; 2],
            rcp_int,
        }
    }

    pub fn step(&mut self, cycles: u64) {
        self.counter -= cycles as i64;

        if self.counter < 0 && self.dma_count > 0 {
            self.dma_count -= 1;
            trace!("AI DMA Count: {}", self.dma_count);

            if self.dma_count > 0 {
                self.samples[0] = self.samples[1];
                self.start_dma();
            }
        }
    }

    fn start_dma(&mut self) {
        let frequency = DAC_FREQUENCY / (self.dac_rate as i64 + 1);
        self.counter = (125000000 * self.samples[0]) / frequency;
        trace!("AI Counter: {}", self.counter);
        self.rcp_int.raise(RcpIntType::AI);
    }
}

impl Reader for AudioInterface {
    type Value = u32;

    fn read_register(&self, address: u32) -> u32 {
        match address {
            0x04 => {
                // AI_LENGTH
                // TODO: This is currently a bit of a hack
                if self.dma_count > 0 {
                    self.length
                } else {
                    0
                }
            }
            0x0c => {
                // AI_STATUS
                let mut value = 0x0110_0000;

                if self.dma_enabled {
                    value |= 0x0200_0000;
                }

                if self.dma_count >= 1 {
                    value |= 0x4000_0000;

                    if self.dma_count >= 2 {
                        value |= 0x8000_0001;
                    }
                }

                value
            }
            _ => unimplemented!("Audio Interface Register Read: {:08X}", address),
        }
    }
}

impl Writer for AudioInterface {
    type SideEffect = ();

    fn write_register(&mut self, address: u32, value: Masked<u32>) {
        match address {
            0x00 => {
                self.dram_addr = value.apply(self.dram_addr) & 0x00ff_fff8;
                trace!("AI_DRAM_ADDR: {:08X}", self.dram_addr);
            }
            0x04 => {
                self.length = value.apply(self.length) & 0x0003_fff8;
                trace!("AI_LENGTH: {:08X}", self.length);

                if self.dma_count < 2 {
                    self.samples[self.dma_count as usize] = self.length as i64 / 4;

                    self.dma_count += 1;
                    trace!("AI DMA Count: {}", self.dma_count);

                    if self.dma_count < 2 {
                        self.start_dma();
                    }
                }
            }
            0x08 => {
                self.dma_enabled = (value.get() & 0x01) != 0;
                trace!("AI_CONTROL: DMA Enabled = {}", self.dma_enabled);
            }
            0x0c => {
                // AI_STATUS
                self.rcp_int.clear(RcpIntType::AI);
            }
            0x10 => {
                self.dac_rate = value.apply(self.dac_rate) & 0x3fff;
                trace!("AI_DACRATE: {}", self.dac_rate);
            }
            0x14 => {
                self.bit_rate = value.apply(self.bit_rate) & 0x0f;
                trace!("AI_BITRATE: {}", self.bit_rate);
            }
            _ => {
                unimplemented!("Audio Interface Register Write: {:08X}", address);
            }
        }
    }
}
