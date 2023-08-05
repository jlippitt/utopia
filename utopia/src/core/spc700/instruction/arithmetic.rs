use super::super::{Bus, Core};
use tracing::debug;

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
            result = (result.wrapping_sub(divisor)) & 0x0001_ffff;
        }
    }

    core.a = result as u8;
    core.y = (result >> 9) as u8;
    core.flags.n = core.a;
    core.flags.v = (result & 0x0100) != 0;
    core.flags.z = core.a;
}

pub fn das(core: &mut Core<impl Bus>) {
    debug!("DAS A");
    core.read(core.pc);

    if !core.flags.c || core.a > 0x99 {
        core.a = core.a.wrapping_sub(0x60);
        core.flags.c = false;
    }

    if !core.flags.h || (core.a & 0x0f) > 0x09 {
        core.a = core.a.wrapping_sub(0x06);
    }

    core.set_nz(core.a);
}
