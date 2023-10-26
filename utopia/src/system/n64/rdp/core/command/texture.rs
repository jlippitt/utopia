use super::super::primitive::{Rectangle, TextureLayout};
use super::super::tmem::{SetTileParams, TextureImage};
use super::super::Core;
use super::Image;
use bitfield_struct::bitfield;
use tracing::trace;

pub fn set_texture_image(core: &mut Core, _rdram: &mut [u8], cmd: u64) {
    let op = Image::from(cmd);
    trace!("SET_TEXTURE_IMAGE {:?}", op);
    core.tmem.set_texture_image(TextureImage {
        format: (op.format(), op.size()).into(),
        dram_addr: op.dram_addr() as usize,
        dram_width: op.width() as usize,
    });
}

pub fn set_tile(core: &mut Core, _rdram: &mut [u8], cmd: u64) {
    let op = SetTile::from(cmd);
    trace!("SET_TILE {:?}", op);
    core.tmem.set_tile(
        op.tile(),
        SetTileParams {
            format: (op.format(), op.size()).into(),
            tmem_addr: op.tmem_addr(),
        },
    )
}

pub fn set_tile_size(core: &mut Core, _rdram: &mut [u8], cmd: u64) {
    let op = TileSize::from(cmd);
    trace!("SET_TILE_SIZE {:?}", op);
    core.tmem.set_tile_size(
        op.tile(),
        // H and L reversed for consistency with other rectangles
        &Rectangle {
            xh: op.sl() as f32 / 4.0,
            yh: op.tl() as f32 / 4.0,
            xl: op.sh() as f32 / 4.0,
            yl: op.th() as f32 / 4.0,
        },
    )
}

pub fn load_tile(core: &mut Core, rdram: &mut [u8], cmd: u64) {
    let op = TileSize::from(cmd);
    trace!("LOAD_TILE {:?}", op);
    core.tmem.load_tile(
        op.tile(),
        rdram,
        // H and L reversed for consistency with other rectangles
        &Rectangle {
            xh: op.sl() as f32 / 4.0,
            yh: op.tl() as f32 / 4.0,
            xl: op.sh() as f32 / 4.0,
            yl: op.th() as f32 / 4.0,
        },
    )
}

pub fn load_block(core: &mut Core, rdram: &mut [u8], cmd: u64) {
    let op = TileSize::from(cmd);
    trace!("LOAD_BLOCK {:?}", op);
    core.tmem
        .load_block(op.tile(), rdram, op.sl(), op.tl(), op.sh(), op.th())
}

pub fn sync_tile(_core: &mut Core, _rdram: &mut [u8], _cmd: u64) {
    trace!("SYNC_TILE");
}

#[bitfield(u64)]
struct SetTile {
    #[bits(4)]
    shift_s: u32,
    #[bits(4)]
    mask_s: u32,
    mirror_s: bool,
    clamp_s: bool,
    #[bits(4)]
    shift_t: u32,
    #[bits(4)]
    mask_t: u32,
    mirror_t: bool,
    clamp_t: bool,
    #[bits(4)]
    palette: u32,
    #[bits(3)]
    tile: u32,
    #[bits(5)]
    __: u32,
    #[bits(9)]
    tmem_addr: u32,
    #[bits(9)]
    line: u32,
    __: bool,
    #[bits(2)]
    size: u32,
    #[bits(3)]
    format: TextureLayout,
    __: u8,
}

#[bitfield(u64)]
struct TileSize {
    #[bits(12)]
    th: u32,
    #[bits(12)]
    sh: u32,
    #[bits(3)]
    tile: u32,
    #[bits(5)]
    __: u32,
    #[bits(12)]
    tl: u32,
    #[bits(12)]
    sl: u32,
    __: u8,
}
