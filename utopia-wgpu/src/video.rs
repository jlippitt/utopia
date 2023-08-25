use std::error::Error;
use wgpu::util::DeviceExt;
use winit::dpi::{PhysicalSize, Size};
use winit::event_loop::EventLoop;
use winit::window::{Fullscreen, Window, WindowBuilder};

mod geometry;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

const DEFAULT_CLIP_RECT: [[f32; 2]; 4] = [[-1.0, 1.0], [-1.0, -1.0], [1.0, 1.0], [1.0, -1.0]];

#[rustfmt::skip]
const INDICES: &[u16] = &[
    0, 1, 2,
    2, 1, 3,
];

struct WgpuState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    texture: wgpu::Texture,
    texture_size: wgpu::Extent3d,
    _texture_view: wgpu::TextureView,
    texture_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

pub struct VideoController {
    wgpu: WgpuState,
    window: Window,
    source_size: PhysicalSize<u32>,
    full_screen: bool,
}

impl VideoController {
    pub fn new(
        event_loop: &EventLoop<()>,
        source_size: PhysicalSize<u32>,
        full_screen: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let monitor = event_loop.available_monitors().next().unwrap();

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

        let window = window_builder.build(&event_loop)?;
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
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ))?;

        let capabilities = surface.get_capabilities(&adapter);

        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: target_size.width,
            height: target_size.height,
            present_mode: capabilities.present_modes[0],
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: Vec::new(),
        };

        surface.configure(&device, &config);

        let texture_size = wgpu::Extent3d {
            width: source_size.width,
            height: source_size.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Source Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let texture_bind_group = device.create_bind_group({
            &wgpu::BindGroupDescriptor {
                label: Some("Texture Bind Group"),
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture_sampler),
                    },
                ],
            }
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let vertices = vertices_from_clip_rect(clip_rect);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Ok(Self {
            wgpu: WgpuState {
                surface,
                device,
                queue,
                config,
                texture,
                texture_size,
                _texture_view: texture_view,
                texture_bind_group,
                render_pipeline,
                vertex_buffer,
                index_buffer,
            },
            window,
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

    pub fn toggle_full_screen(&mut self) -> Result<(), String> {
        self.full_screen = !self.full_screen;

        let monitor = self.window.current_monitor().unwrap();

        let (target_size, clip_rect) = if self.full_screen {
            let default_video_mode = monitor.video_modes().next().unwrap();
            let video_mode =
                geometry::best_fit(self.source_size, monitor).unwrap_or(default_video_mode);
            let clip_rect = geometry::clip(self.source_size, video_mode.size());

            self.window
                .set_fullscreen(Some(Fullscreen::Exclusive(video_mode)));

            (self.source_size, Some(clip_rect))
        } else {
            let monitor_size = monitor.size();
            let target_size = geometry::upscale(self.source_size, monitor_size);
            let position = geometry::center(target_size, monitor_size);

            self.window.set_fullscreen(None);
            self.window.set_outer_position(position);
            self.window.set_resizable(false);

            (target_size, None)
        };

        self.window.set_inner_size(target_size);

        let vertices = vertices_from_clip_rect(clip_rect);

        self.wgpu
            .queue
            .write_buffer(&self.wgpu.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        Ok(())
    }

    pub fn on_window_size_changed(&mut self) -> Result<(), Box<dyn Error>> {
        let new_size = self.window.inner_size();

        if new_size.width > 0 && new_size.height > 0 {
            self.wgpu.config.width = new_size.width;
            self.wgpu.config.height = new_size.height;
            self.wgpu
                .surface
                .configure(&self.wgpu.device, &self.wgpu.config);
        }

        Ok(())
    }

    pub fn render(&mut self, pixels: &[u8], pitch: usize) -> Result<(), Box<dyn Error>> {
        self.wgpu.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.wgpu.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(pitch.try_into()?),
                rows_per_image: Some((pixels.len() / pitch).try_into()?),
            },
            self.wgpu.texture_size,
        );

        let output = self.wgpu.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.wgpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.wgpu.render_pipeline);
            render_pass.set_bind_group(0, &self.wgpu.texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.wgpu.vertex_buffer.slice(..));
            render_pass
                .set_index_buffer(self.wgpu.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }

        self.wgpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn vertices_from_clip_rect(clip_rect: Option<[[f32; 2]; 4]>) -> [Vertex; 4] {
    let clip = clip_rect.unwrap_or(DEFAULT_CLIP_RECT);

    [
        // Top Left
        Vertex {
            position: clip[0],
            tex_coords: [0.0, 0.0],
        },
        // Bottom Left
        Vertex {
            position: clip[1],
            tex_coords: [0.0, 1.0],
        },
        // Top Right
        Vertex {
            position: clip[2],
            tex_coords: [1.0, 0.0],
        },
        // Bottom Right
        Vertex {
            position: clip[3],
            tex_coords: [1.0, 1.0],
        },
    ]
}
