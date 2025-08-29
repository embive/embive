//! GDB support through the `gdbstub` crate.

use core::num::NonZeroI32;

use gdbstub::{
    common::Signal,
    conn::ConnectionExt,
    target::{
        ext::{
            base::{
                single_register_access::{SingleRegisterAccess, SingleRegisterAccessOps},
                singlethread::{
                    SingleThreadBase, SingleThreadResume, SingleThreadResumeOps,
                    SingleThreadSingleStep, SingleThreadSingleStepOps,
                },
                BaseOps,
            },
            breakpoints::{Breakpoints, BreakpointsOps, SwBreakpoint, SwBreakpointOps},
        },
        Target, TargetError, TargetResult,
    },
};
use gdbstub_arch::riscv::{reg, Riscv32};

use super::{Debugger, ExecMode};
use crate::interpreter::{memory::Memory, registers::CSOperation, Error, SYSCALL_ARGS};

/// Base target implementation
impl<
        M: Memory,
        C: ConnectionExt,
        F: FnMut(i32, &[i32; SYSCALL_ARGS], &mut M) -> Result<Result<i32, NonZeroI32>, Error>,
        const N: usize,
    > Target for Debugger<'_, M, C, F, N>
{
    type Arch = Riscv32;
    type Error = Error;

    #[inline(always)]
    fn base_ops(&mut self) -> gdbstub::target::ext::base::BaseOps<'_, Self::Arch, Self::Error> {
        BaseOps::SingleThread(self)
    }

    #[inline(always)]
    fn support_breakpoints(&mut self) -> Option<BreakpointsOps<'_, Self>> {
        Some(self)
    }
}

/// Single thread target implementation
impl<
        M: Memory,
        C: ConnectionExt,
        F: FnMut(i32, &[i32; SYSCALL_ARGS], &mut M) -> Result<Result<i32, NonZeroI32>, Error>,
        const N: usize,
    > SingleThreadBase for Debugger<'_, M, C, F, N>
{
    fn read_registers(&mut self, regs: &mut reg::RiscvCoreRegs<u32>) -> TargetResult<(), Self> {
        for (i, reg) in regs.x.iter_mut().enumerate() {
            *reg = self.interpreter.registers.cpu.inner[i] as u32;
        }

        regs.pc = self.interpreter.program_counter;

        Ok(())
    }

    fn write_registers(&mut self, regs: &reg::RiscvCoreRegs<u32>) -> TargetResult<(), Self> {
        for (i, reg) in regs.x.iter().enumerate() {
            self.interpreter.registers.cpu.inner[i] = *reg as i32;
        }

        self.interpreter.program_counter = regs.pc;

        Ok(())
    }

    fn read_addrs(
        &mut self,
        start_addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
        data: &mut [u8],
    ) -> TargetResult<usize, Self> {
        let res = self
            .interpreter
            .memory
            .load_bytes(start_addr, data.len())
            .map_err(TargetError::Fatal)?;
        data.copy_from_slice(res);

        Ok(data.len())
    }

    fn write_addrs(
        &mut self,
        start_addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
        data: &[u8],
    ) -> TargetResult<(), Self> {
        self.interpreter
            .memory
            .store_bytes(start_addr, data)
            .map_err(TargetError::Fatal)?;

        Ok(())
    }

    #[inline(always)]
    fn support_resume(&mut self) -> Option<SingleThreadResumeOps<'_, Self>> {
        Some(self)
    }

    #[inline(always)]
    fn support_single_register_access(&mut self) -> Option<SingleRegisterAccessOps<'_, (), Self>> {
        Some(self)
    }
}

/// Breakpoint implementation
impl<
        M: Memory,
        C: ConnectionExt,
        F: FnMut(i32, &[i32; SYSCALL_ARGS], &mut M) -> Result<Result<i32, NonZeroI32>, Error>,
        const N: usize,
    > Breakpoints for Debugger<'_, M, C, F, N>
{
    #[inline(always)]
    fn support_sw_breakpoint(&mut self) -> Option<SwBreakpointOps<'_, Self>> {
        Some(self)
    }
}

