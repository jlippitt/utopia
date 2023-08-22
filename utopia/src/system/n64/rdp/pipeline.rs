use tracing::debug;

pub struct Pipeline {
    //
}

impl Pipeline {
    pub fn new() -> Self {
        Self {}
    }

    pub fn step(&mut self, _ram: &[u8], command: u64) {
        match (command >> 56) as u8 {
            opcode => debug!("{:02X}", opcode),
        }
    }
}
