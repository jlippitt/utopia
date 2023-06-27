use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture, TextureCreator, TextureValueError};
use sdl2::video::{Window, WindowContext};
use sdl2::Sdl;
use std::cmp;
use std::error::Error;

pub struct VideoOptions {
    pub disable_vsync: bool,
    pub upscale: Option<u32>,
}

pub struct Video {
    width: u32,
    height: u32,
    pitch: usize,
    canvas: Canvas<Window>,
}

impl Video {
    pub fn new(
        sdl_context: &Sdl,
        width: u32,
        height: u32,
        options: VideoOptions,
    ) -> Result<Self, Box<dyn Error>> {
        let video = sdl_context.video()?;

        let pitch = width as usize * 4;

        let (scaled_width, scaled_height) = if let Some(scale) = options.upscale {
            (width * scale, height * scale)
        } else {
            let display_mode = video.current_display_mode(0)?;

            let width_ratio = display_mode.w as u32 / width;
            let height_ratio = display_mode.h as u32 / width;

            let scale = cmp::min(width_ratio, height_ratio);

            (width * scale, height * scale)
        };

        let window = video
            .window("Utopia", scaled_width, scaled_height)
            .position_centered()
            .build()?;

        let mut canvas_builder = window.into_canvas();

        if !options.disable_vsync {
            canvas_builder = canvas_builder.present_vsync();
        }

        let canvas = canvas_builder.build()?;

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
        texture.update(None, pixels, self.pitch)?;

        self.canvas.clear();
        self.canvas.copy(texture, None, None)?;
        self.canvas.present();

        Ok(())
    }
}
