use bitfield_struct::bitfield;

#[bitfield(u32)]
pub struct Index {
    #[bits(6)]
    pub index: u32,
    #[bits(25)]
    __: u32,
    pub probe_failed: bool,
}

#[bitfield(u32)]
pub struct Status {
    pub ie: bool,
    pub exl: bool,
    pub erl: bool,
    #[bits(2)]
    pub ksu: u32,
    pub ux: bool,
    pub sx: bool,
    pub kx: bool,
    pub im: u8,
    #[bits(9)]
    pub ds: u32,
    pub re: bool,
    pub fr: bool,
    pub rp: bool,
    pub cu0: bool,
    pub cu1: bool,
    pub cu2: bool,
    pub cu3: bool,
}

#[bitfield(u32)]
pub struct Cause {
    #[bits(2)]
    __: u32,
    #[bits(5)]
    pub exc_code: u32,
    __: bool,
    pub ip: u8,
    #[bits(12)]
    __: u32,
    #[bits(2)]
    pub ce: u32,
    __: bool,
    pub bd: bool,
}
