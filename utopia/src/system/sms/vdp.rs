use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use tracing::warn;

#[derive(Copy, Clone, Debug, Eq, PartialEq, FromPrimitive)]
enum Command {
    VramRead = 0,
    VramWrite = 1,
    RegisterWrite = 2,
    CramWrite = 3,
}

pub struct Vdp {
    write_buffer: Option<u8>,
}

impl Vdp {
    pub fn new() -> Self {
        Self { write_buffer: None }
    }

    pub fn write_command(&mut self, value: u8) {
        if let Some(low) = self.write_buffer.take() {
            let command = Command::from_u8(value >> 6).unwrap();
            let address = u16::from_le_bytes([low, value & 0x3f]);
            warn!("VDP Command: {:?} {:04X}", command, address);
        } else {
            self.write_buffer = Some(value)
        }
    }

    pub fn write_data(&mut self, value: u8) {
        warn!("VDP Data: {:02X}", value);
    }
}
