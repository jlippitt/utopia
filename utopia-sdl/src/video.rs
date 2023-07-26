use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator, TextureValueError};
use sdl2::video::{Window, WindowContext};
use sdl2::Sdl;
use sdl2::VideoSubsystem;
use std::cmp;
use std::error::Error;

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
    full_screen: bool,
    upscale: Option<u32>,
}

impl Video {
    pub fn new(sdl_context: &Sdl, options: VideoOptions) -> Result<Self, Box<dyn Error>> {
        let video = sdl_context.video()?;

        let pitch = options.width as usize * 4;

        let clipped_height = options.height - options.clip_top - options.clip_bottom;

        let (window_width, window_height) = calculate_window_size(
            &video,
            options.width,
            clipped_height,
            options.full_screen,
            options.upscale,
        )?;

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

        let target_rect = calculate_target_rect(
            &video,
            options.width,
            clipped_height,
            options.full_screen,
            options.upscale,
        )?;

        Ok(Self {
            video,
            width: options.width,
            height: options.height,
            pitch,
            canvas,
            source_rect,
            target_rect,
            full_screen: options.full_screen,
            upscale: options.upscale,
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
        self.target_rect = calculate_target_rect(
            &self.video,
            self.source_rect.width(),
            self.source_rect.height(),
            self.full_screen,
            self.upscale,
        )?;

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

fn calculate_window_size(
    video: &VideoSubsystem,
    width: u32,
    height: u32,
    full_screen: bool,
    upscale: Option<u32>,
) -> Result<(u32, u32), Box<dyn Error>> {
    if !full_screen {
        let rect = calculate_target_rect(video, width, height, full_screen, upscale)?;
        return Ok((rect.width(), rect.height()));
    }

    let mut max_scale: u32 = 0;

    for index in 0..video.num_display_modes(0)? {
        let display_mode = video.display_mode(0, index)?;

        if display_mode.format.byte_size_per_pixel() < 3 {
            // Needs at least 24-bit colour to be considered
            continue;
        }

        let width_ratio = display_mode.w as u32 / width;
        let height_ratio = display_mode.h as u32 / height;
        let scale = cmp::min(width_ratio, height_ratio);

        if scale <= max_scale {
            continue;
        }

        max_scale = scale;
    }

    Ok(apply_upscale(max_scale, upscale, width, height))
}

fn calculate_target_rect(
    video: &VideoSubsystem,
    width: u32,
    height: u32,
    full_screen: bool,
    upscale: Option<u32>,
) -> Result<Rect, Box<dyn Error>> {
    let display_bounds = if full_screen {
        video.display_bounds(0)?
    } else {
        video.display_usable_bounds(0)?
    };

    let width_ratio = display_bounds.width() / width;
    let height_ratio = display_bounds.height() / height;
    let max_scale = cmp::min(width_ratio, height_ratio);

    let (scaled_width, scaled_height) = apply_upscale(max_scale, upscale, width, height);

    let (offset_x, offset_y) = if full_screen {
        (
            (display_bounds.width() - scaled_width) >> 1,
            (display_bounds.height() - scaled_height) >> 1,
        )
    } else {
        (0, 0)
    };

    let rect = Rect::new(
        offset_x as i32,
        offset_y as i32,
        scaled_width,
        scaled_height,
    );

    return Ok(rect);
}

fn apply_upscale(max_scale: u32, upscale: Option<u32>, width: u32, height: u32) -> (u32, u32) {
    let scale = upscale.map_or(max_scale, |scale| cmp::min(scale, max_scale));
    let scaled_width = width * scale;
    let scaled_height = height * scale;
    (scaled_width, scaled_height)
}
