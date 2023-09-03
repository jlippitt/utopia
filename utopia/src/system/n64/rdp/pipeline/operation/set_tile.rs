use super::super::Pipeline;
use crate::WgpuContext;
use bitfield_struct::bitfield;
use tracing::debug;

#[bitfield(u64)]
pub struct SetTile {
    #[bits(4)]
    shift_s: u8,
    #[bits(4)]
    mask_s: u8,
    mirror_s: bool,
    clamp_s: bool,
    #[bits(4)]
    shift_t: u8,
    #[bits(4)]
    mask_t: u8,
    mirror_t: bool,
    clamp_t: bool,
    #[bits(4)]
    palette: u8,
    #[bits(3)]
    tile: u8,
    #[bits(5)]
    __: u8,
    #[bits(9)]
    tmem_address: u16,
    #[bits(9)]
    line: u16,
    __: bool,
    #[bits(2)]
    size: u8,
    #[bits(3)]
    format: u8,
    __: u8,
}

impl SetTile {
    pub fn exec(&self, _pipeline: &mut Pipeline, _rdram: &mut [u8], _ctx: &WgpuContext) {
        debug!("{:?}", self);
    }
}
