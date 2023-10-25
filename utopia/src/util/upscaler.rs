use crate::util::size::Size;
use crate::WgpuContext;
use std::cell::Cell;
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

#[rustfmt::skip]
const INDICES: &[u16] = &[
    0, 1, 2,
    2, 1, 3,
];

pub struct Upscaler {
    ctx: WgpuContext,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    _bind_group_layout: wgpu::BindGroupLayout,
    target_size: Cell<Size>,
    _resample: bool,
    _sampler_nearest: wgpu::Sampler,
    _sampler_linear: wgpu::Sampler,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl Upscaler {
    pub fn new(ctx: WgpuContext, source_size: Size, target_size: Size, resample: bool) -> Self {
        let WgpuContext {
            device,
            output_format,
            ..
        } = &ctx;

        let sampler_nearest = create_sampler(device, wgpu::FilterMode::Nearest);
        let sampler_linear = create_sampler(device, wgpu::FilterMode::Linear);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Upscaler Texture Bind Group Layout"),
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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Upscaler Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./upscaler.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Upscaler Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Upscaler Render Pipeline"),
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
                    format: *output_format,
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

        let texture = create_texture(device, source_size);

        let bind_group = create_bind_group(
            device,
            &bind_group_layout,
            &texture,
            if resample {
                &sampler_linear
            } else {
                &sampler_nearest
            },
        );

        let clip_rect = create_clip_rect(source_size, target_size);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Upscaler Vertex Buffer"),
            contents: bytemuck::cast_slice(&clip_rect),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Upscaler Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            ctx,
            texture,
            bind_group,
            _bind_group_layout: bind_group_layout,
            target_size: Cell::new(target_size),
            _resample: resample,
            _sampler_linear: sampler_nearest,
            _sampler_nearest: sampler_linear,
            render_pipeline,
            vertex_buffer,
            index_buffer,
        }
    }

    // pub fn set_resample(&mut self, resample: bool) {
    //     if resample == self.resample {
    //         return;
    //     }

    //     if let Some(texture) = &self.texture {
    //         self.bind_group = Some(create_bind_group(
    //             &self.ctx.device,
    //             &self.bind_group_layout,
    //             texture,
    //             if resample {
    //                 &self.sampler_linear
    //             } else {
    //                 &self.sampler_nearest
    //             },
    //         ));
    //     }

    //     self.resample = resample;
    // }

    // pub fn set_source_size(&mut self, size: Size) {
    //     let WgpuContext { device, .. } = &self.ctx;

    //     if size != self.texture.size().into() {
    //         self.texture = create_texture(device, size);

    //         self.bind_group = create_bind_group(
    //             device,
    //             &self.bind_group_layout,
    //             &self.texture,
    //             if self.resample {
    //                 &self.sampler_linear
    //             } else {
    //                 &self.sampler_nearest
    //             },
    //         );
    //     }
    // }

    pub fn update(&self, pixels: &[u8]) {
        self.ctx.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.texture.width() * 4),
                rows_per_image: Some(self.texture.height()),
            },
            self.texture.size(),
        );
    }

    pub fn render(&self, canvas: &wgpu::Texture) {
        let WgpuContext { device, queue, .. } = &self.ctx;

        let canvas_size = canvas.size().into();

        if self.target_size.get() != canvas_size {
            self.target_size.set(canvas_size);

            let clip_rect = create_clip_rect(self.texture.size().into(), canvas_size);

            self.ctx
                .queue
                .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&clip_rect));
        }

        let view = canvas.create_view(&Default::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Upscaler Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Upscaler Render Pass"),
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

        queue.submit(std::iter::once(encoder.finish()));
    }
}

fn create_texture(device: &wgpu::Device, size: Size) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Upscaler Texture"),
        size: size.into(),
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
            label: Some("Upscaler Texture Bind Group"),
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

fn create_clip_rect(source: Size, target: Size) -> [Vertex; 4] {
    let scale_factor = (target.width / source.width).min(target.height / source.height);

    let scaled_width = source.width * scale_factor;
    let scaled_height = source.height * scale_factor;

    let offset_x = (target.width - scaled_width) / 2;
    let offset_y = (target.height - scaled_height) / 2;

    let left = (offset_x as f32 / target.width as f32) * 2.0 - 1.0;
    let right = ((offset_x + scaled_width) as f32 / target.width as f32) * 2.0 - 1.0;
    let top = 1.0 - (offset_y as f32 / target.height as f32) * 2.0;
    let bottom = 1.0 - ((offset_y + scaled_height) as f32 / target.height as f32) * 2.0;

    [
        // Top Left
        Vertex {
            position: [left, top],
            tex_coords: [0.0, 0.0],
        },
        // Bottom Left
        Vertex {
            position: [left, bottom],
            tex_coords: [0.0, 1.0],
        },
        // Top Right
        Vertex {
            position: [right, top],
            tex_coords: [1.0, 0.0],
        },
        // Bottom Right
        Vertex {
            position: [right, bottom],
            tex_coords: [1.0, 1.0],
        },
    ]
}
