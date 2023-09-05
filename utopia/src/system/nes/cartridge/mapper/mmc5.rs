use super::{Interrupt, InterruptType, Mapper, Mappings, CHR_PAGE_SIZE};
use crate::util::MirrorVec;
use bitflags::bitflags;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use tracing::debug;

const PRG_BANK_SIZE: usize = 8192;
const ERAM_SIZE: usize = 1024;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, FromPrimitive)]
enum NameTable {
    Low,
    High,
    Eram,
    Fill,
}

struct Control {
    sprite_mode: bool,
    scanline_count: u8,
    no_read_count: u64,
    prev_address: u16,
    same_address_count: u8,
    same_line_reads: u8,
}

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    struct EramFlags: u8 {
        const NAMETABLE = 0x01;
        const EXTENDED_ATTR = 0x02;
        const READ = 0x04;
        const WRITE = 0x08;
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    struct ScanlineIrqStatus: u8 {
        const IN_FRAME = 0x40;
        const PENDING = 0x80;
    }
}

pub struct Mmc5 {
    prg_mode: u8,
    prg_bank: [u8; 5],
    chr_mode: u8,
    chr_bank: [u8; 12],
    name_bank: [NameTable; 4],
    ctrl: Control,
    eram: MirrorVec<u8>,
    eram_flags: EramFlags,
    fill_mode_name: u8,
    fill_mode_attr: u8,
    scanline_irq_compare: u8,
    scanline_irq_enable: bool,
    scanline_irq_status: ScanlineIrqStatus,
    interrupt: Interrupt,
}

impl Mmc5 {
    pub fn new(interrupt: Interrupt) -> Self {
        Self {
            prg_mode: 3,
            prg_bank: [0, 0, 0, 0, 0xff],
            chr_mode: 0,
            chr_bank: [0; 12],
            name_bank: [NameTable::Low; 4],
            ctrl: Control {
                sprite_mode: false,
                scanline_count: 0,
                no_read_count: 0,
                prev_address: 0xffff,
                same_address_count: 1,
                same_line_reads: 1,
            },
            eram: MirrorVec::new(ERAM_SIZE),
            eram_flags: EramFlags::empty(),
            fill_mode_name: 0,
            fill_mode_attr: 0,
            scanline_irq_compare: 0,
            scanline_irq_enable: false,
            scanline_irq_status: ScanlineIrqStatus::empty(),
            interrupt,
        }
    }

    fn update_prg_mappings(&mut self, mappings: &mut Mappings) {
        match self.prg_mode {
            0 => {
                map_prg(mappings, 6, 2, self.prg_bank[0] & !0x80);
                map_prg(mappings, 8, 8, self.prg_bank[4] | 0x80 & 0xfc);
            }
            1 => {
                map_prg(mappings, 6, 2, self.prg_bank[0] & !0x80);
                map_prg(mappings, 8, 4, self.prg_bank[2] & 0xfe);
                map_prg(mappings, 12, 4, self.prg_bank[4] | 0x80 & 0xfe);
            }
            2 => {
                map_prg(mappings, 6, 2, self.prg_bank[0] & !0x80);
                map_prg(mappings, 8, 4, self.prg_bank[2] & 0xfe);
                map_prg(mappings, 12, 2, self.prg_bank[3]);
                map_prg(mappings, 14, 2, self.prg_bank[4] | 0x80);
            }
            3 => {
                map_prg(mappings, 6, 2, self.prg_bank[0] & !0x80);
                map_prg(mappings, 8, 2, self.prg_bank[1]);
                map_prg(mappings, 10, 2, self.prg_bank[2]);
                map_prg(mappings, 12, 2, self.prg_bank[3]);
                map_prg(mappings, 14, 2, self.prg_bank[4] | 0x80);
            }
            _ => unreachable!(),
        }

        debug!("MMC5 PRG Read Mappings: {:?}", mappings.prg_read);
        debug!("MMC5 PRG Write Mappings: {:?}", mappings.prg_write);
    }

