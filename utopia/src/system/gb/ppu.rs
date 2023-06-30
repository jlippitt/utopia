use super::interrupt::{Interrupt, InterruptType};
use crate::util::MirrorVec;
use render::RenderState;
use screen::Screen;
use tracing::debug;

pub use screen::{HEIGHT, WIDTH};

mod render;
mod screen;

const VRAM_SIZE: usize = 8192;

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
    mode: Mode,
    line: u8,
    dot: u64,
    control: Control,
    interrupt_enable: InterruptEnable,
    scroll_y: u8,
    scroll_x: u8,
    bg_palette: u8,
    lcd_y_compare: u8,
    render: RenderState,
    screen: Screen,
    vram: MirrorVec<u8>,
    oam: [u8; 160],
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            ready: false,
            mode: Mode::Oam,
            line: 0,
            dot: 0,
            control: Control {
                lcd_enable: false,
                bg_enable: false,
                bg_tile_offset: BASE_TILE_OFFSET,
                bg_chr_select: false,
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
            lcd_y_compare: 0,
            bg_palette: 0,
            render: RenderState::new(0),
            screen: Screen::new(),
            vram: MirrorVec::new(VRAM_SIZE),
            oam: [0; 160],
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
            0x40 => self.control.raw,
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
            0x44 => self.line as u8,
            0x45 => self.lcd_y_compare,
            0x47 => self.bg_palette,
            _ => panic!("PPU register read {:02X} not yet implemented", address),
        }
    }

    pub fn write_register(&mut self, address: u8, value: u8) {
        match address {
            0x40 => {
                let lcd_enable = (value & 0x80) != 0;

                if lcd_enable && !self.control.lcd_enable {
                    debug!("Screen On");
                    debug!("Mode: {:?}", self.mode);
                    self.control.lcd_enable = true;
                    self.screen.reset();
                } else if !lcd_enable && self.control.lcd_enable {
                    debug!("Screen Off");
                    self.mode = Mode::Oam;
                    self.line = 0;
                    self.dot = 0;
                    self.control.lcd_enable = false;
                }

                self.control.bg_enable = (value & 0x01) != 0;
                self.control.bg_tile_offset = BASE_TILE_OFFSET + ((value as u16 & 0x08) << 7);
                self.control.bg_chr_select = (value & 0x10) != 0;
                debug!("BG Enable: {}", self.control.bg_enable);
                debug!("BG Tile Offset: {:04X}", self.control.bg_tile_offset);
                debug!("BG CHR Select: {}", self.control.bg_chr_select);

                self.control.raw = value;
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
                self.bg_palette = value;
                debug!("BG Palette: {:08b}", self.bg_palette);
            }
            _ => debug!("PPU register write {:02X} not yet implemented", address),
        }
    }

    pub fn read_vram(&self, address: u16) -> u8 {
        self.vram[address as usize]
    }

    pub fn write_vram(&mut self, address: u16, value: u8) {
        self.vram[address as usize] = value;
    }

    pub fn read_oam(&self, address: u8) -> u8 {
        self.oam[address as usize]
    }

    pub fn write_oam(&mut self, address: u8, value: u8) {
        self.oam[address as usize] = value;
    }

    pub fn step(&mut self, interrupt: &mut Interrupt, cycles: u64) {
        if !self.control.lcd_enable {
            return;
        }

        for _ in 0..cycles {
            match self.mode {
                Mode::HBlank => {
                    self.dot += 1;

                    if self.dot == DOTS_PER_LINE {
                        self.next_line();

                        if self.line == VBLANK_LINE {
                            self.ready = true;
                            self.screen.reset();
                            self.set_mode(Mode::VBlank);
                            interrupt.raise(InterruptType::VBlank);
                        } else {
                            self.set_mode(Mode::Oam);
                        }
                    }
                }
                Mode::VBlank => {
                    self.dot += 1;

                    if self.dot == DOTS_PER_LINE {
                        self.next_line();

                        if self.line == 0 {
                            self.set_mode(Mode::Oam);
                        }
                    }
                }
                Mode::Oam => {
                    self.dot += 1;

                    if self.dot == OAM_SEARCH_LENGTH {
                        self.set_mode(Mode::Vram);
                        self.reset_renderer();
                    }
                }
                Mode::Vram => {
                    self.dot += 1;

                    let done = self.step_renderer();

                    if done {
                        self.set_mode(Mode::HBlank);
                    }
                }
            }
        }
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        debug!("Mode: {:?}", self.mode);

        // TODO: Mode interrupts
    }

    fn next_line(&mut self) {
        self.dot = 0;
        self.line += 1;

        if self.line == TOTAL_LINES {
            self.line = 0;
        }

        // TODO: LYC check
    }
}
