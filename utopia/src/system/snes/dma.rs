use super::clock::SLOW_CYCLES;
use tracing::{debug, warn};

type Mode = ([u8; 4], usize);

const MODES: [Mode; 8] = [
    ([0, 0, 0, 0], 1),
    ([0, 1, 0, 1], 2),
    ([0, 0, 0, 0], 2),
    ([0, 0, 1, 1], 4),
    ([0, 1, 2, 3], 4),
    ([0, 1, 0, 1], 4),
    ([0, 0, 0, 0], 2),
    ([0, 0, 1, 1], 4),
];

struct Control {
    mode: &'static Mode,
    fixed: bool,
    decrement: bool,
    hdma_indirect: bool,
    reverse: bool,
    raw: u8,
}

struct DmaChannel {
    ctrl: Control,
    destination: u8,
    source: u32,
    indirect: u32,
    table: u16,
    counter: u8,
    unknown: u8,
}

impl DmaChannel {
    fn new() -> Self {
        Self {
            ctrl: Control {
                mode: &MODES[7],
                fixed: true,
                decrement: true,
                hdma_indirect: true,
                reverse: true,
                raw: 0xff,
            },
            destination: 0xff,
            source: 0x00ff_ffff,
            indirect: 0x00ff_ffff,
            table: 0xffff,
            counter: 0xff,
            unknown: 0xff,
        }
    }
}

pub struct Dma {
    dma_enabled: u8,
    channels: [DmaChannel; 8],
}

impl Dma {
    pub fn new() -> Self {
        Self {
            dma_enabled: 0,
            channels: [
                DmaChannel::new(),
                DmaChannel::new(),
                DmaChannel::new(),
                DmaChannel::new(),
                DmaChannel::new(),
                DmaChannel::new(),
                DmaChannel::new(),
                DmaChannel::new(),
            ],
        }
    }

    pub fn requested(&self) -> bool {
        self.dma_enabled != 0
    }

    pub fn set_dma_enabled(&mut self, value: u8) {
        self.dma_enabled = value;
        debug!("DMA Enabled: {:08b}", self.dma_enabled);
    }

    pub fn read(&self, address: u8, prev_value: u8) -> u8 {
        if address >= 0x80 {
            warn!("Unmapped DMA read: {:02X}", address);
            return prev_value;
        }

        let id = (address >> 4) as usize;
        let channel = &self.channels[id];

        match address & 0x0f {
            0x00 => channel.ctrl.raw,
            0x01 => channel.destination,
            0x02 => channel.source as u8,
            0x03 => (channel.source >> 8) as u8,
            0x04 => (channel.source >> 16) as u8,
            0x05 => channel.indirect as u8,
            0x06 => (channel.indirect >> 8) as u8,
            0x07 => (channel.indirect >> 16) as u8,
            0x08 => channel.table as u8,
            0x09 => (channel.table >> 8) as u8,
            0x0a => channel.counter,
            0x0b | 0x0f => channel.unknown,
            _ => {
                warn!("Unmapped DMA read: {:02X}", address);
                prev_value
            }
        }
    }

