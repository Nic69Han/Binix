//! JavaScript runtime implementation

use super::{JsCallback, JsValue};
use crate::utils::{error::JsError, Result};
use std::collections::HashMap;

/// JavaScript runtime context
pub struct JsRuntime {
    global_functions: HashMap<String, JsCallback>,
    // TODO: Add V8 isolate and context
}

impl JsRuntime {
    /// Create a new JavaScript runtime
    pub fn new() -> Self {
        Self {
            global_functions: HashMap::new(),
        }
    }

    /// Execute JavaScript code
    pub fn execute(&mut self, code: &str) -> Result<JsValue> {
        // TODO: Implement actual V8 execution
        // For now, return a placeholder
        if code.trim().is_empty() {
            return Ok(JsValue::Undefined);
        }

        // Very basic expression evaluation for testing
        if let Some(result) = self.try_eval_simple(code) {
            return Ok(result);
        }

        Ok(JsValue::Undefined)
    }

    /// Evaluate a JavaScript expression
    pub fn eval(&mut self, expression: &str) -> Result<JsValue> {
        self.execute(expression)
    }

    /// Register a global function
    pub fn register_function(&mut self, name: &str, callback: JsCallback) -> Result<()> {
        self.global_functions.insert(name.to_string(), callback);
        Ok(())
    }

    /// Try to evaluate simple expressions (for testing)
    fn try_eval_simple(&self, code: &str) -> Option<JsValue> {
        let code = code.trim();

        // Boolean literals
        if code == "true" {
            return Some(JsValue::Boolean(true));
        }
        if code == "false" {
            return Some(JsValue::Boolean(false));
        }

        // Null/undefined
        if code == "null" {
            return Some(JsValue::Null);
        }
        if code == "undefined" {
            return Some(JsValue::Undefined);
        }

        // Number literals
        if let Ok(n) = code.parse::<f64>() {
            return Some(JsValue::Number(n));
        }

        // String literals (simple)
        if (code.starts_with('"') && code.ends_with('"'))
            || (code.starts_with('\'') && code.ends_with('\''))
        {
            let s = &code[1..code.len() - 1];
            return Some(JsValue::String(s.to_string()));
        }

        None
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
}

