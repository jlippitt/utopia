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

    pub fn source_size(&self) -> PhysicalSize<u32> {
        self.source_size
    }

    pub fn set_source_size(
        &mut self,
        window_target: &EventLoopWindowTarget<()>,
        source_size: PhysicalSize<u32>,
    ) {
        self.source_size = source_size;

        let viewport = Viewport::new(window_target, source_size, self.full_screen);

        if !self.full_screen {
            self.window.set_outer_position(viewport.offset());
            self.window.set_inner_size(Size::Physical(viewport.size()));
        }

        self.renderer.update(source_size, viewport.clip_rect());
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
        self.renderer.resize(new_size)?;
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
