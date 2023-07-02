use super::{BiosLoader, System};
use crate::core::gbz80::{Bus, Core, State};
use crate::util::MirrorVec;
use cartridge::Cartridge;
use interrupt::Interrupt;
use ppu::Ppu;
use std::error::Error;
use std::fmt;
use timer::Timer;
use tracing::{debug, warn};

mod cartridge;
mod interrupt;
mod ppu;
mod timer;

const WRAM_SIZE: usize = 8192;
const HRAM_SIZE: usize = 128;

const M_CYCLE_LENGTH: u64 = 4;

pub fn create(
    rom_data: Vec<u8>,
    bios_loader: &impl BiosLoader,
    skip_boot: bool,
) -> Result<Box<dyn System>, Box<dyn Error>> {
    let (initial_state, bios_data) = if skip_boot {
        // TODO: This post-boot state should depend on hardware model
        let initial_state = State {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xd8,
            h: 0x01,
            l: 0x4d,
            sp: 0xfffe,
            pc: 0x0100,
            f: 0xb0, // TODO: H & C should depend on header checksum
        };

        (Some(initial_state), None)
    } else {
        let bios_data = bios_loader.load("dmg")?;

        (None, Some(bios_data))
    };

    // TODO: Should skip boot sequence for other hardware components as well
    let hw = Hardware::new(rom_data, bios_data, skip_boot);

    let core = Core::new(hw, initial_state);

    Ok(Box::new(GameBoy { core }))
}

pub struct GameBoy {
    core: Core<Hardware>,
}

impl System for GameBoy {
    fn width(&self) -> usize {
        ppu::WIDTH
    }

    fn height(&self) -> usize {
        ppu::HEIGHT
    }

    fn pixels(&self) -> &[u8] {
        self.core.bus().ppu.pixels()
    }

    fn run_frame(&mut self) {
        let core = &mut self.core;

        core.bus_mut().ppu.start_frame();

        while !core.bus().ppu.ready() {
            debug!("{}", core);
            core.step();
        }
    }
}

struct Hardware {
    cycles: u64,
    interrupt: Interrupt,
    timer: Timer,
    hram: MirrorVec<u8>,
    wram: MirrorVec<u8>,
    cartridge: Cartridge,
    ppu: Ppu,
    bios_data: Option<MirrorVec<u8>>,
}

impl Hardware {
    pub fn new(rom_data: Vec<u8>, bios_data: Option<Vec<u8>>, skip_boot: bool) -> Self {
        Self {
            cycles: 0,
            interrupt: Interrupt::new(),
            timer: Timer::new(),
            hram: MirrorVec::new(HRAM_SIZE),
            wram: MirrorVec::new(WRAM_SIZE),
            cartridge: Cartridge::new(rom_data),
            ppu: Ppu::new(skip_boot),
            bios_data: bios_data.map(MirrorVec::from),
        }
    }

    fn step(&mut self) {
        self.cycles += M_CYCLE_LENGTH;
        self.timer.step(&mut self.interrupt, M_CYCLE_LENGTH);
        self.ppu.step(&mut self.interrupt, M_CYCLE_LENGTH);
    }

    fn read_high_impl(&mut self, address: u8) -> u8 {
        match address {
            0x00 => 0xff, // TODO: Joypad
            0x04..=0x07 => self.timer.read(address),
            0x0f => self.interrupt.flag(),
            0x10..=0x3f => {
                warn!("APU register reads not yet implemented");
                0
            }
            0x40..=0x4f => self.ppu.read_register(address),
            0x80..=0xfe => self.hram[address as usize],
            0xff => self.interrupt.enable(),
            _ => panic!("Unmapped register read"),
        }
    }

    fn write_high_impl(&mut self, address: u8, value: u8) {
        match address {
            0x00 => (),        // TODO: Joypad
            0x01 | 0x02 => (), // TODO: Serial port
            0x04..=0x07 => self.timer.write(address, value),
            0x0f => self.interrupt.set_flag(value),
            0x10..=0x3f => debug!("APU register writes not yet implemented"),
            0x40..=0x4f => self.ppu.write_register(&mut self.interrupt, address, value),
            0x50 => {
                self.bios_data = None;
                debug!("BIOS disabled");
            }
            0x80..=0xfe => self.hram[address as usize] = value,
            0xff => self.interrupt.set_enable(value),
            _ => warn!("Unmapped register write: {:02X}", address),
        }
    }
}

impl Bus for Hardware {
    fn idle(&mut self) {
        self.step();
    }

    fn read(&mut self, address: u16) -> u8 {
        self.step();

        match address >> 13 {
            0 => {
                if address < 0x0100 {
                    if let Some(bios_data) = &self.bios_data {
                        bios_data[address as usize]
                    } else {
                        self.cartridge.read_rom(address)
                    }
                } else {
                    self.cartridge.read_rom(address)
                }
            }
            1 | 2 | 3 => self.cartridge.read_rom(address),
            4 => self.ppu.read_vram(address),
            5 => self.cartridge.read_ram(address),
            6 => self.wram[address as usize],
            7 => match address {
                0xff00..=0xffff => self.read_high_impl(address as u8),
                0xfe00..=0xfe9f => self.ppu.read_oam(address as u8),
                0xfea0..=0xfeff => 0xff,
                _ => panic!("Read from unmapped location"),
            },
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        self.step();

        match address >> 13 {
            0 | 1 | 2 | 3 => self.cartridge.write_register(address, value),
            4 => self.ppu.write_vram(address, value),
            5 => self.cartridge.write_ram(address, value),
            6 => self.wram[address as usize] = value,
            7 => match address {
                0xff00..=0xffff => self.write_high_impl(address as u8, value),
                0xfe00..=0xfe9f => self.ppu.write_oam(address as u8, value),
                0xfea0..=0xfeff => (),
                _ => warn!("Write to unmapped location"),
            },
            _ => unreachable!(),
        }
    }

    fn read_high(&mut self, address: u8) -> u8 {
        self.step();
        self.read_high_impl(address)
    }

    fn write_high(&mut self, address: u8, value: u8) {
        self.step();
        self.write_high_impl(address, value);
    }

    fn poll(&self) -> u8 {
        self.interrupt.poll()
    }

    fn acknowledge(&mut self, mask: u8) {
        self.interrupt.acknowledge(mask);
    }
}

impl fmt::Display for Hardware {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "V={} H={} T={}",
            self.ppu.line(),
            self.ppu.dot(),
            self.cycles,
        )
    }
}
