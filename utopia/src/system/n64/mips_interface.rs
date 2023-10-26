use super::interrupt::{RcpIntType, RcpInterrupt};
use super::memory::{Masked, Reader, Writer};
use bitfield_struct::bitfield;
use tracing::{trace, warn};

const MI_VERSION: u32 = 0x0202_0102;

pub struct MipsInterface {
    mode: Mode,
    rcp_int: RcpInterrupt,
}

impl MipsInterface {
    pub fn new(rcp_int: RcpInterrupt) -> Self {
        Self {
            mode: Mode::new(),
            rcp_int,
        }
    }
}

impl Reader for MipsInterface {
    fn read_u32(&self, address: u32) -> u32 {
        match (address >> 2) & 3 {
            0 => self.mode.into(),
            1 => MI_VERSION,
            2 => self.rcp_int.status() as u32,
            3 => self.rcp_int.mask() as u32,
            _ => unreachable!(),
        }
    }
}

impl Writer for MipsInterface {
    type SideEffect = ();

    fn write_u32(&mut self, address: u32, value: Masked<u32>) {
        match (address >> 2) & 3 {
            0 => {
                let input = value.apply(self.mode.init_length());

                self.mode.set_init_length(input & 0x7f);

                if (input & 0x0080) != 0 {
                    self.mode.set_init_mode(false);
                }

                if (input & 0x0100) != 0 {
                    self.mode.set_init_mode(true);
                }

                if (input & 0x0200) != 0 {
                    self.mode.set_ebus_test_mode(false);
                }

                if (input & 0x0400) != 0 {
                    self.mode.set_ebus_test_mode(true);
                }

                if (input & 0x0800) != 0 {
                    self.rcp_int.clear(RcpIntType::DP);
                }

                if (input & 0x1000) != 0 {
                    self.mode.set_rdram_register_mode(false);
                }

                if (input & 0x2000) != 0 {
                    self.mode.set_rdram_register_mode(true);
                }

                trace!("MI_MODE: {:?}", self.mode);
            }
            1 => {
                warn!("MI_VERSION cannot be written");
            }
            2 => {
                warn!("MI_INTERRUPT cannot be written");
            }
            3 => {
                let input = value.get();
                let mut mask = self.rcp_int.mask();

                for bit_out in 0..6 {
                    let bit_in = bit_out << 1;

                    if (input & (1 << bit_in)) != 0 {
                        mask &= !(1 << bit_out);
                    }

                    if (input & (1 << (bit_in + 1))) != 0 {
                        mask |= 1 << bit_out;
                    }
                }

                self.rcp_int.set_mask(mask);
            }
            _ => unreachable!(),
        }
    }
}

#[bitfield(u32)]
struct Mode {
    #[bits(7)]
    init_length: u32,
    init_mode: bool,
    ebus_test_mode: bool,
    rdram_register_mode: bool,
    #[bits(22)]
    __: u32,
}
