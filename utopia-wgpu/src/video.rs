use renderer::Renderer;
use std::error::Error;
use viewport::Viewport;
use winit::dpi::{PhysicalSize, Size};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;

mod renderer;
mod viewport;

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
        let (window, viewport) = Viewport::create_window(window_target, source_size, full_screen)?;

        let renderer = Renderer::new(&window, source_size, viewport.size(), viewport.clip_rect())?;

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

        let (window, viewport) =
            Viewport::create_window(window_target, self.source_size, self.full_screen)?;

        self.window = window;

        self.renderer = Renderer::new(
            &self.window,
            self.source_size,
            viewport.size(),
            viewport.clip_rect(),
        )?;

        Ok(())
    }

    pub fn on_window_size_changed(&mut self) -> Result<(), Box<dyn Error>> {
        let new_size = self.window.inner_size();
        self.renderer.on_window_size_changed(new_size)?;
        Ok(())
    }

    pub fn on_target_changed(&mut self, window_target: &EventLoopWindowTarget<()>) {
        let viewport = Viewport::new(window_target, self.source_size, self.full_screen);

        if !self.full_screen {
            self.window.set_outer_position(viewport.offset());
            self.window.set_inner_size(Size::Physical(viewport.size()));
        }
    }

    pub fn render(&mut self, pixels: &[u8], pitch: usize) -> Result<(), Box<dyn Error>> {
        self.renderer.render(pixels, pitch)
    }
}
