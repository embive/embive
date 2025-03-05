use crate::instruction::embive::CAddi4spn;
use crate::instruction::embive::InstructionImpl;
use crate::interpreter::registers::CPURegister;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::Execute;

impl<M: Memory> Execute<M> for CAddi4spn {
    #[inline(always)]
    fn execute(&self, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        // Check if illegal instruction
        if self.0.imm == 0 {
            return Err(Error::IllegalInstruction(interpreter.program_counter));
        }

        // Load the immediate value + sp into the register.
        let sp = interpreter.registers.cpu.get(CPURegister::SP as u8)?;
        let reg = interpreter.registers.cpu.get_mut(self.0.rd)?;
        *reg = sp.wrapping_add(self.0.imm);

        // Go to next instruction
        interpreter.program_counter = interpreter
            .program_counter
            .wrapping_add(Self::size() as u32);

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeCIW},
        instruction::embive::InstructionImpl,
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_caddi4spn() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default());
        *interpreter
            .registers
            .cpu
            .get_mut(CPURegister::SP as u8)
            .unwrap() = 0x1;
        let caddi4spn = TypeCIW { rd: 10, imm: 0x100 };

        let result = CAddi4spn::decode(caddi4spn.to_embive()).execute(&mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.registers.cpu.get(10).unwrap(), 0x101);
        assert_eq!(interpreter.program_counter, 0x2);
    }

    #[test]
    fn test_illegal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default());
        let caddi4spn = TypeCIW { rd: 10, imm: 0x0 };

        let result = CAddi4spn::decode(caddi4spn.to_embive()).execute(&mut interpreter);
        assert_eq!(
            result,
            Err(Error::IllegalInstruction(interpreter.program_counter))
        );
    }
}
