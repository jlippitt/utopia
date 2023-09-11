use super::interrupt::{Interrupt, InterruptType};
use crate::util::MirrorVec;
use oam::Oam;
use palette::Palette;
use render::RenderState;
use screen::Screen;
use tracing::debug;

pub use screen::{HEIGHT, WIDTH};

mod oam;
mod palette;
mod render;
mod screen;

const VRAM_BANK_SIZE: usize = 8192;

const VBLANK_LINE: u8 = 144;
const TOTAL_LINES: u8 = 154;

const OAM_SEARCH_LENGTH: u64 = 80;
const DOTS_PER_LINE: u64 = 456;

const BASE_TILE_OFFSET: u16 = 0x1800;

#[derive(Copy, Clone, Debug)]
enum Mode {
    HBlank = 0,
    VBlank = 1,
    Oam = 2,
    Vram = 3,
}

struct Control {
    lcd_enable: bool,
    bg_enable: bool,
    bg_tile_offset: u16,
    bg_chr_select: bool,
    window_enable: bool,
    window_tile_offset: u16,
    obj_enable: bool,
    obj_size: bool,
    raw: u8,
}

struct InterruptEnable {
    hblank: bool,
    vblank: bool,
    oam: bool,
    lcd_y: bool,
}

pub struct Ppu {
    ready: bool,
    is_cgb: bool,
    mode: Mode,
    line: u8,
    dot: u64,
    ctrl: Control,
    interrupt_enable: InterruptEnable,
    scroll_y: u8,
    scroll_x: u8,
    window_y: u8,
    window_x: u8,
    lcd_y_compare: u8,
    dmg_palette_bg: u8,
    dmg_palette_obj: [u8; 2],
    cgb_palette_bg: Palette,
    cgb_palette_obj: Palette,
    render: RenderState,
    screen: Screen,
    vram: MirrorVec<u8>,
    vram_bank_offset: usize,
    oam: Oam,
}

impl Ppu {
    pub fn new(is_cgb: bool, skip_boot: bool) -> Self {
        Self {
            ready: false,
            is_cgb,
            mode: Mode::Oam,
            line: 0,
            dot: 0,
            ctrl: Control {
                lcd_enable: skip_boot,
                bg_enable: false,
                bg_tile_offset: BASE_TILE_OFFSET,
                bg_chr_select: false,
                window_enable: false,
                window_tile_offset: BASE_TILE_OFFSET,
                obj_enable: false,
                obj_size: false,
                raw: 0,
            },
            interrupt_enable: InterruptEnable {
                hblank: false,
                vblank: false,
                oam: false,
                lcd_y: false,
            },
            scroll_y: 0,
            scroll_x: 0,
            window_y: 0,
            window_x: 0,
            lcd_y_compare: 0,
            dmg_palette_bg: 0,
            dmg_palette_obj: [0; 2],
            cgb_palette_bg: Palette::new("BG"),
            cgb_palette_obj: Palette::new("OBJ"),
            render: Default::default(),
            screen: Screen::new(),
            vram: MirrorVec::new(VRAM_BANK_SIZE * if is_cgb { 2 } else { 1 }),
            vram_bank_offset: 0,
            oam: Oam::new(),
        }
    }

    pub fn ready(&self) -> bool {
        self.ready
    }

    pub fn start_frame(&mut self) {
        self.ready = false;
    }

    pub fn line(&self) -> u8 {
        self.line
    }

    pub fn dot(&self) -> u64 {
        self.dot
    }

    pub fn pixels(&self) -> &[u8] {
        self.screen.pixels()
    }

