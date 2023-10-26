use super::{Bus, Core, Cp0, Cp1, Cp2};

mod arithmetic;
mod control;
mod load;
mod logic;
mod misc;
mod mul_div;
mod shift;
mod store;
mod trap;

pub fn dispatch<T: Bus>(core: &mut Core<T>, word: u32) {
    match word >> 26 {
        0o00 => special(core, word),
        0o01 => regimm(core, word),
        0o02 => control::j::<false>(core, word),
        0o03 => control::j::<true>(core, word),
        0o04 => control::beq::<false>(core, word),
        0o05 => control::bne::<false>(core, word),
        0o06 => control::blez::<false>(core, word),
        0o07 => control::bgtz::<false>(core, word),
        0o10 => arithmetic::addi(core, word),
        0o11 => arithmetic::addiu(core, word),
        0o12 => arithmetic::slti(core, word),
        0o13 => arithmetic::sltiu(core, word),
        0o14 => logic::andi(core, word),
        0o15 => logic::ori(core, word),
        0o16 => logic::xori(core, word),
        0o17 => load::lui(core, word),
        0o20 => cop0(core, word),
        0o21 => cop1(core, word),
        0o22 => cop2(core, word),
        0o24 => control::beq::<true>(core, word),
        0o25 => control::bne::<true>(core, word),
        0o26 => control::blez::<true>(core, word),
        0o27 => control::bgtz::<true>(core, word),
        0o30 => arithmetic::daddi(core, word),
        0o31 => arithmetic::daddiu(core, word),
        0o32 => load::ldl(core, word),
        0o33 => load::ldr(core, word),
        0o40 => load::lb(core, word),
        0o41 => load::lh(core, word),
        0o42 => load::lwl(core, word),
        0o43 => load::lw(core, word),
        0o44 => load::lbu(core, word),
        0o45 => load::lhu(core, word),
        0o46 => load::lwr(core, word),
        0o47 => load::lwu(core, word),
        0o50 => store::sb(core, word),
        0o51 => store::sh(core, word),
        0o52 => store::swl(core, word),
        0o53 => store::sw(core, word),
        0o54 => store::sdl(core, word),
        0o55 => store::sdr(core, word),
        0o56 => store::swr(core, word),
        0o57 => misc::cache(core, word),
        0o60 => load::ll(core, word),
        0o61 => T::Cp1::lwc1(core, word),
        0o62 => T::Cp2::lwc2(core, word),
        0o64 => load::lld(core, word),
        0o65 => T::Cp1::ldc1(core, word),
        0o67 => load::ld(core, word),
        0o70 => store::sc(core, word),
        0o71 => T::Cp1::swc1(core, word),
        0o72 => T::Cp2::swc2(core, word),
        0o74 => store::scd(core, word),
        0o75 => T::Cp1::sdc1(core, word),
        0o77 => store::sd(core, word),
        opcode => unimplemented!("{} Opcode {:02o} [PC:{:08X}]", T::NAME, opcode, core.pc()),
    }
}

