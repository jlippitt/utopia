use tracing::debug;

pub struct Dma {
    src_address: u16,
    dst_address: u16,
}

impl Dma {
    pub fn new() -> Self {
        Self {
            src_address: 0,
            dst_address: 0,
        }
    }

    pub fn write(&mut self, address: u8, value: u8) {
        match address {
            0x51 => {
                self.src_address = (self.src_address & 0xff) | ((value as u16) << 8);
                debug!("DMA Source Address: {:04X}", self.src_address);
            }
            0x52 => {
                self.src_address = (self.src_address & 0xff00) | value as u16;
                debug!("DMA Source Address: {:04X}", self.src_address);
            }
            0x53 => {
                self.dst_address = (self.dst_address & 0xff) | ((value as u16) << 8);
                debug!("DMA Destination Address: {:04X}", self.dst_address);
            }
            0x54 => {
                self.dst_address = (self.dst_address & 0xff00) | value as u16;
                debug!("DMA Destination Address: {:04X}", self.dst_address);
            }
            _ => unimplemented!("DMA Write: {:04X} <= {:02X}", address, value),
        }
    }
}
