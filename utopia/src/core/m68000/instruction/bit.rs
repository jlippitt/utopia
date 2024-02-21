use super::{AddressMode, Bus, Core};
use tracing::trace;

pub fn btst_dynamic(core: &mut Core<impl Bus>, word: u16) {
    let dst = AddressMode::from(word);

    if dst.is_areg() {
        todo!("MOVEP")
    }

    let src = (word >> 9) & 7;
    trace!("BTST D{}, {}", src, dst);
    let bit: u8 = core.dreg(src as usize);

    // TODO: Move to shared btst function, or btst operator??
    let (value, mask) = if dst.is_dreg() {
        (core.dreg(dst.reg()), 31)
    } else {
        (dst.read::<u8>(core) as u32, 7)
    };

    core.set_ccr(|flags| {
        flags.z = (value & (1 << (bit & mask))) == 0;
    });
}
