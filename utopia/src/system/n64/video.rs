use crate::util::facade::{DataReader, DataWriter};
use tracing::debug;

pub struct VideoInterface {
    ready: bool,
    cycles: u64,
    line: u32,
    field: bool,
    v_intr: u32,
    v_current: u32,
    v_sync: u32,
    h_sync: u32,
}

impl VideoInterface {
    pub fn new() -> Self {
        Self {
            ready: false,
            cycles: 0,
            line: 0,
            field: false,
            v_intr: 0x3ff,
            v_current: 0,
            v_sync: 0x3ff,
            h_sync: 0x7ff,
        }
    }

    pub fn ready(&self) -> bool {
        self.ready
    }

    pub fn start_frame(&mut self) {
        self.ready = false;
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
                self.ready = true;
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
            0x0c => self.v_intr,
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
            0x00 => {
                // VI_CTRL: Ignore for now
            }
            0x04 => {
                // VI_ORIGIN: Ignore for now
            }
            0x08 => {
                // VI_WIDTH: Ignore for now
            }
            0x0c => {
                self.v_intr = value & 0x3ff;
                debug!("VI_V_INTR: {}", self.v_intr);
            }
            0x10 => {
                self.v_current = value & 0x3ff;
                debug!("VI_V_CURRENT: {}", self.v_current);
                // TODO: Clear interrupt
            }
            0x14 => {
                // VI_BURST: Ignore for now
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
            0x20 => {
                // VI_H_SYNC_LEAP: Ignore for now
            }
            0x24 => {
                // VI_H_VIDEO: Ignore for now
            }
            0x28 => {
                // VI_V_VIDEO: Ignore for now
            }
            0x2c => {
                // VI_V_BURST: Ignore for now
            }
            0x30 => {
                // VI_X_SCALE: Ignore for now
            }
            0x34 => {
                // VI_Y_SCALE: Ignore for now
            }
            _ => unimplemented!("Video Interface Write: {:08X} <= {:08X}", address, value),
        }
    }
}
