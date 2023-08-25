use renderer::Renderer;
use std::error::Error;
use winit::dpi::{PhysicalSize, Size};
use winit::error::OsError;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Fullscreen, Window, WindowBuilder};

mod geometry;
mod renderer;

struct Projection {
    window: Window,
    target_size: PhysicalSize<u32>,
    clip_rect: Option<[[f32; 2]; 4]>,
}

pub struct VideoController {
    window: Window,
    renderer: Renderer,
    source_size: PhysicalSize<u32>,
    full_screen: bool,
}

impl VideoController {
    pub fn new(
        window_target: &EventLoopWindowTarget<()>,
        source_size: PhysicalSize<u32>,
        full_screen: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let Projection {
            window,
            target_size,
            clip_rect,
        } = Projection::new(window_target, source_size, full_screen)?;

        let renderer = Renderer::new(&window, source_size, target_size, clip_rect)?;

        Ok(Self {
            window,
            renderer,
            source_size,
            full_screen,
        })
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn set_source_size(&mut self, size: PhysicalSize<u32>) -> Result<(), Box<dyn Error>> {
        // self.viewport
        //     .set_base_resolution(screen_width, screen_height);

        //let window_size = self.viewport.window_size(&self.video, self.full_screen)?;

        //self.window.set_size(window_size.0, window_size.1)?;

        self.window
            .set_inner_size(PhysicalSize::new(size.width, size.height));

        Ok(())
    }

    pub fn toggle_full_screen(
        &mut self,
        window_target: &EventLoopWindowTarget<()>,
    ) -> Result<(), Box<dyn Error>> {
        self.full_screen = !self.full_screen;

        let Projection {
            window,
            target_size,
            clip_rect,
        } = Projection::new(window_target, self.source_size, self.full_screen)?;

        self.window = window;
        self.renderer = Renderer::new(&self.window, self.source_size, target_size, clip_rect)?;

        Ok(())
    }

    pub fn on_window_size_changed(&mut self) -> Result<(), Box<dyn Error>> {
        let new_size = self.window.inner_size();
        self.renderer.on_window_size_changed(new_size)?;
        Ok(())
    }

    pub fn render(&mut self, pixels: &[u8], pitch: usize) -> Result<(), Box<dyn Error>> {
        self.renderer.render(pixels, pitch)
    }
}

impl Projection {
    pub fn new(
        window_target: &EventLoopWindowTarget<()>,
        source_size: PhysicalSize<u32>,
        full_screen: bool,
    ) -> Result<Projection, OsError> {
        let monitor = window_target.available_monitors().next().unwrap();

        let window_builder = WindowBuilder::new().with_title("Utopia");

        let (target_size, clip_rect, window_builder) = if full_screen {
            let default_video_mode = monitor.video_modes().next().unwrap();
            let video_mode = geometry::best_fit(source_size, monitor).unwrap_or(default_video_mode);
            let clip_rect = geometry::clip(source_size, video_mode.size());

            let window_builder =
                window_builder.with_fullscreen(Some(Fullscreen::Exclusive(video_mode)));

            (source_size, Some(clip_rect), window_builder)
        } else {
            let monitor_size = monitor.size();
            let target_size = geometry::upscale(source_size, monitor_size);
            let position = geometry::center(target_size, monitor_size);

            let window_builder = window_builder
                .with_inner_size(Size::Physical(target_size))
                .with_position(position)
                .with_resizable(false);

            (target_size, None, window_builder)
        };

        let window = window_builder.build(window_target)?;

        Ok(Self {
            window,
            target_size,
            clip_rect,
        })
    }
}
