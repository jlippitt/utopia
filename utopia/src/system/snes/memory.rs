use super::header::Header;
use tracing::trace;

pub const TOTAL_PAGES: usize = 2048;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Page {
    Rom(u32),
    Sram(u32),
    Wram(u32),
    ExternalRegisters,
    InternalRegisters,
    OpenBus,
}

fn mirror(size: usize, index: usize) -> usize {
    if size == 0 {
        return 0;
    }

    if index < size {
        return index;
    }

    let floor: usize = if index > 0 { 1 << index.ilog2() } else { 0 };

    if size <= (index & floor) as usize {
        return mirror(size, index - floor);
    }

    return floor + mirror(size - floor, index - floor);
}

pub fn map(header: &Header) -> [Page; TOTAL_PAGES] {
    let mut pages = [Page::OpenBus; TOTAL_PAGES];

    match header.map_mode & 0x0f {
        0x00 => map_lo_rom(&mut pages, header),
        0x01 => map_hi_rom(&mut pages, header),
        _ => unimplemented!("Map Mode {:02X}", header.map_mode),
    }

    // Map system pages
    map_system_pages(&mut pages, 0x00..=0x3f);
    map_system_pages(&mut pages, 0x80..=0xbf);

    // Map WRAM
    for offset in 0..16 {
        let index = (0x7e << 3) + offset;
        pages[index] = Page::Wram((offset as u32) << 13);
    }

    for (index, page) in pages.iter().enumerate() {
        trace!(
            "{:06X}-{:06X}: {:?}",
            index << 13,
            ((index + 1) << 13) - 1,
            page
        );
    }

    pages
}

fn map_system_pages(pages: &mut [Page], banks: impl Iterator<Item = u8>) {
    for bank in banks {
        let index = (bank as usize) << 3;
        pages[index | 0] = Page::Wram(0);
        pages[index | 1] = Page::ExternalRegisters;
        pages[index | 2] = Page::InternalRegisters;
    }
}

fn map_lo_rom(pages: &mut [Page], header: &Header) {
    let rom_size = header.rom_size;
    let sram_size = header.sram_size;

    for bank in 0x00..=0x7f {
        let index = bank << 3;
        let offset = (bank as usize) << 15;
        pages[index | 4] = Page::Rom(mirror(rom_size, offset | 0x0000) as u32);
        pages[index | 5] = Page::Rom(mirror(rom_size, offset | 0x2000) as u32);
        pages[index | 6] = Page::Rom(mirror(rom_size, offset | 0x4000) as u32);
        pages[index | 7] = Page::Rom(mirror(rom_size, offset | 0x6000) as u32);
    }

    if sram_size > 0 {
        for bank in 0x70..=0x7f {
            let index = bank << 3;
            let offset = ((bank - 0x70) as u32) << 15;
            pages[index | 0] = Page::Sram(offset | 0x0000);
            pages[index | 1] = Page::Sram(offset | 0x2000);
            pages[index | 2] = Page::Sram(offset | 0x4000);
            pages[index | 3] = Page::Sram(offset | 0x6000);
        }
    }

    pages.copy_within(0..(TOTAL_PAGES / 2), TOTAL_PAGES / 2);
}

fn map_hi_rom(pages: &mut [Page], header: &Header) {
    let rom_size = header.rom_size;
    let sram_size = header.sram_size;

    for bank in 0x00..=0x3f {
        let index = bank << 3;
        let offset = (bank as usize) << 16;
        pages[index | 4] = Page::Rom(mirror(rom_size, offset | 0x8000) as u32);
        pages[index | 5] = Page::Rom(mirror(rom_size, offset | 0xa000) as u32);
        pages[index | 6] = Page::Rom(mirror(rom_size, offset | 0xc000) as u32);
        pages[index | 7] = Page::Rom(mirror(rom_size, offset | 0xe000) as u32);
    }

    for bank in 0x40..=0x7f {
        let index = bank << 3;
        let offset = (bank - 0x40 as usize) << 16;
        pages[index | 0] = Page::Rom(mirror(rom_size, offset | 0x0000) as u32);
        pages[index | 1] = Page::Rom(mirror(rom_size, offset | 0x2000) as u32);
        pages[index | 2] = Page::Rom(mirror(rom_size, offset | 0x4000) as u32);
        pages[index | 3] = Page::Rom(mirror(rom_size, offset | 0x6000) as u32);
        pages[index | 4] = Page::Rom(mirror(rom_size, offset | 0x8000) as u32);
        pages[index | 5] = Page::Rom(mirror(rom_size, offset | 0xa000) as u32);
        pages[index | 6] = Page::Rom(mirror(rom_size, offset | 0xc000) as u32);
        pages[index | 7] = Page::Rom(mirror(rom_size, offset | 0xe000) as u32);
    }

    if sram_size > 0 {
        for bank in 0x20..=0x3f {
            let index = (bank as usize) << 3;
            let offset = (bank - 0x20 as u32) << 13;
            pages[index | 3] = Page::Sram(offset);
        }
    }

    pages.copy_within(0..(TOTAL_PAGES / 2), TOTAL_PAGES / 2);
}
