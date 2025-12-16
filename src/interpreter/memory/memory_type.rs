//! Memory Type Module
//!
//! This module defines the MemoryType trait for types that can be loaded from and stored to memory.
use super::Memory;

use crate::interpreter::Error;

/// Memory Type Trait
///
/// This trait represents types that can be accessed to/from memory directly.
///
/// All types that implement this trait must handle conversion between native and RISC-V format (e.g., endianness).
///
/// Default implementation for the following types is provided:
/// - Integers (u8, u16, u32, u64, u128, i8, i16, i32, i64, i128)
/// - Floating-point numbers (f32, f64)
/// - Boolean (bool)
pub trait MemoryType<'a, M: Memory>: Sized {
    /// Load value from memory.
    ///
    /// Arguments:
    /// - `address`: Memory address to get (code or RAM).
    /// - `len`: Number of bytes to load.
    ///
    /// Returns:
    /// - `Ok(Self)`: Loaded value.
    /// - `Err(Error)`: An error occurred. Ex.: Memory address is out of bounds.
    fn load(memory: &'a mut M, address: u32) -> Result<Self, Error>;

    /// Store value to memory.
    ///
    /// Arguments:
    /// - `address`: Memory address to set (code or RAM).
    ///
    /// Returns:
    /// - `Ok(())`: Value was stored successfully.
    /// - `Err(Error)`: An error occurred. Ex.: Memory address is out of bounds.
    fn store(&self, memory: &'a mut M, address: u32) -> Result<(), Error>;
}

/// Number Memory Type Implementation
macro_rules! impl_memory_type_for_number {
    ($t:ty) => {
        impl<'a, M: Memory> MemoryType<'a, M> for $t {
            #[inline]
            fn load(memory: &'a mut M, address: u32) -> Result<Self, Error> {
                let bytes = memory.load_bytes(address, core::mem::size_of::<$t>())?;
                let array: [u8; core::mem::size_of::<$t>()] = bytes
                    .try_into()
                    .map_err(|_| Error::InvalidMemoryAccessLength(core::mem::size_of::<$t>()))?;
                Ok(Self::from_le_bytes(array))
            }

            #[inline]
            fn store(&self, memory: &'a mut M, address: u32) -> Result<(), Error> {
                memory.store_bytes(address, &self.to_le_bytes())
            }
        }
    };
}

impl_memory_type_for_number!(i8);
impl_memory_type_for_number!(i16);
impl_memory_type_for_number!(i32);
impl_memory_type_for_number!(i64);
impl_memory_type_for_number!(i128);
impl_memory_type_for_number!(u8);
impl_memory_type_for_number!(u16);
impl_memory_type_for_number!(u32);
impl_memory_type_for_number!(u64);
impl_memory_type_for_number!(u128);
impl_memory_type_for_number!(f32);
impl_memory_type_for_number!(f64);

impl<'a, M: Memory> MemoryType<'a, M> for bool {
    #[inline]
    fn load(memory: &'a mut M, address: u32) -> Result<Self, Error> {
        let byte = u8::load(memory, address)?;
        Ok(byte != 0)
    }

    #[inline]
    fn store(&self, memory: &'a mut M, address: u32) -> Result<(), Error> {
        let byte = *self as u8;
        byte.store(memory, address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::memory::{SliceMemory, RAM_OFFSET};

    #[test]
    fn test_i8_load_store() {
        let mut ram = [0; 1];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let address = RAM_OFFSET;

        // Test storing
        let value = -42i8;
        assert!(value.store(&mut memory, address).is_ok());

        // Test loading
        let result = i8::load(&mut memory, address);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), value);
    }
    #[test]
    fn test_u8_load_store() {
        let mut ram = [0; 1];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let address = RAM_OFFSET;

        let value = 255u8;
        assert!(value.store(&mut memory, address).is_ok());

        let result = u8::load(&mut memory, address);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), value);
    }

    #[test]
    fn test_i16_load_store() {
        let mut ram = [0; 2];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let address = RAM_OFFSET;

        let value = -12345i16;
        assert!(value.store(&mut memory, address).is_ok());

        let result = i16::load(&mut memory, address);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), value);
    }

    #[test]
    fn test_u16_load_store() {
        let mut ram = [0; 2];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let address = RAM_OFFSET;

        let value = 54321u16;
        assert!(value.store(&mut memory, address).is_ok());

        let result = u16::load(&mut memory, address);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), value);
    }

    #[test]
    fn test_i32_load_store() {
        let mut ram = [0; 4];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let address = RAM_OFFSET;

        let value = -123456789i32;
        assert!(value.store(&mut memory, address).is_ok());

        let result = i32::load(&mut memory, address);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), value);
    }

    #[test]
    fn test_u32_load_store() {
        let mut ram = [0; 4];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let address = RAM_OFFSET;

        let value = 123456789u32;
        assert!(value.store(&mut memory, address).is_ok());

        let result = u32::load(&mut memory, address);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), value);
    }

    #[test]
    fn test_i64_load_store() {
        let mut ram = [0; 8];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let address = RAM_OFFSET;

        let value = -1234567890123456789i64;
        assert!(value.store(&mut memory, address).is_ok());

        let result = i64::load(&mut memory, address);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), value);
    }

    #[test]
    fn test_u64_load_store() {
        let mut ram = [0; 8];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let address = RAM_OFFSET;

        let value = 12345678901234567890u64;
        assert!(value.store(&mut memory, address).is_ok());

        let result = u64::load(&mut memory, address);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), value);
    }

    #[test]
    fn test_i128_load_store() {
        let mut ram = [0; 16];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let address = RAM_OFFSET;

        let value = -123456789012345678901234567890123456i128;
        assert!(value.store(&mut memory, address).is_ok());

        let result = i128::load(&mut memory, address);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), value);
    }

    #[test]
    fn test_u128_load_store() {
        let mut ram = [0; 16];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let address = RAM_OFFSET;

        let value = 123456789012345678901234567890123456u128;
        assert!(value.store(&mut memory, address).is_ok());

        let result = u128::load(&mut memory, address);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), value);
    }

    #[test]
    fn test_f32_load_store() {
        let mut ram = [0; 4];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let address = RAM_OFFSET;

        let value = core::f32::consts::PI;
        assert!(value.store(&mut memory, address).is_ok());

        let result = f32::load(&mut memory, address);
        assert!(result.is_ok());
        assert!((result.unwrap() - value).abs() < f32::EPSILON);
    }

    #[test]
    fn test_f64_load_store() {
        let mut ram = [0; 8];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let address = RAM_OFFSET;

        let value = core::f64::consts::E;
        assert!(value.store(&mut memory, address).is_ok());

        let result = f64::load(&mut memory, address);
        assert!(result.is_ok());
        assert!((result.unwrap() - value).abs() < f64::EPSILON);
    }

    #[test]
    fn test_bool_load_store() {
        let mut ram = [0; 1];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let address = RAM_OFFSET;

        let value = true;
        assert!(value.store(&mut memory, address).is_ok());

        let result = bool::load(&mut memory, address);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), value);
    }

    #[test]
    fn test_i32_store_fail() {
        let mut ram = [0; 1];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let address = RAM_OFFSET;

        let value = i32::MAX;
        assert!(value.store(&mut memory, address).is_err());
    }

    #[test]
    fn test_i32_load_fail() {
        let mut ram = [0; 1];
        let mut memory = SliceMemory::new(&[], &mut ram);
        let address = RAM_OFFSET;

        let value = i32::MAX;
        assert!(value.store(&mut memory, address).is_err());
    }
}
