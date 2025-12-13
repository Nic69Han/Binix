//! WebAssembly runtime for Binix browser
//!
//! Provides WebAssembly execution with SIMD and threading support.

mod runtime;

pub use runtime::{WasmInstance, WasmModule, WasmRuntime, WasmValue};
