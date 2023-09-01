use utopia_winit::{DefaultMemoryMapper, UtopiaWinitOptions};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsError;
use web_sys::HtmlCanvasElement;

pub struct BiosLoader(Option<Vec<u8>>);

impl utopia_winit::BiosLoader for BiosLoader {
    fn load(&self, _name: &str) -> Result<Vec<u8>, utopia_winit::Error> {
        let bios = self
            .0
            .as_ref()
            .ok_or_else(|| utopia_winit::Error("No bios provided".into()))?;

        Ok(bios.clone())
    }
}

#[wasm_bindgen]
pub fn run(
    canvas: HtmlCanvasElement,
    rom_path: &str,
    rom_data: Vec<u8>,
    bios_data: Option<Vec<u8>>,
) -> Result<(), JsError> {
    let _ = canvas;

    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    utopia_winit::run(UtopiaWinitOptions {
        rom_path: rom_path.into(),
        rom_data,
        bios_loader: Box::new(BiosLoader(bios_data)),
        memory_mapper: DefaultMemoryMapper,
        skip_boot: true,
        full_screen: false,
        sync: None,
        #[cfg(target_arch = "wasm32")]
        canvas,
    })
    .map_err(|err| JsError::new(&err.to_string()))
}
