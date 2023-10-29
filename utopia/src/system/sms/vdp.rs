use super::interrupt::{Interrupt, InterruptType};
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
    line_cycles: u64,
    line_counter: u16,
    vblank_line: u16,
    command: Command,
    address: u16,
    mode_select: u8,
    write_buffer: Option<u8>,
    interrupt: Interrupt,
}

impl Vdp {
    const TOTAL_LINES: u16 = 262;
    const CYCLES_PER_LINE: u64 = 1368;

    pub fn new(interrupt: Interrupt) -> Self {
        Self {
            line_cycles: 0,
            line_counter: 0,
            vblank_line: 193,
            command: Command::ReadVram,
            address: 0,
            mode_select: 0,
            write_buffer: None,
            interrupt,
        }
    }

    pub fn v_counter(&self) -> u16 {
        self.line_counter
    }

    pub fn h_counter(&self) -> u16 {
        (self.line_cycles >> 2) as u16
    }

    pub fn write_control(&mut self, value: u8) {
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

    pub fn step(&mut self, cycles: u64) {
        self.line_cycles += cycles;

        while self.line_cycles >= Self::CYCLES_PER_LINE {
            self.line_cycles -= Self::CYCLES_PER_LINE;
            self.line_counter += 1;

            if self.line_counter == Self::TOTAL_LINES {
                self.line_counter = 0;
            } else if self.line_counter == self.vblank_line {
                self.interrupt.raise(InterruptType::FrameIrq);
            }
        }
    }

    fn write_register(&mut self, reg: u8, value: u8) {
        warn!("VDP Register Write: {:02X} <= {:02X}", reg, value);

        match reg {
            0x00 => {
                self.update_mode(
                    (self.mode_select & 0b0101) | (value & 0b0010) | ((value & 0b0100) << 1),
                );
            }
            0x01 => {
                self.update_mode(
                    (self.mode_select & 0b1010)
                        | ((value & 0b1_0000) >> 4)
                        | ((value & 0b1000) >> 1),
                );
            }
            _ => (),
        }
    }

    fn update_mode(&mut self, mode_select: u8) {
        self.mode_select = mode_select;
        trace!("Mode Select: {:04b}", self.mode_select);

        if (mode_select & 0b1000) == 0 {
            unimplemented!("TMS9918 Modes");
        }

        self.vblank_line = match mode_select & 0b111 {
            0b1001 | 0b1101 => panic!("Invalid video mode: {:04b}", mode_select),
            0b1110 => 241,
            0b1011 => 225,
            _ => 193,
        };

        trace!("VBlank Line: {}", self.vblank_line);
    }
}
