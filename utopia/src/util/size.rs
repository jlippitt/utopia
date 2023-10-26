#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl From<(u32, u32)> for Size {
    fn from((width, height): (u32, u32)) -> Self {
        Size { width, height }
    }
}

impl From<Size> for (u32, u32) {
    fn from(value: Size) -> Self {
        (value.width, value.height)
    }
}

impl From<wgpu::Extent3d> for Size {
    fn from(value: wgpu::Extent3d) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

impl From<Size> for wgpu::Extent3d {
    fn from(value: Size) -> Self {
        wgpu::Extent3d {
            width: value.width,
            height: value.height,
            depth_or_array_layers: 1,
        }
    }
}
