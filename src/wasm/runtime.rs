//! WebAssembly runtime implementation using wasmtime

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wasmtime::*;

use crate::utils::error::{BinixError, JsError, RenderError, Result};

/// WebAssembly value types
#[derive(Debug, Clone, PartialEq)]
pub enum WasmValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl From<WasmValue> for Val {
    fn from(value: WasmValue) -> Self {
        match value {
            WasmValue::I32(v) => Val::I32(v),
            WasmValue::I64(v) => Val::I64(v),
            WasmValue::F32(v) => Val::F32(v.to_bits()),
            WasmValue::F64(v) => Val::F64(v.to_bits()),
        }
    }
}

impl TryFrom<Val> for WasmValue {
    type Error = BinixError;

    fn try_from(val: Val) -> Result<Self> {
        match val {
            Val::I32(v) => Ok(WasmValue::I32(v)),
            Val::I64(v) => Ok(WasmValue::I64(v)),
            Val::F32(v) => Ok(WasmValue::F32(f32::from_bits(v))),
            Val::F64(v) => Ok(WasmValue::F64(f64::from_bits(v))),
            _ => Err(BinixError::JavaScript(JsError::Wasm(
                "Unsupported WASM value type".to_string(),
            ))),
        }
    }
}

/// WebAssembly module wrapper
#[derive(Clone)]
pub struct WasmModule {
    module: Module,
}

impl WasmModule {
    /// Get exported function names
    pub fn exports(&self) -> Vec<String> {
        self.module
            .exports()
            .filter_map(|e| {
                if matches!(e.ty(), ExternType::Func(_)) {
                    Some(e.name().to_string())
                } else {
                    None
                }
            })
            .collect()
    }
}

/// WebAssembly instance wrapper
pub struct WasmInstance {
    instance: Instance,
    store: Arc<Mutex<Store<()>>>,
}

impl WasmInstance {
    /// Call an exported function
    pub fn call(&self, name: &str, args: &[WasmValue]) -> Result<Vec<WasmValue>> {
        let mut store = self.store.lock().map_err(|_| {
            BinixError::JavaScript(JsError::Wasm("Failed to lock WASM store".to_string()))
        })?;

        let func = self
            .instance
            .get_func(&mut *store, name)
            .ok_or_else(|| {
                BinixError::JavaScript(JsError::Wasm(format!("Function '{}' not found", name)))
            })?;

        let params: Vec<Val> = args.iter().cloned().map(Val::from).collect();
        let func_ty = func.ty(&*store);
        let result_count = func_ty.results().len();
        let mut results = vec![Val::I32(0); result_count];

        func.call(&mut *store, &params, &mut results).map_err(|e| {
            BinixError::JavaScript(JsError::Wasm(format!("WASM call error: {}", e)))
        })?;

        results
            .into_iter()
            .map(WasmValue::try_from)
            .collect()
    }

    /// Get memory export if available
    pub fn get_memory(&self) -> Option<Memory> {
        let mut store = self.store.lock().ok()?;
        self.instance.get_memory(&mut *store, "memory")
    }
}

/// WebAssembly runtime
pub struct WasmRuntime {
    engine: Engine,
    modules: HashMap<String, WasmModule>,
}

impl WasmRuntime {
    /// Create a new WASM runtime
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.wasm_simd(true);
        config.wasm_threads(true);
        config.wasm_multi_memory(true);
        config.wasm_bulk_memory(true);

        let engine = Engine::new(&config).map_err(|e| {
            BinixError::JavaScript(JsError::Wasm(format!(
                "Failed to create WASM engine: {}",
                e
            )))
        })?;

