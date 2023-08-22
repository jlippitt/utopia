use tracing::debug;

pub struct Pipeline {
    //
}

impl Pipeline {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&mut self, _ram: &[u8], commands: &[u64]) {
        let mut iter = commands.iter();

        while let Some(command) = iter.next() {
            match (command >> 56) as u8 {
                0x24 => debug!("TextureRectangle {:016X}", iter.next().unwrap()),
                0x28 => debug!("SyncTile"),
                0x2d => debug!("SetScissor"),
                0x2f => debug!("SetOtherModes"),
                0x30 => debug!("LoadTlut"),
                0x34 => debug!("LoadTile"),
                0x35 => debug!("SetTile"),
                0x36 => debug!("FillRectangle"),
                0x37 => debug!("SetFillColor"),
                0x3c => debug!("SetCombineMode"),
                0x3d => debug!("SetTextureImage"),
                0x3f => debug!("SetColorImage"),
                opcode => debug!("{:02X}", opcode),
            }
        }
    }
}
