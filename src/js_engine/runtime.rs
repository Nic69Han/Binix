//! JavaScript runtime implementation using Boa engine

use super::{JsCallback, JsValue, advanced_apis, web_apis};
use crate::utils::{Result, error::JsError};
use boa_engine::{
    Context, JsValue as BoaJsValue, NativeFunction, Source,
    object::ObjectInitializer,
    property::Attribute,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Captured output from a JS execution session
#[derive(Debug, Clone, Default)]
pub struct JsOutput {
    pub logs: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl JsOutput {
    pub fn all_logs(&self) -> Vec<String> { self.logs.clone() }
    pub fn all_errors(&self) -> Vec<String> { self.errors.clone() }
}

/// JavaScript runtime context using Boa engine
pub struct JsRuntime {
    context: Context,
    global_functions: HashMap<String, JsCallback>,
    output: Arc<Mutex<JsOutput>>,
}

impl JsRuntime {
    pub fn new() -> Self { Self::with_url("about:blank") }

    pub fn with_url(url: &str) -> Self {
        let output = Arc::new(Mutex::new(JsOutput::default()));
        let mut context = Context::default();
        Self::init_console(&mut context, Arc::clone(&output));
        web_apis::init_web_apis(&mut context, url);
        advanced_apis::init_advanced_apis(&mut context);
        Self { context, global_functions: HashMap::new(), output }
    }

    fn init_console(context: &mut Context, output: Arc<Mutex<JsOutput>>) {
        let out_log = Arc::clone(&output);
        let log_fn = NativeFunction::from_copy_closure(move |_this, args, ctx| {
            let msg = format_console_args(args, ctx);
            log::info!("[JS] {}", msg);
            if let Ok(mut o) = out_log.lock() { o.logs.push(msg); }
            Ok(BoaJsValue::undefined())
        });

        let out_warn = Arc::clone(&output);
        let warn_fn = NativeFunction::from_copy_closure(move |_this, args, ctx| {
            let msg = format_console_args(args, ctx);
            log::warn!("[JS WARN] {}", msg);
            if let Ok(mut o) = out_warn.lock() {
                o.warnings.push(msg.clone());
                o.logs.push(format!("[warn] {}", msg));
            }
            Ok(BoaJsValue::undefined())
        });

        let out_err = Arc::clone(&output);
        let error_fn = NativeFunction::from_copy_closure(move |_this, args, ctx| {
            let msg = format_console_args(args, ctx);
            log::error!("[JS ERROR] {}", msg);
            if let Ok(mut o) = out_err.lock() {
                o.errors.push(msg.clone());
                o.logs.push(format!("[error] {}", msg));
            }
            Ok(BoaJsValue::undefined())
        });

        let out_info = Arc::clone(&output);
        let info_fn = NativeFunction::from_copy_closure(move |_this, args, ctx| {
            let msg = format_console_args(args, ctx);
            log::info!("[JS INFO] {}", msg);
            if let Ok(mut o) = out_info.lock() { o.logs.push(msg); }
            Ok(BoaJsValue::undefined())
        });

        let console = ObjectInitializer::new(context)
            .function(log_fn, boa_engine::js_string!("log"), 0)
            .function(warn_fn, boa_engine::js_string!("warn"), 0)
            .function(error_fn, boa_engine::js_string!("error"), 0)
            .function(info_fn, boa_engine::js_string!("info"), 0)
            .build();

        context
            .register_global_property(boa_engine::js_string!("console"), console, Attribute::all())
            .expect("Failed to register console");
    }

    pub fn execute(&mut self, code: &str) -> Result<JsValue> {
        if code.trim().is_empty() { return Ok(JsValue::Undefined); }
        let source = Source::from_bytes(code);
        match self.context.eval(source) {
            Ok(result) => Ok(Self::convert_boa_value(&result, &mut self.context)),
            Err(e) => Err(JsError::Execution(e.to_string()).into()),
        }
    }

    pub fn eval(&mut self, expression: &str) -> Result<JsValue> { self.execute(expression) }

    pub fn register_function(&mut self, name: &str, callback: JsCallback) -> Result<()> {
        self.global_functions.insert(name.to_string(), callback);
        Ok(())
    }

    /// Drain all captured console output since last call
    pub fn take_output(&self) -> JsOutput {
        self.output.lock().map(|mut o| std::mem::take(&mut *o)).unwrap_or_default()
    }

    /// Execute a batch of scripts and return combined console output + errors
    pub fn execute_scripts(&mut self, scripts: &[String]) -> JsOutput {
        let mut combined = JsOutput::default();
        for (i, script) in scripts.iter().enumerate() {
            log::info!("Executing script {} ({} bytes)", i + 1, script.len());
            if let Err(e) = self.execute(script) {
                let msg = format!("Script {} error: {}", i + 1, e);
                log::warn!("{}", msg);
                combined.errors.push(msg);
            }
            let partial = self.take_output();
            combined.logs.extend(partial.logs);
            combined.warnings.extend(partial.warnings);
            combined.errors.extend(partial.errors);
        }
        combined
    }

    fn convert_boa_value(value: &BoaJsValue, context: &mut Context) -> JsValue {
        if value.is_undefined() { JsValue::Undefined }
        else if value.is_null() { JsValue::Null }
        else if let Some(b) = value.as_boolean() { JsValue::Boolean(b) }
        else if let Some(n) = value.as_number() { JsValue::Number(n) }
        else if let Some(s) = value.as_string() { JsValue::String(s.to_std_string_escaped()) }
        else if value.is_object() {
            if let Ok(array) = value.to_object(context) {
                if array.is_array() {
                    if let Ok(len_val) = array.get(boa_engine::js_string!("length"), context) {
                        if let Some(length) = len_val.as_number() {
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
            JsValue::Object(HashMap::new())
        } else { JsValue::Undefined }
    }
}

fn format_console_args(args: &[BoaJsValue], context: &mut Context) -> String {
    args.iter().map(|arg| {
        if arg.is_undefined() { "undefined".to_string() }
        else if arg.is_null() { "null".to_string() }
        else if let Some(s) = arg.as_string() { s.to_std_string_escaped() }
        else if let Some(n) = arg.as_number() { n.to_string() }
        else if let Some(b) = arg.as_boolean() { b.to_string() }
        else if arg.is_object() {
            if let Ok(Some(json)) = arg.to_json(context) { json.to_string() }
            else { "[object Object]".to_string() }
        } else { format!("{:?}", arg) }
    }).collect::<Vec<_>>().join(" ")
}

impl Default for JsRuntime {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_numbers() {
        let mut r = JsRuntime::new();
        assert_eq!(r.eval("42").unwrap(), JsValue::Number(42.0));
    }

    #[test]
    fn test_eval_arithmetic() {
        let mut r = JsRuntime::new();
        assert_eq!(r.eval("2 + 3").unwrap(), JsValue::Number(5.0));
    }

    #[test]
    fn test_eval_variables() {
        let mut r = JsRuntime::new();
        r.execute("let x = 10;").unwrap();
        assert_eq!(r.eval("x + 5").unwrap(), JsValue::Number(15.0));
    }

    #[test]
    fn test_console_capture() {
        let mut r = JsRuntime::new();
        r.execute("console.log('hello'); console.log('world');").unwrap();
        let output = r.take_output();
        assert_eq!(output.logs, vec!["hello", "world"]);
    }

    #[test]
    fn test_execute_scripts_batch() {
        let mut r = JsRuntime::new();
        let scripts = vec!["let a = 1;".to_string(), "let b = 2; console.log(a + b);".to_string()];
        let output = r.execute_scripts(&scripts);
        assert_eq!(output.logs, vec!["3"]);
        assert!(output.errors.is_empty());
    }
}
