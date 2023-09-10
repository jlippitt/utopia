use super::vram::Vram;
use bitfield_struct::bitfield;
use tracing::debug;

const TOTAL_SPRITES: usize = 64;

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
}
