use super::{Bus, Core};

mod alu;
mod bit;
mod block;
mod control;
mod load;
mod misc;

pub fn dispatch(core: &mut Core<impl Bus>) {
    use super::address_mode as addr;
    use super::condition as cond;

    match core.fetch() {
        // Page 0: Misc Ops

        // +0x00 / +0x08
        0x00 => misc::nop(core),
        //0x08 => instr::ld_u16_sp(core),
        0x10 => control::djnz(core),
        0x18 => control::jr(core),
        0x20 => control::jr_conditional::<cond::NZ>(core),
        0x28 => control::jr_conditional::<cond::Z>(core),
        0x30 => control::jr_conditional::<cond::NC>(core),
        0x38 => control::jr_conditional::<cond::C>(core),

        // +0x01 / +0x09
        0x01 => load::ld::<u16, addr::BC, addr::Immediate>(core),
        0x09 => alu::add16::<addr::BC>(core),
        0x11 => load::ld::<u16, addr::DE, addr::Immediate>(core),
        0x19 => alu::add16::<addr::DE>(core),
        0x21 => load::ld::<u16, addr::HL, addr::Immediate>(core),
        0x29 => alu::add16::<addr::HL>(core),
        0x31 => load::ld::<u16, addr::SP, addr::Immediate>(core),
        0x39 => alu::add16::<addr::SP>(core),

        // +0x02 / +0x0a
        0x02 => load::ld::<u8, addr::BCIndirect, addr::A>(core),
        0x0a => load::ld::<u8, addr::A, addr::BCIndirect>(core),
        0x12 => load::ld::<u8, addr::DEIndirect, addr::A>(core),
        0x1a => load::ld::<u8, addr::A, addr::DEIndirect>(core),
        0x22 => load::ld::<u16, addr::Absolute, addr::HL>(core),
        0x2a => load::ld::<u16, addr::HL, addr::Absolute>(core),
        0x32 => load::ld::<u8, addr::Absolute, addr::A>(core),
        0x3a => load::ld::<u8, addr::A, addr::Absolute>(core),

        // +0x03 / +0x0b
        0x03 => alu::inc16::<addr::BC>(core),
        0x0b => alu::dec16::<addr::BC>(core),
        0x13 => alu::inc16::<addr::DE>(core),
        0x1b => alu::dec16::<addr::DE>(core),
        0x23 => alu::inc16::<addr::HL>(core),
        0x2b => alu::dec16::<addr::HL>(core),
        0x33 => alu::inc16::<addr::SP>(core),
        0x3b => alu::dec16::<addr::SP>(core),

        // +0x04 / +0x0c
        0x04 => alu::inc::<addr::B>(core),
        0x0c => alu::inc::<addr::C>(core),
        0x14 => alu::inc::<addr::D>(core),
        0x1c => alu::inc::<addr::E>(core),
        0x24 => alu::inc::<addr::H>(core),
        0x2c => alu::inc::<addr::L>(core),
        0x34 => alu::inc::<addr::HLIndirect>(core),
        0x3c => alu::inc::<addr::A>(core),

        // +0x05 / +0x0d
        0x05 => alu::dec::<addr::B>(core),
        0x0d => alu::dec::<addr::C>(core),
        0x15 => alu::dec::<addr::D>(core),
        0x1d => alu::dec::<addr::E>(core),
        0x25 => alu::dec::<addr::H>(core),
        0x2d => alu::dec::<addr::L>(core),
        0x35 => alu::dec::<addr::HLIndirect>(core),
        0x3d => alu::dec::<addr::A>(core),

        // +0x06 / +0x0e
        0x06 => load::ld::<u8, addr::B, addr::Immediate>(core),
        0x0e => load::ld::<u8, addr::C, addr::Immediate>(core),
        0x16 => load::ld::<u8, addr::D, addr::Immediate>(core),
        0x1e => load::ld::<u8, addr::E, addr::Immediate>(core),
        0x26 => load::ld::<u8, addr::H, addr::Immediate>(core),
        0x2e => load::ld::<u8, addr::L, addr::Immediate>(core),
        0x36 => load::ld::<u8, addr::HLIndirect, addr::Immediate>(core),
        0x3e => load::ld::<u8, addr::A, addr::Immediate>(core),

        // +0x07 / 0x0f
        0x07 => bit::rlca(core),
        0x0f => bit::rrca(core),
        0x17 => bit::rla(core),
        0x1f => bit::rra(core),
        0x27 => misc::daa(core),
        0x2f => misc::cpl(core),
        0x37 => misc::scf(core),
        0x3f => misc::ccf(core),

        // Page 1: 8-bit Loads

        // 0x40
        0x40 => load::ld::<u8, addr::B, addr::B>(core),
        0x41 => load::ld::<u8, addr::B, addr::C>(core),
        0x42 => load::ld::<u8, addr::B, addr::D>(core),
        0x43 => load::ld::<u8, addr::B, addr::E>(core),
        0x44 => load::ld::<u8, addr::B, addr::H>(core),
        0x45 => load::ld::<u8, addr::B, addr::L>(core),
        0x46 => load::ld::<u8, addr::B, addr::HLIndirect>(core),
        0x47 => load::ld::<u8, addr::B, addr::A>(core),

        // 0x48
        0x48 => load::ld::<u8, addr::C, addr::B>(core),
        0x49 => load::ld::<u8, addr::C, addr::C>(core),
        0x4a => load::ld::<u8, addr::C, addr::D>(core),
        0x4b => load::ld::<u8, addr::C, addr::E>(core),
        0x4c => load::ld::<u8, addr::C, addr::H>(core),
        0x4d => load::ld::<u8, addr::C, addr::L>(core),
        0x4e => load::ld::<u8, addr::C, addr::HLIndirect>(core),
        0x4f => load::ld::<u8, addr::C, addr::A>(core),

        // 0x50
        0x50 => load::ld::<u8, addr::D, addr::B>(core),
        0x51 => load::ld::<u8, addr::D, addr::C>(core),
        0x52 => load::ld::<u8, addr::D, addr::D>(core),
        0x53 => load::ld::<u8, addr::D, addr::E>(core),
        0x54 => load::ld::<u8, addr::D, addr::H>(core),
        0x55 => load::ld::<u8, addr::D, addr::L>(core),
        0x56 => load::ld::<u8, addr::D, addr::HLIndirect>(core),
        0x57 => load::ld::<u8, addr::D, addr::A>(core),

        // 0x58
        0x58 => load::ld::<u8, addr::E, addr::B>(core),
        0x59 => load::ld::<u8, addr::E, addr::C>(core),
        0x5a => load::ld::<u8, addr::E, addr::D>(core),
        0x5b => load::ld::<u8, addr::E, addr::E>(core),
        0x5c => load::ld::<u8, addr::E, addr::H>(core),
        0x5d => load::ld::<u8, addr::E, addr::L>(core),
        0x5e => load::ld::<u8, addr::E, addr::HLIndirect>(core),
        0x5f => load::ld::<u8, addr::E, addr::A>(core),

        // 0x60
        0x60 => load::ld::<u8, addr::H, addr::B>(core),
        0x61 => load::ld::<u8, addr::H, addr::C>(core),
        0x62 => load::ld::<u8, addr::H, addr::D>(core),
        0x63 => load::ld::<u8, addr::H, addr::E>(core),
        0x64 => load::ld::<u8, addr::H, addr::H>(core),
        0x65 => load::ld::<u8, addr::H, addr::L>(core),
        0x66 => load::ld::<u8, addr::H, addr::HLIndirect>(core),
        0x67 => load::ld::<u8, addr::H, addr::A>(core),

        // 0x68
        0x68 => load::ld::<u8, addr::L, addr::B>(core),
        0x69 => load::ld::<u8, addr::L, addr::C>(core),
        0x6a => load::ld::<u8, addr::L, addr::D>(core),
        0x6b => load::ld::<u8, addr::L, addr::E>(core),
        0x6c => load::ld::<u8, addr::L, addr::H>(core),
        0x6d => load::ld::<u8, addr::L, addr::L>(core),
        0x6e => load::ld::<u8, addr::L, addr::HLIndirect>(core),
        0x6f => load::ld::<u8, addr::L, addr::A>(core),

        // 0x70
        0x70 => load::ld::<u8, addr::HLIndirect, addr::B>(core),
        0x71 => load::ld::<u8, addr::HLIndirect, addr::C>(core),
        0x72 => load::ld::<u8, addr::HLIndirect, addr::D>(core),
        0x73 => load::ld::<u8, addr::HLIndirect, addr::E>(core),
        0x74 => load::ld::<u8, addr::HLIndirect, addr::H>(core),
        0x75 => load::ld::<u8, addr::HLIndirect, addr::L>(core),
        0x76 => misc::halt(core),
        0x77 => load::ld::<u8, addr::HLIndirect, addr::A>(core),

        // 0x78
        0x78 => load::ld::<u8, addr::A, addr::B>(core),
        0x79 => load::ld::<u8, addr::A, addr::C>(core),
        0x7a => load::ld::<u8, addr::A, addr::D>(core),
        0x7b => load::ld::<u8, addr::A, addr::E>(core),
        0x7c => load::ld::<u8, addr::A, addr::H>(core),
        0x7d => load::ld::<u8, addr::A, addr::L>(core),
        0x7e => load::ld::<u8, addr::A, addr::HLIndirect>(core),
        0x7f => load::ld::<u8, addr::A, addr::A>(core),

        // Page 2: 8-bit Arithmetic & Logic

        // 0x80
        0x80 => alu::add::<addr::B>(core),
        0x81 => alu::add::<addr::C>(core),
        0x82 => alu::add::<addr::D>(core),
        0x83 => alu::add::<addr::E>(core),
        0x84 => alu::add::<addr::H>(core),
        0x85 => alu::add::<addr::L>(core),
        0x86 => alu::add::<addr::HLIndirect>(core),
        0x87 => alu::add::<addr::A>(core),

        // 0x88
        0x88 => alu::adc::<addr::B>(core),
        0x89 => alu::adc::<addr::C>(core),
        0x8a => alu::adc::<addr::D>(core),
        0x8b => alu::adc::<addr::E>(core),
        0x8c => alu::adc::<addr::H>(core),
        0x8d => alu::adc::<addr::L>(core),
        0x8e => alu::adc::<addr::HLIndirect>(core),
        0x8f => alu::adc::<addr::A>(core),

        // 0x90
        0x90 => alu::sub::<addr::B>(core),
        0x91 => alu::sub::<addr::C>(core),
        0x92 => alu::sub::<addr::D>(core),
        0x93 => alu::sub::<addr::E>(core),
        0x94 => alu::sub::<addr::H>(core),
        0x95 => alu::sub::<addr::L>(core),
        0x96 => alu::sub::<addr::HLIndirect>(core),
        0x97 => alu::sub::<addr::A>(core),

        // 0x98
        0x98 => alu::sbc::<addr::B>(core),
        0x99 => alu::sbc::<addr::C>(core),
        0x9a => alu::sbc::<addr::D>(core),
        0x9b => alu::sbc::<addr::E>(core),
        0x9c => alu::sbc::<addr::H>(core),
        0x9d => alu::sbc::<addr::L>(core),
        0x9e => alu::sbc::<addr::HLIndirect>(core),
        0x9f => alu::sbc::<addr::A>(core),

        // 0xA0
        0xa0 => alu::and::<addr::B>(core),
        0xa1 => alu::and::<addr::C>(core),
        0xa2 => alu::and::<addr::D>(core),
        0xa3 => alu::and::<addr::E>(core),
        0xa4 => alu::and::<addr::H>(core),
        0xa5 => alu::and::<addr::L>(core),
        0xa6 => alu::and::<addr::HLIndirect>(core),
        0xa7 => alu::and::<addr::A>(core),

        // 0xA8
        0xa8 => alu::xor::<addr::B>(core),
        0xa9 => alu::xor::<addr::C>(core),
        0xaa => alu::xor::<addr::D>(core),
        0xab => alu::xor::<addr::E>(core),
        0xac => alu::xor::<addr::H>(core),
        0xad => alu::xor::<addr::L>(core),
        0xae => alu::xor::<addr::HLIndirect>(core),
        0xaf => alu::xor::<addr::A>(core),

        // 0xB0
        0xb0 => alu::or::<addr::B>(core),
        0xb1 => alu::or::<addr::C>(core),
        0xb2 => alu::or::<addr::D>(core),
        0xb3 => alu::or::<addr::E>(core),
        0xb4 => alu::or::<addr::H>(core),
        0xb5 => alu::or::<addr::L>(core),
        0xb6 => alu::or::<addr::HLIndirect>(core),
        0xb7 => alu::or::<addr::A>(core),

        // 0xB8
        0xb8 => alu::cp::<addr::B>(core),
        0xb9 => alu::cp::<addr::C>(core),
        0xba => alu::cp::<addr::D>(core),
        0xbb => alu::cp::<addr::E>(core),
        0xbc => alu::cp::<addr::H>(core),
        0xbd => alu::cp::<addr::L>(core),
        0xbe => alu::cp::<addr::HLIndirect>(core),
        0xbf => alu::cp::<addr::A>(core),

        // Page 3: Misc Ops 2

        // +0x00 / 0x08
        0xc0 => control::ret_conditional::<cond::NZ>(core),
        0xc8 => control::ret_conditional::<cond::Z>(core),
        0xd0 => control::ret_conditional::<cond::NC>(core),
        0xd8 => control::ret_conditional::<cond::C>(core),
        //0xe0 => instr::ld::<u8, addr::High, addr::A>(core),
        //0xe8 => instr::add_sp_i8(core),
        //0xf0 => instr::ld::<u8, addr::A, addr::High>(core),
        //0xf8 => instr::ld_hl_sp_i8(core),

        // +0x01 / 0x09
        0xc1 => load::pop::<addr::BC>(core),
        0xc9 => control::ret(core),
        0xd1 => load::pop::<addr::DE>(core),
        0xd9 => misc::exx(core),
        0xe1 => load::pop::<addr::HL>(core),
        0xe9 => control::jp_hl(core),
        0xf1 => load::pop::<addr::AF>(core),
        0xf9 => load::ld_sp_hl(core),

        // +0x02 / 0x0a
        0xc2 => control::jp_conditional::<cond::NZ>(core),
        0xca => control::jp_conditional::<cond::Z>(core),
        0xd2 => control::jp_conditional::<cond::NC>(core),
        0xda => control::jp_conditional::<cond::C>(core),
        // 0xe2 => instr::ld::<u8, addr::CIndirect, addr::A>(core),
        // 0xea => instr::ld::<u8, addr::Absolute, addr::A>(core),
        // 0xf2 => instr::ld::<u8, addr::A, addr::CIndirect>(core),
        // 0xfa => instr::ld::<u8, addr::A, addr::Absolute>(core),

        // +0x03 / 0x0b
        0xc3 => control::jp(core),
        0xcb => prefix_cb(core),
        0xd3 => load::out_n(core),
        0xdb => load::in_n(core),
        0xf3 => misc::di(core),
        0xfb => misc::ei(core),

        // +0x04 / 0x0c
        0xc4 => control::call_conditional::<cond::NZ>(core),
        0xcc => control::call_conditional::<cond::Z>(core),
        0xd4 => control::call_conditional::<cond::NC>(core),
        0xdc => control::call_conditional::<cond::C>(core),

        // +0x05 / 0x0d
        0xc5 => load::push::<addr::BC>(core),
        0xcd => control::call(core),
        0xd5 => load::push::<addr::DE>(core),
        0xe5 => load::push::<addr::HL>(core),
        0xed => prefix_ed(core),
        0xf5 => load::push::<addr::AF>(core),

        // +0x06 / 0x0e
        0xc6 => alu::add::<addr::Immediate>(core),
        0xce => alu::adc::<addr::Immediate>(core),
        0xd6 => alu::sub::<addr::Immediate>(core),
        0xde => alu::sbc::<addr::Immediate>(core),
        0xe6 => alu::and::<addr::Immediate>(core),
        0xee => alu::xor::<addr::Immediate>(core),
        0xf6 => alu::or::<addr::Immediate>(core),
        0xfe => alu::cp::<addr::Immediate>(core),

        // +0x07 / 0x0f
        0xc7 => control::rst(core, 0x00),
        0xcf => control::rst(core, 0x08),
        0xd7 => control::rst(core, 0x10),
        0xdf => control::rst(core, 0x18),
        0xe7 => control::rst(core, 0x20),
        0xef => control::rst(core, 0x28),
        0xf7 => control::rst(core, 0x30),
        0xff => control::rst(core, 0x38),

        opcode => unimplemented!("Z80 Opcode: {:02X}", opcode),
    }
}

