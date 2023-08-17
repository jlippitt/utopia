use super::super::{Bus, Core, REGS};
use super::{Flags, RoundingMode};
use num_traits::{FromPrimitive, ToPrimitive};
use tracing::debug;

pub fn cfc1(core: &mut Core<impl Bus>, rt: usize, rd: usize) {
    debug!("{:08X} CFC1 {}, ${}", core.pc, REGS[rt], rd);

    let result = match rd {
        31 => {
            // CONTROL/STATUS
            let ctrl = &core.cp1.ctrl;
            let mut value = 0;
            value |= ctrl.rm.to_u32().unwrap();
            value |= Into::<u32>::into(ctrl.flags) << 2;
            value |= Into::<u32>::into(ctrl.enable) << 7;
            value |= Into::<u32>::into(ctrl.cause) << 12;
            value |= (ctrl.c as u32) << 23;
            value |= (ctrl.fs as u32) << 24;
            value
        }
        _ => todo!("CP1 Register Read: ${}", rd),
    };

    core.set(rt, result);
}

pub fn ctc1(core: &mut Core<impl Bus>, rt: usize, rd: usize) {
    debug!("{:08X} CTC1 {}, ${}", core.pc, REGS[rt], rd);

    let value = core.get(rt);

    match rd {
        31 => {
            // CONTROL/STATUS
            let ctrl = &mut core.cp1.ctrl;
            ctrl.rm = RoundingMode::from_u32(value & 3).unwrap();
            ctrl.flags = Flags::from((value >> 2) & 31);
            ctrl.enable = Flags::from((value >> 7) & 31);
            ctrl.cause = Flags::from((value >> 12) & 63);
            ctrl.c = (value & 0x0080_0000) != 0;
            ctrl.fs = (value & 0x0100_0000) != 0;
            debug!("  CP1 Rounding Mode: {:?}", ctrl.rm);
            debug!("  CP1 Flags: {}", ctrl.flags);
            debug!("  CP1 Enable: {}", ctrl.enable);
            debug!("  CP1 Cause: {}", ctrl.cause);
            debug!("  CP1 Compare: {}", ctrl.c);
            debug!("  CP1 Flash: {}", ctrl.fs);
        }
        _ => todo!("CP1 Register Write: ${} <= {:08X}", rd, value),
    }
}

pub fn mfc1(core: &mut Core<impl Bus>, rt: usize, rd: usize) {
    debug!("{:08X} MFC1 {}, $F{}", core.pc, REGS[rt], rd);
    core.set(rt, core.cp1.w(rd) as u32);
}

pub fn mtc1(core: &mut Core<impl Bus>, rt: usize, rd: usize) {
    debug!("{:08X} MTC1 {}, $F{}", core.pc, REGS[rt], rd);
    core.cp1.set_w(rd, core.get(rt) as i32);
}
