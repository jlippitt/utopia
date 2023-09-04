use super::super::Pipeline;
use crate::WgpuContext;
use bitfield_struct::bitfield;
use tracing::debug;

#[bitfield(u64)]
pub struct FillTriangle {
    #[bits(15)]
    yh: u16,
    __: bool,
    #[bits(15)]
    ym: u16,
    __: bool,
    #[bits(15)]
    yl: u16,
    __: bool,
    #[bits(3)]
    tile: u8,
    #[bits(3)]
    level: u8,
    __: bool,
    dir: bool,
    __: u8,
}

impl FillTriangle {
    pub fn exec(&self, pipeline: &mut Pipeline, _rdram: &mut [u8], _ctx: &WgpuContext) {
        debug!("{:?}", self);
        let (xl, dxldy) = edge(pipeline.next_word());
        debug!(" XL: ({}, {})", xl, dxldy);
        let (xh, dxhdy) = edge(pipeline.next_word());
        debug!(" XH: ({}, {})", xh, dxhdy);
        let (xm, dxmdy) = edge(pipeline.next_word());
        debug!(" XM: ({}, {})", xm, dxmdy);
    }
}

fn edge(word: u64) -> (f32, f32) {
    let x = (word >> 32) as f32 / 65536.0;
    let dxdy = (word & 0xffff_ffff) as f32 / 65536.0;
    (x, dxdy)
}
