pub use flag::*;
pub use interrupt::*;
pub use meta::*;
pub use misc::*;
pub use register::*;

mod flag;
mod interrupt;
mod meta;
mod misc;
mod register;

const fn size(byte: bool) -> char {
    if byte {
        'B'
    } else {
        'W'
    }
}
