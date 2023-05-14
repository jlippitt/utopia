use std::error::Error;
use std::path::Path;
use system::System;

mod core;
mod system;

pub fn create(rom_path: &str, rom_data: Vec<u8>) -> Result<Box<dyn System>, Box<dyn Error>> {
    let extension = Path::new(rom_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    system::create(extension, rom_data)
}
