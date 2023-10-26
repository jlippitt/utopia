use super::fragment::{CombineMode, CombinerInputs, CycleType};
use super::primitive::{Rectangle, TextureLayout};
use super::target::OutputFormat;
use super::{Core, Mode};
use bitfield_struct::bitfield;
use tracing::trace;

mod rectangle;
mod texture;
mod triangle;

pub fn dispatch(
    core: &mut Core,
    rdram: &mut [u8],
    cmd: u64,
    commands: &mut impl Iterator<Item = u64>,
) {
    match (cmd >> 56) & 0x3f {
        0x08 => triangle::fill_triangle(core, rdram, cmd, commands),
        0x09 => triangle::fill_z_buffer_triangle(core, rdram, cmd, commands),
        0x0a => triangle::texture_triangle(core, rdram, cmd, commands),
        0x0b => triangle::texture_z_buffer_triangle(core, rdram, cmd, commands),
        0x0c => triangle::shade_triangle(core, rdram, cmd, commands),
        0x0d => triangle::shade_z_buffer_triangle(core, rdram, cmd, commands),
        0x0e => triangle::shade_texture_triangle(core, rdram, cmd, commands),
        0x0f => triangle::shade_texture_z_buffer_triangle(core, rdram, cmd, commands),
        0x24 => rectangle::texture_rectangle(core, rdram, cmd, commands),
        0x25 => rectangle::texture_rectangle_flip(core, rdram, cmd, commands),
        0x27 => sync_pipe(core, rdram, cmd),
        0x28 => texture::sync_tile(core, rdram, cmd),
        0x29 => sync_full(core, rdram, cmd),
        0x2d => set_scissor(core, rdram, cmd),
        0x2e => set_prim_depth(core, rdram, cmd),
        0x2f => set_other_modes(core, rdram, cmd),
        0x32 => texture::set_tile_size(core, rdram, cmd),
        0x33 => texture::load_block(core, rdram, cmd),
        0x34 => texture::load_tile(core, rdram, cmd),
        0x35 => texture::set_tile(core, rdram, cmd),
        0x36 => rectangle::fill_rectangle(core, rdram, cmd),
        0x37 => set_fill_color(core, rdram, cmd),
        0x38 => set_fog_color(core, rdram, cmd),
        0x39 => set_blend_color(core, rdram, cmd),
        0x3a => set_prim_color(core, rdram, cmd),
        0x3b => set_env_color(core, rdram, cmd),
        0x3c => set_combine_mode(core, rdram, cmd),
        0x3d => texture::set_texture_image(core, rdram, cmd),
        0x3f => set_color_image(core, rdram, cmd),
        opcode => trace!("{:02X}: {:016X}", opcode, cmd),
    }
}

fn sync_pipe(_core: &mut Core, _rdram: &mut [u8], _cmd: u64) {
    trace!("SYNC_PIPE");
}

fn sync_full(core: &mut Core, _rdram: &mut [u8], _cmd: u64) {
    trace!("SYNC_FULL");
    core.set_interrupt(true);
}

fn set_scissor(core: &mut Core, _rdram: &mut [u8], cmd: u64) {
    let op = SetScissor::from(cmd);

    trace!("SET_SCISSOR: {:?}", op);

    core.target.set_scissor(Rectangle {
        xh: op.xh() as f32 / 4.0,
        yh: op.yh() as f32 / 4.0,
        xl: op.xl() as f32 / 4.0,
        yl: op.yl() as f32 / 4.0,
    });
}

fn set_prim_depth(core: &mut Core, _rdram: &mut [u8], cmd: u64) {
    let z = cmd as u32 as f32 / 65536.0;
    trace!("SET_PRIM_DEPTH: {}", z);
    core.set_prim_depth(z);
}

fn set_other_modes(core: &mut Core, _rdram: &mut [u8], cmd: u64) {
    let op = SetOtherModes::from(cmd);

    trace!("SET_OTHER_MODES: {:?}", op);

    core.set_mode(Mode {
        use_prim_depth: op.z_source_sel(),
    });

    core.fragment.set_cycle_type(match op.cycle_type() {
        0 => CycleType::Cycle1,
        1 => CycleType::Cycle2,
        2 => CycleType::Copy,
        _ => CycleType::Fill,
    });

    core.fragment.set_blend_mode(op.blend_mode());
}

fn set_fill_color(core: &mut Core, _rdram: &mut [u8], cmd: u64) {
    let op = cmd as u32;
    trace!("SET_FILL_COLOR: {:08X}", op);
    core.target.set_fill_color(op);
}

fn set_blend_color(core: &mut Core, _rdram: &mut [u8], cmd: u64) {
    let op = Color::from(cmd);

    trace!("SET_BLEND_COLOR: {:?}", op);

    core.fragment.set_blend_color([
        op.red() as f32 / 255.0,
        op.green() as f32 / 255.0,
        op.blue() as f32 / 255.0,
        op.alpha() as f32 / 255.0,
    ]);
}

fn set_fog_color(core: &mut Core, _rdram: &mut [u8], cmd: u64) {
    let op = Color::from(cmd);

    trace!("SET_FOG_COLOR: {:?}", op);

    core.fragment.set_fog_color([
        op.red() as f32 / 255.0,
        op.green() as f32 / 255.0,
        op.blue() as f32 / 255.0,
        op.alpha() as f32 / 255.0,
    ]);
}

