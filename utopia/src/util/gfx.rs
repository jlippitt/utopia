use crate::WgpuContext;

pub fn write_pixels_to_texture(
    ctx: &WgpuContext,
    texture: &wgpu::Texture,
    pixels: &[u8],
    pitch: usize,
) {
    ctx.queue.write_texture(
        wgpu::ImageCopyTexture {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        pixels,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(pitch.try_into().unwrap()),
            rows_per_image: Some((pixels.len() / pitch).try_into().unwrap()),
        },
        texture.size(),
    );
}
