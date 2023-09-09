use super::super::{Bus, Core, ZERO_PAGE};

pub trait BranchOperator {
    const NAME: &'static str;
    fn apply(core: &mut Core<impl Bus>) -> bool;
}

pub struct Bpl;

impl BranchOperator for Bpl {
    const NAME: &'static str = "BPL";

    fn apply(core: &mut Core<impl Bus>) -> bool {
        (core.flags.n & 0x80) == 0
    }
}

pub struct Bmi;

impl BranchOperator for Bmi {
    const NAME: &'static str = "BMI";

    fn apply(core: &mut Core<impl Bus>) -> bool {
        (core.flags.n & 0x80) != 0
    }
}

pub struct Bvc;

impl BranchOperator for Bvc {
    const NAME: &'static str = "BVC";

    fn apply(core: &mut Core<impl Bus>) -> bool {
        (core.flags.v & 0x80) == 0
    }
}

pub struct Bvs;

impl BranchOperator for Bvs {
    const NAME: &'static str = "BVS";

    fn apply(core: &mut Core<impl Bus>) -> bool {
        (core.flags.v & 0x80) != 0
    }
}

pub struct Bcc;

impl BranchOperator for Bcc {
    const NAME: &'static str = "BCC";

    fn apply(core: &mut Core<impl Bus>) -> bool {
        !core.flags.c
    }
}

pub struct Bcs;

impl BranchOperator for Bcs {
    const NAME: &'static str = "BCS";

    fn apply(core: &mut Core<impl Bus>) -> bool {
        core.flags.c
    }
}

pub struct Bne;

impl BranchOperator for Bne {
    const NAME: &'static str = "BNE";

    fn apply(core: &mut Core<impl Bus>) -> bool {
        core.flags.z != 0
    }
}

pub struct Beq;

impl BranchOperator for Beq {
    const NAME: &'static str = "BEQ";

    fn apply(core: &mut Core<impl Bus>) -> bool {
        core.flags.z == 0
    }
}

pub struct Bra;

impl BranchOperator for Bra {
    const NAME: &'static str = "BRA";

    fn apply(_core: &mut Core<impl Bus>) -> bool {
        true
    }
}

pub struct Bbs<const BIT: u8>;

impl<const BIT: u8> BranchOperator for Bbs<BIT> {
    const NAME: &'static str =
        unsafe { std::str::from_utf8_unchecked(&[b'B', b'B', b'S', b'0' + BIT]) };

    fn apply(core: &mut Core<impl Bus>) -> bool {
        let address = core.next_byte();
        let value = core.read_physical(ZERO_PAGE | address as u32);
        (value & (1 << BIT)) != 0
    }
}

pub struct Bbr<const BIT: u8>;

impl<const BIT: u8> BranchOperator for Bbr<BIT> {
    const NAME: &'static str =
        unsafe { std::str::from_utf8_unchecked(&[b'B', b'B', b'R', b'0' + BIT]) };

    fn apply(core: &mut Core<impl Bus>) -> bool {
        let address = core.next_byte();
        let value = core.read_physical(ZERO_PAGE | address as u32);
        (value & (1 << BIT)) == 0
    }
}
