//! JavaScript engine integration
//!
//! Provides integration with Boa JavaScript engine (pure Rust) and DOM bindings.

mod dom_bindings;
mod runtime;

pub use dom_bindings::{DomBindings, DomNode, DomNodeType, NodeId};
pub use runtime::JsRuntime;

use crate::utils::Result;

/// Trait for JavaScript engines
pub trait JavaScriptEngine {
    /// Execute JavaScript code and return the result
    fn execute(&mut self, code: &str) -> Result<JsValue>;

    /// Evaluate an expression
    fn eval(&mut self, expression: &str) -> Result<JsValue>;

    /// Register a global function
    fn register_function(&mut self, name: &str, callback: JsCallback) -> Result<()>;
}

/// JavaScript value types
#[derive(Debug, Clone, PartialEq)]
pub enum JsValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsValue>),
    Object(std::collections::HashMap<String, JsValue>),
}

impl JsValue {
    /// Check if the value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Undefined | Self::Null => false,
            Self::Boolean(b) => *b,
            Self::Number(n) => *n != 0.0 && !n.is_nan(),
            Self::String(s) => !s.is_empty(),
            Self::Array(_) | Self::Object(_) => true,
        }
    }

    /// Convert to string representation
    pub fn to_js_string(&self) -> String {
        match self {
            Self::Undefined => "undefined".to_string(),
            Self::Null => "null".to_string(),
            Self::Boolean(b) => b.to_string(),
            Self::Number(n) => n.to_string(),
            Self::String(s) => s.clone(),
            Self::Array(arr) => format!("[{}]", arr.len()),
            Self::Object(_) => "[object Object]".to_string(),
        }
    }
}

/// JavaScript callback function type
pub type JsCallback = Box<dyn Fn(Vec<JsValue>) -> Result<JsValue> + Send + Sync>;

/// Default JavaScript engine using V8
pub struct DefaultJsEngine {
    runtime: JsRuntime,
}

impl DefaultJsEngine {
    /// Create a new JavaScript engine
    pub fn new() -> Self {
        Self {
            runtime: JsRuntime::new(),
        }
    }
}

impl Default for DefaultJsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl JavaScriptEngine for DefaultJsEngine {
    fn execute(&mut self, code: &str) -> Result<JsValue> {
        self.runtime.execute(code)
    }

    fn eval(&mut self, expression: &str) -> Result<JsValue> {
        self.runtime.eval(expression)
    }

    fn register_function(&mut self, name: &str, callback: JsCallback) -> Result<()> {
        self.runtime.register_function(name, callback)
    }
}
