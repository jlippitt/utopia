use super::Interrupt;
use crate::util::mirror::{Mirror, MirrorVec};
use crate::{Mapped, MemoryMapper};
use mapper::{Mapper, MapperType, Mappings, MirrorMode, PrgRead, PrgWrite};
use tracing::info;

mod mapper;

const HEADER_SIZE: usize = 16;
const TRAINER_SIZE: usize = 512;
const PRG_ROM_MULTIPLIER: usize = 16384;
const CHR_ROM_MULTIPLIER: usize = 8192;
const PRG_RAM_SIZE: usize = 8192;
const CI_RAM_SIZE: usize = 2048;

pub struct Cartridge<T: Mapped> {
    prg_rom: MirrorVec<u8>,
    prg_ram: Mirror<T>,
    chr_data: MirrorVec<u8>,
    chr_writable: bool,
    ci_ram: MirrorVec<u8>,
    mappings: Mappings,
    mapper: MapperType,
}

impl<T: Mapped> Cartridge<T> {
    pub fn new<U: MemoryMapper<Mapped = T>>(
        data: Vec<u8>,
        memory_mapper: &U,
        interrupt: Interrupt,
    ) -> Result<Self, crate::Error> {
        let mapper_number = ((data[6] & 0xf0) >> 4) | (data[7] & 0xf0);
        let prg_rom_size = PRG_ROM_MULTIPLIER * (data[4] as usize);
        let chr_rom_size = CHR_ROM_MULTIPLIER * (data[5] as usize);

        info!("Mapper Number: {}", mapper_number);
        info!("PRG ROM Size: {}", prg_rom_size);

        let trainer_present = (data[6] & 0x04) != 0;
        let prg_rom_start = HEADER_SIZE + if trainer_present { TRAINER_SIZE } else { 0 };
        let prg_rom_end = prg_rom_start + prg_rom_size;
        let prg_rom = Vec::from(&data[prg_rom_start..prg_rom_end]);

        let chr_data = if chr_rom_size > 0 {
            info!("CHR ROM Size: {}", chr_rom_size);
            let chr_rom_end = prg_rom_end + chr_rom_size;
            Vec::from(&data[prg_rom_end..chr_rom_end])
        } else {
            info!("CHR RAM Size: {}", CHR_ROM_MULTIPLIER);
            vec![0; CHR_ROM_MULTIPLIER]
        };

        let mirror_mode = if (data[6] & 0x01) != 0 {
            MirrorMode::Vertical
        } else {
            MirrorMode::Horizontal
        };

        info!("Mirror Mode: {:?}", mirror_mode);

        let mut mappings = Mappings::new(mirror_mode);
        let mut mapper = MapperType::new(mapper_number, prg_rom_size, interrupt);
        mapper.init_mappings(&mut mappings);

        let battery_backed = (data[6] & 0x02) != 0;
        info!("Battery Backed: {}", battery_backed);

        Ok(Self {
            prg_rom: prg_rom.into(),
            prg_ram: memory_mapper.open(PRG_RAM_SIZE, battery_backed)?.into(),
            chr_data: chr_data.into(),
            chr_writable: chr_rom_size == 0,
            ci_ram: MirrorVec::new(CI_RAM_SIZE),
            mappings,
            mapper,
        })
    }

    pub fn read_prg(&mut self, address: u16, prev_value: u8) -> u8 {
        match self.mappings.prg_read[address as usize >> 12] {
            PrgRead::Rom(offset) => self.prg_rom[offset as usize | (address as usize & 0x0fff)],
            PrgRead::Ram(offset) => self.prg_ram[offset as usize | (address as usize & 0x0fff)],
            PrgRead::Register => self
                .mapper
                .read_register(&mut self.mappings, address, prev_value),
            PrgRead::None => prev_value,
        }
    }

    pub fn write_prg(&mut self, address: u16, value: u8) {
        match self.mappings.prg_write[address as usize >> 12] {
            PrgWrite::Ram(offset) => {
                self.prg_ram[offset as usize | (address as usize & 0x0fff)] = value
            }
            PrgWrite::Register => self
                .mapper
                .write_register(&mut self.mappings, address, value),
            PrgWrite::None => (),
        }
    }

    pub fn read_vram(&mut self, address: u16) -> u8 {
        if address >= 0x2000 {
            self.mapper
                .read_name(&mut self.mappings, &self.ci_ram, address)
        } else {
            let offset = self.mappings.chr[(address >> 10) as usize];
            let value = self.chr_data[offset as usize | ((address as usize) & 0x03ff)];
            self.mapper.on_ppu_chr_fetch(&mut self.mappings, address);
            value
        }
    }

    pub fn write_vram(&mut self, address: u16, value: u8) {
        if address >= 0x2000 {
            self.mapper
                .write_name(&mut self.mappings, &mut self.ci_ram, address, value);
        } else if self.chr_writable {
            let offset = self.mappings.chr[(address >> 10) as usize];
            self.chr_data[offset as usize | ((address as usize) & 0x03ff)] = value;
        }
    }

    pub fn on_cpu_cycle(&mut self) {
        self.mapper.on_cpu_cycle(&mut self.mappings);
    }

    pub fn on_ppu_address_changed(&mut self, ppu_address: u16) {
        self.mapper.on_ppu_address_changed(ppu_address);
    }

    pub fn audio_output(&self) -> f32 {
        self.mapper.audio_output()
    }
}
