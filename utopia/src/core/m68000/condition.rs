use super::Flags;

pub trait Condition {
    const NAME: &'static str;
    fn apply(flags: &Flags) -> bool;
}

pub struct T;

impl Condition for T {
    const NAME: &'static str = "T";

    fn apply(_flags: &Flags) -> bool {
        true
    }
}

pub struct F;

impl Condition for F {
    const NAME: &'static str = "F";

    fn apply(_flags: &Flags) -> bool {
        false
    }
}

pub struct HI;

impl Condition for HI {
    const NAME: &'static str = "HI";

    fn apply(flags: &Flags) -> bool {
        !flags.z && !flags.c
    }
}

pub struct LS;

impl Condition for LS {
    const NAME: &'static str = "LS";

    fn apply(flags: &Flags) -> bool {
        flags.z || flags.c
    }
}

pub struct CC;

impl Condition for CC {
    const NAME: &'static str = "CC";

    fn apply(flags: &Flags) -> bool {
        !flags.c
    }
}

pub struct CS;

impl Condition for CS {
    const NAME: &'static str = "CS";

    fn apply(flags: &Flags) -> bool {
        flags.c
    }
}

pub struct NE;

impl Condition for NE {
    const NAME: &'static str = "NE";

    fn apply(flags: &Flags) -> bool {
        !flags.z
    }
}

pub struct EQ;

impl Condition for EQ {
    const NAME: &'static str = "EQ";

    fn apply(flags: &Flags) -> bool {
        flags.z
    }
}

pub struct VC;

impl Condition for VC {
    const NAME: &'static str = "VC";

    fn apply(flags: &Flags) -> bool {
        !flags.v
    }
}

pub struct VS;

impl Condition for VS {
    const NAME: &'static str = "VS";

    fn apply(flags: &Flags) -> bool {
        flags.v
    }
}

pub struct PL;

impl Condition for PL {
    const NAME: &'static str = "PL";

    fn apply(flags: &Flags) -> bool {
        !flags.n
    }
}

pub struct MI;

impl Condition for MI {
    const NAME: &'static str = "MI";

    fn apply(flags: &Flags) -> bool {
        flags.n
    }
}

pub struct GE;

impl Condition for GE {
    const NAME: &'static str = "GE";

    fn apply(flags: &Flags) -> bool {
        flags.n == flags.v
    }
}

pub struct LT;

impl Condition for LT {
    const NAME: &'static str = "LT";

    fn apply(flags: &Flags) -> bool {
        flags.n != flags.v
    }
}

pub struct GT;

impl Condition for GT {
    const NAME: &'static str = "GT";

    fn apply(flags: &Flags) -> bool {
        !flags.z && flags.n == flags.v
    }
}

pub struct LE;

impl Condition for LE {
    const NAME: &'static str = "LE";

    fn apply(flags: &Flags) -> bool {
        flags.z || flags.n != flags.v
    }
}