        Ok(Self {
            engine,
            modules: HashMap::new(),
        })
    }

    /// Compile a WASM module from bytes
    pub fn compile(&mut self, name: &str, bytes: &[u8]) -> Result<WasmModule> {
        let module = Module::new(&self.engine, bytes).map_err(|e| {
            BinixError::Render(RenderError::ResourceLoad(format!(
                "Failed to compile WASM module: {}",
                e
            )))
        })?;

        let wasm_module = WasmModule { module };
        self.modules.insert(name.to_string(), wasm_module.clone());
        Ok(wasm_module)
    }

    /// Compile a WASM module from WAT text format
    pub fn compile_wat(&mut self, name: &str, wat: &str) -> Result<WasmModule> {
        let module = Module::new(&self.engine, wat).map_err(|e| {
            BinixError::Render(RenderError::ResourceLoad(format!(
                "Failed to compile WAT: {}",
                e
            )))
        })?;

        let wasm_module = WasmModule { module };
        self.modules.insert(name.to_string(), wasm_module.clone());
        Ok(wasm_module)
    }

    /// Instantiate a compiled module
    pub fn instantiate(&self, module: &WasmModule) -> Result<WasmInstance> {
        let store = Arc::new(Mutex::new(Store::new(&self.engine, ())));

        let instance = {
            let mut store_guard = store.lock().map_err(|_| {
                BinixError::JavaScript(JsError::Wasm("Failed to lock store".to_string()))
            })?;
            Instance::new(&mut *store_guard, &module.module, &[]).map_err(|e| {
                BinixError::JavaScript(JsError::Wasm(format!(
                    "Failed to instantiate module: {}",
                    e
                )))
            })?
        };

        Ok(WasmInstance { instance, store })
    }

    /// Get a cached module by name
    pub fn get_module(&self, name: &str) -> Option<&WasmModule> {
        self.modules.get(name)
    }

    /// Check if SIMD is supported
    pub fn supports_simd(&self) -> bool {
        true
    }

    /// Check if threading is supported
    pub fn supports_threads(&self) -> bool {
        true
    }
}

impl Default for WasmRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create WASM runtime")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_runtime_creation() {
        let runtime = WasmRuntime::new();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_compile_wat() {
        let mut runtime = WasmRuntime::new().unwrap();
        let wat = r#"
            (module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                )
            )
        "#;
        let module = runtime.compile_wat("test", wat);
        assert!(module.is_ok());
    }

    #[test]
    fn test_call_wasm_function() {
        let mut runtime = WasmRuntime::new().unwrap();
        let wat = r#"
            (module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                )
                (func (export "multiply") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.mul
                )
            )
        "#;

        let module = runtime.compile_wat("math", wat).unwrap();
        let instance = runtime.instantiate(&module).unwrap();

        // Test add
        let result = instance.call("add", &[WasmValue::I32(5), WasmValue::I32(3)]).unwrap();
        assert_eq!(result, vec![WasmValue::I32(8)]);

        // Test multiply
        let result = instance.call("multiply", &[WasmValue::I32(4), WasmValue::I32(7)]).unwrap();
        assert_eq!(result, vec![WasmValue::I32(28)]);
    }

    #[test]
    fn test_wasm_exports() {
        let mut runtime = WasmRuntime::new().unwrap();
        let wat = r#"
            (module
                (func (export "foo"))
                (func (export "bar"))
            )
        "#;

        let module = runtime.compile_wat("exports", wat).unwrap();
        let exports = module.exports();

        assert!(exports.contains(&"foo".to_string()));
        assert!(exports.contains(&"bar".to_string()));
    }

    #[test]
    fn test_wasm_value_conversion() {
        assert_eq!(WasmValue::I32(42), WasmValue::I32(42));
        assert_eq!(WasmValue::F64(3.14), WasmValue::F64(3.14));
    }

    #[test]
    fn test_simd_support() {
        let runtime = WasmRuntime::new().unwrap();
        assert!(runtime.supports_simd());
        assert!(runtime.supports_threads());
    }

    #[test]
    fn test_fibonacci_wasm() {
        let mut runtime = WasmRuntime::new().unwrap();
        let wat = r#"
            (module
                (func $fib (export "fib") (param i32) (result i32)
                    (if (result i32) (i32.le_s (local.get 0) (i32.const 1))
                        (then (local.get 0))
                        (else
                            (i32.add
                                (call $fib (i32.sub (local.get 0) (i32.const 1)))
                                (call $fib (i32.sub (local.get 0) (i32.const 2)))
                            )
                        )
                    )
                )
            )
        "#;

        let module = runtime.compile_wat("fib", wat).unwrap();
        let instance = runtime.instantiate(&module).unwrap();

        let result = instance.call("fib", &[WasmValue::I32(10)]).unwrap();
        assert_eq!(result, vec![WasmValue::I32(55)]);
    }
}

