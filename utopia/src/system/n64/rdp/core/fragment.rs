use super::primitive::Color;
use buffer::{Buffer, FragmentState};
use std::fmt::{self, Debug};
use tracing::trace;

mod buffer;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CombinerInputs<T = u8> {
    pub sub_a: T,
    pub sub_b: T,
    pub mul: T,
    pub add: T,
}

#[derive(Debug)]
pub struct CombineMode {
    pub rgb0: CombinerInputs<u8>,
    pub alpha0: CombinerInputs<u8>,
    pub rgb1: CombinerInputs<u8>,
    pub alpha1: CombinerInputs<u8>,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[repr(i32)]
pub enum CycleType {
    #[default]
    Cycle1 = 0,
    Cycle2 = 1,
    Copy = 2,
    Fill = 3,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[repr(i32)]
enum RgbInput {
    #[default]
    PrevColor = 0,
    Texel0Color = 1,
    Texel1Color = 2,
    PrimColor = 3,
    ShadeColor = 4,
    EnvColor = 5,
    KeyCenter = 6,
    KeyScale = 7,
    PrevAlpha = 8,
    Texel0Alpha = 9,
    Texel1Alpha = 10,
    PrimAlpha = 11,
    ShadeAlpha = 12,
    EnvAlpha = 13,
    LodFraction = 14,
    PrimLodFraction = 15,
    Noise = 16,
    ConvertK4 = 17,
    ConvertK5 = 18,
    Constant1 = 19,
    Constant0 = 20,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[repr(i32)]
enum AlphaInput {
    #[default]
    PrevAlpha = 0,
    Texel0Alpha = 1,
    Texel1Alpha = 2,
    PrimAlpha = 3,
    ShadeAlpha = 4,
    EnvAlpha = 5,
    LodFraction = 6,
    PrimLodFraction = 7,
    Constant1 = 8,
    Constant0 = 9,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct BlenderInputs {
    p: BlenderPmInput,
    m: BlenderPmInput,
    a: BlenderAInput,
    b: BlenderBInput,
}

#[allow(clippy::enum_variant_names)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[repr(i32)]
enum BlenderPmInput {
    #[default]
    PrevColor = 0,
    MemoryColor = 1,
    BlendColor = 2,
    FogColor = 3,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[repr(i32)]
enum BlenderAInput {
    #[default]
    PrevAlpha = 0,
    FogAlpha = 1,
    ShadeAlpha = 2,
    Constant0 = 3,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[repr(i32)]
enum BlenderBInput {
    #[default]
    InvAlpha = 0,
    MemoryAlpha = 1,
    Constant1 = 2,
    Constant0 = 3,
}

pub struct FragmentControl {
    rgb0: CombinerInputs<RgbInput>,
    alpha0: CombinerInputs<AlphaInput>,
    rgb1: CombinerInputs<RgbInput>,
    alpha1: CombinerInputs<AlphaInput>,
    blend0: BlenderInputs,
    blend1: BlenderInputs,
    prim_color: Color,
    env_color: Color,
    blend_color: Color,
    fog_color: Color,
    cycle_type: CycleType,
    dirty: bool,
    bind_group_layout: wgpu::BindGroupLayout,
    buffers: Vec<Buffer>,
}

impl FragmentControl {
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("RDP Combiner Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        Self {
            rgb0: CombinerInputs::default(),
            alpha0: CombinerInputs::default(),
            rgb1: CombinerInputs::default(),
            alpha1: CombinerInputs::default(),
            blend0: BlenderInputs::default(),
            blend1: BlenderInputs::default(),
            prim_color: Color::default(),
            env_color: Color::default(),
            blend_color: Color::default(),
            fog_color: Color::default(),
            cycle_type: CycleType::default(),
            dirty: true,
            bind_group_layout,
            buffers: Vec::new(),
        }
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn set_combine_mode(&mut self, mode: CombineMode) {
        let rgb0 = mode.rgb0.into();
        let alpha0 = mode.alpha0.into();
        let rgb1 = mode.rgb1.into();
        let alpha1 = mode.alpha1.into();

        self.dirty = self.dirty
            || rgb0 != self.rgb0
            || alpha0 != self.alpha0
            || rgb1 != self.rgb1
            || alpha1 != self.alpha1;

        self.rgb0 = rgb0;
        self.alpha0 = alpha0;
        self.rgb1 = rgb1;
        self.alpha1 = alpha1;

        trace!("Cycle 0 RGB: {}", self.rgb0);
        trace!("Cycle 0 Alpha: {}", self.alpha0);
        trace!("Cycle 1 RGB: {}", self.rgb1);
        trace!("Cycle 1 Alpha: {}", self.alpha1);
    }

