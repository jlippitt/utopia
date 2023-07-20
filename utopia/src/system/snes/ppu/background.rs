use tracing::debug;

pub struct BackgroundLayer {
    tile_map: u16,
    chr_map: u16,
    id: u32,
}

impl BackgroundLayer {
    pub fn new(id: u32) -> Self {
        Self {
            tile_map: 0,
            chr_map: 0,
            id,
        }
    }

    pub fn set_tile_map(&mut self, value: u8) {
        // TODO: Mirroring
        self.tile_map = ((value & 0xfc) as u16) << 8;
        debug!("BG{} Tile Map: {:04X}", self.id, self.tile_map);
    }

    pub fn set_chr_map(&mut self, value: u8) {
        self.tile_map = (value as u16) << 8;
        debug!("BG{} CHR Map: {:04X}", self.id, self.chr_map);
    }
}
