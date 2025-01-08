//! Transpiler module
//! Convert RISC-V ELF to Embive binary.

mod convert;
mod error;

use elf::{
    abi::{SHF_ALLOC, SHF_EXECINSTR, SHT_PROGBITS},
    endian::LittleEndian,
    file::Class,
    ElfBytes,
};

#[doc(inline)]
pub use error::Error;

use convert::convert;

const RISCV_MACHINE: u16 = 0xF3;

/// Parse RISC-V ELF, extracting the binary data and converting the instructions to the Embive format.
///
/// How it works:
/// - Iterate over the ELF sections
///     - If the section is of type `ProgBits` and has the flag `Alloc`:
///         - Iterate over the ELF segments
///             - If the segment contains the section:
///                 - Translate virtual address to physical address
///         - Write the section data to the output buffer (handling the alignment and address translation)
///         - If the section has the flag `Execinstr`:
///            - Convert the RISC-V instructions to Embive instructions
///
/// # Arguments
/// - `elf`: The RISC-V ELF file.
/// - `output`: The output buffer to write the Embive binary format.
///
/// # Returns
/// - `Ok(usize)`: Transpilation was successful, returns the size of the binary.
/// - `Err(Error)`: An error occurred during the transpilation.
pub fn transpile_elf(elf: &[u8], output: &mut [u8]) -> Result<usize, Error> {
    let elf_bytes = ElfBytes::<LittleEndian>::minimal_parse(elf)?;

    let segments = elf_bytes.segments().ok_or(Error::NoProgramHeader)?;
    let sections = elf_bytes.section_headers().ok_or(Error::NoSectionHeader)?;

    // Check if the ELF is a RISC-V 32-bit ELF
    if elf_bytes.ehdr.e_machine != RISCV_MACHINE || elf_bytes.ehdr.class != Class::ELF32 {
        return Err(Error::InvalidPlatform);
    }

    let entry = elf_bytes.ehdr.e_entry as u32;
    let mut binary_size = 0;
    let mut needs_padding = false;
    // Iterate over the ELF sections
    'section: for (i, section) in sections.iter().enumerate() {
        // If the section is of type `ProgBits` and has the flag `Alloc`
        if section.sh_type == SHT_PROGBITS && (section.sh_flags as u32 & SHF_ALLOC) != 0 {
            let addr = section.sh_addr as u32;
            'segment: {
                // Iterate over the ELF segments
                for segment in segments.iter() {
                    // If the segment contains the section
                    if addr >= segment.p_vaddr as u32
                        && addr + section.sh_size as u32
                            <= segment.p_vaddr as u32 + segment.p_memsz as u32
                    {
                        // Translate virtual address to physical address
                        let paddr = addr - segment.p_vaddr as u32 + segment.p_paddr as u32;

                        // Get the section offset from the entry point (next aligned address)
                        let alignment = section.sh_addralign as u32;
                        let offset = ((paddr - entry).div_ceil(alignment) * alignment) as usize;

                        // Write the section data to the output buffer
                        let data = elf
                            .get(
                                section.sh_offset as usize
                                    ..section.sh_offset as usize + section.sh_size as usize,
                            )
                            .ok_or(Error::MissingSectionData(i))?;

                        let end_offset = offset + section.sh_size as usize;

                        // Ignore empty sections
                        if end_offset == paddr as usize {
                            continue 'section;
                        }

                        // Update the binary size if needed
                        if end_offset > binary_size {
                            binary_size = end_offset;
                        }

                        // Padding is only needed if the last bytes are a compressed instruction
                        // Interpreter fetches 4 bytes at a time, even if it is a compressed instruction
                        needs_padding = false;
                        output
                            .get_mut(offset..end_offset)
                            .ok_or(Error::BufferTooSmall)?
                            .copy_from_slice(data);

                        // If the section has the flag `Execinstr`
                        if (section.sh_flags as u32 & SHF_EXECINSTR) != 0 {
                            // Convert the RISC-V instructions to Embive instructions
                            let section_size = section.sh_size as usize;
                            needs_padding =
                                transpile_raw(&mut output[offset..offset + section_size])?;
                        }

                        break 'segment;
                    }
                }

                // Segment not found for the section
                return Err(Error::NoSegmentForSection(i));
            }
        }
    }

    // Add padding if needed
    if needs_padding {
        output
            .get_mut(binary_size..binary_size + 2)
            .ok_or(Error::BufferTooSmall)?
            .copy_from_slice(&[0, 0]);

        binary_size += 2;
    }

    Ok(binary_size)
}

/// Transpile raw RISC-V instructions to Embive instructions.
/// The code buffer must be padded to multiple of 4 bytes.
///
/// # Arguments
/// - `code`: The raw RISC-V instructions.
///
/// # Returns
/// - `Ok(bool)`: Transpilation was successful, returns if the code buffer needs padding.
/// - `Err(Error)`: An error occurred during the transpilation.
pub(crate) fn transpile_raw(code: &mut [u8]) -> Result<bool, Error> {
    let code_size = code.len();
    let mut needs_padding = false;

    let mut i = 0;
    while i + 2 <= code_size {
        // Last instruction may be a compressed instruction (2 bytes)
        let raw = if i + 4 > code_size {
            needs_padding = true;
            // Unwrap is safe because the slice is 2 bytes
            u16::from_le_bytes(code[i..i + 2].try_into().unwrap()) as u32
        } else {
            // Unwrap is safe because the slice is 4 bytes
            u32::from_le_bytes(code[i..i + 4].try_into().unwrap())
        };

        // Convert the RISC-V instruction to Embive instruction
        let instruction = convert(raw)?;
        let inst_bytes = instruction.data.to_le_bytes();
        let inst_size = instruction.size as usize;

        // Copy back to the code buffer
        code[i..i + inst_size].copy_from_slice(&inst_bytes[..inst_size]);

        // Move to the next instruction
        i += inst_size;
    }

    Ok(needs_padding)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transpile() {
        let elf = include_bytes!("../tests/test.elf");
        let mut output = [0; 16384];

        let result = transpile_elf(elf, &mut output);
        assert!(result.is_ok());

        let expected = include_bytes!("../tests/test.bin");
        assert_eq!(&output[..result.unwrap()], expected);
    }
}