    pub fn set_cycle_type(&mut self, cycle_type: CycleType) {
        self.dirty = self.dirty || cycle_type != self.cycle_type;
        self.cycle_type = cycle_type;
        trace!("Cycle Type: {:?}", cycle_type);
    }

    pub fn set_blend_mode(&mut self, blend_mode: u16) {
        let blend0 = blend_mode.into();
        let blend1 = (blend_mode >> 2).into();

        self.dirty = self.dirty || blend0 != self.blend0 || blend1 != self.blend1;

        self.blend0 = blend0;
        self.blend1 = blend1;

        trace!("Cycle 0 Blend: {}", self.blend0);
        trace!("Cycle 1 Blend: {}", self.blend1);
    }

    pub fn set_prim_color(&mut self, color: Color) {
        self.dirty = self.dirty || color != self.prim_color;
        self.prim_color = color;
    }

    pub fn set_env_color(&mut self, color: Color) {
        self.dirty = self.dirty || color != self.env_color;
        self.env_color = color;
    }

    pub fn set_blend_color(&mut self, color: Color) {
        self.dirty = self.dirty || color != self.blend_color;
        self.blend_color = color;
    }

    pub fn set_fog_color(&mut self, color: Color) {
        self.dirty = self.dirty || color != self.fog_color;
        self.fog_color = color;
    }

    pub fn cache_key(&mut self, device: &wgpu::Device) -> usize {
        if self.dirty {
            self.buffers.push(Buffer::new(
                device,
                &self.bind_group_layout,
                FragmentState {
                    rgb0: [
                        self.rgb0.sub_a as i32,
                        self.rgb0.sub_b as i32,
                        self.rgb0.mul as i32,
                        self.rgb0.add as i32,
                    ],
                    alpha0: [
                        self.alpha0.sub_a as i32,
                        self.alpha0.sub_b as i32,
                        self.alpha0.mul as i32,
                        self.alpha0.add as i32,
                    ],
                    rgb1: [
                        self.rgb1.sub_a as i32,
                        self.rgb1.sub_b as i32,
                        self.rgb1.mul as i32,
                        self.rgb1.add as i32,
                    ],
                    alpha1: [
                        self.alpha1.sub_a as i32,
                        self.alpha1.sub_b as i32,
                        self.alpha1.mul as i32,
                        self.alpha1.add as i32,
                    ],
                    blend0: [
                        self.blend0.p as i32,
                        self.blend0.m as i32,
                        self.blend0.a as i32,
                        self.blend0.b as i32,
                    ],
                    blend1: [
                        self.blend1.p as i32,
                        self.blend1.m as i32,
                        self.blend1.a as i32,
                        self.blend1.b as i32,
                    ],
                    prim_color: self.prim_color,
                    env_color: self.env_color,
                    blend_color: self.blend_color,
                    fog_color: self.fog_color,
                    cycle_type: self.cycle_type as i32,
                    ..Default::default()
                },
            ));

            self.dirty = false;
        }

        self.buffers.len() - 1
    }

