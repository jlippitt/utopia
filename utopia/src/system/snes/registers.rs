use crate::core::wdc65c816::INT_NMI;
use tracing::debug;

pub struct Registers {
    nmi_occurred: bool,
    nmi_active: bool,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            nmi_occurred: false,
            nmi_active: false,
        }
    }

    pub fn nmi_raised(&mut self) -> bool {
        self.nmi_occurred && self.nmi_active
    }

    pub fn set_nmi_occurred(&mut self, nmi_occurred: bool) {
        self.nmi_occurred = nmi_occurred;
        debug!("NMI Occurred: {}", self.nmi_occurred);
    }
}

impl super::Hardware {
    pub(super) fn read_register(&mut self, address: u8, prev_value: u8) -> u8 {
        match address {
            0x10 => {
                let mut value = (prev_value & 0x70) | 0x02;

                if self.regs.nmi_occurred {
                    self.regs.set_nmi_occurred(false);
                    self.interrupt &= !INT_NMI;
                    value |= 0x80;
                }

                value
            }
            _ => todo!("Register read {:02X}", address),
        }
    }

    pub(super) fn write_register(&mut self, address: u8, value: u8) {
        match address {
            0x00 => {
                let nmi_active = (value & 0x80) != 0;

                if !nmi_active {
                    self.interrupt &= !INT_NMI;
                } else if nmi_active && self.regs.nmi_occurred && !self.regs.nmi_active {
                    self.interrupt |= INT_NMI;
                }

                self.regs.nmi_active = nmi_active;
                debug!("NMI Active: {}", self.regs.nmi_active);
            }
            _ => (),
        }
    }
}
