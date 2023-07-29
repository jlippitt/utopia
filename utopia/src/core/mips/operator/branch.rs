pub trait BranchOperator {
    const NAME: &'static str;
    fn apply(lhs: u32, rhs: u32) -> bool;
}

pub struct Beq;

impl BranchOperator for Beq {
    const NAME: &'static str = "BEQ";

    fn apply(lhs: u32, rhs: u32) -> bool {
        lhs == rhs
    }
}

pub struct Bne;

impl BranchOperator for Bne {
    const NAME: &'static str = "BNE";

    fn apply(lhs: u32, rhs: u32) -> bool {
        lhs != rhs
    }
}
