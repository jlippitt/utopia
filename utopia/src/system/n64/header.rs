use crate::util::facade::ReadFacade;

pub struct Header {
    pub title: String,
    pub boot_address: u32,
}

pub fn parse(rom: &[u8]) -> Header {
    let boot_address = rom.read_be::<u32>(0x08);

    let title = String::from_utf8_lossy(&rom[0x20..=0x33]).into_owned();

    Header {
        title,
        boot_address,
    }
}
