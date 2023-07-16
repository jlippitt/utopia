use super::super::Flags;

pub trait BranchOperator {
    const NAME: &'static str;
    fn apply(flags: &Flags) -> bool;
}

pub struct Bra;

impl BranchOperator for Bra {
    const NAME: &'static str = "BRA";

    fn apply(_flags: &Flags) -> bool {
        true
    }
}

pub struct Bpl;

impl BranchOperator for Bpl {
    const NAME: &'static str = "BPL";

    fn apply(flags: &Flags) -> bool {
        !flags.n
    }
}

pub struct Bmi;

impl BranchOperator for Bmi {
    const NAME: &'static str = "BMI";

    fn apply(flags: &Flags) -> bool {
        flags.n
    }
}

pub struct Bvc;

impl BranchOperator for Bvc {
    const NAME: &'static str = "BVC";

    fn apply(flags: &Flags) -> bool {
        !flags.v
    }
}

pub struct Bvs;

impl BranchOperator for Bvs {
    const NAME: &'static str = "BVS";

    fn apply(flags: &Flags) -> bool {
        flags.v
    }
}

pub struct Bcc;

impl BranchOperator for Bcc {
    const NAME: &'static str = "BCC";

    fn apply(flags: &Flags) -> bool {
        !flags.c
    }
}

pub struct Bcs;

impl BranchOperator for Bcs {
    const NAME: &'static str = "BCS";

    fn apply(flags: &Flags) -> bool {
        flags.c
    }
}

pub struct Bne;

impl BranchOperator for Bne {
    const NAME: &'static str = "BNE";

    fn apply(flags: &Flags) -> bool {
        flags.z != 0
    }
}

pub struct Beq;

impl BranchOperator for Beq {
    const NAME: &'static str = "BEQ";

    fn apply(flags: &Flags) -> bool {
        flags.z == 0
    }
}
