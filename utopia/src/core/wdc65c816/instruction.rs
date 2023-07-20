pub use block_move::*;
pub use control::*;
pub use flag::*;
pub use interrupt::*;
pub use meta::*;
pub use misc::*;
pub use register::*;
pub use stack::*;

mod block_move;
mod control;
mod flag;
mod interrupt;
mod meta;
mod misc;
mod register;
mod stack;

const fn size(byte: bool) -> char {
    if byte {
        'B'
    } else {
        'W'
    }
}
