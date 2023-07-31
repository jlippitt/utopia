use super::super::{Bus, Core, REGS};
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

pub fn ldr_immediate<const PUW: u8>(core: &mut Core<impl Bus>, pc: u32, word: u32) {
    let rn = ((word >> 16) & 15) as usize;
    let rd = ((word >> 12) & 15) as usize;
    let offset = word & 0x0000_0fff;

    debug!(
        "{:08X} LDR {}, {}",
        pc,
        REGS[rd],
        format_immediate::<PUW>(rn, offset),
    );

    let address = resolve::<PUW>(core, rn, offset);
    let result = core.read_word(address);
    core.set(rd, result);
}

pub fn ldrb_immediate<const PUW: u8>(core: &mut Core<impl Bus>, pc: u32, word: u32) {
    let rn = ((word >> 16) & 15) as usize;
    let rd = ((word >> 12) & 15) as usize;
    let offset = word & 0x0000_0fff;

    debug!(
        "{:08X} LDRB {}, {}",
        pc,
        REGS[rd],
        format_immediate::<PUW>(rn, offset),
    );

    let address = resolve::<PUW>(core, rn, offset);
    let result = core.read_byte(address);
    core.set(rd, result as u32);
}
