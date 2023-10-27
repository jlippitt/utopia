use super::rsp::Rsp;
use crate::core::mips::Bus;
use tracing::debug;

#[derive(Debug)]
pub struct DmaRequest {
    pub src: u32,
    pub dst: u32,
    pub len: u32,
    pub mode: bool,
}

impl super::Bus {
    pub(super) fn pi_dma_transfer(&mut self, request: DmaRequest) {
        let DmaRequest {
            src,
            dst,
            len,
            mode,
        } = request;

        // TODO: Room for optimisation here when addresses & len are word or halfword-aligned
        if mode {
            for index in 0..=len {
                self.write_data(src + index, self.read_data::<u8>(dst + index));
            }

            debug!(
                "PI DMA: {} bytes read from {:08X} to {:08X}",
                len + 1,
                dst,
                src
            );
        } else {
            for index in 0..=len {
                self.write_data(dst + index, self.read_data::<u8>(src + index));
            }

            debug!(
                "PI DMA: {} bytes written from {:08X} to {:08X}",
                len + 1,
                src,
                dst
            );
        }

        self.pi.finish_dma();
    }

    pub(super) fn si_dma_transfer(&mut self, request: DmaRequest) {
        let DmaRequest {
            src,
            dst,
            len,
            mode,
        } = request;

        if mode {
            for index in 0..len {
                let value: u8 = self.si.pif().read(dst.wrapping_add(index));

                self.rdram
                    .data_mut()
                    .write(src.wrapping_add(index) as usize, value);
            }

            debug!("SI DMA: {} bytes read from {:08X} to {:08X}", len, dst, src,);
        } else {
            for index in 0..len {
                let value: u8 = self.rdram.data().read(src.wrapping_add(index) as usize);
                self.si.pif_mut().write(dst.wrapping_add(index), value);
            }

            debug!(
                "SI DMA: {} bytes written from {:08X} to {:08X}",
                len, src, dst
            );

            self.si.pif_mut().upload();
        }

        self.si.finish_dma();
    }

    pub fn rsp_dma_transfer(&mut self, request: DmaRequest) {
        let DmaRequest {
            src,
            dst,
            len,
            mode,
        } = request;

        let imem = (src & 0x1000) != 0;
        let rdlen = ((len & 0xff8) + 8) as usize;
        let count = ((len >> 12) & 0xff) + 1;
        let skip = ((len >> 20) & 0xff8) as usize;

        debug!("RSP DMA: RDLEN={}, COUNT={}, SKIP={}", rdlen, count, skip);

        let rsp_mem = self.rsp.mem_mut();
        let rdram = self.rdram.data_mut();

        let mut rsp_start = src as usize & 0x1ff8;
        let mut rdram_start = dst as usize & 0x00ff_fff8;

        if mode {
            for _ in 0..count {
                for offset in 0..rdlen {
                    rsp_mem[(rsp_start & 0x1000) | ((rsp_start + offset) & 0xfff)] =
                        rdram[(rdram_start + offset) & 0x00ff_ffff];
                }
                rsp_start = (rsp_start & 0x1000) | ((rsp_start + rdlen) & 0xfff);
                rdram_start += rdlen + skip;
            }

            debug!(
                "RSP DMA: {} bytes read from {:08X} to {}:{:03X}",
                rdlen * count as usize,
                dst,
                if imem { "IMEM" } else { "DMEM" },
                src,
            );
        } else {
            for _ in 0..count {
                for offset in 0..rdlen {
                    rdram[(rdram_start + offset) & 0x00ff_ffff] =
                        rsp_mem[(rsp_start & 0x1000) | ((rsp_start + offset) & 0xfff)];
                }
                rsp_start = (rsp_start & 0x1000) | ((rsp_start + rdlen) & 0xfff);
                rdram_start += rdlen + skip;
            }

            debug!(
                "RSP DMA: {} bytes written from {}:{:03X} to {:08X}",
                rdlen * count as usize,
                if imem { "IMEM" } else { "DMEM" },
                src,
                dst
            );
        }

        self.rsp
            .regs_mut()
            .finish_rsp_dma(rsp_start as u32, rdram_start as u32);
    }

    pub fn rdp_dma_transfer(&mut self, request: DmaRequest) {
        let DmaRequest { src, len, mode, .. } = request;

        let commands = if mode {
            let dmem = &self.rsp.mem()[0..Rsp::DMEM_SIZE];
            &dmem[src as usize..(src as usize + len as usize)]
        } else {
            let rdram = self.rdram.data();
            &rdram[src as usize..(src as usize + len as usize)]
        };

        self.rdp.upload(commands);

        debug!(
            "RDP DMA: {} bytes uploaded from {}:{:08X}",
            len,
            if mode { "DMEM" } else { "RAM" },
            src
        );

        self.rdp.run(self.rsp.regs_mut(), self.rdram.data_mut());
    }
}
