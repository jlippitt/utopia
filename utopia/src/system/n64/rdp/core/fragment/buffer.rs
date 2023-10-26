use tracing::trace;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FragmentState {
    pub rgb0: [i32; 4],
    pub alpha0: [i32; 4],
    pub rgb1: [i32; 4],
    pub alpha1: [i32; 4],
    pub blend0: [i32; 4],
    pub blend1: [i32; 4],
    pub prim_color: [f32; 4],
    pub env_color: [f32; 4],
    pub blend_color: [f32; 4],
    pub fog_color: [f32; 4],
    pub cycle_type: i32,
    pub _pad0: i32,
    pub _pad1: i32,
    pub _pad2: i32,
}

pub struct Buffer {
    _buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Buffer {
    pub fn new(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        data: FragmentState,
    ) -> Self {
        trace!("{:?}", data);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("RDP Combiner Buffer"),
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group({
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            }
        });

        Self {
            _buffer: buffer,
            bind_group,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
