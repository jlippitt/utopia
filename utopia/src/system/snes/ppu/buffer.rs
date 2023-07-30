use super::WIDTH;

pub const TILE_BUFFER_SIZE: usize = 67;
pub const OFFSET_BUFFER_SIZE: usize = TILE_BUFFER_SIZE / 2;
pub const PIXEL_BUFFER_SIZE: usize = WIDTH >> 1;

pub const LAYER_BACKDROP: u8 = 0x20;

#[derive(Copy, Clone, Debug, Default)]
pub struct Tile {
    pub chr_data: u64,
    pub flip_mask: u16,
    pub priority: u8,
    pub palette: u16,
    pub pos_x: u16,
}

pub type TileBuffer = [Tile; TILE_BUFFER_SIZE];

#[derive(Copy, Clone, Debug, Default)]
pub struct Offset {
    pub x: u16,
    pub y: u16,
}

pub type OffsetBuffer = [Offset; OFFSET_BUFFER_SIZE];

#[derive(Copy, Clone, Debug, Default)]
pub struct Pixel {
    pub color: u16,
    pub priority: u8,
    pub layer: u8,
}

pub type PixelBuffer = [Pixel; PIXEL_BUFFER_SIZE];
