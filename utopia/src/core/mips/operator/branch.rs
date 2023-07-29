pub trait BranchOperator {
    const NAME: &'static str;
    const UNARY: bool;
    fn apply(rs: u32, rt: u32) -> bool;
}

pub struct Beq;

impl BranchOperator for Beq {
    const NAME: &'static str = "BEQ";
    const UNARY: bool = false;

    fn apply(rs: u32, rt: u32) -> bool {
        rs == rt
    }
}

pub struct Bne;

impl BranchOperator for Bne {
    const NAME: &'static str = "BNE";
    const UNARY: bool = false;

    fn apply(rs: u32, rt: u32) -> bool {
        rs != rt
    }
}

pub struct Blez;

impl BranchOperator for Blez {
    const NAME: &'static str = "BLEZ";
    const UNARY: bool = true;

    fn apply(rs: u32, _rt: u32) -> bool {
        (rs as i32) <= 0
    }
}

pub struct Bgtz;

impl BranchOperator for Bgtz {
    const NAME: &'static str = "BGTZ";
    const UNARY: bool = true;

    fn apply(rs: u32, _rt: u32) -> bool {
        (rs as i32) > 0
    }
}

pub struct Bltz;

impl BranchOperator for Bltz {
    const NAME: &'static str = "BLTZ";
    const UNARY: bool = true;

    fn apply(rs: u32, _rt: u32) -> bool {
        (rs as i32) < 0
    }
}

pub struct Bgez;

impl BranchOperator for Bgez {
    const NAME: &'static str = "BGEZ";
    const UNARY: bool = true;

    fn apply(rs: u32, _rt: u32) -> bool {
        (rs as i32) >= 0
    }
}
