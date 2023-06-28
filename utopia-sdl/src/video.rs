use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator, TextureValueError};
use sdl2::video::{Window, WindowContext};
use sdl2::Sdl;
use std::cmp;
use std::error::Error;

pub struct VideoOptions {
    pub width: u32,
    pub height: u32,
    pub clip_top: u32,
    pub clip_bottom: u32,
    pub upscale: Option<u32>,
    pub disable_vsync: bool,
}

pub struct Video {
    width: u32,
    height: u32,
    pitch: usize,
    canvas: Canvas<Window>,
    src_rect: Rect,
}

impl Video {
    pub fn new(sdl_context: &Sdl, options: VideoOptions) -> Result<Self, Box<dyn Error>> {
        let video = sdl_context.video()?;

        let pitch = options.width as usize * 4;

        let clipped_height = options.height - options.clip_top - options.clip_bottom;

        let (scaled_width, scaled_height) = if let Some(scale) = options.upscale {
            (options.width * scale, clipped_height * scale)
        } else {
            let bounds = video.display_usable_bounds(0)?;

            let width_ratio = bounds.w as u32 / options.width;
            let height_ratio = bounds.h as u32 / clipped_height;

            let scale = cmp::min(width_ratio, height_ratio);

            (options.width * scale, clipped_height * scale)
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
            width: options.width,
            height: options.height,
            pitch,
            canvas,
            src_rect: Rect::new(
                0,
                options.clip_top.try_into()?,
                options.width,
                clipped_height,
            ),
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
        self.canvas.copy(texture, self.src_rect, None)?;
        self.canvas.present();

        Ok(())
    }
}
