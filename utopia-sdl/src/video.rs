use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture, TextureCreator, TextureValueError};
use sdl2::video::{Window, WindowContext};
use sdl2::Sdl;
use std::error::Error;

pub struct Video {
    width: u32,
    height: u32,
    pitch: usize,
    canvas: Canvas<Window>,
}

impl Video {
    pub fn new(sdl_context: &Sdl, width: u32, height: u32) -> Result<Self, Box<dyn Error>> {
        let video = sdl_context.video()?;

        let pitch = width as usize * 4;

        let window = video
            .window("Utopia", width, height)
            .position_centered()
            .build()?;

        let canvas = window.into_canvas().present_vsync().build()?;

        Ok(Self {
            width,
            height,
            pitch,
            canvas,
        })
    }

    pub fn texture_creator(&self) -> TextureCreator<WindowContext> {
        self.canvas.texture_creator()
    }

    pub fn create_texture<'a>(
        &mut self,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Result<Texture<'a>, TextureValueError> {
        texture_creator.create_texture_streaming(PixelFormatEnum::BGR888, self.width, self.height)
    }

    pub fn update(
        &mut self,
        texture: &mut Texture<'_>,
        pixels: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        texture.update(None, &pixels, self.pitch)?;

        self.canvas.clear();
        self.canvas.copy(&texture, None, None)?;
        self.canvas.present();

        Ok(())
    }
}
