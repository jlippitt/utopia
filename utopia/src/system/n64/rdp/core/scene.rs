use super::fragment::FragmentControl;
use super::tmem::Tmem;
use std::ops::Range;
use tracing::trace;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[derive(Debug)]
enum DisplayListItem {
    Triangles(Range<u32>),
    Quads(Range<u32>),
    BindTexture(usize),
    BindFragmentState(usize),
}

pub struct Scene {
    display_list: Vec<DisplayListItem>,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    bound_texture: usize,
    bound_fragment_state: Option<usize>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            display_list: Vec::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
            bound_texture: 0,
            bound_fragment_state: None,
        }
    }

    pub fn clear(&mut self) {
        self.display_list = Vec::new();
        self.vertices = Vec::new();
        self.indices = Vec::new();
        self.bound_texture = 0;
        self.bound_fragment_state = None;
    }

    pub fn push_triangle(&mut self, vertices: &[Vertex; 3]) {
        self.vertices.extend_from_slice(vertices);

        let end = self.vertices.len().try_into().unwrap();

        match self.display_list.last_mut() {
            Some(DisplayListItem::Triangles(existing_range)) => {
                *existing_range = existing_range.start..end;
            }
            _ => {
                self.display_list
                    .push(DisplayListItem::Triangles((end - 3)..end));
            }
        }
    }

    pub fn push_quad(&mut self, vertices: &[Vertex; 4]) {
        let base_vertex = self.vertices.len().try_into().unwrap();

        self.vertices.extend_from_slice(vertices);

        self.indices.extend_from_slice(&[
            base_vertex,
            base_vertex + 1,
            base_vertex + 2,
            base_vertex + 2,
            base_vertex + 1,
            base_vertex + 3,
        ]);

        let end = self.indices.len().try_into().unwrap();

        match self.display_list.last_mut() {
            Some(DisplayListItem::Quads(existing_range)) => {
                *existing_range = existing_range.start..end;
            }
            _ => {
                self.display_list
                    .push(DisplayListItem::Quads((end - 6)..end));
            }
        }
    }

    pub fn bind_texture(&mut self, key: usize) {
        if key == self.bound_texture {
            return;
        }

        self.display_list.push(DisplayListItem::BindTexture(key));

        self.bound_texture = key;
    }

    pub fn bind_fragment_state(&mut self, key: usize) {
        if Some(key) == self.bound_fragment_state {
            return;
        }

        self.display_list
            .push(DisplayListItem::BindFragmentState(key));

        self.bound_fragment_state = Some(key);
    }

    pub fn upload_buffers(
        &self,
        queue: &wgpu::Queue,
        vertex_buffer: &wgpu::Buffer,
        index_buffer: &wgpu::Buffer,
    ) {
        queue.write_buffer(vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        queue.write_buffer(index_buffer, 0, bytemuck::cast_slice(&self.indices));
    }

    pub fn render<'a, 'b: 'a>(
        &'a mut self,
        tmem: &'b Tmem,
        fragment: &'b FragmentControl,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        use DisplayListItem::*;

        trace!("Display List: {:?}", self.display_list);

        for item in &self.display_list {
            match item {
                Triangles(range) => render_pass.draw(range.clone(), 0..1),
                Quads(range) => render_pass.draw_indexed(range.clone(), 0, 0..1),
                BindTexture(key) => {
                    render_pass.set_bind_group(1, tmem.bind_group_from_key(*key), &[])
                }
                BindFragmentState(key) => {
                    render_pass.set_bind_group(2, fragment.bind_group_from_key(*key), &[])
                }
            }
        }
    }
}
