use super::dma::DmaRequest;
use super::interrupt::{RcpIntType, RcpInterrupt};
use super::rsp::{DmaType, Registers as RspRegisters};
use super::WgpuContext;
use crate::util::memory::{Masked, Reader, Writer};
use core::Core;
use std::ops::{Deref, DerefMut};
use tracing::{debug_span, trace};

mod core;

pub struct Rdp {
    rcp_int: RcpInterrupt,
    commands: Vec<u64>,
    core: Core,
}

impl Rdp {
    pub fn new(ctx: WgpuContext, rcp_int: RcpInterrupt) -> Self {
        Self {
            rcp_int,
            commands: Vec::new(),
            core: Core::new(ctx),
        }
    }

    pub fn command<T: Deref<Target = RspRegisters>>(&self, rsp_regs: T) -> CommandRegisters<T> {
        CommandRegisters::new(rsp_regs)
    }

    pub fn command_mut<T: Deref<Target = RspRegisters> + DerefMut<Target = RspRegisters>>(
        &self,
        rsp_regs: T,
    ) -> CommandRegisters<T> {
        CommandRegisters::new(rsp_regs)
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

pub struct CommandRegisters<T: Deref<Target = RspRegisters>> {
    regs: T,
}

impl<T: Deref<Target = RspRegisters>> CommandRegisters<T> {
    fn new(regs: T) -> Self {
        Self { regs }
    }
}

impl<T: Deref<Target = RspRegisters>> Reader for CommandRegisters<T> {
    fn read_u32(&self, address: u32) -> u32 {
        self.regs.get(8 + (address as usize >> 2))
    }
}

impl<T: Deref<Target = RspRegisters> + DerefMut<Target = RspRegisters>> Writer
    for CommandRegisters<T>
{
    type SideEffect = Option<DmaRequest>;

    fn write_u32(&mut self, address: u32, value: Masked<u32>) -> Option<DmaRequest> {
        self.regs.set(8 + (address as usize >> 2), value);

        match self.regs.take_dma_type() {
            DmaType::Rsp(..) => unreachable!(),
            DmaType::Rdp(request) => Some(request),
            DmaType::None => None,
        }
    }
}
