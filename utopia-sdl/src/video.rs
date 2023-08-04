use display_mode::DisplayMode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator, TextureValueError};
use sdl2::video::{Window, WindowContext};
use sdl2::Sdl;
use sdl2::VideoSubsystem;
use std::error::Error;

mod display_mode;

pub struct VideoOptions {
    pub width: u32,
    pub height: u32,
    pub clip_top: u32,
    pub clip_bottom: u32,
    pub disable_vsync: bool,
    pub full_screen: bool,
    pub upscale: Option<u32>,
}

pub struct Video {
    video: VideoSubsystem,
    width: u32,
    height: u32,
    pitch: usize,
    canvas: Canvas<Window>,
    source_rect: Rect,
    target_rect: Rect,
    display_mode: DisplayMode,
    full_screen: bool,
}

impl Video {
    pub fn new(sdl_context: &Sdl, options: VideoOptions) -> Result<Self, Box<dyn Error>> {
        let video = sdl_context.video()?;

        let pitch = options.width as usize * 4;

        let clipped_height = options.height - options.clip_top - options.clip_bottom;

        let display_mode = DisplayMode::new(options.width, clipped_height, options.upscale);

        let (window_width, window_height) =
            display_mode.window_size(&video, options.full_screen)?;

        let mut window_builder = video.window("Utopia", window_width, window_height);

        if options.full_screen {
            window_builder.fullscreen();
        } else {
            window_builder.position_centered();
        }

        let window = window_builder.allow_highdpi().build()?;

        let mut canvas_builder = window.into_canvas();

        if !options.disable_vsync {
            canvas_builder = canvas_builder.present_vsync();
        }

        let canvas = canvas_builder.build()?;

        let source_rect = Rect::new(
            0,
            options.clip_top.try_into()?,
            options.width,
            clipped_height,
        );

        let target_rect = display_mode.target_rect(&video, options.full_screen)?;

        Ok(Self {
            video,
            width: options.width,
            height: options.height,
            pitch,
            canvas,
            source_rect,
            target_rect,
            display_mode,
            full_screen: options.full_screen,
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

    pub fn on_size_changed(&mut self) -> Result<(), Box<dyn Error>> {
        self.target_rect = self
            .display_mode
            .target_rect(&self.video, self.full_screen)?;

        Ok(())
    }

    pub fn update(
        &mut self,
        texture: &mut Texture<'_>,
        pixels: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        texture.update(None, pixels, self.pitch)?;

        self.canvas.clear();
        self.canvas
            .copy(texture, self.source_rect, self.target_rect)?;
        self.canvas.present();

        Ok(())
    }
}
