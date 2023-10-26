use super::super::primitive::{Color, Position};
use super::super::Core;
use bitfield_struct::bitfield;
use tracing::trace;

type Edge = [f32; 2];

pub fn fill_triangle(
    core: &mut Core,
    _rdram: &mut [u8],
    cmd: u64,
    reader: &mut impl Iterator<Item = u64>,
) {
    let (edges, _) = triangle("FILL_TRIANGLE", cmd, reader);
    let color = fill(core);
    let position = prim_depth(core, edges);
    core.push_triangle(position, color, None);
}

pub fn fill_z_buffer_triangle(
    core: &mut Core,
    _rdram: &mut [u8],
    cmd: u64,
    reader: &mut impl Iterator<Item = u64>,
) {
    let (edges, _) = triangle("FILL_Z_BUFFER_TRIANGLE", cmd, reader);
    let color = fill(core);
    let position = z_buffer(core, reader, edges, &Offsets::new(&edges));
    core.push_triangle(position, color, None);
}

pub fn texture_triangle(
    core: &mut Core,
    _rdram: &mut [u8],
    cmd: u64,
    reader: &mut impl Iterator<Item = u64>,
) {
    let (edges, tile) = triangle("TEXTURE_TRIANGLE", cmd, reader);
    let color = Default::default();
    let tex_coords = texture(reader, &Offsets::new(&edges));
    let position = prim_depth(core, edges);
    core.push_triangle(position, color, Some((tile, tex_coords)));
}

pub fn texture_z_buffer_triangle(
    core: &mut Core,
    _rdram: &mut [u8],
    cmd: u64,
    reader: &mut impl Iterator<Item = u64>,
) {
    let (edges, tile) = triangle("TEXTURE_Z_BUFFER_TRIANGLE", cmd, reader);
    let color = Default::default();
    let offsets = Offsets::new(&edges);
    let tex_coords = texture(reader, &offsets);
    let position = z_buffer(core, reader, edges, &offsets);
    core.push_triangle(position, color, Some((tile, tex_coords)));
}

pub fn shade_triangle(
    core: &mut Core,
    _rdram: &mut [u8],
    cmd: u64,
    reader: &mut impl Iterator<Item = u64>,
) {
    let (edges, _) = triangle("SHADE_TRIANGLE", cmd, reader);
    let color = shade(reader, &Offsets::new(&edges));
    let position = prim_depth(core, edges);
    core.push_triangle(position, color, None);
}

pub fn shade_z_buffer_triangle(
    core: &mut Core,
    _rdram: &mut [u8],
    cmd: u64,
    reader: &mut impl Iterator<Item = u64>,
) {
    let (edges, _) = triangle("SHADE_Z_BUFFER_TRIANGLE", cmd, reader);
    let offsets = Offsets::new(&edges);
    let color = shade(reader, &offsets);
    let position = z_buffer(core, reader, edges, &offsets);
    core.push_triangle(position, color, None);
}

pub fn shade_texture_triangle(
    core: &mut Core,
    _rdram: &mut [u8],
    cmd: u64,
    reader: &mut impl Iterator<Item = u64>,
) {
    let (edges, tile) = triangle("SHADE_TEXTURE_TRIANGLE", cmd, reader);
    let offsets = Offsets::new(&edges);
    let color = shade(reader, &offsets);
    let tex_coords = texture(reader, &offsets);
    let position = prim_depth(core, edges);
    core.push_triangle(position, color, Some((tile, tex_coords)));
}

pub fn shade_texture_z_buffer_triangle(
    core: &mut Core,
    _rdram: &mut [u8],
    cmd: u64,
    reader: &mut impl Iterator<Item = u64>,
) {
    let (edges, tile) = triangle("SHADE_TEXTURE_Z_BUFFER_TRIANGLE", cmd, reader);
    let offsets = Offsets::new(&edges);
    let color = shade(reader, &offsets);
    let tex_coords = texture(reader, &offsets);
    let position = z_buffer(core, reader, edges, &offsets);
    core.push_triangle(position, color, Some((tile, tex_coords)));
}

fn triangle(
    name: &'static str,
    cmd: u64,
    reader: &mut impl Iterator<Item = u64>,
) -> ([Edge; 3], u32) {
    let op = Triangle::from(cmd);
    let low = EdgeCoefficient::from(reader.next().unwrap());
    let high = EdgeCoefficient::from(reader.next().unwrap());
    let mid = EdgeCoefficient::from(reader.next().unwrap());

    trace!("{}: {:?}", name, op);
    trace!("  Edge Coefficients: {:?} {:?}, {:?}", low, high, mid);

    let yl = op.yl() as f32 / 4.0;
    let yh = op.yh() as f32 / 4.0;
    let ym = op.ym() as f32 / 4.0;

    let xl = low.x() as i32 as f32 / 65536.0;
    let xh = high.x() as i32 as f32 / 65536.0;
    let dxhdy = high.dxdy() as i32 as f32 / 65536.0;

    // Origin -> Major -> Minor
    let edges = [[xh, yh], [xh + (yl - yh) * dxhdy, yl], [xl, ym]];

    (edges, op.tile())
}

fn fill(core: &Core) -> [Color; 3] {
    [core.target.fill_color(); 3]
}

