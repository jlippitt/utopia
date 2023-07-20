use super::WIDTH;

pub const TILE_BUFFER_SIZE: usize = 34;
pub const PIXEL_BUFFER_SIZE: usize = WIDTH >> 1;

#[derive(Copy, Clone, Debug, Default)]
pub struct Tile {
    pub chr_data: u16,
    pub flip_mask: u16,
    pub priority: u8,
    pub palette: u16,
}

pub type TileBuffer = [Tile; TILE_BUFFER_SIZE];

#[derive(Copy, Clone, Debug, Default)]
pub struct Pixel {
    pub color: u16,
    pub priority: u8,
}

pub type PixelBuffer = [Pixel; PIXEL_BUFFER_SIZE];