    fn update_chr_mappings(&mut self, mappings: &mut Mappings) {
        if !self.ctrl.sprite_mode
            || !self
                .scanline_irq_status
                .contains(ScanlineIrqStatus::IN_FRAME)
            || (64..80).contains(&self.ctrl.same_line_reads)
        {
            debug!("MMC5 Lower CHR Banks Selected");

            match self.chr_mode {
                0 => {
                    mappings.map_chr(0, 8, CHR_PAGE_SIZE * self.chr_bank[7] as usize);
                }
                1 => {
                    mappings.map_chr(0, 4, CHR_PAGE_SIZE * self.chr_bank[3] as usize);
                    mappings.map_chr(4, 4, CHR_PAGE_SIZE * self.chr_bank[7] as usize);
                }
                2 => {
                    mappings.map_chr(0, 2, CHR_PAGE_SIZE * self.chr_bank[1] as usize);
                    mappings.map_chr(2, 2, CHR_PAGE_SIZE * self.chr_bank[3] as usize);
                    mappings.map_chr(4, 2, CHR_PAGE_SIZE * self.chr_bank[5] as usize);
                    mappings.map_chr(6, 2, CHR_PAGE_SIZE * self.chr_bank[7] as usize);
                }
                3 => {
                    mappings.map_chr(0, 1, CHR_PAGE_SIZE * self.chr_bank[0] as usize);
                    mappings.map_chr(1, 1, CHR_PAGE_SIZE * self.chr_bank[1] as usize);
                    mappings.map_chr(2, 1, CHR_PAGE_SIZE * self.chr_bank[2] as usize);
                    mappings.map_chr(3, 1, CHR_PAGE_SIZE * self.chr_bank[3] as usize);
                    mappings.map_chr(4, 1, CHR_PAGE_SIZE * self.chr_bank[4] as usize);
                    mappings.map_chr(5, 1, CHR_PAGE_SIZE * self.chr_bank[5] as usize);
                    mappings.map_chr(6, 1, CHR_PAGE_SIZE * self.chr_bank[6] as usize);
                    mappings.map_chr(7, 1, CHR_PAGE_SIZE * self.chr_bank[7] as usize);
                }
                _ => unreachable!(),
            }
        } else {
            debug!("MMC5 Upper CHR Banks Selected");

            match self.chr_mode {
                0 | 1 => {
                    mappings.map_chr(0, 8, CHR_PAGE_SIZE * self.chr_bank[11] as usize);
                }
                2 => {
                    mappings.map_chr(0, 2, CHR_PAGE_SIZE * self.chr_bank[9] as usize);
                    mappings.map_chr(2, 2, CHR_PAGE_SIZE * self.chr_bank[11] as usize);
                    mappings.map_chr(4, 2, CHR_PAGE_SIZE * self.chr_bank[9] as usize);
                    mappings.map_chr(6, 2, CHR_PAGE_SIZE * self.chr_bank[11] as usize);
                }
                3 => {
                    mappings.map_chr(0, 1, CHR_PAGE_SIZE * self.chr_bank[8] as usize);
                    mappings.map_chr(1, 1, CHR_PAGE_SIZE * self.chr_bank[9] as usize);
                    mappings.map_chr(2, 1, CHR_PAGE_SIZE * self.chr_bank[10] as usize);
                    mappings.map_chr(3, 1, CHR_PAGE_SIZE * self.chr_bank[11] as usize);
                    mappings.map_chr(4, 1, CHR_PAGE_SIZE * self.chr_bank[8] as usize);
                    mappings.map_chr(5, 1, CHR_PAGE_SIZE * self.chr_bank[9] as usize);
                    mappings.map_chr(6, 1, CHR_PAGE_SIZE * self.chr_bank[10] as usize);
                    mappings.map_chr(7, 1, CHR_PAGE_SIZE * self.chr_bank[11] as usize);
                }
                _ => unreachable!(),
            }
        }

        debug!("MMC5 CHR Mappings: {:?}", mappings.chr);
    }
}

impl Mapper for Mmc5 {
    fn init_mappings(&mut self, mappings: &mut Mappings) {
        mappings.map_registers(2, 2);
        mappings.map_registers_with_read(5, 1);
        self.update_prg_mappings(mappings);
    }

