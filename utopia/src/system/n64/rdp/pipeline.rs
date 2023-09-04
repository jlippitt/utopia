use crate::WgpuContext;
use std::array;
use std::collections::VecDeque;
use tracing::debug;

mod operation;

#[derive(Default)]
pub struct Pipeline {
    commands: VecDeque<u64>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            commands: VecDeque::new(),
        }
    }

    pub fn upload(&mut self, commands: &[u8]) {
        self.commands.clear();

        for chunk in commands.chunks_exact(8) {
            let bytes: [u8; 8] = array::from_fn(|index| chunk[index]);
            self.commands.push_back(u64::from_be_bytes(bytes));
        }
    }

    pub fn next_word(&mut self) -> u64 {
        self.commands.pop_front().unwrap()
    }

    pub fn run(&mut self, rdram: &mut [u8], ctx: &WgpuContext) {
        use operation as op;

        while let Some(word) = self.commands.pop_front() {
            match (word >> 56) as u8 {
                0x08 => op::FillTriangle::from(word).exec(self, rdram, ctx),
                0x24 => debug!("TextureRectangle {:016X}", self.next_word()),
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
                0x3d => op::FillTriangle::from(word).exec(self, rdram, ctx),
                0x3f => debug!("SetColorImage"),
                opcode => debug!("{:02X}", opcode),
            }
        }
    }
}
