use tracing::trace;

pub const TOTAL_PAGES: usize = 2048;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Page {
    Rom(u32),
    //Sram(u32),
    Wram(u32),
    ExternalRegisters,
    InternalRegisters,
    OpenBus,
}

pub fn map() -> [Page; TOTAL_PAGES] {
    let mut pages = [Page::OpenBus; TOTAL_PAGES];

    // Assume LoROM for now
    map_lo_rom(&mut pages);

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

fn map_lo_rom(pages: &mut [Page]) {
    for bank in 0x00..=0x7f {
        let index = bank << 3;
        let offset = (bank as u32) << 14;
        pages[index | 4] = Page::Rom(offset | 0x0000);
        pages[index | 5] = Page::Rom(offset | 0x2000);
        pages[index | 6] = Page::Rom(offset | 0x4000);
        pages[index | 7] = Page::Rom(offset | 0x6000);
    }

    pages.copy_within(0..(TOTAL_PAGES / 2), TOTAL_PAGES / 2);
}
