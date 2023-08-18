use std::error::Error;
use std::fmt;
use utopia::{Options, System};
use wasm_bindgen::prelude::*;

#[derive(Debug)]
#[wasm_bindgen]
pub struct UtopiaError {
    message: String,
}

#[wasm_bindgen]
impl UtopiaError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for UtopiaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for UtopiaError {}

pub struct BiosLoader;

impl utopia::BiosLoader for BiosLoader {
    fn load(&self, _name: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Err(Box::new(UtopiaError::new(
            "BIOS loading not supported yet in WASM front-end",
        )))
    }
}

pub struct MemoryMapper;

impl utopia::MemoryMapper for MemoryMapper {
    type Mapped = Vec<u8>;

    fn open(&self, len: usize, _battery_backed: bool) -> Result<Vec<u8>, Box<dyn Error>> {
        // TODO: Some way to save games?
        Ok(vec![0; len])
    }
}

#[wasm_bindgen]
pub struct Utopia {
    system: Box<dyn System>,
}

#[wasm_bindgen]
impl Utopia {
    #[wasm_bindgen(constructor)]
    pub fn new(rom_path: &str, rom_data: Vec<u8>) -> Result<Utopia, UtopiaError> {
        let options = Options {
            bios_loader: BiosLoader,
            memory_mapper: MemoryMapper,
            skip_boot: true,
        };

        let system = utopia::create(rom_data, rom_path, &options)
            .map_err(|err| UtopiaError::new(err.to_string()))?;

        Ok(Self { system })
    }

    #[wasm_bindgen(js_name = getScreenWidth)]
    pub fn screen_width(&self) -> u32 {
        self.system.screen_width()
    }

    #[wasm_bindgen(js_name = getScreenHeight)]
    pub fn screen_height(&self) -> u32 {
        self.system.screen_height()
    }
}
