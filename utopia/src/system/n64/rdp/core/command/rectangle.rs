use super::super::primitive::Rectangle;
use super::super::Core;
use bitfield_struct::bitfield;
use tracing::trace;

pub fn fill_rectangle(core: &mut Core, _rdram: &mut [u8], cmd: u64) {
    let (rect, _) = rectangle("FILL_RECTANGLE: {:?}", cmd);
    core.push_rect(rect, None);
}

pub fn texture_rectangle(
    core: &mut Core,
    _rdram: &mut [u8],
    cmd: u64,
    reader: &mut impl Iterator<Item = u64>,
) {
    let (rect, tile) = rectangle("TEXTURE_RECTANGLE: {:?}", cmd);
    let bounds = texture(reader, false, &rect);
    core.push_rect(rect, Some((tile, bounds)));
}

pub fn texture_rectangle_flip(
    core: &mut Core,
    _rdram: &mut [u8],
    cmd: u64,
    reader: &mut impl Iterator<Item = u64>,
) {
    let (rect, tile) = rectangle("TEXTURE_RECTANGLE_FLIP: {:?}", cmd);
    let bounds = texture(reader, true, &rect);
    core.push_rect(rect, Some((tile, bounds)));
}

fn rectangle(name: &'static str, cmd: u64) -> (Rectangle, u32) {
    let op = RectangleRaw::from(cmd);

    trace!("{}: {:?}", name, op);

    let rect = Rectangle {
        xh: op.xh() as f32 / 4.0,
        yh: op.yh() as f32 / 4.0,
        xl: op.xl() as f32 / 4.0,
        yl: op.yl() as f32 / 4.0,
    };

    (rect, op.tile())
}

fn texture(reader: &mut impl Iterator<Item = u64>, flip: bool, rect: &Rectangle) -> Rectangle {
    let word = reader.next().unwrap();

    let start_s = bound(word, 48, 32.0);
    let start_t = bound(word, 32, 32.0);
    let dsdx = bound(word, 16, 1024.0);
    let dtdy = bound(word, 0, 1024.0);

    let end_s = start_s + (rect.xl - rect.xh) * dsdx;
    let end_t = start_t + (rect.yl - rect.yh) * dtdy;

    let bounds = if flip {
        Rectangle {
            xh: start_t,
            yh: start_s,
            xl: end_t,
            yl: end_s,
        }
    } else {
        Rectangle {
            xh: start_s,
            yh: start_t,
            xl: end_s,
            yl: end_t,
        }
    };

    trace!("  Texture: {:?}", bounds);

    bounds
}

fn bound(word: u64, shift: u32, divisor: f32) -> f32 {
    let raw = (word >> shift) as u32 & 0xffff;
    let value = (raw & 0x7fff) as f32 / divisor;

    if (word & 0x8000) != 0 {
        -value
    } else {
        value
    }
}

#[bitfield(u64)]
struct RectangleRaw {
    #[bits(12)]
    yh: i32,
    #[bits(12)]
    xh: i32,
    #[bits(3)]
    tile: u32,
    #[bits(5)]
    __: u32,
    #[bits(12)]
    yl: i32,
    #[bits(12)]
    xl: i32,
    __: u8,
}
