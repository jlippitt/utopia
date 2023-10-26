use super::primitive::{Color, Rectangle};
use crate::WgpuContext;
use tracing::trace;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Scissor {
    pub scale: [f32; 2],
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OutputFormat {
    Index8,
    Rgba16,
    Rgba32,
}

pub struct Output {
    pub color_texture: wgpu::Texture,
    pub depth_texture: wgpu::Texture,
    pub sync_buffer: wgpu::Buffer,
}

pub struct Target {
    output: Option<Output>,
    output_format: OutputFormat,
    scissor: Scissor,
    texture_width: u32,
    texture_height: u32,
    sync_dram_addr: u32,
    sync_width: u32,
    fill16: Color,
    fill32: Color,
}

impl Target {
    pub fn new() -> Self {
        Self {
            output: None,
            output_format: OutputFormat::Index8,
            scissor: Scissor::default(),
            texture_width: 0,
            texture_height: 0,
            sync_dram_addr: 0,
            sync_width: 0,
            fill16: Default::default(),
            fill32: Default::default(),
        }
    }

    pub fn scissor(&self) -> Scissor {
        self.scissor
    }

    pub fn set_scissor(&mut self, rect: Rectangle) {
        let width = rect.xl - rect.xh;
        let height = rect.yl - rect.yh;

        // TODO: Scissor offset
        self.scissor = Scissor {
            scale: [width, height],
        };

        self.texture_width = width as u32;
        self.texture_height = height as u32;

        trace!("  Target Scissor: {:?}", self.scissor);
        trace!(
            "  Target Texture Size: {}x{}",
            self.texture_width,
            self.texture_height
        );
    }

    pub fn set_color_image(&mut self, dram_addr: u32, width: u32, format: OutputFormat) {
        self.sync_dram_addr = dram_addr;
        self.sync_width = width;
        self.output_format = format;
        trace!("  Target Sync Destination: {:08X}", self.sync_dram_addr);
        trace!("  Target Sync Width: {}", self.sync_width);
        trace!("  Target Output Format: {:?}", self.output_format);
    }

    pub fn fill_color(&self) -> Color {
        match self.output_format {
            OutputFormat::Index8 => todo!("Index8 fill color"),
            OutputFormat::Rgba16 => self.fill16,
            OutputFormat::Rgba32 => self.fill32,
        }
    }

    pub fn set_fill_color(&mut self, word: u32) {
        // TODO: In 16-bit mode, there are actually two colors which alternate
        self.fill16 = [
            ((word >> 11) & 31) as f32 / 31.0,
            ((word >> 6) & 31) as f32 / 31.0,
            ((word >> 1) & 31) as f32 / 31.0,
            (word & 1) as f32,
        ];

        self.fill32 = [
            ((word >> 24) & 255) as f32 / 255.0,
            ((word >> 16) & 255) as f32 / 255.0,
            ((word >> 8) & 255) as f32 / 255.0,
            (word & 255) as f32 / 255.0,
        ];
    }

    pub fn prepare(&mut self, ctx: &WgpuContext) {
        if self.texture_width != 0 && self.texture_height != 0 {
            let should_update = if let Some(output) = &self.output {
                self.texture_width != output.color_texture.width()
                    || self.texture_height != output.color_texture.height()
            } else {
                true
            };

            if should_update {
                let size = wgpu::Extent3d {
                    width: self.sync_width,
                    height: self.texture_height,
                    depth_or_array_layers: 1,
                };

                let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("RDP Target Texture"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });

                let depth_texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("RDP Target Depth Texture"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth32Float,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });

                let sync_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("RDP Target Sync Buffer"),
                    size: self.sync_width as u64 * self.texture_height as u64 * 4,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                self.output = Some(Output {
                    color_texture: texture,
                    depth_texture,
                    sync_buffer,
                })
            }
        } else {
            self.output = None;
        }
    }

    pub fn output(&self) -> &Output {
        self.output
            .as_ref()
            .expect("No available RDP output texture")
    }

    pub fn copy_to_rdram(&self, rdram: &mut [u8], pixel_data: &[u8]) {
        // TODO: What happens when sync width is not the same as texture width?

        match self.output_format {
            OutputFormat::Index8 => todo!("Index8 output format"),
            OutputFormat::Rgba16 => {
                let start = self.sync_dram_addr as usize;
                let end = start + pixel_data.len() / 2;

                let mut iter = pixel_data.chunks_exact(4).flat_map(|chunk| {
                    let color = ((chunk[0] as u16 >> 3) << 11)
                        | ((chunk[1] as u16 >> 3) << 6)
                        | ((chunk[2] as u16 >> 3) << 1)
                        | (chunk[3] as u16 >> 7);

                    color.to_be_bytes()
                });

                rdram[start..end].fill_with(|| iter.next().unwrap());
            }
            OutputFormat::Rgba32 => {
                let start = self.sync_dram_addr as usize;
                let end = start + pixel_data.len();
                rdram[start..end].copy_from_slice(pixel_data);
            }
        }
    }
}
