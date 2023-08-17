use super::super::operator::BranchOperator;

pub struct Bc1f;

impl BranchOperator for Bc1f {
    const NAME: &'static str = "BC1F";
    const UNARY: bool = false;

    fn apply(rs: u64, rt: u64) -> bool {
        rs == rt
    }
}
