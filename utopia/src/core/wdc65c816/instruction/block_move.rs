use super::super::{Bus, Core};
use tracing::trace;

fn block_move(core: &mut Core<impl Bus>) {
    core.dbr = (core.next_byte() as u32) << 16;
    let src_bank = (core.next_byte() as u32) << 16;
    let value = core.read(src_bank | (core.x as u32));
    core.write(core.dbr | (core.y as u32), value);
    core.a = core.a.wrapping_sub(1);

    if core.a != 0xffff {
        core.pc = (core.pc & 0xffff_0000) | (core.pc.wrapping_sub(3) & 0xffff);
    }

    core.idle();
    core.idle();
}

pub fn mvp<const X: bool>(core: &mut Core<impl Bus>) {
    trace!("MVP src, dst");
    block_move(core);
    core.x = core.x.wrapping_sub(1);
    core.y = core.y.wrapping_sub(1);

    if X {
        core.x &= 0xff;
        core.y &= 0xff
    }
}

pub fn mvn<const X: bool>(core: &mut Core<impl Bus>) {
    trace!("MVN src, dst");
    block_move(core);
    core.x = core.x.wrapping_add(1);
    core.y = core.y.wrapping_add(1);

    if X {
        core.x &= 0xff;
        core.y &= 0xff
    }
}
