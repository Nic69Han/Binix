//! JavaScript runtime implementation using Boa engine

use super::{JsCallback, JsValue};
use crate::utils::{error::JsError, Result};
use boa_engine::{Context, Source, JsValue as BoaJsValue};
use std::collections::HashMap;

/// JavaScript runtime context using Boa engine
pub struct JsRuntime {
    context: Context,
    global_functions: HashMap<String, JsCallback>,
}

impl JsRuntime {
    /// Create a new JavaScript runtime
    pub fn new() -> Self {
        Self {
            context: Context::default(),
            global_functions: HashMap::new(),
        }
    }

    /// Execute JavaScript code
    pub fn execute(&mut self, code: &str) -> Result<JsValue> {
        if code.trim().is_empty() {
            return Ok(JsValue::Undefined);
        }

        let source = Source::from_bytes(code);
        match self.context.eval(source) {
            Ok(result) => Ok(Self::convert_boa_value(&result, &mut self.context)),
            Err(e) => Err(JsError::Execution(e.to_string()).into()),
        }
    }

    /// Evaluate a JavaScript expression
    pub fn eval(&mut self, expression: &str) -> Result<JsValue> {
        self.execute(expression)
    }

    /// Register a global function
    pub fn register_function(&mut self, name: &str, callback: JsCallback) -> Result<()> {
        self.global_functions.insert(name.to_string(), callback);
        // TODO: Actually register the function in Boa context
        Ok(())
    }

    /// Convert Boa JsValue to our JsValue
    fn convert_boa_value(value: &BoaJsValue, context: &mut Context) -> JsValue {
        if value.is_undefined() {
            JsValue::Undefined
        } else if value.is_null() {
            JsValue::Null
        } else if let Some(b) = value.as_boolean() {
            JsValue::Boolean(b)
        } else if let Some(n) = value.as_number() {
            JsValue::Number(n)
        } else if let Some(s) = value.as_string() {
            JsValue::String(s.to_std_string_escaped())
        } else if value.is_object() {
            // Check if it's an array
            if let Ok(array) = value.to_object(context) {
                if array.is_array() {
                    // Get length property
                    if let Ok(length_val) = array.get(boa_engine::js_string!("length"), context) {
                        if let Some(length) = length_val.as_number() {
                            let mut items = Vec::new();
                            for i in 0..(length as u32) {
                                if let Ok(item) = array.get(i, context) {
                                    items.push(Self::convert_boa_value(&item, context));
                                }
                            }
                            return JsValue::Array(items);
                        }
                    }
                }
            }
            // Regular object - convert to HashMap
            JsValue::Object(HashMap::new())
        } else {
            JsValue::Undefined
        }
    }
}

impl Default for JsRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_numbers() {
        let mut runtime = JsRuntime::new();
        assert_eq!(runtime.eval("42").unwrap(), JsValue::Number(42.0));
        assert_eq!(runtime.eval("3.14").unwrap(), JsValue::Number(3.14));
    }

    #[test]
    fn test_eval_booleans() {
        let mut runtime = JsRuntime::new();
        assert_eq!(runtime.eval("true").unwrap(), JsValue::Boolean(true));
        assert_eq!(runtime.eval("false").unwrap(), JsValue::Boolean(false));
    }

    #[test]
    fn test_eval_strings() {
        let mut runtime = JsRuntime::new();
        assert_eq!(
            runtime.eval("\"hello\"").unwrap(),
            JsValue::String("hello".to_string())
        );
    }

    #[test]
    fn test_eval_arithmetic() {
        let mut runtime = JsRuntime::new();
        assert_eq!(runtime.eval("2 + 3").unwrap(), JsValue::Number(5.0));
        assert_eq!(runtime.eval("10 * 5").unwrap(), JsValue::Number(50.0));
        assert_eq!(runtime.eval("20 / 4").unwrap(), JsValue::Number(5.0));
    }

    #[test]
    fn test_eval_variables() {
        let mut runtime = JsRuntime::new();
        runtime.execute("let x = 10;").unwrap();
        assert_eq!(runtime.eval("x + 5").unwrap(), JsValue::Number(15.0));
    }

    #[test]
    fn test_eval_functions() {
        let mut runtime = JsRuntime::new();
        runtime.execute("function add(a, b) { return a + b; }").unwrap();
        assert_eq!(runtime.eval("add(3, 4)").unwrap(), JsValue::Number(7.0));
    }

    #[test]
    fn test_eval_arrays() {
        let mut runtime = JsRuntime::new();
        let result = runtime.eval("[1, 2, 3]").unwrap();
        if let JsValue::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], JsValue::Number(1.0));
            assert_eq!(arr[1], JsValue::Number(2.0));
            assert_eq!(arr[2], JsValue::Number(3.0));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_eval_string_operations() {
        let mut runtime = JsRuntime::new();
        assert_eq!(
            runtime.eval("\"hello\" + \" world\"").unwrap(),
            JsValue::String("hello world".to_string())
        );
    }
}

