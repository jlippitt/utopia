use tracing::debug;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum InterruptType {
    VBlank = 0x01,
    //LcdStat = 0x02,
    Timer = 0x04,
    //Serial = 0x08,
    //Joypad = 0x10,
}

pub struct Interrupt {
    flag: u8,
    enable: u8,
}

impl Interrupt {
    pub fn new() -> Self {
        Self { flag: 0, enable: 0 }
    }

    pub fn poll(&self) -> u8 {
        self.flag & self.enable
    }

    pub fn flag(&self) -> u8 {
        0xe0 | self.flag
    }

    pub fn set_flag(&mut self, value: u8) {
        self.flag = value;
        debug!("Interrupt Flag: {:05b}", self.flag);
    }

    pub fn enable(&self) -> u8 {
        0xe0 | self.enable
    }

    pub fn set_enable(&mut self, value: u8) {
        self.enable = value;
        debug!("Interrupt Enable: {:05b}", self.enable);
    }

    pub fn raise(&mut self, interrupt_type: InterruptType) {
        self.flag |= interrupt_type as u8;
        debug!("Interrupt Raised: {:?}", interrupt_type);
    }

    pub fn acknowledge(&mut self, mask: u8) {
        self.flag &= !mask;
    }
}
