use super::super::vce::Color;
use super::vram::Vram;
use bitfield_struct::bitfield;
use tracing::debug;

const TOTAL_SPRITES: usize = 64;
const MAX_SPRITES_PER_LINE: usize = 64;

#[bitfield(u16)]
struct SpriteAttributes {
    #[bits(4)]
    palette_offset: u8,
    #[bits(3)]
    __: u8,
    foreground: bool,
    width: bool,
    #[bits(2)]
    __: u8,
    flip_x: bool,
    #[bits(2)]
    height: u8,
    __: bool,
    flip_y: bool,
}

#[derive(Copy, Clone, Debug, Default)]
struct Sprite {
    pos_y: u16,
    pos_x: u16,
    chr_index: u16,
    attr: SpriteAttributes,
}

pub struct SpriteLayer {
    enabled: bool,
    dma_scheduled: bool,
    dma_repeat: bool,
    table_address: u16,
    sprites: [Sprite; TOTAL_SPRITES],
}

impl SpriteLayer {
    pub fn new() -> Self {
        Self {
            enabled: false,
            dma_scheduled: false,
            dma_repeat: false,
            table_address: 0,
            sprites: [Sprite::default(); TOTAL_SPRITES],
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        debug!("Sprite Layer Enabled: {}", self.enabled);
    }

    pub fn set_dma_repeat(&mut self, dma_repeat: bool) {
        self.dma_repeat = dma_repeat;
        debug!("Sprite DMA Repeat: {}", self.dma_repeat);
    }

    pub fn set_table_address(&mut self, msb: bool, value: u8) {
        self.table_address = if msb {
            (self.table_address & 0xff) | ((value as u16) << 8)
        } else {
            (self.table_address & 0xff00) | value as u16
        };

        debug!("Sprite Table Address: {:04X}", self.table_address);

        if msb {
            self.dma_scheduled = true;
            debug!("Sprite DMA Scheduled: {}", self.dma_repeat);
        }
    }

    pub fn transfer_dma(&mut self, vram: &Vram) {
        if !self.dma_scheduled {
            return;
        }

        debug!("Sprite DMA Begin");

        for (index, sprite) in self.sprites.iter_mut().enumerate() {
            let base_address = self.table_address as usize + (index << 2);

            sprite.pos_y = vram.get(base_address) & 0x03ff;
            debug!("Sprite {} Pos Y: {}", index, sprite.pos_y);

            sprite.pos_x = vram.get(base_address + 1) & 0x03ff;
            debug!("Sprite {} Pos X: {}", index, sprite.pos_x);

            sprite.chr_index = vram.get(base_address + 2) & 0x07ff;
            debug!("Sprite {} CHR Index: {}", index, sprite.chr_index);

            sprite.attr = vram.get(base_address + 3).into();
            debug!("Sprite {} Attr: {:?}", index, sprite.attr);
        }

        debug!("Sprite DMA End");

        self.dma_scheduled = self.dma_repeat;
        debug!("Sprite DMA Scheduled: {}", self.dma_repeat);
    }

    pub fn render_line(
        &self,
        line_buffer: &mut [Color],
        vram: &Vram,
        palette: &[Color],
        line: u16,
    ) {
        if !self.enabled {
            return;
        }

        let raster_line = line + 64;

        let mut sprites_selected = 0;

        for sprite in &self.sprites {
            if sprite.pos_y > raster_line
                || (sprite.pos_y + (16 << sprite.attr.height())) <= raster_line
            {
                continue;
            }

            sprites_selected += 1;

            if sprites_selected > MAX_SPRITES_PER_LINE {
                // TODO: Sprite overflow flag/interrupt
                break;
            }

            let pixel_y = {
                let offset_y = raster_line - sprite.pos_y;

                if sprite.attr.flip_y() {
                    offset_y ^ ((16 << sprite.attr.height()) - 1)
                } else {
                    offset_y
                }
            };

            let (cell_flip_x, pixel_flip_x) = if sprite.attr.flip_x() {
                (sprite.attr.width() as usize, 15)
            } else {
                (0, 0)
            };

            let cell_y = pixel_y >> 4;
            let fine_y = pixel_y & 15;

            for cell_x in 0..=(sprite.attr.width() as usize) {
                let base_address = ((sprite.chr_index as usize) << 5)
                    + ((cell_y as usize) << (6 + sprite.attr.width() as u32))
                    + ((cell_x ^ cell_flip_x) << 6)
                    + fine_y as usize;

                let chr0 = vram.get(base_address);
                let chr1 = vram.get(base_address + 16);
                let chr2 = vram.get(base_address + 32);
                let chr3 = vram.get(base_address + 48);

                for pixel_x in 0..16 {
                    let pos_x = sprite.pos_x as usize + (cell_x << 4) + pixel_x as usize;

                    if pos_x < 32 || pos_x >= line_buffer.len() + 32 {
                        continue;
                    }

                    // TODO: Foreground flag

                    let shift = (15 - pixel_x) ^ pixel_flip_x;
                    let color0 = (chr0 >> shift) & 1;
                    let color1 = (chr1 >> shift) & 1;
                    let color2 = (chr2 >> shift) & 1;
                    let color3 = (chr3 >> shift) & 1;

                    let color_index = (color3 << 3) | (color2 << 2) | (color1 << 1) | color0;

                    if color_index == 0 {
                        continue;
                    }

                    let palette_index =
                        256 + ((sprite.attr.palette_offset() as usize) << 4) + color_index as usize;

                    line_buffer[pos_x - 32] = palette[palette_index];
                }
            }
        }
    }
}
