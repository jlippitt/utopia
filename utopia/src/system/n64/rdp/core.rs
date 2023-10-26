use super::super::WgpuContext;
use fragment::FragmentControl;
use primitive::{Color, Position, Rectangle};
use scene::{Scene, Vertex};
use target::{Output, Target};
use tmem::Tmem;
use tracing::{debug, debug_span, trace};
use wgpu::util::DeviceExt;

mod command;
mod fragment;
mod primitive;
mod scene;
mod target;
mod tmem;

#[derive(Debug)]
pub struct Mode {
    pub use_prim_depth: bool,
}

pub struct Core {
    sync_required: bool,
    interrupt: bool,
    mode: Mode,
    prim_depth: f32,
    target: Target,
    tmem: Tmem,
    fragment: FragmentControl,
    ctx: WgpuContext,
    render_pipeline: wgpu::RenderPipeline,
    scissor_buffer: wgpu::Buffer,
    scissor_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    scene: Scene,
}

impl Core {
    pub const Z_BUFFER_MAX: f32 = 65536.0;

    pub fn new(ctx: WgpuContext) -> Self {
        let WgpuContext { device, .. } = &ctx;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("RDP Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./rdp.wgsl").into()),
        });

        let scissor_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("RDP Scissor Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let tmem = Tmem::new(&ctx);

        let fragment = FragmentControl::new(device);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("RDP Render Pipeline Layout"),
                bind_group_layouts: &[
                    &scissor_bind_group_layout,
                    tmem.bind_group_layout(),
                    fragment.bind_group_layout(),
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("RDP Render Pipeline"),
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
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let target = Target::new();

        let scissor_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("RDP Scissor Buffer"),
            contents: bytemuck::cast_slice(&[target.scissor()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let scissor_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("RDP Scissor Bind Group"),
            layout: &scissor_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scissor_buffer.as_entire_binding(),
            }],
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RDP Vertex Buffer"),
            size: 262144,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: 131072,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            sync_required: false,
            interrupt: false,
            mode: Mode {
                use_prim_depth: false,
            },
            prim_depth: Self::Z_BUFFER_MAX,
            target,
            tmem,
            fragment,
            ctx,
            render_pipeline,
            scissor_buffer,
            scissor_bind_group,
            vertex_buffer,
            index_buffer,
            scene: Scene::new(),
        }
    }

    pub fn sync_required(&self) -> bool {
        self.sync_required
    }

    pub fn set_sync_required(&mut self, sync_required: bool) {
        self.sync_required = sync_required;
    }

    pub fn interrupt(&self) -> bool {
        self.interrupt
    }

    pub fn set_interrupt(&mut self, interrupt: bool) {
        self.interrupt = interrupt;
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn prim_depth(&self) -> f32 {
        self.prim_depth
    }

    pub fn set_prim_depth(&mut self, prim_depth: f32) {
        self.prim_depth = prim_depth;
    }

    pub fn push_triangle(
        &mut self,
        position: [Position; 3],
        color: [Color; 3],
        texture: Option<(u32, [[f32; 4]; 3])>,
    ) {
        let (key, tex_coords) = if let Some((tile_id, in_coords)) = texture {
            let (key, _) = self.tmem.request_key(&self.ctx, tile_id);
            // TODO: 'W' coordinate?
            (key, in_coords.map(|coord| [coord[0], coord[1]]))
        } else {
            (0, Default::default())
        };

        let vertices = [
            Vertex {
                position: position[0],
                color: color[0],
                tex_coords: tex_coords[0],
            },
            Vertex {
                position: position[1],
                color: color[1],
                tex_coords: tex_coords[1],
            },
            Vertex {
                position: position[2],
                color: color[2],
                tex_coords: tex_coords[2],
            },
        ];

        trace!("  Triangle: {:?}", vertices);

        self.scene.bind_texture(key);

        self.scene
            .bind_fragment_state(self.fragment.cache_key(&self.ctx.device));

        self.scene.push_triangle(&vertices);
    }

    pub fn push_rect(&mut self, rect: Rectangle, texture: Option<(u32, Rectangle)>) {
        let (key, color, tex_coords) = if let Some((tile_id, in_coords)) = texture {
            let (key, (width, height)) = self.tmem.request_key(&self.ctx, tile_id);

            let out_coords = Rectangle {
                xh: in_coords.xh / width,
                yh: in_coords.yh / height,
                xl: in_coords.xl / width,
                yl: in_coords.yl / height,
            };

            (key, Default::default(), out_coords)
        } else {
            (0, self.target.fill_color(), Default::default())
        };

        let vertices = [
            Vertex {
                position: [rect.xh, rect.yh, self.prim_depth],
                color,
                tex_coords: [tex_coords.xh, tex_coords.yh],
            },
            Vertex {
                position: [rect.xh, rect.yl, self.prim_depth],
                color,
                tex_coords: [tex_coords.xh, tex_coords.yl],
            },
            Vertex {
                position: [rect.xl, rect.yh, self.prim_depth],
                color,
                tex_coords: [tex_coords.xl, tex_coords.yh],
            },
            Vertex {
                position: [rect.xl, rect.yl, self.prim_depth],
                color,
                tex_coords: [tex_coords.xl, tex_coords.yl],
            },
        ];

        trace!("  Rectangle: {:?}", vertices);

        self.scene.bind_texture(key);

        self.scene
            .bind_fragment_state(self.fragment.cache_key(&self.ctx.device));

        self.scene.push_quad(&vertices);
    }

    pub fn run(&mut self, rdram: &mut [u8], mut commands: impl Iterator<Item = u64>) {
        while let Some(cmd) = commands.next() {
            command::dispatch(self, rdram, cmd, &mut commands);
        }
    }

    pub fn sync_full(&mut self, rdram: &mut [u8]) {
        let _span = debug_span!("rdp").entered();
        debug!("RDP Sync to RDRAM");

        self.set_sync_required(false);

        self.target.prepare(&self.ctx);

        let WgpuContext { device, queue, .. } = &self.ctx;

        // Upload buffer data
        queue.write_buffer(
            &self.scissor_buffer,
            0,
            bytemuck::cast_slice(&[self.target.scissor()]),
        );

        self.scene
            .upload_buffers(queue, &self.vertex_buffer, &self.index_buffer);

        let fragment_key = self.fragment.cache_key(device);

        let Output {
            color_texture,
            depth_texture,
            sync_buffer,
        } = self.target.output();

        // Render the scene
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("RDP Render Encoder"),
        });

        {
            let color_texture_view =
                color_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let depth_texture_view =
                depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("RDP Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_texture_view,
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Self::Z_BUFFER_MAX),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, &self.scissor_bind_group, &[]);
            render_pass.set_bind_group(1, self.tmem.bind_group_from_key(0), &[]);
            render_pass.set_bind_group(2, self.fragment.bind_group_from_key(fragment_key), &[]);

            self.scene
                .render(&self.tmem, &self.fragment, &mut render_pass);
        }

        self.scene.clear();

        // Copy the color texture into RDRAM
        encoder.copy_texture_to_buffer(
            color_texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: sync_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(color_texture.width() * 4),
                    rows_per_image: Some(color_texture.height()),
                },
            },
            color_texture.size(),
        );

        queue.submit(std::iter::once(encoder.finish()));

        // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();

        let buffer_slice = sync_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        device.poll(wgpu::Maintain::Wait);

        if let Some(Ok(())) = pollster::block_on(receiver.receive()) {
            let pixel_data = &sync_buffer.slice(..).get_mapped_range();
            self.target.copy_to_rdram(rdram, pixel_data);
        }

        sync_buffer.unmap();
    }
}
