use super::Flags;

pub trait Condition {
    const NAME: &'static str;
    fn test(flags: &Flags) -> bool;
}

pub struct NZ;

impl Condition for NZ {
    const NAME: &'static str = "NZ";

    fn test(flags: &Flags) -> bool {
        flags.z != 0
    }
}

pub struct Z;

impl Condition for Z {
    const NAME: &'static str = "Z";

    fn test(flags: &Flags) -> bool {
        flags.z == 0
    }
}

pub struct NC;

impl Condition for NC {
    const NAME: &'static str = "NC";

    fn test(flags: &Flags) -> bool {
        !flags.c
    }
}

pub struct C;

impl Condition for C {
    const NAME: &'static str = "C";

    fn test(flags: &Flags) -> bool {
        flags.c
    }
}
