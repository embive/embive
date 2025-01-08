use crate::instruction::embive::CAddi4spn;
use crate::interpreter::registers::CPURegister;
use crate::interpreter::{memory::Memory, Error, Interpreter, State};

use super::super::DecodeExecute;

impl<M: Memory> DecodeExecute<M> for CAddi4spn {
    #[inline(always)]
    fn decode_execute(data: u32, interpreter: &mut Interpreter<'_, M>) -> Result<State, Error> {
        let inst = Self::decode(data);

        // Check if illegal instruction
        if inst.imm == 0 {
            return Err(Error::IllegalInstruction(data));
        }

        // Load the immediate value + sp into the register.
        let sp = interpreter.registers.cpu.get(CPURegister::SP as u8)?;
        let reg = interpreter.registers.cpu.get_mut(inst.rd)?;
        *reg = sp.wrapping_add(inst.imm);

        // Go to next instruction
        interpreter.program_counter = interpreter.program_counter.wrapping_add(Self::SIZE as u32);

        Ok(State::Running)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Format, TypeCIW},
        interpreter::memory::SliceMemory,
    };

    use super::*;

    #[test]
    fn test_caddi4spn() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        *interpreter
            .registers
            .cpu
            .get_mut(CPURegister::SP as u8)
            .unwrap() = 0x1;
        let caddi4spn = TypeCIW { rd: 10, imm: 0x100 };

        let result = CAddi4spn::decode_execute(caddi4spn.to_embive(), &mut interpreter);
        assert_eq!(result, Ok(State::Running));
        assert_eq!(interpreter.registers.cpu.get(10).unwrap(), 0x101);
        assert_eq!(interpreter.program_counter, 0x2);
    }

    #[test]
    fn test_illegal() {
        let mut memory = SliceMemory::new(&[], &mut []);
        let mut interpreter = Interpreter::new(&mut memory, Default::default()).unwrap();
        let caddi4spn = TypeCIW { rd: 10, imm: 0x0 };

        let result = CAddi4spn::decode_execute(caddi4spn.to_embive(), &mut interpreter);
        assert_eq!(
            result,
            Err(Error::IllegalInstruction(caddi4spn.to_embive()))
        );
    }
}
