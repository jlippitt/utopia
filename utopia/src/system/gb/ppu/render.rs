use super::{Ppu, WIDTH};
use fifo::Fifo;
use tracing::trace;

mod fifo;

pub struct RenderState {
    pos_x: usize,
    //bg_ready: bool,
    bg_step: u32,
    bg_coarse_x: u16,
    //bg_fine_x: u8,
    bg_fifo: Fifo,
    bg_tile: u8,
    bg_chr: (u8, u8),
}

impl RenderState {
    pub fn new(scroll_x: u8) -> Self {
        Self {
            pos_x: 0,
            //bg_ready: false,
            bg_step: 0,
            bg_coarse_x: scroll_x as u16 >> 3,
            //bg_fine_x: 0,
            bg_fifo: Fifo::new(),
            bg_tile: 0,
            bg_chr: (0, 0),
        }
    }
}

impl Ppu {
    pub(super) fn reset_renderer(&mut self) {
        self.render = RenderState::new(self.scroll_x);
    }

    pub(super) fn step_renderer(&mut self) -> bool {
        self.fetch_bg();

        if let Some((low, high)) = self.render.bg_fifo.pop() {
            let shift = (high << 2) | (low << 1);
            let color = (self.bg_palette >> shift) & 3;
            self.screen.draw_pixel(color);
            self.render.pos_x += 1;
        }

        self.render.pos_x == WIDTH
    }

    fn fetch_bg(&mut self) {
        match self.render.bg_step {
            0 | 2 | 4 => self.render.bg_step += 1,
            1 => {
                let address = self.control.bg_tile_offset
                    + (((self.bg_pos_y() as u16) << 2) & !0x1f)
                    + self.render.bg_coarse_x;

                trace!("BG Tile Address: {:04X}", address);

                self.render.bg_tile = self.vram[address as usize];
                self.render.bg_coarse_x += 1;
                self.render.bg_step += 1;
            }
            3 => {
                let address = self.bg_chr_address();
                trace!("BG CHR Low Address: {:04X}", address);
                self.render.bg_chr.0 = self.vram[address as usize];
                self.render.bg_step += 1;
            }
            5 => {
                let address = self.bg_chr_address() + 1;
                trace!("BG CHR High Address: {:04X}", address);
                self.render.bg_chr.1 = self.vram[address as usize];
                self.render.bg_step += 1;
            }
            6 => {
                // Push
                if self.render.bg_fifo.try_push(self.render.bg_chr) {
                    self.render.bg_step = 0;
                }
            }
            _ => unreachable!(),
        }
    }

    fn bg_pos_y(&self) -> u8 {
        self.scroll_y.wrapping_add(self.line as u8)
    }

    fn bg_chr_address(&self) -> u16 {
        let mut tile = self.render.bg_tile as u16;

        if tile < 128 && !self.control.bg_chr_select {
            tile += 256;
        }

        let fine_y = self.bg_pos_y() as u16 & 7;

        (tile << 4) | (fine_y << 1)
    }
}