fn special<T: Bus>(core: &mut Core<T>, word: u32) {
    match word & 0o77 {
        0o00 => shift::sll(core, word),
        0o02 => shift::srl(core, word),
        0o03 => shift::sra(core, word),
        0o04 => shift::sllv(core, word),
        0o06 => shift::srlv(core, word),
        0o07 => shift::srav(core, word),
        0o10 => control::jr(core, word),
        0o11 => control::jalr(core, word),
        0o14 => T::Cp0::syscall(core, word),
        0o15 => T::Cp0::break_(core, word),
        0o17 => misc::sync(core, word),
        0o20 => mul_div::mfhi(core, word),
        0o21 => mul_div::mthi(core, word),
        0o22 => mul_div::mflo(core, word),
        0o23 => mul_div::mtlo(core, word),
        0o24 => shift::dsllv(core, word),
        0o26 => shift::dsrlv(core, word),
        0o27 => shift::dsrav(core, word),
        0o30 => mul_div::mult(core, word),
        0o31 => mul_div::multu(core, word),
        0o32 => mul_div::div(core, word),
        0o33 => mul_div::divu(core, word),
        0o34 => mul_div::dmult(core, word),
        0o35 => mul_div::dmultu(core, word),
        0o36 => mul_div::ddiv(core, word),
        0o37 => mul_div::ddivu(core, word),
        0o40 => arithmetic::add(core, word),
        0o41 => arithmetic::addu(core, word),
        0o42 => arithmetic::sub(core, word),
        0o43 => arithmetic::subu(core, word),
        0o44 => logic::and(core, word),
        0o45 => logic::or(core, word),
        0o46 => logic::xor(core, word),
        0o47 => logic::nor(core, word),
        0o52 => arithmetic::slt(core, word),
        0o53 => arithmetic::sltu(core, word),
        0o54 => arithmetic::dadd(core, word),
        0o55 => arithmetic::daddu(core, word),
        0o56 => arithmetic::dsub(core, word),
        0o57 => arithmetic::dsubu(core, word),
        0o64 => trap::teq(core, word),
        0o66 => trap::tne(core, word),
        0o70 => shift::dsll(core, word),
        0o72 => shift::dsrl(core, word),
        0o73 => shift::dsra(core, word),
        0o74 => shift::dsll32(core, word),
        0o76 => shift::dsrl32(core, word),
        0o77 => shift::dsra32(core, word),
        opcode => unimplemented!(
            "{} Special Opcode {:02o} [PC:{:08X}]",
            T::NAME,
            opcode,
            core.pc()
        ),
    }
}

fn regimm<T: Bus>(core: &mut Core<T>, word: u32) {
    match (word >> 16) & 0o37 {
        0o00 => control::bltz::<false, false>(core, word),
        0o01 => control::bgez::<false, false>(core, word),
        0o02 => control::bltz::<false, true>(core, word),
        0o03 => control::bgez::<false, true>(core, word),
        0o20 => control::bltz::<true, false>(core, word),
        0o21 => control::bgez::<true, false>(core, word),
        0o22 => control::bltz::<true, true>(core, word),
        0o23 => control::bgez::<true, true>(core, word),
        opcode => unimplemented!(
            "{} RegImm Opcode {:02o} [PC:{:08X}]",
            T::NAME,
            opcode,
            core.pc()
        ),
    }
}

fn cop0<T: Bus>(core: &mut Core<T>, word: u32) {
    match (word >> 21) & 0o37 {
        0o00 => T::Cp0::mfc0(core, word),
        0o01 => T::Cp0::dmfc0(core, word),
        0o04 => T::Cp0::mtc0(core, word),
        0o05 => T::Cp0::dmtc0(core, word),
        0o20..=0o37 => T::Cp0::cop0(core, word),
        opcode => unimplemented!(
            "{} COP0 Opcode {:02o} [PC:{:08X}]",
            T::NAME,
            opcode,
            core.pc()
        ),
    }
}

fn cop1<T: Bus>(core: &mut Core<T>, word: u32) {
    match (word >> 21) & 0o37 {
        0o00 => T::Cp1::mfc1(core, word),
        0o01 => T::Cp1::dmfc1(core, word),
        0o02 => T::Cp1::cfc1(core, word),
        0o04 => T::Cp1::mtc1(core, word),
        0o05 => T::Cp1::dmtc1(core, word),
        0o06 => T::Cp1::ctc1(core, word),
        0o10 => T::Cp1::bc1(core, word),
        0o20..=0o37 => T::Cp1::cop1(core, word),
        opcode => unimplemented!(
            "{} COP1 Opcode {:02o} [PC:{:08X}]",
            T::NAME,
            opcode,
            core.pc()
        ),
    }
}

fn cop2<T: Bus>(core: &mut Core<T>, word: u32) {
    match (word >> 21) & 0o37 {
        0o00 => T::Cp2::mfc2(core, word),
        0o02 => T::Cp2::cfc2(core, word),
        0o04 => T::Cp2::mtc2(core, word),
        0o06 => T::Cp2::ctc2(core, word),
        0o20..=0o37 => T::Cp2::cop2(core, word),
        opcode => unimplemented!(
            "{} COP2 Opcode {:02o} [PC:{:08X}]",
            T::NAME,
            opcode,
            core.pc()
        ),
    }
}