    pub fn bind_group_from_key(&self, key: usize) -> &wgpu::BindGroup {
        self.buffers[key].bind_group()
    }
}

impl<T: Debug> fmt::Display for CombinerInputs<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({:?} - {:?}) * {:?} + {:?}",
            self.sub_a, self.sub_b, self.mul, self.add
        )
    }
}

impl From<CombinerInputs<u8>> for CombinerInputs<RgbInput> {
    fn from(value: CombinerInputs<u8>) -> Self {
        Self {
            sub_a: match value.sub_a {
                0 => RgbInput::PrevColor,
                1 => RgbInput::Texel0Color,
                2 => RgbInput::Texel1Color,
                3 => RgbInput::PrimColor,
                4 => RgbInput::ShadeColor,
                5 => RgbInput::EnvColor,
                6 => RgbInput::Constant1,
                7 => RgbInput::Noise,
                _ => RgbInput::Constant0,
            },
            sub_b: match value.sub_b {
                0 => RgbInput::PrevColor,
                1 => RgbInput::Texel0Color,
                2 => RgbInput::Texel1Color,
                3 => RgbInput::PrimColor,
                4 => RgbInput::ShadeColor,
                5 => RgbInput::EnvColor,
                6 => RgbInput::KeyCenter,
                7 => RgbInput::ConvertK4,
                _ => RgbInput::Constant0,
            },
            mul: match value.mul {
                0 => RgbInput::PrevColor,
                1 => RgbInput::Texel0Color,
                2 => RgbInput::Texel1Color,
                3 => RgbInput::PrimColor,
                4 => RgbInput::ShadeColor,
                5 => RgbInput::EnvColor,
                6 => RgbInput::KeyScale,
                7 => RgbInput::PrevAlpha,
                8 => RgbInput::Texel0Alpha,
                9 => RgbInput::Texel1Alpha,
                10 => RgbInput::PrimAlpha,
                11 => RgbInput::ShadeAlpha,
                12 => RgbInput::EnvAlpha,
                13 => RgbInput::LodFraction,
                14 => RgbInput::PrimLodFraction,
                15 => RgbInput::ConvertK5,
                _ => RgbInput::Constant0,
            },
            add: match value.add {
                0 => RgbInput::PrevColor,
                1 => RgbInput::Texel0Color,
                2 => RgbInput::Texel1Color,
                3 => RgbInput::PrimColor,
                4 => RgbInput::ShadeColor,
                5 => RgbInput::EnvColor,
                6 => RgbInput::Constant1,
                _ => RgbInput::Constant0,
            },
        }
    }
}

impl fmt::Display for BlenderInputs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({:?} * {:?} + {:?} * {:?}) / ({:?} + {:?})",
            self.a, self.p, self.b, self.m, self.a, self.b,
        )
    }
}

impl From<CombinerInputs<u8>> for CombinerInputs<AlphaInput> {
    fn from(value: CombinerInputs<u8>) -> Self {
        Self {
            sub_a: match value.sub_a {
                0 => AlphaInput::PrevAlpha,
                1 => AlphaInput::Texel0Alpha,
                2 => AlphaInput::Texel1Alpha,
                3 => AlphaInput::PrimAlpha,
                4 => AlphaInput::ShadeAlpha,
                5 => AlphaInput::EnvAlpha,
                6 => AlphaInput::Constant1,
                _ => AlphaInput::Constant0,
            },
            sub_b: match value.sub_b {
                0 => AlphaInput::PrevAlpha,
                1 => AlphaInput::Texel0Alpha,
                2 => AlphaInput::Texel1Alpha,
                3 => AlphaInput::PrimAlpha,
                4 => AlphaInput::ShadeAlpha,
                5 => AlphaInput::EnvAlpha,
                6 => AlphaInput::Constant1,
                _ => AlphaInput::Constant0,
            },
            mul: match value.mul {
                0 => AlphaInput::LodFraction,
                1 => AlphaInput::Texel0Alpha,
                2 => AlphaInput::Texel1Alpha,
                3 => AlphaInput::PrimAlpha,
                4 => AlphaInput::ShadeAlpha,
                5 => AlphaInput::EnvAlpha,
                6 => AlphaInput::PrimLodFraction,
                _ => AlphaInput::Constant0,
            },
            add: match value.add {
                0 => AlphaInput::PrevAlpha,
                1 => AlphaInput::Texel0Alpha,
                2 => AlphaInput::Texel1Alpha,
                3 => AlphaInput::PrimAlpha,
                4 => AlphaInput::ShadeAlpha,
                5 => AlphaInput::EnvAlpha,
                6 => AlphaInput::Constant1,
                _ => AlphaInput::Constant0,
            },
        }
    }
}

impl From<u16> for BlenderInputs {
    fn from(value: u16) -> Self {
        Self {
            p: match (value >> 12) & 3 {
                0 => BlenderPmInput::PrevColor,
                1 => BlenderPmInput::MemoryColor,
                2 => BlenderPmInput::BlendColor,
                _ => BlenderPmInput::FogColor,
            },
            m: match (value >> 4) & 3 {
                0 => BlenderPmInput::PrevColor,
                1 => BlenderPmInput::MemoryColor,
                2 => BlenderPmInput::BlendColor,
                _ => BlenderPmInput::FogColor,
            },
            a: match (value >> 8) & 3 {
                0 => BlenderAInput::PrevAlpha,
                1 => BlenderAInput::FogAlpha,
                2 => BlenderAInput::ShadeAlpha,
                _ => BlenderAInput::Constant0,
            },
            b: match (value >> 2) & 3 {
                0 => BlenderBInput::InvAlpha,
                1 => BlenderBInput::MemoryAlpha,
                2 => BlenderBInput::Constant1,
                _ => BlenderBInput::Constant0,
            },
        }
    }
}