fn shade(reader: &mut impl Iterator<Item = u64>, offsets: &Offsets) -> [Color; 3] {
    let args = [
        reader.next().unwrap(),
        reader.next().unwrap(),
        reader.next().unwrap(),
        reader.next().unwrap(),
        reader.next().unwrap(),
        reader.next().unwrap(),
        reader.next().unwrap(),
        reader.next().unwrap(),
    ];

    let base = coefficient(args[0], args[2]);
    let dx = coefficient(args[1], args[3]);
    let de = coefficient(args[4], args[6]);
    let dy = coefficient(args[5], args[7]);

    trace!(
        "  Shade Coefficients: Base={:?} DX={:?} DE={:?} DY={:?}",
        base,
        dx,
        de,
        dy
    );

    [base, offsets.major(base, de), offsets.minor(base, de, dx)]
        .map(|color| color.map(|component| component.clamp(0.0, 255.0) / 255.0))
}

fn texture(reader: &mut impl Iterator<Item = u64>, offsets: &Offsets) -> [[f32; 4]; 3] {
    let args = [
        reader.next().unwrap(),
        reader.next().unwrap(),
        reader.next().unwrap(),
        reader.next().unwrap(),
        reader.next().unwrap(),
        reader.next().unwrap(),
        reader.next().unwrap(),
        reader.next().unwrap(),
    ];

    let base = coefficient(args[0], args[2]);
    let dx = coefficient(args[1], args[3]);
    let de = coefficient(args[4], args[6]);
    let dy = coefficient(args[5], args[7]);

    trace!(
        "  Texture Coefficients: Base={:?} DX={:?} DE={:?} DY={:?}",
        base,
        dx,
        de,
        dy
    );

    [base, offsets.major(base, de), offsets.minor(base, de, dx)]
        .map(|coord| coord.map(|component| component.clamp(0.0, 255.0) / 255.0))
}

fn prim_depth(core: &Core, edges: [Edge; 3]) -> [Position; 3] {
    edges.map(|[x, y]| [x, y, core.prim_depth()])
}

fn z_buffer(
    core: &Core,
    reader: &mut impl Iterator<Item = u64>,
    edges: [Edge; 3],
    offsets: &Offsets,
) -> [Position; 3] {
    let args = [reader.next().unwrap(), reader.next().unwrap()];

    if core.mode().use_prim_depth {
        return prim_depth(core, edges);
    }

    let base = (args[0] >> 32) as i32 as f32 / 65536.0;
    let dx = args[0] as i32 as f32 / 65536.0;
    let de = (args[1] >> 32) as i32 as f32 / 65536.0;
    let dy = args[1] as i32 as f32 / 65536.0;

    trace!("  Z-Buffer Coefficients: {}, {}, {}, {}", base, dx, de, dy);

    let major = offsets.major_component(base, de * 0.5);
    let minor = offsets.minor_component(base, de * 0.5, dx * 0.5);

    [
        [edges[0][0], edges[0][1], base],
        [edges[1][0], edges[1][1], major],
        [edges[2][0], edges[2][1], minor],
    ]
}

#[bitfield(u64)]
struct Triangle {
    #[bits(14)]
    yh: i32,
    #[bits(2)]
    __: u32,
    #[bits(14)]
    ym: i32,
    #[bits(2)]
    __: u32,
    #[bits(14)]
    yl: i32,
    #[bits(2)]
    __: u32,
    #[bits(3)]
    tile: u32,
    #[bits(3)]
    level: u32,
    __: bool,
    right: bool,
    __: u8,
}

#[bitfield(u64)]
struct EdgeCoefficient {
    #[bits(32)]
    dxdy: u32,
    #[bits(32)]
    x: u32,
}

struct Offsets {
    major_e: f32,
    minor_e: f32,
    minor_x: f32,
}

impl Offsets {
    fn new(edges: &[Edge; 3]) -> Self {
        let major_e = edges[1][1] - edges[0][1];
        let major_x = edges[1][0] - edges[0][0];
        let minor_e = edges[2][1] - edges[0][1];
        let minor_x = edges[2][0] - edges[0][0] - major_x * minor_e / major_e;

        Self {
            major_e,
            minor_e,
            minor_x,
        }
    }

    fn major_component(&self, base: f32, de: f32) -> f32 {
        base + self.major_e * de
    }

    fn minor_component(&self, base: f32, de: f32, dx: f32) -> f32 {
        base + self.minor_e * de + self.minor_x * dx
    }

    fn major(&self, base: [f32; 4], de: [f32; 4]) -> [f32; 4] {
        std::array::from_fn(|index| self.major_component(base[index], de[index]))
    }

    fn minor(&self, base: [f32; 4], de: [f32; 4], dx: [f32; 4]) -> [f32; 4] {
        std::array::from_fn(|index| self.minor_component(base[index], de[index], dx[index]))
    }
}

fn coefficient(integer: u64, fraction: u64) -> [f32; 4] {
    [
        component(integer, fraction, 48),
        component(integer, fraction, 32),
        component(integer, fraction, 16),
        component(integer, fraction, 0),
    ]
}

fn component(integer: u64, fraction: u64, shift: u32) -> f32 {
    ((((integer >> shift) as i16 as i32) << 16) | ((fraction >> shift) as i16 as i32)) as f32
        / 65536.0
}
