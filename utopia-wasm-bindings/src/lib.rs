use std::error;
use std::fmt;
use std::path::Path;
use utopia::{DefaultMemoryMapper, Instance, InstanceOptions, SystemOptions};
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;

#[derive(Clone, Debug)]
#[wasm_bindgen]
pub struct Error {
    message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<utopia::Error> for Error {
    fn from(value: utopia::Error) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl error::Error for Error {}

pub struct BiosLoader(Option<Vec<u8>>);

impl utopia::BiosLoader for BiosLoader {
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
    instance: Box<dyn Instance>,
}

#[wasm_bindgen]
impl Utopia {
    #[wasm_bindgen(constructor)]
    pub fn new(
        rom_path: &str,
        rom_data: Vec<u8>,
        bios_data: Option<Vec<u8>>,
    ) -> Result<Utopia, Error> {
        let system_type = Path::new(rom_path).try_into().map_err(Error::from)?;

        let options = SystemOptions {
            system_type,
            bios_loader: BiosLoader(bios_data),
            memory_mapper: DefaultMemoryMapper,
            skip_boot: true,
        };

        let system = utopia::create(options).map_err(Error::from)?;

        let instance = system
            .create_instance(InstanceOptions { rom_data })
            .map_err(Error::from)?;

        Ok(Self { instance })
    }

    #[wasm_bindgen(js_name = getScreenWidth)]
    pub fn screen_width(&self) -> u32 {
        self.instance.resolution().0
    }

    #[wasm_bindgen(js_name = getScreenHeight)]
    pub fn screen_height(&self) -> u32 {
        self.instance.resolution().1
    }

    #[wasm_bindgen(js_name = getPixels)]
    pub fn pixels(&self) -> Clamped<Vec<u8>> {
        Clamped(Vec::from(self.instance.pixels()))
    }

    #[wasm_bindgen(js_name = getSampleRate)]
    pub fn sample_rate(&self) -> u32 {
        self.instance.sample_rate().try_into().unwrap()
    }

    #[wasm_bindgen(js_name = getSampleBuffer)]
    pub fn sample_buffer(&mut self) -> SampleBuffer {
        let (left, right) = if let Some(queue) = self.instance.audio_queue() {
            queue.drain(..).unzip()
        } else {
            (Vec::new(), Vec::new())
        };

        SampleBuffer { left, right }
    }

    #[wasm_bindgen(js_name = runFrame)]
    pub fn run_frame(&mut self, joypad_state: JoypadState) {
        self.instance.run_frame(&joypad_state.into())
    }
}