    pub fn read_register(&self, address: u8) -> u8 {
        match address {
            0x40 => self.ctrl.raw,
            0x41 => {
                let mut value: u8 = 0x80 | (self.mode as u8);

                if self.line == self.lcd_y_compare {
                    value |= 0x04;
                }

                if self.interrupt_enable.hblank {
                    value |= 0x08;
                }

                if self.interrupt_enable.vblank {
                    value |= 0x10;
                }

                if self.interrupt_enable.oam {
                    value |= 0x20;
                }

                if self.interrupt_enable.lcd_y {
                    value |= 0x40;
                }

                value
            }
            0x42 => self.scroll_y,
            0x43 => self.scroll_x,
            0x44 => self.line,
            0x45 => self.lcd_y_compare,
            0x47 => self.dmg_palette_bg,
            0x48 => self.dmg_palette_obj[0],
            0x49 => self.dmg_palette_obj[1],
            0x4a => self.window_y,
            0x4b => self.window_x,
            _ => panic!("PPU register read {:02X} not yet implemented", address),
        }
    }

    pub fn write_register(&mut self, interrupt: &mut Interrupt, address: u8, value: u8) {
        match address {
            0x40 => {
                let lcd_enable = (value & 0x80) != 0;

                if lcd_enable && !self.ctrl.lcd_enable {
                    debug!("Screen On");
                    self.set_mode(interrupt, Mode::Oam);
                    self.ctrl.lcd_enable = true;
                    self.screen.reset();
                } else if !lcd_enable && self.ctrl.lcd_enable {
                    debug!("Screen Off");
                    self.mode = Mode::HBlank;
                    self.line = 0;
                    self.dot = 0;
                    self.ctrl.lcd_enable = false;
                }

                self.ctrl.bg_enable = (value & 0x01) != 0;
                self.ctrl.bg_tile_offset = BASE_TILE_OFFSET + ((value as u16 & 0x08) << 7);
                self.ctrl.bg_chr_select = (value & 0x10) != 0;
                self.ctrl.window_enable = (value & 0x20) != 0;
                self.ctrl.window_tile_offset = BASE_TILE_OFFSET + ((value as u16 & 0x40) << 4);
                self.ctrl.obj_enable = (value & 0x02) != 0;
                self.ctrl.obj_size = (value & 0x04) != 0;
                debug!("BG Enable: {}", self.ctrl.bg_enable);
                debug!("BG Tile Offset: {:04X}", self.ctrl.bg_tile_offset);
                debug!("Window Enable: {}", self.ctrl.window_enable);
                debug!("Window Tile Offset: {:04X}", self.ctrl.window_tile_offset);
                debug!("BG CHR Select: {}", self.ctrl.bg_chr_select);
                debug!("OBJ Enable: {}", self.ctrl.obj_enable);
                debug!("OBJ Size: 8x{}", 8 << self.ctrl.obj_size as u32);

                self.ctrl.raw = value;
            }
            0x41 => {
                self.interrupt_enable.hblank = (value & 0x08) != 0;
                self.interrupt_enable.vblank = (value & 0x10) != 0;
                self.interrupt_enable.oam = (value & 0x20) != 0;
                self.interrupt_enable.lcd_y = (value & 0x40) != 0;
                debug!("HBlank Interrupt Enable: {}", self.interrupt_enable.hblank);
                debug!("VBlank Interrupt Enable: {}", self.interrupt_enable.vblank);
                debug!("OAM Interrupt Enable: {}", self.interrupt_enable.oam);
                debug!("LCD Y Interrupt Enable: {}", self.interrupt_enable.lcd_y);
            }
            0x42 => {
                self.scroll_y = value;
                debug!("Scroll Y: {}", self.scroll_y);
            }
            0x43 => {
                self.scroll_x = value;
                debug!("Scroll X: {}", self.scroll_x);
            }
            0x45 => {
                self.lcd_y_compare = value;
                debug!("LCD Y Compare: {}", self.lcd_y_compare);
            }
            0x47 => {
                self.dmg_palette_bg = value;
                debug!("BG Palette: {:08b}", self.dmg_palette_bg);
            }
            0x48 => {
                self.dmg_palette_obj[0] = value;
                debug!("OBJ Palette 0: {:08b}", self.dmg_palette_obj[0]);
            }
            0x49 => {
                self.dmg_palette_obj[1] = value;
                debug!("OBJ Palette 1: {:08b}", self.dmg_palette_obj[1]);
            }
            0x4a => {
                self.window_y = value;
                debug!("Window Y: {}", self.window_y);
            }
            0x4b => {
                self.window_x = value;
                debug!("Window X: {}", self.window_x);
            }
            0x4c => {
                // Ignore for now
            }
            0x4f => {
                if self.is_cgb {
                    self.vram_bank_offset = VRAM_BANK_SIZE * (value as usize & 0x01);
                    debug!("VRAM Bank Offset: {}", self.vram_bank_offset);
                }
            }
            0x68 => {
                if self.is_cgb {
                    self.cgb_palette_bg.set_address(value);
                }
            }
            0x69 => {
                if self.is_cgb {
                    self.cgb_palette_bg.write(value);
                }
            }
            0x6a => {
                if self.is_cgb {
                    self.cgb_palette_obj.set_address(value);
                }
            }
            0x6b => {
                if self.is_cgb {
                    self.cgb_palette_obj.write(value);
                }
            }
            _ => panic!("PPU register write {:02X} not yet implemented", address),
        }
    }

