const VRAM_SIZE: usize = 98304;

pub struct Ppu {
    vram: Vec<u8>,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            vram: vec![0; VRAM_SIZE],
        }
    }

    pub fn vram(&self) -> &[u8] {
        &self.vram
    }

    pub fn vram_mut(&mut self) -> &mut [u8] {
        &mut self.vram
    }
}