fn set_prim_color(core: &mut Core, _rdram: &mut [u8], cmd: u64) {
    // TODO: There are two additional parameters here that we don't yet handle
    let op = Color::from(cmd);

    trace!("SET_PRIM_COLOR: {:?}", op);

    core.fragment.set_prim_color([
        op.red() as f32 / 255.0,
        op.green() as f32 / 255.0,
        op.blue() as f32 / 255.0,
        op.alpha() as f32 / 255.0,
    ]);
}

fn set_env_color(core: &mut Core, _rdram: &mut [u8], cmd: u64) {
    let op = Color::from(cmd);

    trace!("SET_ENV_COLOR: {:?}", op);

    core.fragment.set_env_color([
        op.red() as f32 / 255.0,
        op.green() as f32 / 255.0,
        op.blue() as f32 / 255.0,
        op.alpha() as f32 / 255.0,
    ]);
}

fn set_combine_mode(core: &mut Core, _rdram: &mut [u8], cmd: u64) {
    let op = SetCombineMode::from(cmd);

    trace!("SET_COMBINE_MODE: {:?}", op);

    core.fragment.set_combine_mode(CombineMode {
        rgb0: CombinerInputs {
            sub_a: op.sub_a_rgb_0(),
            sub_b: op.sub_b_rgb_0(),
            mul: op.mul_rgb_0(),
            add: op.add_rgb_0(),
        },
        alpha0: CombinerInputs {
            sub_a: op.sub_a_alpha_0(),
            sub_b: op.sub_b_alpha_0(),
            mul: op.mul_alpha_0(),
            add: op.add_alpha_0(),
        },
        rgb1: CombinerInputs {
            sub_a: op.sub_a_rgb_1(),
            sub_b: op.sub_b_rgb_1(),
            mul: op.mul_rgb_1(),
            add: op.add_rgb_1(),
        },
        alpha1: CombinerInputs {
            sub_a: op.sub_a_alpha_1(),
            sub_b: op.sub_b_alpha_1(),
            mul: op.mul_alpha_1(),
            add: op.add_alpha_1(),
        },
    });
}

fn set_color_image(core: &mut Core, _rdram: &mut [u8], cmd: u64) {
    let op = Image::from(cmd);

    trace!("SET_COLOR_IMAGE: {:?}", op);

    let output_format = match (op.format(), op.size()) {
        (TextureLayout::Rgba, 3) => OutputFormat::Rgba32,
        (TextureLayout::Rgba, 2) => OutputFormat::Rgba16,
        (TextureLayout::ColorIndex, 1) => OutputFormat::Index8,
        _ => panic!("Invalid output format"),
    };

    core.target
        .set_color_image(op.dram_addr(), op.width() + 1, output_format);
}

#[bitfield(u64)]
struct SetScissor {
    #[bits(12)]
    yl: u32,
    #[bits(12)]
    xl: u32,
    odd_line: bool,
    field: bool,
    #[bits(6)]
    __: u32,
    #[bits(12)]
    yh: u32,
    #[bits(12)]
    xh: u32,
    __: u8,
}

#[bitfield(u64)]
struct SetOtherModes {
    alpha_compare_en: bool,
    dither_alpha_en: bool,
    z_source_sel: bool,
    antialias_en: bool,
    z_compare_en: bool,
    z_update_en: bool,
    image_read_en: bool,
    color_on_cvg: bool,
    #[bits(2)]
    cvg_dest: u32,
    #[bits(2)]
    z_mode: u32,
    cvg_times_alpha: bool,
    alpha_cvg_select: bool,
    force_blend: bool,
    __: bool,
    blend_mode: u16,
    #[bits(4)]
    __: u32,
    #[bits(2)]
    alpha_dither_sel: u32,
    #[bits(2)]
    rgb_dither_sel: u32,
    key_en: bool,
    convert_one: bool,
    bi_lerp_1: bool,
    bi_lerp_0: bool,
    mid_texel: bool,
    sample_type: bool,
    tlut_type: bool,
    en_tlut: bool,
    tex_lod_en: bool,
    sharpen_tex_en: bool,
    detail_tex_en: bool,
    persp_text_en: bool,
    #[bits(2)]
    cycle_type: u32,
    __: bool,
    atomic_prim: bool,
    __: u8,
}

#[bitfield(u64)]
struct Color {
    alpha: u8,
    blue: u8,
    green: u8,
    red: u8,
    __: u32,
}

#[bitfield(u64)]
struct SetCombineMode {
    #[bits(3)]
    add_alpha_1: u8,
    #[bits(3)]
    sub_b_alpha_1: u8,
    #[bits(3)]
    add_rgb_1: u8,
    #[bits(3)]
    add_alpha_0: u8,
    #[bits(3)]
    sub_b_alpha_0: u8,
    #[bits(3)]
    add_rgb_0: u8,
    #[bits(3)]
    mul_alpha_1: u8,
    #[bits(3)]
    sub_a_alpha_1: u8,
    #[bits(4)]
    sub_b_rgb_1: u8,
    #[bits(4)]
    sub_b_rgb_0: u8,
    #[bits(5)]
    mul_rgb_1: u8,
    #[bits(4)]
    sub_a_rgb_1: u8,
    #[bits(3)]
    mul_alpha_0: u8,
    #[bits(3)]
    sub_a_alpha_0: u8,
    #[bits(5)]
    mul_rgb_0: u8,
    #[bits(4)]
    sub_a_rgb_0: u8,
    __: u8,
}

#[bitfield(u64)]
struct Image {
    #[bits(26)]
    dram_addr: u32,
    #[bits(6)]
    __: u32,
    #[bits(10)]
    width: u32,
    #[bits(9)]
    __: u32,
    #[bits(2)]
    size: u32,
    #[bits(3)]
    format: TextureLayout,
    __: u8,
}
