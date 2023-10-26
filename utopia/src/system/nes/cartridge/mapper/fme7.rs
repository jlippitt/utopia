use super::{
    Interrupt, InterruptType, Mapper, Mappings, NameTable, CHR_PAGE_SIZE, MIRROR_HORIZONTAL,
    MIRROR_VERTICAL,
};
use tracing::trace;

const PRG_BANK_SIZE: usize = 8192;

pub struct Fme7 {
    command: u8,
    irq_enable: bool,
    irq_counter_enable: bool,
    irq_counter: u16,
    prg_rom_size: usize,
    interrupt: Interrupt,
}

impl Fme7 {
    pub fn new(prg_rom_size: usize, interrupt: Interrupt) -> Self {
        Self {
            command: 0,
            irq_enable: false,
            irq_counter_enable: false,
            irq_counter: 0,
            prg_rom_size,
            interrupt,
        }
    }
}

impl Mapper for Fme7 {
    fn init_mappings(&mut self, mappings: &mut Mappings) {
        mappings.map_prg_rom(14, 2, self.prg_rom_size - PRG_BANK_SIZE);
        mappings.map_registers(8, 4);
        trace!("FME7 PRG Read Mappings: {:?}", mappings.prg_read);
        trace!("FME7 PRG Write Mappings: {:?}", mappings.prg_write);
    }

    fn write_register(&mut self, mappings: &mut Mappings, address: u16, value: u8) {
        if address < 0xa000 {
            self.command = value & 0x0f;
            return;
        }

        match self.command {
            0..=7 => {
                mappings.map_chr(self.command as usize, 1, CHR_PAGE_SIZE * value as usize);
                trace!("FME7 CHR Mappings: {:?}", mappings.chr);
            }
            8 => {
                if (value & 0x40) == 0 {
                    mappings.map_prg_rom(6, 2, PRG_BANK_SIZE * (value as usize & 0x3f));
                } else if (value & 0x80) != 0 {
                    mappings.map_prg_ram(6, 2, PRG_BANK_SIZE * (value as usize & 0x3f));
                } else {
                    mappings.unmap_prg(6, 2);
                }
                trace!("FME7 PRG Read Mappings: {:?}", mappings.prg_read);
                trace!("FME7 PRG Write Mappings: {:?}", mappings.prg_write);
            }
            9..=11 => {
                mappings.map_prg_rom(
                    8 + 2 * (self.command as usize - 9),
                    2,
                    PRG_BANK_SIZE * (value as usize & 0x3f),
                );
                trace!("FME7 PRG Read Mappings: {:?}", mappings.prg_read);
            }
            12 => {
                mappings.name = match value & 3 {
                    0 => MIRROR_VERTICAL,
                    1 => MIRROR_HORIZONTAL,
                    2 => [NameTable::LOW; 4],
                    3 => [NameTable::HIGH; 4],
                    _ => unreachable!(),
                };
                trace!("FME7 Name Mappings: {:?}", mappings.name);
            }
            13 => {
                self.irq_enable = (value & 0x01) != 0;
                self.irq_counter_enable = (value & 0x80) != 0;
                trace!("FME7 IRQ Enable: {}", self.irq_enable);
                trace!("FME7 IRQ Counter Enable: {}", self.irq_counter_enable);
                self.interrupt.clear(InterruptType::MapperIrq);
            }
            14 => {
                self.irq_counter = (self.irq_counter & 0xff00) | value as u16;
                trace!("FME7 IRQ Counter: {}", self.irq_counter);
            }
            15 => {
                self.irq_counter = (self.irq_counter & 0xff) | ((value as u16) << 8);
                trace!("FME7 IRQ Counter: {}", self.irq_counter);
            }
            _ => unreachable!(),
        }
    }

    fn on_cpu_cycle(&mut self, _mappings: &mut Mappings) {
        if self.irq_counter_enable {
            self.irq_counter = self.irq_counter.wrapping_sub(1);

            if self.irq_counter == 0xffff && self.irq_enable {
                self.interrupt.raise(InterruptType::MapperIrq);
            }
        }
    }
}
