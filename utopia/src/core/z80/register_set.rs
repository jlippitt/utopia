use super::address_mode::{self as addr, WriteAddress};

pub trait RegisterSet {
    type H: WriteAddress<u8>;
    type L: WriteAddress<u8>;
    type HL: WriteAddress<u16>;
    type HLIndirect: WriteAddress<u8>;
    const INDEXED: bool;
}

pub struct RegisterSetDefault;

impl RegisterSet for RegisterSetDefault {
    type H = addr::H;
    type L = addr::L;
    type HL = addr::HL;
    type HLIndirect = addr::HLIndirect;
    const INDEXED: bool = false;
}

pub struct RegisterSetIX;

impl RegisterSet for RegisterSetIX {
    type H = addr::IXH;
    type L = addr::IXL;
    type HL = addr::IX;
    type HLIndirect = addr::IXIndexed;
    const INDEXED: bool = false;
}

pub struct RegisterSetIY;

impl RegisterSet for RegisterSetIY {
    type H = addr::IYH;
    type L = addr::IYL;
    type HL = addr::IY;
    type HLIndirect = addr::IYIndexed;
    const INDEXED: bool = false;
}
