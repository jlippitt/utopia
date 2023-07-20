use super::super::address_mode::{Direct, DirectX, ReadAddress};
use super::super::{Bus, Core};

pub trait BranchOperator {
    const NAME: &'static str;
    fn apply(core: &mut Core<impl Bus>) -> bool;
}

pub struct Bra;

impl BranchOperator for Bra {
    const NAME: &'static str = "BRA";

    fn apply(_core: &mut Core<impl Bus>) -> bool {
        true
    }
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
        !core.flags.v
    }
}

pub struct Bvs;

impl BranchOperator for Bvs {
    const NAME: &'static str = "BVS";

    fn apply(core: &mut Core<impl Bus>) -> bool {
        core.flags.v
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

pub struct CbneDirect;

impl BranchOperator for CbneDirect {
    const NAME: &'static str = "CBNE d,";

    fn apply(core: &mut Core<impl Bus>) -> bool {
        let value = Direct::read(core);
        core.idle();
        core.a != value
    }
}

pub struct CbneDirectX;

impl BranchOperator for CbneDirectX {
    const NAME: &'static str = "CBNE d+X,";

    fn apply(core: &mut Core<impl Bus>) -> bool {
        let value = DirectX::read(core);
        core.idle();
        core.a != value
    }
}

pub struct DbnzY;

impl BranchOperator for DbnzY {
    const NAME: &'static str = "DBNZ Y,";

    fn apply(core: &mut Core<impl Bus>) -> bool {
        core.read(core.pc);
        core.idle();
        core.y = core.y.wrapping_sub(1);
        core.y != 0
    }
}

pub struct DbnzDirect;

impl BranchOperator for DbnzDirect {
    const NAME: &'static str = "DBNZ d,";

    fn apply(core: &mut Core<impl Bus>) -> bool {
        let address = core.next_byte();
        let value = core.read_direct(address);
        let result = value.wrapping_sub(1);
        core.write_direct(address, result);
        result != 0
    }
}
