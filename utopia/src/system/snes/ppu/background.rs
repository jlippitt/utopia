use tracing::debug;

#[repr(u16)]
#[derive(Copy, Clone)]
enum MirrorMask {
    Mask32 = 31,
    Mask64 = 63,
}

#[repr(u16)]
#[derive(Copy, Clone)]
enum NameShift {
    Shift32 = 5,
    Shift64 = 6,
}

pub struct BackgroundLayer {
    tile_map: u16,
    mirror_mask_x: MirrorMask,
    mirror_mask_y: MirrorMask,
    name_shift_y: NameShift,
    chr_map: u16,
    scroll_x: u16,
    scroll_y: u16,
    id: u32,
}

impl BackgroundLayer {
    pub fn new(id: u32) -> Self {
        Self {
            tile_map: 0,
            mirror_mask_x: MirrorMask::Mask32,
            mirror_mask_y: MirrorMask::Mask32,
            name_shift_y: NameShift::Shift32,
            chr_map: 0,
            scroll_x: 0,
            scroll_y: 0,
            id,
        }
    }

    pub fn set_tile_map(&mut self, value: u8) {
        let mirror_x = (value & 0x01) == 0;
        let mirror_y = (value & 0x02) == 0;

        self.mirror_mask_x = if mirror_x {
            MirrorMask::Mask32
        } else {
            MirrorMask::Mask64
        };

        self.mirror_mask_y = if mirror_y {
            MirrorMask::Mask32
        } else {
            MirrorMask::Mask64
        };

        self.name_shift_y = if mirror_x || mirror_y {
            NameShift::Shift32
        } else {
            NameShift::Shift64
        };

        self.tile_map = ((value & 0xfc) as u16) << 8;

        debug!("BG{} Tile Map: {:04X}", self.id, self.tile_map);
        debug!("BG{} Mirror Mask X: {}", self.id, self.mirror_mask_x as u16);
        debug!("BG{} Mirror Mask Y: {}", self.id, self.mirror_mask_y as u16);
        debug!("BG{} Name Shift Y: {}", self.id, self.name_shift_y as u16);
    }

    pub fn set_chr_map(&mut self, value: u8) {
        self.tile_map = (value as u16) << 8;
        debug!("BG{} CHR Map: {:04X}", self.id, self.chr_map);
    }

    pub fn set_scroll_x(&mut self, regs: &mut (u8, u8), value: u8) {
        self.scroll_x =
            (((value & 0x03) as u16) << 8) | ((regs.0 & 0xf8) as u16) | ((regs.1 & 0x07) as u16);

        regs.0 = value;
        regs.1 = value;

        debug!("BG{} Scroll X: {}", self.id, self.scroll_x);
    }

    pub fn set_scroll_y(&mut self, regs: &mut (u8, u8), value: u8) {
        self.scroll_y = (((value & 0x03) as u16) << 8) | (regs.0 as u16);

        regs.0 = value;

        debug!("BG{} Scroll Y: {}", self.id, self.scroll_y);
    }
}
