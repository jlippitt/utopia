use tracing::debug;

pub struct BackgroundLayer {
    enabled: bool,
    scroll_x: u16,
    scroll_y: u16,
    tile_map_width: u16,
    tile_map_height: u16,
}

impl BackgroundLayer {
    pub fn new() -> Self {
        Self {
            enabled: false,
            scroll_x: 0,
            scroll_y: 0,
            tile_map_width: 32,
            tile_map_height: 32,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        debug!("BG Layer Enabled: {}", enabled);
    }

    pub fn set_scroll_x(&mut self, msb: bool, value: u8) {
        self.scroll_x = if msb {
            (self.scroll_x & 0xff) | ((value as u16 & 0x03) << 8)
        } else {
            (self.scroll_x & 0xff00) | value as u16
        };
        debug!("BG Scroll X: {}", self.scroll_x);
    }

    pub fn set_scroll_y(&mut self, msb: bool, value: u8) {
        self.scroll_y = if msb {
            (self.scroll_y & 0xff) | ((value as u16 & 0x03) << 8)
        } else {
            (self.scroll_y & 0xff00) | value as u16
        };
        debug!("BG Scroll Y: {}", self.scroll_x);
    }

    pub fn set_tile_map_size(&mut self, value: u8) {
        self.tile_map_width = match (value >> 4) & 3 {
            0 => 32,
            1 => 64,
            _ => 128,
        };

        self.tile_map_height = if (value & 0x40) != 0 { 64 } else { 32 };

        debug!("BG Tile Map Width: {}", self.tile_map_width);
        debug!("BG Tile Map Height: {}", self.tile_map_height);
    }
}
