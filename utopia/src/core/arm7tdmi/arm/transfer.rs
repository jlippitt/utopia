use super::super::operator::TransferOperator;
use super::super::{Bus, Core, REGS, SIZES};
use super::{apply_shift, SHIFT};
use tracing::debug;

fn format_immediate<const PUW: u8>(rn: usize, offset: u32) -> String {
    match PUW {
        0b000 => format!("[{}], #-0x{:X}", REGS[rn], offset),
        0b010 => format!("[{}], #0x{:X}", REGS[rn], offset),
        0b100 => format!("[{}, #-0x{:X}]", REGS[rn], offset),
        0b101 => format!("[{}, #-0x{:X}]!", REGS[rn], offset),
        0b110 => format!("[{}, #0x{:X}]", REGS[rn], offset),
        0b111 => format!("[{}, #0x{:X}]!", REGS[rn], offset),
        _ => panic!("Invalid address mode: {:03b}", PUW),
    }
}

fn format_register<const PUW: u8>(
    rn: usize,
    rm: usize,
    shift_type: usize,
    shift_amount: &str,
) -> String {
    match PUW {
        0b000 => format!(
            "[{}], -{}, {} {}",
            REGS[rn], REGS[rm], SHIFT[shift_type], shift_amount
        ),
        0b010 => format!(
            "[{}], {}, {} {}",
            REGS[rn], REGS[rm], SHIFT[shift_type], shift_amount
        ),
        0b100 => format!(
            "[{}, -{}, {} {}]",
            REGS[rn], REGS[rm], SHIFT[shift_type], shift_amount
        ),
        0b101 => format!(
            "[{}, -{}, {} {}]",
            REGS[rn], REGS[rm], SHIFT[shift_type], shift_amount
        ),
        0b110 => format!(
            "[{}, {}, {} {}]",
            REGS[rn], REGS[rm], SHIFT[shift_type], shift_amount
        ),
        0b111 => format!(
            "[{}, {}, {} {}]",
            REGS[rn], REGS[rm], SHIFT[shift_type], shift_amount
        ),
        _ => panic!("Invalid address mode: {:03b}", PUW),
    }
}

fn resolve<const PUW: u8>(core: &mut Core<impl Bus>, rn: usize, offset: u32) -> u32 {
    let base = core.get(rn);

    let (address, write_back) = match PUW {
        0b000 => (base, Some(base.wrapping_sub(offset))),
        0b010 => (base, Some(base.wrapping_add(offset))),
        0b100 => (base.wrapping_sub(offset), None),
        0b101 => {
            let address = base.wrapping_sub(offset);
            (address, Some(address))
        }
        0b110 => (base.wrapping_add(offset), None),
        0b111 => {
            let address = base.wrapping_add(offset);
            (address, Some(address))
        }
        _ => panic!("Invalid address mode: {:03b}", PUW),
    };

    if let Some(value) = write_back {
        core.set(rn, value);
    }

    address
}

pub fn mem_immediate<Op: TransferOperator, const SIZE: usize, const PUW: u8>(
    core: &mut Core<impl Bus>,
    pc: u32,
    word: u32,
) {
    let rn = ((word >> 16) & 15) as usize;
    let rd = ((word >> 12) & 15) as usize;
    let offset = word & 0x0000_0fff;

    debug!(
        "{:08X} {}{} {}, {}",
        pc,
        Op::NAME,
        SIZES[SIZE],
        REGS[rd],
        format_immediate::<PUW>(rn, offset),
    );

    let address = resolve::<PUW>(core, rn, offset);
    Op::apply::<SIZE>(core, rd, address);
}

pub fn mem_register<Op: TransferOperator, const SIZE: usize, const PUW: u8>(
    core: &mut Core<impl Bus>,
    pc: u32,
    word: u32,
) {
    let rn = ((word >> 16) & 15) as usize;
    let rd = ((word >> 12) & 15) as usize;
    let rm = (word & 15) as usize;
    let shift_type = ((word >> 5) & 3) as usize;

    let var_shift = (word & 0x10) != 0;

    let (shift_amount, debug_string) = if var_shift {
        let rs = ((word >> 8) & 15) as usize;
        (core.get(rs), format!("{}", REGS[rs]))
    } else {
        let shift_amount = (word >> 7) & 31;
        (shift_amount, format!("#0x{:X}", shift_amount))
    };

    debug!(
        "{:08X} {}{} {}, {}",
        pc,
        Op::NAME,
        SIZES[SIZE],
        REGS[rd],
        format_register::<PUW>(rn, rm, shift_type, &debug_string),
    );

    let offset = if var_shift {
        apply_shift::<false, true, false>(core, rm, shift_type, shift_amount)
    } else {
        apply_shift::<false, false, false>(core, rm, shift_type, shift_amount)
    };

    let address = resolve::<PUW>(core, rn, offset);
    Op::apply::<SIZE>(core, rd, address);
}