    fn read_register(&mut self, _mappings: &mut Mappings, address: u16, _prev_value: u8) -> u8 {
        match address {
            0x5204 => {
                let value = self.scanline_irq_status.bits();
                self.scanline_irq_status.remove(ScanlineIrqStatus::PENDING);
                self.interrupt.clear(InterruptType::MapperIrq);
                value
            }
            0x5c00..=0x5fff => {
                if self.eram_flags.contains(EramFlags::READ) {
                    self.eram[address as usize & 0x03ff]
                } else {
                    0
                }
            }
            _ => unimplemented!("MMC5 Register Read {:04X}", address),
        }
    }

    fn write_register(&mut self, mappings: &mut Mappings, address: u16, value: u8) {
        if address <= 0x3fff {
            match address & 7 {
                0 => {
                    self.ctrl.sprite_mode = (value & 0x20) != 0;
                    debug!("MMC5 Sprite Mode: {}", self.ctrl.sprite_mode);
                    self.update_chr_mappings(mappings);
                }
                1 => {
                    if (value & 0x18) == 0 {
                        self.scanline_irq_status.remove(ScanlineIrqStatus::IN_FRAME);
                        self.update_chr_mappings(mappings);
                    }
                }
                _ => (),
            }

            return;
        }

        match address {
            0x5000..=0x5015 => (), // TODO: MMC5 Audio
            0x5100 => {
                self.prg_mode = value & 0x03;
                debug!("MMC5 PRG Mode: {}", self.prg_mode);
                self.update_prg_mappings(mappings);
            }
            0x5101 => {
                self.chr_mode = value & 0x03;
                debug!("MMC5 CHR Mode: {}", self.chr_mode);
                self.update_chr_mappings(mappings);
            }
            0x5104 => {
                self.eram_flags = match value & 0x03 {
                    0 => EramFlags::NAMETABLE | EramFlags::WRITE,
                    1 => EramFlags::NAMETABLE | EramFlags::EXTENDED_ATTR | EramFlags::WRITE,
                    2 => EramFlags::READ | EramFlags::WRITE,
                    3 => EramFlags::READ,
                    _ => unreachable!(),
                };
                debug!("MMC5 ERAM: {:?}", self.eram_flags);
            }
            0x5105 => {
                self.name_bank[0] = NameTable::from_u8(value & 0x03).unwrap();
                self.name_bank[1] = NameTable::from_u8((value >> 2) & 0x03).unwrap();
                self.name_bank[2] = NameTable::from_u8((value >> 4) & 0x03).unwrap();
                self.name_bank[3] = NameTable::from_u8(value >> 6).unwrap();
                debug!("MMC5 Name Banks: {:?}", self.name_bank);
            }
            0x5106 => {
                self.fill_mode_name = value;
                debug!("MMC5 Fill Mode Name: {:02X}", self.fill_mode_name);
            }
            0x5107 => {
                let attr = value & 0x03;
                self.fill_mode_attr = attr | (attr << 2) | (attr << 4) | (attr << 6);
                debug!("MMC5 Fill Mode Attr: {:02X}", self.fill_mode_attr);
            }
            0x5113..=0x5117 => {
                let index = (address - 0x5113) as usize;
                self.prg_bank[index] = value;
                debug!("MMC5 PRG Bank {}: {:02X}", index, value);
                self.update_prg_mappings(mappings);
            }
            0x5120..=0x512b => {
                let index = (address - 0x5120) as usize;
                self.chr_bank[index] = value;
                debug!("MMC5 CHR Bank {}: {:02X}", index, value);
                self.update_chr_mappings(mappings);
            }
            0x5200 => (), // TODO: Vertical Split
            0x5203 => {
                self.scanline_irq_compare = value;
                debug!("MMC5 Scanline IRQ Compare: {}", self.scanline_irq_compare);
            }
            0x5204 => {
                self.scanline_irq_enable = (value & 0x80) != 0;
                debug!("MMC5 Scanline IRQ Enable: {}", self.scanline_irq_enable);
            }
            0x5c00..=0x5fff => {
                if self.eram_flags.contains(EramFlags::WRITE) {
                    self.eram[address as usize & 0x03ff] = value;
                }
            }
            _ => unimplemented!("MMC5 Register Write {:04X} <= {:02X}", address, value),
        }
    }