fn prefix_cb(core: &mut Core<impl Bus>) {
    use super::address_mode as addr;

    let opcode = core.fetch();

    match opcode {
        // Page 0: Shifts and Rotates

        // 0x00
        0x00 => bit::rlc::<addr::B>(core),
        0x01 => bit::rlc::<addr::C>(core),
        0x02 => bit::rlc::<addr::D>(core),
        0x03 => bit::rlc::<addr::E>(core),
        0x04 => bit::rlc::<addr::H>(core),
        0x05 => bit::rlc::<addr::L>(core),
        0x06 => bit::rlc::<addr::HLIndirect>(core),
        0x07 => bit::rlc::<addr::A>(core),

        // 0x08
        0x08 => bit::rrc::<addr::B>(core),
        0x09 => bit::rrc::<addr::C>(core),
        0x0a => bit::rrc::<addr::D>(core),
        0x0b => bit::rrc::<addr::E>(core),
        0x0c => bit::rrc::<addr::H>(core),
        0x0d => bit::rrc::<addr::L>(core),
        0x0e => bit::rrc::<addr::HLIndirect>(core),
        0x0f => bit::rrc::<addr::A>(core),

        // 0x10
        0x10 => bit::rl::<addr::B>(core),
        0x11 => bit::rl::<addr::C>(core),
        0x12 => bit::rl::<addr::D>(core),
        0x13 => bit::rl::<addr::E>(core),
        0x14 => bit::rl::<addr::H>(core),
        0x15 => bit::rl::<addr::L>(core),
        0x16 => bit::rl::<addr::HLIndirect>(core),
        0x17 => bit::rl::<addr::A>(core),

        // 0x18
        0x18 => bit::rr::<addr::B>(core),
        0x19 => bit::rr::<addr::C>(core),
        0x1a => bit::rr::<addr::D>(core),
        0x1b => bit::rr::<addr::E>(core),
        0x1c => bit::rr::<addr::H>(core),
        0x1d => bit::rr::<addr::L>(core),
        0x1e => bit::rr::<addr::HLIndirect>(core),
        0x1f => bit::rr::<addr::A>(core),

        // 0x20
        0x20 => bit::sla::<addr::B>(core),
        0x21 => bit::sla::<addr::C>(core),
        0x22 => bit::sla::<addr::D>(core),
        0x23 => bit::sla::<addr::E>(core),
        0x24 => bit::sla::<addr::H>(core),
        0x25 => bit::sla::<addr::L>(core),
        0x26 => bit::sla::<addr::HLIndirect>(core),
        0x27 => bit::sla::<addr::A>(core),

        // 0x28
        0x28 => bit::sra::<addr::B>(core),
        0x29 => bit::sra::<addr::C>(core),
        0x2a => bit::sra::<addr::D>(core),
        0x2b => bit::sra::<addr::E>(core),
        0x2c => bit::sra::<addr::H>(core),
        0x2d => bit::sra::<addr::L>(core),
        0x2e => bit::sra::<addr::HLIndirect>(core),
        0x2f => bit::sra::<addr::A>(core),

        // 0x30
        // 0x30 => bit::swap::<addr::B>(core),
        // 0x31 => bit::swap::<addr::C>(core),
        // 0x32 => bit::swap::<addr::D>(core),
        // 0x33 => bit::swap::<addr::E>(core),
        // 0x34 => bit::swap::<addr::H>(core),
        // 0x35 => bit::swap::<addr::L>(core),
        // 0x36 => bit::swap::<addr::HLIndirect>(core),
        // 0x37 => bit::swap::<addr::A>(core),

        // 0x38
        0x38 => bit::srl::<addr::B>(core),
        0x39 => bit::srl::<addr::C>(core),
        0x3a => bit::srl::<addr::D>(core),
        0x3b => bit::srl::<addr::E>(core),
        0x3c => bit::srl::<addr::H>(core),
        0x3d => bit::srl::<addr::L>(core),
        0x3e => bit::srl::<addr::HLIndirect>(core),
        0x3f => bit::srl::<addr::A>(core),

        // Page 1: BIT
        0x40 | 0x48 | 0x50 | 0x58 | 0x60 | 0x68 | 0x70 | 0x78 => bit::bit::<addr::B>(core, opcode),
        0x41 | 0x49 | 0x51 | 0x59 | 0x61 | 0x69 | 0x71 | 0x79 => bit::bit::<addr::C>(core, opcode),
        0x42 | 0x4a | 0x52 | 0x5a | 0x62 | 0x6a | 0x72 | 0x7a => bit::bit::<addr::D>(core, opcode),
        0x43 | 0x4b | 0x53 | 0x5b | 0x63 | 0x6b | 0x73 | 0x7b => bit::bit::<addr::E>(core, opcode),
        0x44 | 0x4c | 0x54 | 0x5c | 0x64 | 0x6c | 0x74 | 0x7c => bit::bit::<addr::H>(core, opcode),
        0x45 | 0x4d | 0x55 | 0x5d | 0x65 | 0x6d | 0x75 | 0x7d => bit::bit::<addr::L>(core, opcode),
        0x46 | 0x4e | 0x56 | 0x5e | 0x66 | 0x6e | 0x76 | 0x7e => {
            bit::bit::<addr::HLIndirect>(core, opcode)
        }
        0x47 | 0x4f | 0x57 | 0x5f | 0x67 | 0x6f | 0x77 | 0x7f => bit::bit::<addr::A>(core, opcode),

        // Page 2: RES
        0x80 | 0x88 | 0x90 | 0x98 | 0xa0 | 0xa8 | 0xb0 | 0xb8 => bit::res::<addr::B>(core, opcode),
        0x81 | 0x89 | 0x91 | 0x99 | 0xa1 | 0xa9 | 0xb1 | 0xb9 => bit::res::<addr::C>(core, opcode),
        0x82 | 0x8a | 0x92 | 0x9a | 0xa2 | 0xaa | 0xb2 | 0xba => bit::res::<addr::D>(core, opcode),
        0x83 | 0x8b | 0x93 | 0x9b | 0xa3 | 0xab | 0xb3 | 0xbb => bit::res::<addr::E>(core, opcode),
        0x84 | 0x8c | 0x94 | 0x9c | 0xa4 | 0xac | 0xb4 | 0xbc => bit::res::<addr::H>(core, opcode),
        0x85 | 0x8d | 0x95 | 0x9d | 0xa5 | 0xad | 0xb5 | 0xbd => bit::res::<addr::L>(core, opcode),
        0x86 | 0x8e | 0x96 | 0x9e | 0xa6 | 0xae | 0xb6 | 0xbe => {
            bit::res::<addr::HLIndirect>(core, opcode)
        }
        0x87 | 0x8f | 0x97 | 0x9f | 0xa7 | 0xaf | 0xb7 | 0xbf => bit::res::<addr::A>(core, opcode),

        // Page 3: SET
        0xc0 | 0xc8 | 0xd0 | 0xd8 | 0xe0 | 0xe8 | 0xf0 | 0xf8 => bit::set::<addr::B>(core, opcode),
        0xc1 | 0xc9 | 0xd1 | 0xd9 | 0xe1 | 0xe9 | 0xf1 | 0xf9 => bit::set::<addr::C>(core, opcode),
        0xc2 | 0xca | 0xd2 | 0xda | 0xe2 | 0xea | 0xf2 | 0xfa => bit::set::<addr::D>(core, opcode),
        0xc3 | 0xcb | 0xd3 | 0xdb | 0xe3 | 0xeb | 0xf3 | 0xfb => bit::set::<addr::E>(core, opcode),
        0xc4 | 0xcc | 0xd4 | 0xdc | 0xe4 | 0xec | 0xf4 | 0xfc => bit::set::<addr::H>(core, opcode),
        0xc5 | 0xcd | 0xd5 | 0xdd | 0xe5 | 0xed | 0xf5 | 0xfd => bit::set::<addr::L>(core, opcode),
        0xc6 | 0xce | 0xd6 | 0xde | 0xe6 | 0xee | 0xf6 | 0xfe => {
            bit::set::<addr::HLIndirect>(core, opcode)
        }
        0xc7 | 0xcf | 0xd7 | 0xdf | 0xe7 | 0xef | 0xf7 | 0xff => bit::set::<addr::A>(core, opcode),
        _ => unimplemented!("Z80 Opcode CB{:02X}", opcode),
    }
}

pub fn prefix_ed(core: &mut Core<impl Bus>) {
    match core.fetch() {
        0x46 => misc::im(core, 0),
        0x56 => misc::im(core, 1),
        0x5e => misc::im(core, 2),
        0xa0 => block::ldi(core),
        0xa3 => block::outi(core),
        0xa8 => block::ldd(core),
        0xab => block::outd(core),
        0xb0 => block::ldir(core),
        0xb3 => block::otir(core),
        0xb8 => block::lddr(core),
        0xbb => block::otdr(core),
        opcode => unimplemented!("Z80 Opcode: ED{:02X}", opcode),
    }
}
