use crate::util::facade::{DataReader, DataWriter};
use tracing::debug;

pub struct VideoInterface {
    cycles: u64,
    line: u32,
    field: bool,
    v_current: u32,
    v_sync: u32,
    h_sync: u32,
}

impl VideoInterface {
    pub fn new() -> Self {
        Self {
            cycles: 0,
            line: 0,
            field: false,
            v_current: 0,
            v_sync: 0x3ff,
            h_sync: 0x7ff,
        }
    }

    pub fn step(&mut self, cycles: u64) {
        self.cycles += cycles;

        // Runs approximately half as fast as the CPU
        let cycles_per_line = (self.h_sync as u64) << 1;

        if self.cycles >= cycles_per_line {
            self.cycles -= cycles_per_line;
            self.line += 1;

            // VCurrent & VSync are given in half lines, with the low bit
            // representing the field in interlace mode
            self.v_current = (self.line << 1) | (self.field as u32);

            // (TODO: Interlace mode)
            if self.v_current >= self.v_sync {
                self.line = 0;
                self.field = !self.field;
                self.v_current = self.field as u32;
                debug!("Field: {}", self.field as u32);
            }

            debug!("Line: {}", self.line);
        }
    }
}

impl DataReader for VideoInterface {
    type Address = u32;
    type Value = u32;

    fn read(&self, address: u32) -> u32 {
        match address {
            0x10 => self.v_current,
            0x18 => self.v_sync,
            0x1c => self.h_sync,
            _ => unimplemented!("Video Interface Read: {:08X}", address),
        }
    }
}

impl DataWriter for VideoInterface {
    fn write(&mut self, address: u32, value: u32) {
        match address {
            0x10 => {
                self.v_current = value & 0x3ff;
                debug!("VI_V_CURRENT: {}", self.v_current);
                // TODO: Clear interrupt
            }
            0x18 => {
                self.v_sync = value & 0x3ff;
                debug!("VI_V_SYNC: {}", self.v_sync);
            }
            0x1c => {
                self.h_sync = value & 0x7ff;
                debug!("VI_H_SYNC: {}", self.h_sync);
                // TODO: Leap (PAL only)
            }
            _ => unimplemented!("Video Interface Write: {:08X} <= {:08X}", address, value),
        }
    }
}