    pub fn write(&mut self, address: u8, value: u8) {
        if address >= 0x80 {
            warn!("Unmapped DMA write: {:02X} <= {:02X}", address, value);
            return;
        }

        let id = (address >> 4) as usize;
        let channel = &mut self.channels[id];

        match address & 0x0f {
            0x00 => {
                let mode_index = (value & 0x07) as usize;
                channel.ctrl.mode = &MODES[mode_index];
                channel.ctrl.fixed = (value & 0x08) != 0;
                channel.ctrl.decrement = (value & 0x10) != 0;
                channel.ctrl.hdma_indirect = (value & 0x40) != 0;
                channel.ctrl.reverse = (value & 0x80) != 0;
                channel.ctrl.raw = value;
                debug!("DMA{} Mode: {}", id, mode_index);
                debug!("DMA{} Fixed: {}", id, channel.ctrl.fixed);
                debug!("DMA{} Decrement: {}", id, channel.ctrl.decrement);
                debug!("DMA{} HDMA Indirect: {}", id, channel.ctrl.hdma_indirect);
                debug!("DMA{} Reverse: {}", id, channel.ctrl.reverse);
            }
            0x01 => {
                channel.destination = value;
                debug!("DMA{} Destination: {:02X}", id, channel.destination);
            }
            0x02 => {
                channel.source = (channel.source & 0xffff_ff00) | (value as u32);
                debug!("DMA{} Source: {:06X}", id, channel.source);
            }
            0x03 => {
                channel.source = (channel.source & 0xffff_00ff) | ((value as u32) << 8);
                debug!("DMA{} Source: {:06X}", id, channel.source);
            }
            0x04 => {
                channel.source = (channel.source & 0xff00_ffff) | ((value as u32) << 16);
                debug!("DMA{} Source: {:06X}", id, channel.source);
            }
            0x05 => {
                channel.indirect = (channel.indirect & 0xffff_ff00) | (value as u32);
                debug!("DMA{} Indirect: {:06X}", id, channel.indirect);
            }
            0x06 => {
                channel.indirect = (channel.indirect & 0xffff_00ff) | ((value as u32) << 8);
                debug!("DMA{} Indirect: {:06X}", id, channel.indirect);
            }
            0x07 => {
                channel.indirect = (channel.indirect & 0xff00_ffff) | ((value as u32) << 16);
                debug!("DMA{} Indirect: {:06X}", id, channel.indirect);
            }
            0x08 => {
                channel.table = (channel.table & 0xff00) | (value as u16);
                debug!("DMA{} Table: {:06X}", id, channel.table);
            }
            0x09 => {
                channel.table = (channel.table & 0x00ff) | ((value as u16) << 8);
                debug!("DMA{} Table: {:06X}", id, channel.table);
            }
            0x0a => {
                channel.counter = value;
                debug!("DMA{} Counter: {:02X}", id, channel.counter);
            }
            0x0b | 0x0f => {
                channel.unknown = value;
                debug!("DMA{} Unknown: {:02X}", id, channel.unknown);
            }
            _ => warn!("Unmapped DMA write: {:02X} <= {:02X}", address, value),
        }
    }
}

impl super::Hardware {
    pub(super) fn transfer_dma(&mut self) {
        // TODO: More accurate timing - but this will do for now
        self.step(SLOW_CYCLES);

        debug!("DMA Transfer Begin");

        for id in 0..8 {
            let mask = 1 << id;

            if (self.dma.dma_enabled & mask) == 0 {
                continue;
            }

            self.step(SLOW_CYCLES);
            self.transfer_dma_for(id);
        }

        self.dma.dma_enabled = 0;

        debug!("DMA Transfer End");
    }

    fn transfer_dma_for(&mut self, id: usize) {
        let mut byte_index = 0;

        loop {
            self.step(SLOW_CYCLES);

            let (port, address, reverse) = {
                let channel = &self.dma.channels[id];
                let port = channel.destination + channel.ctrl.mode.0[byte_index];
                byte_index = byte_index.wrapping_add(1) & 3;
                (port, channel.source, channel.ctrl.reverse)
            };

            if reverse {
                let value = self.read_bus_b(port);
                debug!(
                    "DMA{} Read: {:02X} => {:02X} => {:06X}",
                    id, port, value, address
                );
                self.write_bus_a(address, value);
            } else {
                let value = self.read_bus_a(address);
                debug!(
                    "DMA{} Write: {:02X} <= {:02X} <= {:06X}",
                    id, port, value, address
                );
                self.write_bus_b(port, value);
            }

            let channel = &mut self.dma.channels[id];

            if !channel.ctrl.fixed {
                if channel.ctrl.decrement {
                    channel.source =
                        (channel.source & 0xffff_0000) | (channel.source.wrapping_sub(1) & 0xffff)
                } else {
                    channel.source =
                        (channel.source & 0xffff_0000) | (channel.source.wrapping_add(1) & 0xffff)
                }
            }

            let bytes_remaining = channel.indirect.wrapping_sub(1) & 0xffff;

            channel.indirect = (channel.indirect & 0xffff_0000) | bytes_remaining;

            if bytes_remaining == 0 {
                break;
            }
        }
    }
}
