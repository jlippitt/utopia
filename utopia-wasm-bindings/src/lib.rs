use std::rc::Rc;
use utopia_winit::{App, DefaultMemoryMapper, ResetOptions};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsError;
use web_sys::HtmlCanvasElement;

#[derive(Debug)]
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
pub struct Utopia {
    app: App<DefaultMemoryMapper>,
}

#[wasm_bindgen]
impl Utopia {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Utopia, JsError> {
        Ok(Self { app: App::new() })
    }

    pub fn reset(
        &mut self,
        canvas: HtmlCanvasElement,
        rom_path: &str,
        rom_data: Vec<u8>,
        bios_data: Option<Vec<u8>>,
    ) -> Result<(), JsError> {
        let _ = canvas;

        self.app
            .reset(ResetOptions {
                bios_loader: Rc::new(BiosLoader(bios_data)),
                memory_mapper: DefaultMemoryMapper,
                rom_path: rom_path.into(),
                rom_data,
                skip_boot: true,
                full_screen: false,
                sync: None,
                #[cfg(target_arch = "wasm32")]
                canvas,
            })
            .map_err(|err| JsError::new(&err.to_string()))?;

        Ok(())
    }

    #[wasm_bindgen(js_name = updateViewport)]
    pub fn update_viewport(&mut self) -> Result<(), JsError> {
        self.app
            .update_viewport()
            .map_err(|err| JsError::new(&err.to_string()))
    }
}

#[wasm_bindgen(start)]
fn on_load() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }
}
