use winit::dpi::{PhysicalPosition, PhysicalSize};

pub fn upscale(source: PhysicalSize<u32>, target: PhysicalSize<u32>) -> PhysicalSize<u32> {
    let width_ratio = target.width / source.width;
    let height_ratio = target.height / source.height;
    let scale = width_ratio.min(height_ratio);
    (source.width * scale, source.height * scale).into()
}

pub fn center(source: PhysicalSize<u32>, target: PhysicalSize<u32>) -> PhysicalPosition<u32> {
    let pos_x = (target.width - source.width) / 2;
    let pos_y = (target.height - source.height) / 2;
    (pos_x, pos_y).into()
}
