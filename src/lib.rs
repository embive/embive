//! # A crate for embedding RISC-V programs in Rust.
//! The main focus of this project is to provide a way to run untrusted code in a sandboxed
//! environment, while not requiring dynamic memory allocation or the standard library.
//!
//! The end goal is to have a low-level WebAssembly alternative, where the code inside the
//! sandbox should never impact the host execution (no panics on release builds, no memory exhaustion,
//! no stack overflows, etc.); but also while being easy to use and integrate with existing Rust code.
//!
//! Without any feature enabled, this crates has no external dependencies. Check the `features`
//! section for more information.
#![no_std]
pub mod engine;
pub mod error;
mod instruction;