    fn read_name(&mut self, mappings: &mut Mappings, ci_ram: &MirrorVec<u8>, address: u16) -> u8 {
        self.ctrl.no_read_count = 0;

        if address == self.ctrl.prev_address {
            self.ctrl.same_address_count += 1;

            if self.ctrl.same_address_count == 3 {
                if self
                    .scanline_irq_status
                    .contains(ScanlineIrqStatus::IN_FRAME)
                {
                    self.ctrl.scanline_count += 1;

                    if self.ctrl.scanline_count == self.scanline_irq_compare {
                        self.scanline_irq_status.insert(ScanlineIrqStatus::PENDING);

                        if self.scanline_irq_enable {
                            self.interrupt.raise(InterruptType::MapperIrq);
                        }
                    }
                } else {
                    debug!("MMC5 Frame Start Detected");
                    self.scanline_irq_status.insert(ScanlineIrqStatus::IN_FRAME);
                    self.ctrl.scanline_count = 0;
                    self.interrupt.clear(InterruptType::MapperIrq);
                }

                debug!("MMC5 New Line Detected");
                self.ctrl.same_line_reads = 1;
                self.update_chr_mappings(mappings);
            }
        } else {
            self.ctrl.prev_address = address;
            self.ctrl.same_address_count = 1;
        }

        self.ctrl.same_line_reads += 1;

        match self.ctrl.same_line_reads {
            64 => self.update_chr_mappings(mappings),
            80 => self.update_chr_mappings(mappings),
            _ => (),
        }

        let index = address as usize & 0x0fff;

        // TODO: Extended attributes
        match self.name_bank[index >> 10] {
            NameTable::Low => ci_ram[index & 0x03ff],
            NameTable::High => ci_ram[0x0400 | (index & 0x03ff)],
            NameTable::Eram => {
                if self.eram_flags.contains(EramFlags::NAMETABLE) {
                    self.eram[index & 0x03ff]
                } else {
                    0
                }
            }
            NameTable::Fill => {
                if (index & 0x03ff) < 0x03c0 {
                    self.fill_mode_name
                } else {
                    self.fill_mode_attr
                }
            }
        }
    }

    fn write_name(
        &mut self,
        _mappings: &mut Mappings,
        ci_ram: &mut MirrorVec<u8>,
        address: u16,
        value: u8,
    ) {
        let index = address as usize & 0x0fff;

        // TODO: Extended attributes
        match self.name_bank[index >> 10] {
            NameTable::Low => ci_ram[index & 0x03ff] = value,
            NameTable::High => ci_ram[0x0400 | (index & 0x03ff)] = value,
            NameTable::Eram => {
                if self.eram_flags.contains(EramFlags::NAMETABLE) {
                    self.eram[index & 0x03ff] = value;
                }
            }
            NameTable::Fill => (),
        }
    }

    fn on_ppu_chr_fetch(&mut self, _mappings: &mut Mappings, _ppu_address: u16) {
        self.ctrl.no_read_count = 0;
    }

    fn on_cpu_cycle(&mut self, mappings: &mut Mappings) {
        self.ctrl.no_read_count += 1;

        if self.ctrl.no_read_count == 3 {
            debug!("MMC5 Frame End Detected");
            self.scanline_irq_status.remove(ScanlineIrqStatus::IN_FRAME);
            self.update_chr_mappings(mappings);
        }
    }
}

fn map_prg(mappings: &mut Mappings, start: usize, len: usize, bank: u8) {
    if (bank & 0x80) != 0 {
        mappings.map_prg_rom(start, len, PRG_BANK_SIZE * bank as usize);
        mappings.unmap_prg_write(start, len);
    } else {
        mappings.map_prg_ram(start, len, PRG_BANK_SIZE * bank as usize);
    }
}
