use super::super::{Bus, Core};
use tracing::debug;

pub fn nop(core: &mut Core<impl Bus>) {
    debug!("NOP");
    core.read(core.pc);
}

pub fn auto_inc_read(core: &mut Core<impl Bus>) {
    debug!("MOV A, (X)+");
    core.read(core.pc);
    core.a = core.read_direct(core.x);
    core.set_nz(core.a);
    core.x = core.x.wrapping_add(1);
    core.idle();
}

pub fn auto_inc_write(core: &mut Core<impl Bus>) {
    debug!("MOV (X)+, A");
    core.read(core.pc);
    core.idle();
    core.write_direct(core.x, core.a);
    core.x = core.x.wrapping_add(1);
}

pub fn mul(core: &mut Core<impl Bus>) {
    debug!("MUL YA");
    core.read(core.pc);

    for _ in 0..7 {
        core.idle();
    }

    let [low, high] = ((core.a as u16) * (core.y as u16)).to_le_bytes();
    core.a = low;
    core.y = high;
    core.set_nz(core.y);
}

pub fn div(core: &mut Core<impl Bus>) {
    debug!("DIV YA, X");
    core.read(core.pc);

    for _ in 0..10 {
        core.idle();
    }

    core.flags.h = (core.x & 0x0f) <= (core.y & 0x0f);

    let divisor = (core.x as u32) << 9;
    let mut result = u32::from_le_bytes([core.a, core.y, 0, 0]);

    for _ in 0..9 {
        result <<= 1;

        if (result & 0x0002_0000) != 0 {
            result ^= 0x0002_0001;
        }

        if result >= divisor {
            result ^= 1;
        }

        if (result & 1) != 0 {
            result = (result - divisor) & 0x0001_ffff;
        }
    }

    core.a = result as u8;
    core.y = (result >> 9) as u8;
    core.flags.n = core.a;
    core.flags.v = (result & 0x0100) != 0;
    core.flags.z = core.a;
}
