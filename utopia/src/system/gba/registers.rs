use crate::util::memory::{Masked, Reader, Writer};
use tracing::{trace, warn};

pub struct Registers {
    interrupt_enable: u16,
    interrupt_flags: u16,
    wait_state_control: u16,
    interrupt_master_enable: u16,
    unknown: u8,
    post_boot_flag: u8,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            interrupt_enable: 0,
            interrupt_flags: 0,
            wait_state_control: 0,
            interrupt_master_enable: 0,
            post_boot_flag: 0,
            unknown: 0,
        }
    }
}

impl Reader for Registers {
    type Value = u16;

    fn read_register(&self, address: u32) -> u16 {
        match address {
            0x0200 => self.interrupt_enable,
            0x0202 => self.interrupt_flags,
            0x0204 => self.wait_state_control,
            0x0208 => self.interrupt_master_enable,
            0x0300 => self.post_boot_flag as u16,
            0x0410 => self.unknown as u16,
            _ => todo!("Unmapped I/O Register Read: {:08X}", address),
        }
    }
}

impl Writer for Registers {
    type SideEffect = ();

    fn write_register(&mut self, address: u32, value: Masked<u16>) {
        match address {
            0x0200 => {
                self.interrupt_enable = value.apply(self.interrupt_enable);
                trace!("Interrupt Enable: {:04X}", self.interrupt_enable);
            }
            0x0202 => {
                self.interrupt_flags = value.apply(self.interrupt_flags);
                trace!("Interrupt Flags: {:04X}", self.interrupt_flags);
            }
            0x0204 => {
                self.wait_state_control = value.apply(self.wait_state_control);
                trace!("Wait State Control: {:04X}", self.wait_state_control);
            }
            0x0208 => {
                self.interrupt_master_enable = value.apply(self.interrupt_master_enable);
                trace!(
                    "Interrupt Master Enable: {:04X}",
                    self.interrupt_master_enable
                );
            }
            0x0300 => {
                if (value.get() & 0x8000) != 0 {
                    todo!("Power down");
                }

                self.post_boot_flag = value.apply(self.post_boot_flag as u16) as u8;
                trace!("Post-Boot Flag: {:02X}", self.post_boot_flag);
            }
            0x0410 => {
                self.unknown = value.apply(self.unknown as u16) as u8;
                trace!("Unknown Register: {:02X}", self.unknown);
            }
            _ => warn!(
                "Unmapped I/O Register Write: {:08X} <= {:02X}",
                address,
                value.get()
            ),
        }
    }
}
