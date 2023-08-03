use super::oam::Sprite;
use fifo::{BackgroundFifo, SpriteFifo};
use tracing::trace;

mod fifo;

pub struct RenderState {
    pos_x: u8,
    //bg_ready: bool,
    bg_step: u32,
    bg_coarse_x: u16,
    bg_fine_x: u8,
    bg_tile: u8,
    bg_chr: (u8, u8),
    bg_fifo: BackgroundFifo,
    sprite_step: u32,
    sprite_tile: u8,
    sprite_chr: (u8, u8),
    sprite_fifo: SpriteFifo,
}

impl RenderState {
    pub fn new(scroll_x: u8) -> Self {
        Self {
            pos_x: 0,
            //bg_ready: false,
            bg_step: 0,
            bg_coarse_x: scroll_x as u16 >> 3,
            bg_fine_x: scroll_x & 7,
            bg_tile: 0,
            bg_chr: (0, 0),
            bg_fifo: BackgroundFifo::new(),
            sprite_step: 0,
            sprite_tile: 0,
            sprite_chr: (0, 0),
            sprite_fifo: SpriteFifo::new(),
        }
    }
}

impl super::Ppu {
    pub(super) fn reset_renderer(&mut self) {
        self.render = RenderState::new(self.scroll_x);
    }

    pub(super) fn step_renderer(&mut self) -> bool {
        if self.oam.sprite_ready(self.render.pos_x) {
            if self.render.bg_fifo.is_empty() {
                self.fetch_bg();
                return false;
            } else {
                self.fetch_sprite();

                if self.oam.sprite_ready(self.render.pos_x) {
                    return false;
                }
            }
        } else {
            self.fetch_bg();
        }

        self.draw_pixel();
        self.render.pos_x == (super::WIDTH as u8)
    }

    fn draw_pixel(&mut self) {
        let Some(bg_pixel) = self.render.bg_fifo.pop() else {
            return;
        };

        if self.render.bg_fine_x > 0 {
            self.render.bg_fine_x -= 1;
            return;
        }

        let sprite_pixel = self.render.sprite_fifo.pop();

        let sprite_visible = self.ctrl.obj_enable
            && sprite_pixel.color != 0
            && (bg_pixel == 0 || !sprite_pixel.below_bg);

        let color = if sprite_visible {
            let sprite_palette = self.obj_palette[sprite_pixel.palette as usize];
            (sprite_palette >> (sprite_pixel.color << 1)) & 3
        } else if self.ctrl.bg_enable {
            (self.bg_palette >> (bg_pixel << 1)) & 3
        } else {
            0
        };

        self.screen.draw_pixel(color);
        self.render.pos_x += 1;
    }

    fn fetch_bg(&mut self) {
        match self.render.bg_step {
            0 | 2 | 4 => self.render.bg_step += 1,
            1 => {
                let coarse_y = ((self.bg_pos_y() as u16) >> 3) & 31;
                let coarse_x = self.render.bg_coarse_x & 31;
                let address = self.ctrl.bg_tile_offset + (coarse_y << 5) + coarse_x;
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

    fn fetch_sprite(&mut self) {
        let sprite = self.oam.current_sprite();

        match self.render.sprite_step {
            0 | 2 | 4 => self.render.sprite_step += 1,
            1 => {
                self.render.sprite_tile = sprite.chr;
                self.render.sprite_step += 1;
            }
            3 => {
                let address = self.sprite_chr_address(sprite);
                trace!("Sprite CHR Low Address: {:04X}", address);
                self.render.sprite_chr.0 = self.vram[address as usize];
                self.render.sprite_step += 1;
            }
            5 => {
                let address = self.sprite_chr_address(sprite) + 1;
                trace!("Sprite CHR High Address: {:04X}", address);
                self.render.sprite_chr.1 = self.vram[address as usize];
                self.render.sprite_step += 1;
            }
            6 => {
                // Push
                self.render.sprite_fifo.push(sprite, self.render.sprite_chr);
                self.oam.next_sprite();
                self.render.sprite_step = 0;
            }
            _ => unreachable!(),
        }
    }

    fn bg_pos_y(&self) -> u8 {
        self.scroll_y.wrapping_add(self.line)
    }

    fn bg_chr_address(&self) -> u16 {
        let mut tile = self.render.bg_tile as u16;

        if tile < 128 && !self.ctrl.bg_chr_select {
            tile += 256;
        }

        let fine_y = self.bg_pos_y() as u16 & 7;

        (tile << 4) | (fine_y << 1)
    }

    fn sprite_chr_address(&self, sprite: &Sprite) -> u16 {
        let tile = self.render.sprite_tile as u16;

        let (tile_mask, line_mask) = if self.ctrl.obj_size {
            (0xfe, 15)
        } else {
            (0xff, 7)
        };

        let fine_y = (self.line.wrapping_sub(sprite.y) as u16) & line_mask;
        let flip_mask = if sprite.flip_y { line_mask } else { 0 };

        ((tile & tile_mask) << 4) | ((fine_y ^ flip_mask) << 1)
    }
}
