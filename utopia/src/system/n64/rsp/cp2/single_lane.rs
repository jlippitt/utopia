use super::Cp2;
use crate::core::mips::{Bus, Core};
use bitfield_struct::bitfield;
use tracing::trace;

#[bitfield(u32)]
struct SingleLane {
    #[bits(6)]
    opcode: u32,
    #[bits(5)]
    vd: usize,
    #[bits(5)]
    vd_elem: usize,
    #[bits(5)]
    vt: usize,
    #[bits(4)]
    vt_elem: usize,
    #[bits(7)]
    __: u32,
}

pub fn vmov(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    single_lane("VMOV", core, word, |_cp2, input| input);
}

pub fn vrcp(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    calc_reciprocal("VRCP", core, word, input_double, value_reciprocal, 0);
}

pub fn vrsq(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    calc_reciprocal("VRSQ", core, word, input_double, value_inv_sqrt, 1);
}

pub fn vrcpl(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    calc_reciprocal("VRCPL", core, word, input_low, value_reciprocal, 0);
}

pub fn vrsql(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    calc_reciprocal("VRSQL", core, word, input_low, value_inv_sqrt, 1);
}

pub fn vrcph(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    single_lane("VRCPH", core, word, |cp2, input| {
        cp2.div_in = (input as u32) << 16;
        (cp2.div_out >> 16) as u16
    });
}

pub fn vrsqh(core: &mut Core<impl Bus<Cp2 = Cp2>>, word: u32) {
    single_lane("VRSQH", core, word, |cp2, input| {
        cp2.div_in = (input as u32) << 16;
        (cp2.div_out >> 16) as u16
    });
}

pub fn vnop(core: &mut Core<impl Bus<Cp2 = Cp2>>, _word: u32) {
    trace!("{:08X} VNOP", core.pc());
}

pub fn vnull(core: &mut Core<impl Bus<Cp2 = Cp2>>, _word: u32) {
    trace!("{:08X} VNULL", core.pc());
}

fn single_lane(
    name: &'static str,
    core: &mut Core<impl Bus<Cp2 = Cp2>>,
    word: u32,
    cb: impl Fn(&mut Cp2, u16) -> u16,
) {
    let op = SingleLane::from(word);

    trace!(
        "{:08X} {} V{:02},E({}), V{:02},E({})",
        core.pc(),
        name,
        op.vd(),
        op.vd_elem(),
        op.vt(),
        op.vt_elem(),
    );

    let vd_elem = op.vd_elem() & 0b111;

    let vt_elem = match op.vt_elem() & 15 {
        0..=1 => op.vd_elem() & 0b111,
        2..=3 => (op.vd_elem() & 0b110) | (op.vt_elem() & 0b001),
        4..=7 => (op.vd_elem() & 0b100) | (op.vt_elem() & 0b011),
        8..=15 => op.vt_elem() & 0b111,
        _ => unreachable!(),
    };

    let cp2 = core.cp2_mut();

    let acc_words = cp2.acc_le();
    let input_words = cp2.broadcast_le(op.vt(), op.vt_elem());
    cp2.set_acc_le(std::array::from_fn(|index| {
        (acc_words[index] & !0xffff) | (input_words[index] as u64)
    }));

    let input = cp2.lane(op.vt(), vt_elem);
    let result = cb(cp2, input);
    cp2.set_lane(op.vd(), vd_elem, result);
}

fn calc_reciprocal(
    name: &'static str,
    core: &mut Core<impl Bus<Cp2 = Cp2>>,
    word: u32,
    input_cb: impl Fn(&Cp2, u16) -> i32,
    value_cb: impl Fn(&Cp2, usize, usize) -> u16,
    mod_shift: u32,
) {
    single_lane(name, core, word, |cp2, input| {
        let input = input_cb(cp2, input);
        let mask = input >> 31;
        let div_in = input.wrapping_abs();

        let result = match div_in as u32 {
            0 => 0x7fff_ffff,
            0xffff_8000 => 0xffff_0000,
            _ => {
                let shift = div_in.leading_zeros();
                let index = ((div_in << shift) & 0x7fc0_0000) >> 22;
                let value = value_cb(cp2, index as usize, shift as usize);
                ((((0x10000 | value as u32 as i32) << 14) >> ((31 - shift) >> mod_shift)) ^ mask)
                    as u32
            }
        };

        cp2.div_in = div_in as u32;
        cp2.div_out = result;

        result as u16
    });
}

fn input_double(_cp2: &Cp2, input: u16) -> i32 {
    input as i16 as i32
}

fn input_low(cp2: &Cp2, input: u16) -> i32 {
    ((cp2.div_in & 0xffff_0000) as i32) | input as i16 as i32
}

fn value_reciprocal(cp2: &Cp2, index: usize, _shift: usize) -> u16 {
    cp2.reciprocal[index]
}

fn value_inv_sqrt(cp2: &Cp2, index: usize, shift: usize) -> u16 {
    cp2.inv_sqrt[(index & 0x1fe) | (shift & 1)]
}
