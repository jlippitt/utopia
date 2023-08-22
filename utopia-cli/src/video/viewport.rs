use sdl2::rect::Rect;
use sdl2::VideoSubsystem;
use std::cmp;
use std::error::Error;

pub struct Viewport {
    width: u32,
    height: u32,
    upscale: Option<u32>,
}

impl Viewport {
    pub fn new(width: u32, height: u32, upscale: Option<u32>) -> Self {
        Self {
            width,
            height,
            upscale,
        }
    }

    pub fn set_base_resolution(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn window_size(
        &self,
        video: &VideoSubsystem,
        full_screen: bool,
    ) -> Result<(u32, u32), Box<dyn Error>> {
        if !full_screen {
            let rect = self.target_rect(video, full_screen)?;
            return Ok((rect.width(), rect.height()));
        }

        let mut max_scale: u32 = 0;

        for index in 0..video.num_display_modes(0)? {
            let display_mode = video.display_mode(0, index)?;

            if display_mode.format.byte_size_per_pixel() < 3 {
                // Needs at least 24-bit colour to be considered
                continue;
            }

            let width_ratio = display_mode.w as u32 / self.width;
            let height_ratio = display_mode.h as u32 / self.height;
            let scale = cmp::min(width_ratio, height_ratio);

            if scale <= max_scale {
                continue;
            }

            max_scale = scale;
        }

        Ok(self.apply_upscale(max_scale))
    }

    pub fn target_rect(
        &self,
        video: &VideoSubsystem,
        full_screen: bool,
    ) -> Result<Rect, Box<dyn Error>> {
        let display_bounds = if full_screen {
            video.display_bounds(0)?
        } else {
            video.display_usable_bounds(0)?
        };

        let width_ratio = display_bounds.width() / self.width;
        let height_ratio = display_bounds.height() / self.height;
        let max_scale = cmp::min(width_ratio, height_ratio);

        let (scaled_width, scaled_height) = self.apply_upscale(max_scale);

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

        Ok(rect)
    }

    fn apply_upscale(&self, max_scale: u32) -> (u32, u32) {
        let scale = self
            .upscale
            .map_or(max_scale, |scale| cmp::min(scale, max_scale));
        let scaled_width = self.width * scale;
        let scaled_height = self.height * scale;
        (scaled_width, scaled_height)
    }
}
