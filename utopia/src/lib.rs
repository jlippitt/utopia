use std::path::Path;
use tracing::debug;

pub fn create(rom_path: &str, rom_data: Vec<u8>) {
    let extension = Path::new(rom_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match extension {
        "nes" => debug!("{:?}", rom_data),
        _ => panic!("Unsupported ROM type"),
    }
}
