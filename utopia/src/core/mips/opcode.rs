use bitfield_struct::bitfield;

#[bitfield(u32)]
pub struct RType {
    #[bits(6)]
    pub func: u32,
    #[bits(5)]
    pub sa: u32,
    #[bits(5)]
    pub rd: usize,
    #[bits(5)]
    pub rt: usize,
    #[bits(5)]
    pub rs: usize,
    #[bits(6)]
    pub opcode: u32,
}

#[bitfield(u32)]
pub struct IType {
    #[bits(16)]
    pub imm: u32,
    #[bits(5)]
    pub rt: usize,
    #[bits(5)]
    pub rs: usize,
    #[bits(6)]
    pub opcode: u32,
}

#[bitfield(u32)]
pub struct JType {
    #[bits(26)]
    pub imm: u32,
    #[bits(6)]
    pub opcode: u32,
}
