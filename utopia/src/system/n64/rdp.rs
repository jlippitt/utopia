use super::dma::DmaRequest;
use super::interrupt::{RcpIntType, RcpInterrupt};
use super::rsp::{DmaType, Registers as RspRegisters};
use super::WgpuContext;
use crate::util::memory::{Masked, Reader, Writer};
use core::Core;
use tracing::{debug_span, trace};

mod core;

pub struct Rdp {
    rcp_int: RcpInterrupt,
    commands: Vec<u64>,
    core: Core,
    dma: Option<DmaRequest>,
}

impl Rdp {
    pub fn new(ctx: WgpuContext, rcp_int: RcpInterrupt) -> Self {
        Self {
            rcp_int,
            commands: Vec::new(),
            core: Core::new(ctx),
            dma: None,
        }
    }

    pub fn command<'a>(&'a self, rsp_regs: &'a RspRegisters) -> CommandRegisters<'a> {
        CommandRegisters::new(rsp_regs)
    }

    pub fn command_mut<'a>(
        &'a mut self,
        rsp_regs: &'a mut RspRegisters,
    ) -> CommandRegistersMut<'a> {
        CommandRegistersMut::new(rsp_regs, &mut self.dma)
    }

    pub fn poll_dma(&mut self) -> Option<DmaRequest> {
        self.dma.take()
    }

    pub fn upload(&mut self, command_data: &[u8]) {
        let commands: &[u64] = bytemuck::cast_slice(command_data);
        let iter = commands.iter().map(|value| value.swap_bytes());
        self.commands.splice(.., iter);
    }

    pub fn run(&mut self, rsp_regs: &mut RspRegisters, rdram: &mut [u8]) {
        self.core.set_sync_required(true);

        {
            let _span = debug_span!("rdp").entered();
            trace!("Running {} commands", self.commands.len());
            self.core.run(rdram, self.commands.drain(..));
        }

        rsp_regs.set_dp_ready(true);

        if self.core.interrupt() {
            rsp_regs.clear_buffer_busy();
            self.core.set_interrupt(false);
            self.core.sync_full(rdram);
            self.rcp_int.raise(RcpIntType::DP);
        }
    }

    pub fn sync(&mut self, rdram: &mut [u8]) {
        if !self.core.sync_required() {
            return;
        }

        self.core.sync_full(rdram);
    }
}

pub struct CommandRegisters<'a> {
    regs: &'a RspRegisters,
}

impl<'a> CommandRegisters<'a> {
    fn new(regs: &'a RspRegisters) -> Self {
        Self { regs }
    }
}

impl<'a> Reader for CommandRegisters<'a> {
    type Value = u32;

    fn read_register(&self, address: u32) -> u32 {
        self.regs.get(8 + (address as usize >> 2))
    }
}

pub struct CommandRegistersMut<'a> {
    regs: &'a mut RspRegisters,
    dma: &'a mut Option<DmaRequest>,
}

impl<'a> CommandRegistersMut<'a> {
    fn new(regs: &'a mut RspRegisters, dma: &'a mut Option<DmaRequest>) -> Self {
        Self { regs, dma }
    }
}

impl<'a> Reader for CommandRegistersMut<'a> {
    type Value = u32;

    fn read_register(&self, address: u32) -> u32 {
        self.regs.get(8 + (address as usize >> 2))
    }
}

impl<'a> Writer for CommandRegistersMut<'a> {
    fn write_register(&mut self, address: u32, value: Masked<u32>) {
        self.regs.set(8 + (address as usize >> 2), value);

        *self.dma = match self.regs.take_dma_type() {
            DmaType::Rsp(..) => unreachable!(),
            DmaType::Rdp(request) => Some(request),
            DmaType::None => None,
        };
    }
}