// Software breakpoint implementation
impl<
        M: Memory,
        C: ConnectionExt,
        F: FnMut(i32, &[i32; SYSCALL_ARGS], &mut M) -> Result<Result<i32, NonZeroI32>, Error>,
        const N: usize,
    > SwBreakpoint for Debugger<'_, M, C, F, N>
{
    fn add_sw_breakpoint(&mut self, addr: u32, _kind: usize) -> TargetResult<bool, Self> {
        match self.breakpoints.iter().position(|b| b.is_none()) {
            Some(i) => {
                self.breakpoints[i] = Some(addr);
                Ok(true)
            }
            None => Ok(false),
        }
    }

    fn remove_sw_breakpoint(&mut self, addr: u32, _kind: usize) -> TargetResult<bool, Self> {
        match self.breakpoints.iter().position(|b| *b == Some(addr)) {
            Some(i) => {
                self.breakpoints[i] = None;
                Ok(true)
            }
            None => Ok(false),
        }
    }
}

// Single thread resume implementation
impl<
        M: Memory,
        C: ConnectionExt,
        F: FnMut(i32, &[i32; SYSCALL_ARGS], &mut M) -> Result<Result<i32, NonZeroI32>, Error>,
        const N: usize,
    > SingleThreadResume for Debugger<'_, M, C, F, N>
{
    fn resume(&mut self, _signal: Option<Signal>) -> Result<(), Self::Error> {
        self.exec_mode = ExecMode::Run;
        Ok(())
    }

    #[inline(always)]
    fn support_single_step(&mut self) -> Option<SingleThreadSingleStepOps<'_, Self>> {
        Some(self)
    }
}

// Single thread single step implementation
impl<
        M: Memory,
        C: ConnectionExt,
        F: FnMut(i32, &[i32; SYSCALL_ARGS], &mut M) -> Result<Result<i32, NonZeroI32>, Error>,
        const N: usize,
    > SingleThreadSingleStep for Debugger<'_, M, C, F, N>
{
    fn step(&mut self, _signal: Option<Signal>) -> Result<(), Self::Error> {
        self.exec_mode = ExecMode::Step;
        Ok(())
    }
}

// Single register access implementation
impl<
        M: Memory,
        C: ConnectionExt,
        F: FnMut(i32, &[i32; SYSCALL_ARGS], &mut M) -> Result<Result<i32, NonZeroI32>, Error>,
        const N: usize,
    > SingleRegisterAccess<()> for Debugger<'_, M, C, F, N>
{
    fn read_register(
        &mut self,
        _tid: (),
        reg_id: reg::id::RiscvRegId<u32>,
        buf: &mut [u8],
    ) -> TargetResult<usize, Self> {
        if buf.len() < 4 {
            return Err(TargetError::NonFatal);
        }

        match reg_id {
            reg::id::RiscvRegId::Pc => {
                let pc = self.interpreter.program_counter;
                buf[0..4].copy_from_slice(&pc.to_le_bytes());
            }
            reg::id::RiscvRegId::Gpr(i) => {
                let reg = self
                    .interpreter
                    .registers
                    .cpu
                    .get(i)
                    .map_err(TargetError::Fatal)?;
                buf[0..4].copy_from_slice(&reg.to_le_bytes());
            }
            reg::id::RiscvRegId::Fpr(i) => {
                return Err(TargetError::Fatal(Error::InvalidCPURegister(i)))
            }
            reg::id::RiscvRegId::Csr(i) => {
                let csr = self
                    .interpreter
                    .registers
                    .control_status
                    .operation(None, i)
                    .map_err(TargetError::Fatal)?;
                buf[0..4].copy_from_slice(&csr.to_le_bytes());
            }
            _ => return Err(TargetError::NonFatal),
        }

        Ok(4)
    }

    fn write_register(
        &mut self,
        _tid: (),
        reg_id: reg::id::RiscvRegId<u32>,
        buf: &[u8],
    ) -> TargetResult<(), Self> {
        if buf.len() > 4 {
            return Err(TargetError::NonFatal);
        }

        let mut pad_buf = [0; 4];
        pad_buf[..buf.len()].copy_from_slice(buf);

        let val = u32::from_le_bytes(pad_buf);

        match reg_id {
            reg::id::RiscvRegId::Pc => self.interpreter.program_counter = val,
            reg::id::RiscvRegId::Gpr(i) => {
                let reg = self
                    .interpreter
                    .registers
                    .cpu
                    .get_mut(i)
                    .map_err(TargetError::Fatal)?;
                *reg = val as i32;
            }
            reg::id::RiscvRegId::Fpr(i) => {
                return Err(TargetError::Fatal(Error::InvalidCPURegister(i)))
            }
            reg::id::RiscvRegId::Csr(i) => {
                self.interpreter
                    .registers
                    .control_status
                    .operation(Some(CSOperation::Write(val)), i)
                    .map_err(TargetError::Fatal)?;
            }
            _ => return Err(TargetError::NonFatal),
        }

        Ok(())
    }
}
