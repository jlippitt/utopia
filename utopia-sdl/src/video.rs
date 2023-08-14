use sdl2::mouse::MouseUtil;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator, TextureValueError};
use sdl2::video::{FullscreenType, Window, WindowContext};
use sdl2::Sdl;
use sdl2::VideoSubsystem;
use std::error::Error;
use utopia::System;
use viewport::Viewport;

mod viewport;

pub struct VideoOptions {
    pub disable_vsync: bool,
    pub full_screen: bool,
    pub upscale: Option<u32>,
}

pub struct Video {
    video: VideoSubsystem,
    mouse: MouseUtil,
    canvas: Canvas<Window>,
    viewport: Viewport,
    source_rect: Rect,
    target_rect: Rect,
    full_screen: bool,
}

impl Video {
    pub fn new(
        sdl_context: &Sdl,
        system: &dyn System,
        options: VideoOptions,
    ) -> Result<Self, Box<dyn Error>> {
        let video = sdl_context.video()?;
        let mouse = sdl_context.mouse();

        let clipped_height =
            system.screen_height() - system.screen_clip_top() - system.screen_clip_bottom();

        let display_mode = Viewport::new(system.screen_width(), clipped_height, options.upscale);

        let (window_width, window_height) =
            display_mode.window_size(&video, options.full_screen)?;

        let mut window_builder = video.window("Utopia", window_width, window_height);

        if options.full_screen {
            window_builder.fullscreen();
        } else {
            window_builder.position_centered();
        }

        let window = window_builder.allow_highdpi().build()?;

        mouse.show_cursor(!options.full_screen);

        let mut canvas_builder = window.into_canvas();

        if !options.disable_vsync {
            canvas_builder = canvas_builder.present_vsync();
        }

        let canvas = canvas_builder.build()?;

        let source_rect = Rect::new(
            0,
            system.screen_clip_top().try_into()?,
            system.screen_width(),
            clipped_height,
        );

        let target_rect = display_mode.target_rect(&video, options.full_screen)?;

        Ok(Self {
            video,
            mouse,
            canvas,
            viewport: display_mode,
            source_rect,
            target_rect,
            full_screen: options.full_screen,
        })
    }

    pub fn texture_creator(&self) -> TextureCreator<WindowContext> {
        self.canvas.texture_creator()
    }

    pub fn create_texture<'a>(
        &mut self,
        texture_creator: &'a TextureCreator<WindowContext>,
        screen_width: u32,
        screen_height: u32,
    ) -> Result<Texture<'a>, TextureValueError> {
        texture_creator.create_texture_streaming(
            PixelFormatEnum::BGR888,
            screen_width,
            screen_height,
        )
    }

    pub fn toggle_full_screen(&mut self) -> Result<(), String> {
        self.full_screen = !self.full_screen;

        self.mouse.show_cursor(!self.full_screen);

        let full_screen_type = if self.full_screen {
            FullscreenType::True
        } else {
            FullscreenType::Off
        };

        self.canvas.window_mut().set_fullscreen(full_screen_type)?;

        Ok(())
    }

    pub fn on_size_changed(&mut self) -> Result<(), Box<dyn Error>> {
        self.target_rect = self.viewport.target_rect(&self.video, self.full_screen)?;

        Ok(())
    }

    pub fn update(
        &mut self,
        texture: &mut Texture<'_>,
        pixels: &[u8],
        pitch: usize,
    ) -> Result<(), Box<dyn Error>> {
        texture.update(None, pixels, pitch)?;

        self.canvas.clear();
        self.canvas
            .copy(texture, self.source_rect, self.target_rect)?;
        self.canvas.present();

        Ok(())
    }
}
