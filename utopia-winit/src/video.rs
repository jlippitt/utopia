use super::AppEvent;
use std::error::Error;
use std::sync::Arc;
use utopia::{MemoryMapper, WgpuContext};
use viewport::Viewport;
use winit::dpi::{PhysicalSize, Size};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Fullscreen, Window, WindowBuilder};

#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::{WindowBuilderExtWebSys, WindowExtWebSys};

mod viewport;

pub struct VideoController {
    window: Window,
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    ctx: WgpuContext,
    source_size: PhysicalSize<u32>,
    prev_monitor_size: PhysicalSize<u32>,
    full_screen: bool,
}

impl VideoController {
    pub fn create_with_context(
        window_target: &EventLoopWindowTarget<AppEvent<impl MemoryMapper>>,
        source_size: PhysicalSize<u32>,
        full_screen: bool,
        vsync: bool,
        #[cfg(target_arch = "wasm32")] canvas: HtmlCanvasElement,
    ) -> Result<Self, Box<dyn Error>> {
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
            let window_builder = window_builder.with_inner_size(Size::Physical(viewport.size()));

            if let Some(offset) = viewport.offset() {
                window_builder.with_position(offset)
            } else {
                window_builder
            }
        };

        let window = window_builder.build(window_target)?;

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window)? };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                #[cfg(target_arch = "wasm32")]
                limits: wgpu::Limits::downlevel_webgl2_defaults(),
                #[cfg(not(target_arch = "wasm32"))]
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ))?;

        let capabilities = surface.get_capabilities(&adapter);

        let output_format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let present_mode = if vsync {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: output_format,
            width: viewport.size().width,
            height: viewport.size().height,
            present_mode,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: Vec::new(),
        };

        surface.configure(&device, &config);

        let ctx = WgpuContext {
            device: Arc::new(device),
            queue: Arc::new(queue),
            output_format,
        };

        let monitor_size = window.current_monitor().unwrap().size();

        Ok(Self {
            window,
            surface,
            config,
            ctx,
            source_size,
            prev_monitor_size: monitor_size,
            full_screen,
        })
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn ctx(&self) -> &WgpuContext {
        &self.ctx
    }

    pub fn source_size(&self) -> PhysicalSize<u32> {
        self.source_size
    }

    pub fn set_source_size(
        &mut self,
        window_target: &EventLoopWindowTarget<AppEvent<impl MemoryMapper>>,
        source_size: PhysicalSize<u32>,
    ) {
        self.source_size = source_size;
        self.update_viewport(window_target);
    }

    pub fn toggle_full_screen(
        &mut self,
        window_target: &EventLoopWindowTarget<AppEvent<impl MemoryMapper>>,
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

    pub fn on_window_size_changed(&mut self) -> Result<(), Box<dyn Error>> {
        let new_size = self.window.inner_size();

        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.ctx.device, &self.config);
        }

        Ok(())
    }

    pub fn update_viewport(
        &mut self,
        window_target: &EventLoopWindowTarget<AppEvent<impl MemoryMapper>>,
    ) {
        #[cfg(target_arch = "wasm32")]
        let view_target = {
            _ = window_target;
            &self.window.canvas().unwrap()
        };

        #[cfg(not(target_arch = "wasm32"))]
        let view_target = window_target;

        let viewport = Viewport::new(view_target, self.source_size, self.full_screen);

        if !self.full_screen {
            if let Some(offset) = viewport.offset() {
                self.window.set_outer_position(offset);
            }

            _ = self
                .window
                .request_inner_size(Size::Physical(viewport.size()));
        }
    }

    pub fn redraw(
        &mut self,
        window_target: &EventLoopWindowTarget<AppEvent<impl MemoryMapper>>,
        draw_fn: impl Fn(&wgpu::Texture),
    ) -> Result<(), Box<dyn Error>> {
        let monitor_size = self.window.current_monitor().unwrap().size();

        if monitor_size != self.prev_monitor_size {
            self.prev_monitor_size = monitor_size;
            self.update_viewport(window_target);
            self.on_window_size_changed()?;
        }

        let output = self.surface.get_current_texture()?;
        draw_fn(&output.texture);
        output.present();

        Ok(())
    }
}
