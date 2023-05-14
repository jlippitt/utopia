use std::path::Path;
mod system;

pub fn create(rom_path: &str, rom_data: Vec<u8>) {
    let extension = Path::new(rom_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    system::create(extension, rom_data);
}
