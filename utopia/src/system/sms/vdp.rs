use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use tracing::{trace, warn};

#[derive(Copy, Clone, Debug, Eq, PartialEq, FromPrimitive)]
enum Command {
    ReadVram = 0,
    WriteVram = 1,
    WriteRegister = 2,
    WriteCram = 3,
}

pub struct Vdp {
    command: Command,
    address: u16,
    mode_select: u8,
    write_buffer: Option<u8>,
}

impl Vdp {
    pub fn new() -> Self {
        Self {
            command: Command::ReadVram,
            address: 0,
            mode_select: 0,
            write_buffer: None,
        }
    }

    pub fn write_command(&mut self, value: u8) {
        if let Some(low) = self.write_buffer.take() {
            self.command = Command::from_u8(value >> 6).unwrap();
            self.address = u16::from_le_bytes([low, value & 0x3f]);

            if self.command == Command::WriteRegister {
                self.write_register((self.address >> 8) as u8 & 15, self.address as u8);
            }
        } else {
            self.write_buffer = Some(value)
        }
    }

    pub fn write_data(&mut self, value: u8) {
        trace!("VDP Data: {:02X}", value);
    }

    fn write_register(&mut self, reg: u8, value: u8) {
        warn!("VDP Register Write: {:02X} <= {:02X}", reg, value);

        match reg {
            0x00 => {
                self.mode_select =
                    (self.mode_select & 0b0101) | (value & 0b0010) | ((value & 0b0100) << 1);

                warn!("Mode Select: {:04b}", self.mode_select);
            }
            0x01 => {
                self.mode_select = (self.mode_select & 0b1010)
                    | ((value & 0b1_0000) >> 4)
                    | ((value & 0b1000) >> 1);

                warn!("Mode Select: {:04b}", self.mode_select);
            }
            _ => (),
        }
    }
}
