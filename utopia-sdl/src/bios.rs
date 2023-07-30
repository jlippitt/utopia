use std::error::Error;
use std::fs;
use tracing::warn;

const BIOS_PATH: &str = "./bios";

pub struct BiosLoader;

impl BiosLoader {
    pub fn new() -> Self {
        Self
    }
}

impl utopia::BiosLoader for BiosLoader {
    fn load(&self, name: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let path = format!("{}/{}.bin", BIOS_PATH, name);

        let bios_data = fs::read(&path).map_err(|err| {
            warn!("Failed to load BIOS file '{}': {}", path, err);
            err
        });

        Ok(bios_data?)
    }
}
