use super::rdp::{RdpDma, RdpDmaSource};
use crate::util::facade::{DataReader, DataWriter};
use tracing::debug;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Dma {
    pub src_addr: u32,
    pub dst_addr: u32,
    pub len: u32,
    pub reverse: bool,
}

impl super::Hardware {
    pub(super) fn rsp_dma(&mut self, request: Dma) {
        let Dma {
            src_addr,
            dst_addr,
            len,
            reverse,
        } = request;

        let skip = (len >> 20) & 0xff8;
        let count = (len >> 12) & 0xff;
        let rdlen = (len & 0xff8) + 8;

        if skip != 0 {
            todo!("RSP DMA Skip");
        }

        if count != 0 {
            todo!("RSP DMA Count");
        }

        if reverse {
            for index in 0..rdlen {
                let value: u8 = self.rdram.read_data(dst_addr.wrapping_add(index));
                self.rsp.write_ram(src_addr.wrapping_add(index), value);
            }

            debug!(
                "SP DMA: {} bytes read from {:08X} to {:08X}",
                rdlen, dst_addr, src_addr,
            );
        } else {
            for index in 0..rdlen {
                let value: u8 = self.rsp.read_ram(src_addr.wrapping_add(index));
                self.rdram.write_data(dst_addr.wrapping_add(index), value);
            }

            debug!(
                "SP DMA: {} bytes written from {:08X} to {:08X}",
                rdlen, src_addr, dst_addr,
            );
        }

        self.rsp.finish_dma();
    }

    pub(super) fn rdp_dma(&mut self, request: RdpDma) {
        let RdpDma { source, start, end } = request;

        let cmd_source = match source {
            RdpDmaSource::Rdram => self.rdram.data(),
            RdpDmaSource::Dmem => self.rsp.dmem(),
        };

        self.rdp.upload(&cmd_source[start as usize..end as usize]);
        self.rdp.run(self.rdram.data_mut(), &self.wgpu_context);
    }

    pub(super) fn peripheral_dma(&mut self, request: Dma) {
        let Dma {
            src_addr,
            dst_addr,
            len,
            reverse,
        } = request;

        // TODO: As most transfers will have lengths divisible by 4, this can be
        // better optimised. As (presumably) cart_address can only be ROM or
        // SRAM and dram_address is always RDRAM (possibly registers, though?),
        // we could also try talking directly to the components to save some
        // cycles.

        if reverse {
            todo!("Peripheral DMA Reads");
        } else {
            for index in 0..=len {
                let value: u8 = self.read_physical(src_addr.wrapping_add(index));
                self.write_physical(dst_addr.wrapping_add(index), value);
            }

            debug!(
                "PI DMA: {} bytes written from {:08X} to {:08X}",
                len + 1,
                src_addr,
                dst_addr,
            );
        }

        self.peripheral.finish_dma();
    }

    pub(super) fn serial_dma(&mut self, request: Dma) {
        let Dma {
            src_addr,
            dst_addr,
            len,
            reverse,
        } = request;

        if reverse {
            for index in 0..len {
                let value: u8 = self.serial.pif().read(dst_addr.wrapping_add(index));

                self.rdram.write_data(src_addr.wrapping_add(index), value);
            }

            debug!(
                "SI DMA: {} bytes read from {:08X} to {:08X}",
                len, dst_addr, src_addr,
            );
        } else {
            for index in 0..len {
                let value: u8 = self.rdram.read_data(src_addr.wrapping_add(index));

                self.serial
                    .pif_mut()
                    .write(dst_addr.wrapping_add(index), value);
            }

            debug!(
                "SI DMA: {} bytes written from {:08X} to {:08X}",
                len, src_addr, dst_addr
            );
        }

        self.serial.finish_dma();
    }
}
