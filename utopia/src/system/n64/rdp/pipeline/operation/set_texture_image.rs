use super::super::Pipeline;
use crate::WgpuContext;
use bitfield_struct::bitfield;
use tracing::debug;

#[bitfield(u64)]
pub struct SetTextureImage {
    #[bits(26)]
    dram_address: u32,
    #[bits(6)]
    __: u8,
    #[bits(10)]
    width: u16,
    #[bits(9)]
    __: u16,
    #[bits(2)]
    size: u8,
    #[bits(3)]
    format: u8,
    __: u8,
}

impl SetTextureImage {
    pub fn exec(&self, _pipeline: &mut Pipeline, _rdram: &mut [u8], _ctx: &WgpuContext) {
        debug!("{:?}", self);
    }
}
