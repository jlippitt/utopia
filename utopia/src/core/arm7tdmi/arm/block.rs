use super::super::{Bus, Core, REGS};
use arrayvec::ArrayVec;
use tracing::debug;

const ADDRESS: [&str; 4] = ["DA", "IA", "DB", "IB"];

fn reg_list(word: u32) -> String {
    let mut reg_list: ArrayVec<&str, 9> = ArrayVec::new();

    for reg in 0..=15 {
        let mask = 1 << reg;

        if (word & mask) != 0 {
            reg_list.push(REGS[reg]);
        }
    }

    reg_list.join(", ")
}

pub fn stm<const PU: u8, const S: bool, const W: bool>(
    core: &mut Core<impl Bus>,
    pc: u32,
    word: u32,
) {
    if S {
        todo!("S bit")
    }

    let rn = ((word >> 16) & 15) as usize;
    let mut base = core.get(rn);

    debug!(
        "{:08X} STM{} {}{}, {{ {} }}{}",
        pc,
        ADDRESS[PU as usize],
        REGS[rn],
        if W { "!" } else { "" },
        reg_list(word),
        if S { "^" } else { "" }
    );

    for index in (0..=15).rev() {
        let mask = 1 << index;

        if (word & mask) == 0 {
            continue;
        }

        let address = match PU {
            0b00 => {
                let address = base;
                base = base.wrapping_sub(4);
                address
            }
            0b01 => {
                let address = base;
                base = base.wrapping_add(4);
                address
            }
            0b10 => {
                base = base.wrapping_sub(4);
                base
            }
            0b11 => {
                base = base.wrapping_add(4);
                base
            }
            _ => unreachable!(),
        };

        core.write_word(address, core.get(index));
    }

    if W {
        core.set(rn, base);
    }
}
