use crate::util::facade::{DataReader, DataWriter};

struct DmaChannel {
    _id: u32,
    control: u16,
}

impl DmaChannel {
    fn new(id: u32) -> Self {
        Self {
            _id: id,
            control: 0,
        }
    }

    fn control(&self) -> u16 {
        self.control
    }

    fn set_source_low(&mut self, _value: u16) {
        // TODO
    }

    fn set_source_high(&mut self, _value: u16) {
        // TODO
    }

    fn set_destination_low(&mut self, _value: u16) {
        // TODO
    }

    fn set_destination_high(&mut self, _value: u16) {
        // TODO
    }

    fn set_word_count(&mut self, _value: u16) {
        // TODO
    }

    fn set_control(&mut self, value: u16) {
        if (value & 0x8000) != 0 {
            todo!("DMA transfers");
        }

        self.control = value;

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

impl DataReader for Dma {
    type Address = u32;
    type Value = u16;

    fn read(&self, address: u32) -> u16 {
        match address {
            0xba => self.channels[0].control(),
            0xc6 => self.channels[1].control(),
            0xd2 => self.channels[2].control(),
            0xde => self.channels[3].control(),
            _ => panic!("Unmapped DMA Read: {:02X}", address),
        }
    }
}

impl DataWriter for Dma {
    fn write(&mut self, address: u32, value: u16) {
        match address {
            0xb0 => self.channels[0].set_source_low(value),
            0xb2 => self.channels[0].set_source_high(value),
            0xb4 => self.channels[0].set_destination_low(value),
            0xb6 => self.channels[0].set_destination_high(value),
            0xb8 => self.channels[0].set_word_count(value),
            0xba => self.channels[0].set_control(value),
            0xbc => self.channels[1].set_source_low(value),
            0xbe => self.channels[1].set_source_high(value),
            0xc0 => self.channels[1].set_destination_low(value),
            0xc2 => self.channels[1].set_destination_high(value),
            0xc4 => self.channels[1].set_word_count(value),
            0xc6 => self.channels[1].set_control(value),
            0xc8 => self.channels[2].set_source_low(value),
            0xca => self.channels[2].set_source_high(value),
            0xcc => self.channels[2].set_destination_low(value),
            0xce => self.channels[2].set_destination_high(value),
            0xd0 => self.channels[2].set_word_count(value),
            0xd2 => self.channels[2].set_control(value),
            0xd4 => self.channels[3].set_source_low(value),
            0xd6 => self.channels[3].set_source_high(value),
            0xd8 => self.channels[3].set_destination_low(value),
            0xda => self.channels[3].set_destination_high(value),
            0xdc => self.channels[3].set_word_count(value),
            0xde => self.channels[3].set_control(value),
            _ => panic!("Unmapped DMA Write: {:02X} <= {:04X}", address, value),
        }
    }
}
