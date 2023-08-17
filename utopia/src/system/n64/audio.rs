use crate::util::facade::{DataReader, DataWriter};
use tracing::debug;

pub struct AudioInterface {
    dram_addr: u32,
    length: u32,
    dma_enabled: bool,
    dacrate: u32,
    bitrate: u32,
}

impl AudioInterface {
    pub fn new() -> Self {
        Self {
            dram_addr: 0,
            length: 0,
            dma_enabled: false,
            dacrate: 0,
            bitrate: 0,
        }
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
                self.dram_addr = value & 0x00ff_ffff;
                debug!("AI_DRAM_ADDR: {:08X}", self.dram_addr);
            }
            0x04 => {
                self.length = value & 0x0003_fff8;
                debug!("AI_LENGTH: {:08X}", self.length);
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
