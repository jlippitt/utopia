use crate::util::gfx;
use crate::WgpuContext;
use wgpu::util::DeviceExt;

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

const VERTICES: &[Vertex] = &[
    // Top Left
    Vertex {
        position: [-1.0, 1.0],
        tex_coords: [0.0, 0.0],
    },
    // Bottom Left
    Vertex {
        position: [-1.0, -1.0],
        tex_coords: [0.0, 1.0],
    },
    // Top Right
    Vertex {
        position: [1.0, 1.0],
        tex_coords: [1.0, 0.0],
    },
    // Bottom Right
    Vertex {
        position: [1.0, -1.0],
        tex_coords: [1.0, 1.0],
    },
];

#[rustfmt::skip]
const INDICES: &[u16] = &[
    0, 1, 2,
    2, 1, 3,
];

pub struct Resampler {
    resample: bool,
    texture: wgpu::Texture,
    sampler_nearest: wgpu::Sampler,
    sampler_linear: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl Resampler {
    pub fn new(ctx: &WgpuContext, width: u32, height: u32) -> Self {
        let device = &ctx.device;

        let texture = create_texture(device, width, height);
        let sampler_nearest = create_sampler(device, wgpu::FilterMode::Nearest);
        let sampler_linear = create_sampler(device, wgpu::FilterMode::Linear);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Resampler Texture Bind Group Layout"),
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

        let bind_group = create_bind_group(device, &bind_group_layout, &texture, &sampler_nearest);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Resample Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("resampler.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Resampler Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Resampler Render Pipeline"),
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
                    format: ctx.texture.format(),
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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Resampler Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Resampler Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            resample: false,
            texture,
            sampler_nearest,
            sampler_linear,
            bind_group,
            bind_group_layout,
            render_pipeline,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn update_source_size(&mut self, ctx: &WgpuContext, width: u32, height: u32) {
        self.texture = create_texture(&ctx.device, width, height);

        self.bind_group = create_bind_group(
            &ctx.device,
            &self.bind_group_layout,
            &self.texture,
            if self.resample {
                &self.sampler_linear
            } else {
                &self.sampler_nearest
            },
        );
    }

    pub fn render(&mut self, ctx: &WgpuContext, pixels: &[u8], pitch: usize, resample: bool) {
        gfx::write_pixels_to_texture(ctx, &self.texture, pixels, pitch);

        if resample != self.resample {
            self.bind_group = create_bind_group(
                &ctx.device,
                &self.bind_group_layout,
                &self.texture,
                if resample {
                    &self.sampler_linear
                } else {
                    &self.sampler_nearest
                },
            );

            self.resample = resample;
        }

        let view = ctx
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Resampler Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Resampler Render Pass"),
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }

        ctx.queue.submit(std::iter::once(encoder.finish()));
    }
}

fn create_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Resampler Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn create_sampler(device: &wgpu::Device, mag_filter: wgpu::FilterMode) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

fn create_bind_group(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    texture: &wgpu::Texture,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    device.create_bind_group({
        &wgpu::BindGroupDescriptor {
            label: Some("Resampler Texture Bind Group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        }
    })
}