    pub fn read_vram(&self, address: u16) -> u8 {
        self.vram[self.vram_bank_offset + (address as usize & 0x1fff)]
    }

    pub fn write_vram(&mut self, address: u16, value: u8) {
        self.vram[self.vram_bank_offset + (address as usize & 0x1fff)] = value;
    }

    pub fn read_oam(&self, address: u8) -> u8 {
        self.oam.read(address)
    }

    pub fn write_oam(&mut self, address: u8, value: u8) {
        self.oam.write(address, value)
    }

    pub fn step(&mut self, interrupt: &mut Interrupt, cycles: u64) {
        if !self.ctrl.lcd_enable {
            return;
        }

        for _ in 0..cycles {
            match self.mode {
                Mode::HBlank => {
                    self.dot += 1;

                    if self.dot == DOTS_PER_LINE {
                        self.next_line(interrupt);

                        if self.line == VBLANK_LINE {
                            self.ready = true;
                            self.screen.reset();
                            self.set_mode(interrupt, Mode::VBlank);
                            interrupt.raise(InterruptType::VBlank);
                        } else {
                            self.set_mode(interrupt, Mode::Oam);
                        }
                    }
                }
                Mode::VBlank => {
                    self.dot += 1;

                    if self.dot == DOTS_PER_LINE {
                        self.next_line(interrupt);

                        if self.line == 0 {
                            self.set_mode(interrupt, Mode::Oam);
                        }
                    }
                }
                Mode::Oam => {
                    self.dot += 1;

                    if self.dot == OAM_SEARCH_LENGTH {
                        let obj_size = 8 << self.ctrl.obj_size as u32;
                        self.oam.select_sprites(self.line, obj_size);
                        self.set_mode(interrupt, Mode::Vram);
                        self.init_renderer();
                    }
                }
                Mode::Vram => {
                    self.dot += 1;

                    let done = self.step_renderer();

                    if done {
                        self.set_mode(interrupt, Mode::HBlank);
                    }
                }
            }
        }
    }

    fn set_mode(&mut self, interrupt: &mut Interrupt, mode: Mode) {
        self.mode = mode;
        debug!("Mode: {:?}", self.mode);

        let lcd_stat = match self.mode {
            Mode::HBlank => self.interrupt_enable.hblank,
            Mode::VBlank => self.interrupt_enable.vblank,
            Mode::Oam => self.interrupt_enable.oam,
            Mode::Vram => false,
        };

        if lcd_stat {
            interrupt.raise(InterruptType::LcdStat);
        }
    }

    fn next_line(&mut self, interrupt: &mut Interrupt) {
        self.dot = 0;
        self.line += 1;

        if self.line == TOTAL_LINES {
            self.line = 0;
        }

        if self.line == self.lcd_y_compare && self.interrupt_enable.lcd_y {
            interrupt.raise(InterruptType::LcdStat);
        }
    }
}
