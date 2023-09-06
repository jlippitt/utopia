use super::{
    Interrupt, InterruptType, Mapper, Mappings, NameTable, CHR_PAGE_SIZE, MIRROR_HORIZONTAL,
    MIRROR_VERTICAL,
};
use tracing::debug;

const PRG_BANK_SIZE: usize = 8192;
const IRQ_DIVIDER: i32 = 341;

pub struct Vrc6 {
    irq_mode: bool,
    irq_divider: i32,
    irq_counter: u8,
    irq_latch: u8,
    irq_enable: bool,
    irq_enable_after_ack: bool,
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
}
