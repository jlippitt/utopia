use super::primitive::Rectangle;
use crate::WgpuContext;
use format::TextureFormat;
use texture::{Texture, TextureParams};
use tracing::trace;

mod format;
mod texture;

const TMEM_SIZE: usize = 512;

#[derive(Debug)]
pub struct SetTileParams {
    pub format: TextureFormat,
    pub tmem_addr: u32,
}

#[derive(Clone, Debug, Default)]
pub struct TextureImage {
    pub format: TextureFormat,
    pub dram_addr: usize,
    pub dram_width: usize,
}

#[derive(Clone, Debug, Default)]
struct TileDescriptor {
    format: TextureFormat,
    tmem_addr: u32,
    rect: Rectangle,
}

pub struct Tmem {
    tiles: [TileDescriptor; 8],
    texture_image: TextureImage,
    data: Vec<u64>,
    bind_group_layout: wgpu::BindGroupLayout,
    textures: Vec<Texture>,
}

impl Tmem {
    pub fn new(ctx: &WgpuContext) -> Self {
        let bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("RDP Texture Bind Group Layout"),
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

        let null_texture = Texture::new(
            ctx,
            &bind_group_layout,
            TextureParams {
                format: wgpu::TextureFormat::Rgba8Unorm,
                width: 1,
                height: 1,
                data: &[0, 0, 0, 255],
            },
        );

        Self {
            tiles: Default::default(),
            texture_image: TextureImage::default(),
            data: vec![0; TMEM_SIZE],
            bind_group_layout,
            textures: vec![null_texture],
        }
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn set_texture_image(&mut self, texture_image: TextureImage) {
        self.texture_image = texture_image;
        trace!("{:?}", self.texture_image);
    }

    pub fn set_tile(&mut self, tile_id: u32, params: SetTileParams) {
        let tile = &mut self.tiles[tile_id as usize];
        tile.format = params.format;
        tile.tmem_addr = params.tmem_addr;
        trace!("{:?}", tile);
    }

    pub fn set_tile_size(&mut self, tile_id: u32, rect: &Rectangle) {
        let tile = &mut self.tiles[tile_id as usize];
        tile.rect = Rectangle {
            xh: rect.xh,
            yh: rect.yh,
            xl: rect.xl + 1.0,
            yl: rect.yl + 1.0,
        };
        trace!("{:?}", tile);
    }

    pub fn load_tile(&mut self, tile_id: u32, rdram: &[u8], rect: &Rectangle) {
        self.set_tile_size(tile_id, rect);

        let image = &self.texture_image;
        let tile = &self.tiles[tile_id as usize];

        let width = ((image.dram_width + 1) * image.format.bits_per_pixel()) >> 3;

        // let x_start = (rect.xh as usize * tile.format.bits_per_pixel()) >> 3;
        // let x_end = ((rect.xl as usize + 1) * tile.format.bits_per_pixel()) >> 3;
        let y_start = rect.yh as usize;
        let y_end = rect.yl as usize + 1;

        let tmem = &mut self.data[tile.tmem_addr as usize..];

        let mut tmem_iter = tmem.iter_mut();
        let mut dram_start = image.dram_addr;

        for line in y_start..y_end {
            let dram: &[u64] = bytemuck::cast_slice(&rdram[dram_start..(dram_start + width)]);

            for src in dram {
                *tmem_iter.next().unwrap() = interleave(line, *src);
            }

            dram_start += width;
        }

        trace!(
            "  Tile {}: {} bytes uploaded from {:08X} to {:03X}",
            tile_id,
            (y_end - y_start) * width,
            image.dram_addr,
            tile.tmem_addr,
        );
    }

    pub fn load_block(&mut self, tile_id: u32, rdram: &[u8], sl: u32, tl: u32, sh: u32, dxt: u32) {
        let image = &self.texture_image;
        let tile = &self.tiles[tile_id as usize];

        let len = ((sh as usize - sl as usize + 1) * tile.format.bits_per_pixel()) >> 6;
        let mut line = tl as usize;
        let mut y_delta = 0;

        let tmem = &mut self.data[tile.tmem_addr as usize..];
        let dram: &[u64] = bytemuck::cast_slice(&rdram[image.dram_addr..]);

        for (dst, src) in tmem.iter_mut().zip(dram.iter()) {
            *dst = interleave(line, *src);

            y_delta += dxt;

            if y_delta >= 2048 {
                y_delta -= 2048;
                line += 1;
            }
        }

        trace!(
            "  Tile {}: {} bytes uploaded from {:08X} to {:03X}",
            tile_id,
            len * 8,
            image.dram_addr,
            tile.tmem_addr,
        );
    }

    pub fn request_key(&mut self, ctx: &WgpuContext, tile_id: u32) -> (usize, (f32, f32)) {
        // TODO: Avoid uploading duplicates

        let tile = &self.tiles[tile_id as usize];
        let TileDescriptor { format, rect, .. } = &tile;

        let width = rect.xl - rect.xh;
        let height = rect.yl - rect.yh;

        // TODO: Use X and Y offsets
        let start = tile.tmem_addr as usize;
        let end = (start + ((width as usize * height as usize * format.bits_per_pixel()) >> 6))
            .min(TMEM_SIZE);

        let mut data = Vec::from(&self.data[start..end.min(TMEM_SIZE)]);

        // Reverse interleaving
        let mut word_offset: usize = 0;
        let words_per_line = (width as usize * format.bits_per_pixel()) >> 6;

        for line in rect.yh as usize..rect.yl as usize {
            if (line & 1) != 0 {
                for word in &mut data[word_offset..(word_offset + words_per_line)] {
                    *word = (*word << 32) | (*word >> 32);
                }
            }

            word_offset += words_per_line;
        }

        // TODO: Shifting and mirroring
        let mut decode_buffer = Vec::new();
        let data = format.decode(&mut decode_buffer, bytemuck::cast_slice(&data));

        let texture = Texture::new(
            ctx,
            &self.bind_group_layout,
            TextureParams {
                format: wgpu::TextureFormat::Rgba8Unorm,
                width: width as u32,
                height: (data.len() / (width as u32 * 4) as usize)
                    .try_into()
                    .unwrap(),
                data,
            },
        );

        let key = self.textures.len();

        self.textures.push(texture);

        (key, (width, height))
    }

    pub fn bind_group_from_key(&self, index: usize) -> &wgpu::BindGroup {
        self.textures[index].bind_group()
    }
}

fn interleave(line: usize, input: u64) -> u64 {
    if (line & 1) != 0 {
        (input << 32) | (input >> 32)
    } else {
        input
    }
}
