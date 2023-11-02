use crate::util::memory::{Masked, Reader, Writer};
use tracing::{trace, warn};

struct DmaChannel {
    source: u32,
    destination: u32,
    word_count: u16,
    control: u16,
    id: u32,
}

impl DmaChannel {
    fn new(id: u32) -> Self {
        Self {
            source: 0,
            destination: 0,
            word_count: 0,
            control: 0,
            id,
        }
    }

    fn control(&self) -> u32 {
        (self.control as u32) << 16
    }

    fn set_source(&mut self, value: Masked<u32>) {
        self.source = value.apply(self.source);
        trace!("DMA{} Source: {:08X}", self.id, self.source);
    }

    fn set_destination(&mut self, value: Masked<u32>) {
        self.destination = value.apply(self.destination);
        trace!("DMA{} Destination: {:08X}", self.id, self.destination);
    }

    fn set_control(&mut self, value: Masked<u32>) {
        self.word_count = value.apply(self.word_count as u32) as u16;
        trace!("DMA{} Word Count: {}", self.id, self.word_count);

        self.control = (value.apply((self.control as u32) << 16) >> 16) as u16;

        if (self.control & 0x8000) != 0 {
            todo!("DMA transfers");
        }

        // TODO
    }
}

pub struct Dma {
    channels: [DmaChannel; 4],
}

impl Dma {
    pub fn new() -> Self {
        Self {
            channels: [
                DmaChannel::new(0),
                DmaChannel::new(1),
                DmaChannel::new(2),
                DmaChannel::new(3),
            ],
        }
    }
}

impl Reader for Dma {
    type Value = u32;

    fn read_register(&self, address: u32) -> u32 {
        match address & 0xff {
            0xb8 => self.channels[0].control(),
            0xc4 => self.channels[1].control(),
            0xd0 => self.channels[2].control(),
            0xdc => self.channels[3].control(),
            address => panic!("Unmapped DMA Read: {:02X}", address),
        }
    }
}

impl Writer for Dma {
    type SideEffect = ();

    fn write_register(&mut self, address: u32, value: Masked<u32>) {
        match address & 0xff {
            0xb0 => self.channels[0].set_source(value),
            0xb4 => self.channels[0].set_destination(value),
            0xb8 => self.channels[0].set_control(value),
            0xbc => self.channels[1].set_source(value),
            0xc0 => self.channels[1].set_destination(value),
            0xc4 => self.channels[1].set_control(value),
            0xc8 => self.channels[2].set_source(value),
            0xcc => self.channels[2].set_destination(value),
            0xd0 => self.channels[2].set_control(value),
            0xd4 => self.channels[3].set_source(value),
            0xd8 => self.channels[3].set_destination(value),
            0xdc => self.channels[3].set_control(value),
            address => warn!("Unmapped DMA Write: {:02X} <= {:04X}", address, value.get()),
        }
    }
}
