use super::{Bus, Core};

mod misc;

pub fn dispatch(core: &mut Core<impl Bus>) {
    match core.fetch() {
        0xed => prefix_ed(core),
        0xf3 => misc::di(core),
        opcode => unimplemented!("Z80 Opcode: {:02X}", opcode),
    }
}

pub fn prefix_ed(core: &mut Core<impl Bus>) {
    match core.fetch() {
        0x46 => misc::im(core, 0),
        0x56 => misc::im(core, 1),
        0x5e => misc::im(core, 2),
        opcode => unimplemented!("Z80 Opcode: ED{:02X}", opcode),
    }
}
