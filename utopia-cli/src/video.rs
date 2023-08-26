use renderer::{RenderError, Renderer};
use std::error::Error;
use utopia::WgpuContext;
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
    vsync: bool,
}

impl VideoController {
    pub fn create_with_context(
        window_target: &EventLoopWindowTarget<()>,
        source_size: PhysicalSize<u32>,
        full_screen: bool,
        vsync: bool,
    ) -> Result<(Self, WgpuContext), Box<dyn Error>> {
        let (window, viewport) = Viewport::create_window(window_target, source_size, full_screen)?;

        let (renderer, wgpu_context) = Renderer::create_with_context(
            &window,
            source_size,
            viewport.size(),
            viewport.clip_rect(),
            vsync,
        )?;

        let video = Self {
            window,
            renderer,
            source_size,
            full_screen,
            vsync,
        };

        Ok((video, wgpu_context))
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn source_size(&self) -> PhysicalSize<u32> {
        self.source_size
    }

    pub fn set_source_size(
        &mut self,
        ctx: &mut WgpuContext,
        window_target: &EventLoopWindowTarget<()>,
        source_size: PhysicalSize<u32>,
    ) {
        self.source_size = source_size;

        let viewport = Viewport::new(window_target, source_size, self.full_screen);

        if !self.full_screen {
            self.window.set_outer_position(viewport.offset());
            self.window.set_inner_size(Size::Physical(viewport.size()));
        }

        self.renderer
            .update_viewport(ctx, source_size, viewport.clip_rect());
    }

    pub fn toggle_full_screen(
        &mut self,
        ctx: &mut WgpuContext,
        window_target: &EventLoopWindowTarget<()>,
    ) -> Result<(), Box<dyn Error>> {
        self.full_screen = !self.full_screen;

        let (window, viewport) =
            Viewport::create_window(window_target, self.source_size, self.full_screen)?;

        self.window = window;

        (self.renderer, *ctx) = Renderer::create_with_context(
            &self.window,
            self.source_size,
            viewport.size(),
            viewport.clip_rect(),
            self.vsync,
        )?;

        Ok(())
    }

    pub fn on_window_size_changed(&mut self, ctx: &WgpuContext) -> Result<(), Box<dyn Error>> {
        let new_size = self.window.inner_size();
        self.renderer.resize(ctx, new_size)?;
        Ok(())
    }

    pub fn on_target_changed(&mut self, window_target: &EventLoopWindowTarget<()>) {
        let viewport = Viewport::new(window_target, self.source_size, self.full_screen);

        if !self.full_screen {
            self.window.set_outer_position(viewport.offset());
            self.window.set_inner_size(Size::Physical(viewport.size()));
        }
    }

    pub fn render(&mut self, ctx: &WgpuContext) -> Result<(), RenderError> {
        self.renderer.render(ctx)
    }
}
