use super::{
    Interrupt, InterruptType, Mapper, Mappings, NameTable, CHR_PAGE_SIZE, MIRROR_HORIZONTAL,
    MIRROR_VERTICAL,
};
use pulse::Pulse;
use saw::Saw;
use tracing::debug;

mod pulse;
mod saw;

const PRG_BANK_SIZE: usize = 8192;
const IRQ_DIVIDER: i32 = 341;

// Pulse volume is approximately equal to internal NES pulse channels
const VOLUME_MULTIPLIER: f32 = 95.88 / (8128.0 / 30.0 + 100.0) / 30.0;

pub struct Vrc6 {
    irq_mode: bool,
    irq_divider: i32,
    irq_counter: u8,
    irq_latch: u8,
    irq_enable: bool,
    irq_enable_after_ack: bool,
    audio_halted: bool,
    pulse1: Pulse,
    pulse2: Pulse,
    saw: Saw,
    prg_rom_size: usize,
    interrupt: Interrupt,
}

impl Vrc6 {
    pub fn new(prg_rom_size: usize, interrupt: Interrupt) -> Self {
        Self {
            irq_mode: false,
            irq_divider: IRQ_DIVIDER,
            irq_counter: 0,
            irq_latch: 0,
            irq_enable: false,
            irq_enable_after_ack: false,
            audio_halted: false,
            pulse1: Pulse::new(),
            pulse2: Pulse::new(),
            saw: Saw::new(),
            prg_rom_size,
            interrupt,
        }
    }
}

impl Mapper for Vrc6 {
    fn init_mappings(&mut self, mappings: &mut Mappings) {
        mappings.map_prg_rom(14, 2, self.prg_rom_size - PRG_BANK_SIZE);
        mappings.map_registers(8, 8);
        debug!("VRC6 PRG Read Mappings: {:?}", mappings.prg_read);
        debug!("VRC6 PRG Write Mappings: {:?}", mappings.prg_write);
    }

    fn write_register(&mut self, mappings: &mut Mappings, address: u16, value: u8) {
        match address & 0xf003 {
            0x8000..=0x8003 => {
                mappings.map_prg_rom(8, 4, PRG_BANK_SIZE * 2 * (value as usize & 0x0f));
                debug!("VRC6 PRG Read Mappings: {:?}", mappings.prg_read);
            }
            0x9000 => self.pulse1.set_control(value),
            0x9001 => self.pulse1.set_freq_period_low(value),
            0x9002 => self.pulse1.set_freq_period_high(value),
            0x9003 => {
                self.audio_halted = (value & 0x01) != 0;

                let freq_shift = if (value & 0x04) != 0 {
                    8
                } else if (value & 0x02) != 0 {
                    4
                } else {
                    0
                };

                self.pulse1.set_freq_shift(freq_shift);
                self.pulse2.set_freq_shift(freq_shift);
                self.saw.set_freq_shift(freq_shift);

                debug!("VRC6 Audio Halted: {}", self.audio_halted);
                debug!("VRC6 Frequency Shift: {}", freq_shift);
            }
            0xa000 => self.pulse2.set_control(value),
            0xa001 => self.pulse2.set_freq_period_low(value),
            0xa002 => self.pulse2.set_freq_period_high(value),
            0xb000 => self.saw.set_control(value),
            0xb001 => self.saw.set_freq_period_low(value),
            0xb002 => self.saw.set_freq_period_high(value),
            0xb003 => {
                if (value & 0x33) != 0x20 {
                    unimplemented!("Mode: {:02X}", value);
                }

                mappings.name = match (value >> 2) & 3 {
                    0 => MIRROR_VERTICAL,
                    1 => MIRROR_HORIZONTAL,
                    2 => [NameTable::LOW; 4],
                    3 => [NameTable::HIGH; 4],
                    _ => unreachable!(),
                };

                if (value & 0x80) != 0 {
                    mappings.map_prg_ram(6, 2, 0);
                } else {
                    mappings.unmap_prg(6, 2);
                }

                debug!("VRC6 PRG Read Mappings: {:?}", mappings.prg_read);
                debug!("VRC6 PRG Write Mappings: {:?}", mappings.prg_write);
                debug!("VRC6 Name Mappings: {:?}", mappings.name);
            }
            0xc000..=0xc003 => {
                mappings.map_prg_rom(12, 2, PRG_BANK_SIZE * (value as usize & 0x1f));
                debug!("VRC6 PRG Read Mappings: {:?}", mappings.prg_read);
            }
            0xd000..=0xd003 => {
                mappings.map_chr(address as usize - 0xd000, 1, CHR_PAGE_SIZE * value as usize);
                debug!("VRC6 CHR Mappings: {:?}", mappings.chr);
            }
            0xe000..=0xe003 => {
                mappings.map_chr(
                    4 + address as usize - 0xe000,
                    1,
                    CHR_PAGE_SIZE * value as usize,
                );
                debug!("VRC6 CHR Mappings: {:?}", mappings.chr);
            }
            0xf000 => {
                self.irq_latch = value;
                debug!("VRC6 IRQ Latch: {}", self.irq_latch);
            }
            0xf001 => {
                self.irq_mode = (value & 0x04) != 0;
                self.irq_enable = (value & 0x02) != 0;
                self.irq_enable_after_ack = (value & 0x01) != 0;

                debug!("VRC6 IRQ Mode: {}", self.irq_mode);
                debug!("VRC6 IRQ Enable: {}", self.irq_enable);
                debug!("VRC6 IRQ Enable After ACK: {}", self.irq_enable_after_ack);

                self.interrupt.clear(InterruptType::MapperIrq);

                if self.irq_enable {
                    self.irq_divider = IRQ_DIVIDER;
                    self.irq_counter = self.irq_latch;
                    debug!("VRC6 IRQ Counter: {}", self.irq_counter);
                }
            }
            0xf002 => {
                self.interrupt.clear(InterruptType::MapperIrq);
                self.irq_enable = self.irq_enable_after_ack;
                debug!("VRC6 IRQ Enable: {}", self.irq_enable);
            }
            _ => (),
        }
    }

    fn on_cpu_cycle(&mut self, _mappings: &mut Mappings) {
        if !self.audio_halted {
            self.pulse1.step();
            self.pulse2.step();
            self.saw.step();
        }

        if !self.irq_enable {
            return;
        }

        if !self.irq_mode {
            self.irq_divider -= 3;

            if self.irq_divider > 0 {
                return;
            }

            self.irq_divider += IRQ_DIVIDER;
        };

        if self.irq_counter != 0xff {
            self.irq_counter = self.irq_counter.wrapping_add(1);
            debug!("VRC6 IRQ Counter: {}", self.irq_counter);
            return;
        }

        self.irq_counter = self.irq_latch;
        debug!("VRC6 IRQ Counter: {}", self.irq_counter);

        self.interrupt.raise(InterruptType::MapperIrq);
    }

    fn audio_output(&self) -> f32 {
        let output = self.pulse1.output() + self.pulse2.output() + self.saw.output();

        output as f32 * VOLUME_MULTIPLIER
    }
}
