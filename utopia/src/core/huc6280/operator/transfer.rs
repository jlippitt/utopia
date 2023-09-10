pub trait TransferOperator {
    const NAME: &'static str;
    fn apply(src: &mut u16, dst: &mut u16, alternate: bool);
}

pub struct Tii;

impl TransferOperator for Tii {
    const NAME: &'static str = "TII";

    fn apply(src: &mut u16, dst: &mut u16, _alternate: bool) {
        *src = src.wrapping_add(1);
        *dst = dst.wrapping_add(1);
    }
}

pub struct Tdd;

impl TransferOperator for Tdd {
    const NAME: &'static str = "TDD";

    fn apply(src: &mut u16, dst: &mut u16, _alternate: bool) {
        *src = src.wrapping_sub(1);
        *dst = dst.wrapping_sub(1);
    }
}

pub struct Tin;

impl TransferOperator for Tin {
    const NAME: &'static str = "TIN";

    fn apply(src: &mut u16, _dst: &mut u16, _alternate: bool) {
        *src = src.wrapping_add(1);
    }
}

pub struct Tia;

impl TransferOperator for Tia {
    const NAME: &'static str = "TIA";

    fn apply(src: &mut u16, dst: &mut u16, alternate: bool) {
        *src = src.wrapping_add(1);

        if alternate {
            *dst = dst.wrapping_sub(1);
        } else {
            *dst = dst.wrapping_add(1);
        }
    }
}

pub struct Tai;

impl TransferOperator for Tai {
    const NAME: &'static str = "TAI";

    fn apply(src: &mut u16, dst: &mut u16, alternate: bool) {
        if alternate {
            *src = src.wrapping_sub(1);
        } else {
            *src = src.wrapping_add(1);
        }

        *dst = dst.wrapping_add(1);
    }
}
