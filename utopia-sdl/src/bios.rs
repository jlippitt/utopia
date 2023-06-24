use std::error::Error;
use std::fs;

const BIOS_PATH: &'static str = "./bios";

pub struct BiosLoader;

impl BiosLoader {
    pub fn new() -> Self {
        Self
    }
}

impl utopia::BiosLoader for BiosLoader {
    fn load(&self, name: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let path = format!("{}/{}.bin", BIOS_PATH, name);
        let result = fs::read(path)?;
        Ok(result)
    }
}
