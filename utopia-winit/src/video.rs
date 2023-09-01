use renderer::Renderer;
use std::error::Error;
use utopia::WgpuContext;
use viewport::Viewport;
use winit::dpi::{PhysicalSize, Size};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Fullscreen, Window, WindowBuilder};

#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::{WindowBuilderExtWebSys, WindowExtWebSys};

mod renderer;
mod viewport;

pub struct VideoController {
    window: Window,
    renderer: Renderer,
    source_size: PhysicalSize<u32>,
    prev_monitor_size: PhysicalSize<u32>,
    full_screen: bool,
}

impl VideoController {
    pub fn create_with_context(
        window_target: &EventLoopWindowTarget<()>,
        source_size: PhysicalSize<u32>,
        full_screen: bool,
        vsync: bool,
        #[cfg(target_arch = "wasm32")] canvas: HtmlCanvasElement,
    ) -> Result<(Self, WgpuContext), Box<dyn Error>> {
        #[cfg(target_arch = "wasm32")]
        let view_target = &canvas;

        #[cfg(not(target_arch = "wasm32"))]
        let view_target = window_target;

        let viewport = Viewport::new(view_target, source_size, full_screen);

        let window_builder = WindowBuilder::new().with_title("Utopia");

        #[cfg(target_arch = "wasm32")]
        let window_builder = window_builder.with_canvas(Some(canvas));

        let window_builder = if full_screen {
            window_builder.with_fullscreen(Some(Fullscreen::Exclusive(
                viewport.video_mode().unwrap().clone(),
            )))
        } else {
            window_builder
                .with_inner_size(Size::Physical(viewport.size()))
                .with_position(viewport.offset())
        };

        let window = window_builder.build(window_target)?;

        let (renderer, wgpu_context) = Renderer::create_with_context(
            &window,
            source_size,
            viewport.size(),
            viewport.clip_rect(),
            vsync,
        )?;

        let monitor_size = window.current_monitor().unwrap().size();

        let video = Self {
            window,
            renderer,
            source_size,
            prev_monitor_size: monitor_size,
            full_screen,
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

        #[cfg(target_arch = "wasm32")]
        let view_target = {
            _ = window_target;
            &self.window.canvas().unwrap()
        };

        #[cfg(not(target_arch = "wasm32"))]
        let view_target = window_target;

        let viewport = Viewport::new(view_target, source_size, self.full_screen);

        if !self.full_screen {
            self.window.set_outer_position(viewport.offset());
            _ = self
                .window
                .request_inner_size(Size::Physical(viewport.size()));
        }

        self.renderer.update_source_size(ctx, source_size);
        self.renderer.update_clip_rect(ctx, viewport.clip_rect());
    }

    pub fn toggle_full_screen(
        &mut self,
        _ctx: &WgpuContext,
        window_target: &EventLoopWindowTarget<()>,
    ) -> Result<(), Box<dyn Error>> {
        self.full_screen = !self.full_screen;

        #[cfg(target_arch = "wasm32")]
        let view_target = {
            _ = window_target;
            &self.window.canvas().unwrap()
        };

        #[cfg(not(target_arch = "wasm32"))]
        let view_target = window_target;

        let viewport = Viewport::new(view_target, self.source_size, self.full_screen);

        if self.full_screen {
            self.window.set_fullscreen(Some(Fullscreen::Exclusive(
                viewport.video_mode().unwrap().clone(),
            )))
        } else {
            self.window.set_fullscreen(None);
        }

        Ok(())
    }

    pub fn on_window_size_changed(&mut self, ctx: &WgpuContext) -> Result<(), Box<dyn Error>> {
        let new_size = self.window.inner_size();
        self.renderer.resize(ctx, new_size)?;
        Ok(())
    }

    pub fn render(
        &mut self,
        ctx: &WgpuContext,
        window_target: &EventLoopWindowTarget<()>,
    ) -> Result<(), Box<dyn Error>> {
        let monitor_size = self.window.current_monitor().unwrap().size();

        if monitor_size != self.prev_monitor_size {
            #[cfg(target_arch = "wasm32")]
            let view_target = {
                _ = window_target;
                &self.window.canvas().unwrap()
            };

            #[cfg(not(target_arch = "wasm32"))]
            let view_target = window_target;

            let viewport = Viewport::new(view_target, self.source_size, self.full_screen);

            if !self.full_screen {
                self.window.set_outer_position(viewport.offset());
                _ = self
                    .window
                    .request_inner_size(Size::Physical(viewport.size()));
            }

            self.on_window_size_changed(ctx)?;

            self.renderer.update_clip_rect(ctx, viewport.clip_rect());

            self.prev_monitor_size = monitor_size;
        }

        self.renderer.render(ctx)?;

        Ok(())
    }
}
