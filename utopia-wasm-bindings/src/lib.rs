use std::error;
use std::fmt;
use utopia::{CreateOptions, DefaultMemoryMapper, System};
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;

#[derive(Clone, Debug)]
#[wasm_bindgen]
pub struct Error {
    message: String,
}

#[wasm_bindgen]
impl Error {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for Error {}

pub struct BiosLoader(Option<Vec<u8>>);

impl utopia::BiosLoader for BiosLoader {
    type Error = utopia::Error;

    fn load(&self, _name: &str) -> Result<Vec<u8>, utopia::Error> {
        let bios = self
            .0
            .as_ref()
            .ok_or_else(|| utopia::Error("No bios provided".into()))?;

        Ok(bios.clone())
    }
}

#[derive(Default)]
#[wasm_bindgen]
pub struct JoypadState {
    inner: utopia::JoypadState,
}

#[wasm_bindgen]
impl JoypadState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    #[wasm_bindgen(js_name = getAxis)]
    pub fn axis(&self, index: usize) -> f64 {
        self.inner.axes[index] as f64 / i32::MAX as f64
    }

    #[wasm_bindgen(js_name = setAxis)]
    pub fn set_axis(&mut self, index: usize, value: f64) {
        self.inner.axes[index] = (value * i32::MAX as f64) as i32;
    }

    #[wasm_bindgen(js_name = getButton)]
    pub fn button(&self, index: usize) -> bool {
        self.inner.buttons[index]
    }

    #[wasm_bindgen(js_name = setButton)]
    pub fn set_button(&mut self, index: usize, value: bool) {
        self.inner.buttons[index] = value;
    }
}

impl From<JoypadState> for utopia::JoypadState {
    fn from(value: JoypadState) -> Self {
        value.inner
    }
}

#[wasm_bindgen]
pub struct SampleBuffer {
    left: Vec<f32>,
    right: Vec<f32>,
}

#[wasm_bindgen]
impl SampleBuffer {
    #[wasm_bindgen(js_name = getLeft)]
    pub fn left(&self) -> Vec<f32> {
        self.left.clone()
    }

    #[wasm_bindgen(js_name = getRight)]
    pub fn right(&self) -> Vec<f32> {
        self.right.clone()
    }
}

#[wasm_bindgen]
pub struct Utopia {
    system: Box<dyn System>,
}

#[wasm_bindgen]
impl Utopia {
    #[wasm_bindgen(constructor)]
    pub fn new(
        rom_path: &str,
        rom_data: Vec<u8>,
        bios_data: Option<Vec<u8>>,
    ) -> Result<Utopia, Error> {
        let options = CreateOptions {
            bios_loader: BiosLoader(bios_data),
            memory_mapper: DefaultMemoryMapper,
            skip_boot: true,
        };

        let system = utopia::create(rom_path, rom_data, &options)
            .map_err(|err| Error::new(err.to_string()))?;

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

    #[wasm_bindgen(js_name = getPixels)]
    pub fn pixels(&self) -> Clamped<Vec<u8>> {
        Clamped(Vec::from(self.system.pixels()))
    }

    #[wasm_bindgen(js_name = getSampleRate)]
    pub fn sample_rate(&self) -> u32 {
        self.system.sample_rate().try_into().unwrap()
    }

    #[wasm_bindgen(js_name = getSampleBuffer)]
    pub fn sample_buffer(&mut self) -> SampleBuffer {
        let (left, right) = if let Some(queue) = self.system.audio_queue() {
            queue.drain(..).unzip()
        } else {
            (Vec::new(), Vec::new())
        };

        SampleBuffer { left, right }
    }

    #[wasm_bindgen(js_name = runFrame)]
    pub fn run_frame(&mut self, joypad_state: JoypadState) {
        self.system.run_frame(&joypad_state.into())
    }
}
