use crate::WgpuContext;
use tracing::debug;

mod operation;

#[derive(Default)]
pub struct Pipeline;

impl Pipeline {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&mut self, rdram: &mut [u8], ctx: &WgpuContext, commands: &[u64]) {
        use operation as op;

        let mut iter = commands.iter();

        while let Some(&word) = iter.next() {
            match (word >> 56) as u8 {
                0x08 => debug!(
                    "FillTriangle {:016X} {:016X} {:016X}",
                    iter.next().unwrap(),
                    iter.next().unwrap(),
                    iter.next().unwrap(),
                ),
                0x24 => debug!("TextureRectangle {:016X}", iter.next().unwrap()),
                0x27 => debug!("SyncPipe"),
                0x28 => debug!("SyncTile"),
                0x29 => debug!("SyncFull"),
                0x2d => debug!("SetScissor"),
                0x2f => debug!("SetOtherModes"),
                0x30 => debug!("LoadTlut"),
                0x34 => debug!("LoadTile"),
                0x35 => op::SetTile::from(word).exec(self, rdram, ctx),
                0x36 => debug!("FillRectangle"),
                0x37 => debug!("SetFillColor"),
                0x39 => debug!("SetBlendColor"),
                0x3c => debug!("SetCombineMode"),
                0x3d => op::SetTextureImage::from(word).exec(self, rdram, ctx),
                0x3f => debug!("SetColorImage"),
                opcode => debug!("{:02X}", opcode),
            }
        }
    }
}
