//! Web APIs implementation for JavaScript runtime
//!
//! Implements browser-like global objects and functions:
//! - window object
//! - document object (basic)
//! - setTimeout/setInterval
//! - localStorage
//! - navigator
//! - location

use boa_engine::{
    Context, JsValue as BoaJsValue, NativeFunction,
    object::ObjectInitializer,
    property::Attribute,
    JsResult, JsArgs,
};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Pending timers for setTimeout/setInterval
#[derive(Default)]
pub struct TimerManager {
    /// Pending timeouts: (id, code, delay_ms, created_at, is_interval)
    pub timers: Vec<(u32, String, u64, std::time::Instant, bool)>,
    next_id: u32,
}

impl TimerManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_timeout(&mut self, code: String, delay_ms: u64) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.timers.push((id, code, delay_ms, std::time::Instant::now(), false));
        id
    }

    pub fn add_interval(&mut self, code: String, delay_ms: u64) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.timers.push((id, code, delay_ms, std::time::Instant::now(), true));
        id
    }

    pub fn clear(&mut self, id: u32) {
        self.timers.retain(|(timer_id, _, _, _, _)| *timer_id != id);
    }
}

/// LocalStorage implementation
#[derive(Default, Clone)]
pub struct LocalStorage {
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl LocalStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_item(&self, key: &str) -> Option<String> {
        self.data.lock().ok()?.get(key).cloned()
    }

    pub fn set_item(&self, key: &str, value: &str) {
        if let Ok(mut data) = self.data.lock() {
            data.insert(key.to_string(), value.to_string());
        }
    }

    pub fn remove_item(&self, key: &str) {
        if let Ok(mut data) = self.data.lock() {
            data.remove(key);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut data) = self.data.lock() {
            data.clear();
        }
    }

    pub fn length(&self) -> usize {
        self.data.lock().map(|d| d.len()).unwrap_or(0)
    }
}

/// Initialize all Web APIs in the JavaScript context
pub fn init_web_apis(context: &mut Context, current_url: &str) {
    init_window(context, current_url);
    init_document(context);
    init_timers(context);
    init_local_storage(context);
    init_navigator(context);
    init_location(context, current_url);
}

/// Initialize the window object
fn init_window(context: &mut Context, url: &str) {
    let url_clone = url.to_string();

    // Create alert function
    let alert_fn = NativeFunction::from_copy_closure(|_this, args, ctx| {
        let message = args.get_or_undefined(0).to_string(ctx)?;
        println!("[JS alert] {}", message.to_std_string_escaped());
        Ok(BoaJsValue::undefined())
    });

    // Create confirm function (always returns true for now)
    let confirm_fn = NativeFunction::from_copy_closure(|_this, args, ctx| {
        let message = args.get_or_undefined(0).to_string(ctx)?;
        println!("[JS confirm] {}", message.to_std_string_escaped());
        Ok(BoaJsValue::from(true))
    });

    // Create prompt function (returns null for now)
    let prompt_fn = NativeFunction::from_copy_closure(|_this, args, ctx| {
        let message = args.get_or_undefined(0).to_string(ctx)?;
        println!("[JS prompt] {}", message.to_std_string_escaped());
        Ok(BoaJsValue::null())
    });

    // Build window object
    let window = ObjectInitializer::new(context)
        .function(alert_fn, boa_engine::js_string!("alert"), 1)
        .function(confirm_fn, boa_engine::js_string!("confirm"), 1)
        .function(prompt_fn, boa_engine::js_string!("prompt"), 1)
        .property(
            boa_engine::js_string!("innerWidth"),
            BoaJsValue::from(1920),
            Attribute::READONLY,
        )
        .property(
            boa_engine::js_string!("innerHeight"),
            BoaJsValue::from(1080),
            Attribute::READONLY,
        )
        .property(
            boa_engine::js_string!("outerWidth"),
            BoaJsValue::from(1920),
            Attribute::READONLY,
        )
        .property(
            boa_engine::js_string!("outerHeight"),
            BoaJsValue::from(1080),
            Attribute::READONLY,
        )
        .build();

    context
        .register_global_property(boa_engine::js_string!("window"), window, Attribute::all())
        .expect("Failed to register window object");
}

/// Initialize a basic document object
fn init_document(context: &mut Context) {
    // document.getElementById - returns null for now
    let get_element_by_id = NativeFunction::from_copy_closure(|_this, args, ctx| {
        let _id = args.get_or_undefined(0).to_string(ctx)?;
        // Return null - real DOM binding would be needed
        Ok(BoaJsValue::null())
    });

    // document.querySelector - returns null for now
    let query_selector = NativeFunction::from_copy_closure(|_this, args, ctx| {
        let _selector = args.get_or_undefined(0).to_string(ctx)?;
        Ok(BoaJsValue::null())
    });

    // document.querySelectorAll - returns empty array
    let query_selector_all = NativeFunction::from_copy_closure(|_this, _args, ctx| {
        let array = boa_engine::object::builtins::JsArray::new(ctx);
        Ok(array.into())
    });

    // document.createElement - returns a basic object
    let create_element = NativeFunction::from_copy_closure(|_this, args, ctx| {
        let tag_name = args.get_or_undefined(0).to_string(ctx)?;
        let element = ObjectInitializer::new(ctx)
            .property(
                boa_engine::js_string!("tagName"),
                BoaJsValue::from(tag_name.clone()),
                Attribute::all(),
            )
            .property(
                boa_engine::js_string!("innerHTML"),
                BoaJsValue::from(boa_engine::js_string!("")),
                Attribute::all(),
            )
            .property(
                boa_engine::js_string!("textContent"),
                BoaJsValue::from(boa_engine::js_string!("")),
                Attribute::all(),
            )
            .build();
        Ok(element.into())
    });

    // document.createTextNode
    let create_text_node = NativeFunction::from_copy_closure(|_this, args, ctx| {
        let text = args.get_or_undefined(0).to_string(ctx)?;
        let node = ObjectInitializer::new(ctx)
            .property(
                boa_engine::js_string!("nodeType"),
                BoaJsValue::from(3), // TEXT_NODE
                Attribute::READONLY,
            )
            .property(
                boa_engine::js_string!("textContent"),
                BoaJsValue::from(text),
                Attribute::all(),
            )
            .build();
        Ok(node.into())
    });

    // Build basic document object first
    let document = ObjectInitializer::new(context)
        .function(get_element_by_id, boa_engine::js_string!("getElementById"), 1)
        .function(query_selector, boa_engine::js_string!("querySelector"), 1)
        .function(query_selector_all, boa_engine::js_string!("querySelectorAll"), 1)
        .function(create_element, boa_engine::js_string!("createElement"), 1)
        .function(create_text_node, boa_engine::js_string!("createTextNode"), 1)
        .property(boa_engine::js_string!("readyState"), BoaJsValue::from(boa_engine::js_string!("complete")), Attribute::READONLY)
        .property(boa_engine::js_string!("cookie"), BoaJsValue::from(boa_engine::js_string!("")), Attribute::all())
        .property(boa_engine::js_string!("title"), BoaJsValue::from(boa_engine::js_string!("")), Attribute::all())
        .property(boa_engine::js_string!("domain"), BoaJsValue::from(boa_engine::js_string!("")), Attribute::all())
        .property(boa_engine::js_string!("URL"), BoaJsValue::from(boa_engine::js_string!("")), Attribute::all())
        .property(boa_engine::js_string!("referrer"), BoaJsValue::from(boa_engine::js_string!("")), Attribute::all())
        .build();

    context
        .register_global_property(boa_engine::js_string!("document"), document, Attribute::all())
        .expect("Failed to register document object");

    // Add complete DOM structure via JavaScript for better compatibility
    let dom_js = r#"
        // Create Element prototype methods
        function _createElementMethods(el) {
            el.addEventListener = function() {};
            el.removeEventListener = function() {};
            el.dispatchEvent = function() { return true; };
            el.appendChild = function(child) { return child; };
            el.removeChild = function(child) { return child; };
            el.insertBefore = function(node, ref) { return node; };
            el.replaceChild = function(newChild, oldChild) { return oldChild; };
            el.cloneNode = function(deep) { return {}; };
            el.getAttribute = function(name) { return null; };
            el.setAttribute = function(name, value) {};
            el.removeAttribute = function(name) {};
            el.hasAttribute = function(name) { return false; };
            el.getBoundingClientRect = function() {
                return { x: 0, y: 0, width: 0, height: 0, top: 0, right: 0, bottom: 0, left: 0 };
            };
            el.getElementsByTagName = function() { return []; };
            el.getElementsByClassName = function() { return []; };
            el.querySelector = function() { return null; };
            el.querySelectorAll = function() { return []; };
            return el;
        }

        // Create body element
        document.body = _createElementMethods({
            tagName: "BODY",
            nodeName: "BODY",
            nodeType: 1,
            innerHTML: "",
            textContent: "",
            className: "",
            id: "",
            style: {},
            children: [],
            childNodes: [],
            parentNode: null,
            parentElement: null,
            firstChild: null,
            lastChild: null,
            previousSibling: null,
            nextSibling: null,
            scrollTop: 0,
            scrollLeft: 0,
            scrollWidth: 0,
            scrollHeight: 0,
            clientWidth: 1920,
            clientHeight: 1080,
            offsetWidth: 1920,
            offsetHeight: 1080,
            offsetTop: 0,
            offsetLeft: 0
        });

        // Create head element
        document.head = _createElementMethods({
            tagName: "HEAD",
            nodeName: "HEAD",
            nodeType: 1,
            children: [],
            childNodes: []
        });

        // Create documentElement (html)
        document.documentElement = _createElementMethods({
            tagName: "HTML",
            nodeName: "HTML",
            nodeType: 1,
            scrollTop: 0,
            scrollLeft: 0,
            clientWidth: 1920,
            clientHeight: 1080
        });

        // Add additional document methods
        document.getElementsByTagName = function(tag) { return []; };
        document.getElementsByClassName = function(cls) { return []; };
        document.getElementsByName = function(name) { return []; };
        document.createDocumentFragment = function() {
            return _createElementMethods({ nodeType: 11, children: [], childNodes: [] });
        };
        document.createEvent = function(type) {
            return {
                type: "",
                initEvent: function() {},
                preventDefault: function() {},
                stopPropagation: function() {},
                stopImmediatePropagation: function() {}
            };
        };
        document.createComment = function(data) {
            return { nodeType: 8, data: data };
        };
        document.addEventListener = function() {};
        document.removeEventListener = function() {};
        document.dispatchEvent = function() { return true; };
        document.hasFocus = function() { return true; };
        document.activeElement = document.body;
    "#;

    if let Err(e) = context.eval(boa_engine::Source::from_bytes(dom_js)) {
        eprintln!("Failed to initialize DOM: {:?}", e);
    }
}

/// Initialize setTimeout and setInterval
fn init_timers(context: &mut Context) {
    // setTimeout - stores code but can't execute asynchronously yet
    let set_timeout = NativeFunction::from_copy_closure(|_this, args, ctx| {
        let callback = args.get_or_undefined(0);
        let delay = args.get_or_undefined(1).to_u32(ctx).unwrap_or(0);

        // Log the timeout registration
        if callback.is_callable() {
            log::debug!("[JS] setTimeout registered with delay {}ms", delay);
        } else if let Some(code) = callback.as_string() {
            log::debug!("[JS] setTimeout('{}', {})", code.to_std_string_escaped(), delay);
        }

        // Return a timer ID (we can't actually execute async yet)
        static TIMER_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
        let id = TIMER_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(BoaJsValue::from(id))
    });

    // clearTimeout
    let clear_timeout = NativeFunction::from_copy_closure(|_this, args, ctx| {
        let _id = args.get_or_undefined(0).to_u32(ctx).unwrap_or(0);
        Ok(BoaJsValue::undefined())
    });

    // setInterval
    let set_interval = NativeFunction::from_copy_closure(|_this, args, ctx| {
        let callback = args.get_or_undefined(0);
        let delay = args.get_or_undefined(1).to_u32(ctx).unwrap_or(0);

        if callback.is_callable() {
            log::debug!("[JS] setInterval registered with delay {}ms", delay);
        }

        static TIMER_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1000);
        let id = TIMER_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(BoaJsValue::from(id))
    });

    // clearInterval
    let clear_interval = NativeFunction::from_copy_closure(|_this, args, ctx| {
        let _id = args.get_or_undefined(0).to_u32(ctx).unwrap_or(0);
        Ok(BoaJsValue::undefined())
    });

    // requestAnimationFrame - returns a frame ID
    let request_animation_frame = NativeFunction::from_copy_closure(|_this, _args, _ctx| {
        static FRAME_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
        let id = FRAME_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(BoaJsValue::from(id))
    });

    // cancelAnimationFrame
    let cancel_animation_frame = NativeFunction::from_copy_closure(|_this, _args, _ctx| {
        Ok(BoaJsValue::undefined())
    });

    context
        .register_global_builtin_callable(boa_engine::js_string!("setTimeout"), 2, set_timeout)
        .expect("Failed to register setTimeout");
    context
        .register_global_builtin_callable(boa_engine::js_string!("clearTimeout"), 1, clear_timeout)
        .expect("Failed to register clearTimeout");
    context
        .register_global_builtin_callable(boa_engine::js_string!("setInterval"), 2, set_interval)
        .expect("Failed to register setInterval");
    context
        .register_global_builtin_callable(boa_engine::js_string!("clearInterval"), 1, clear_interval)
        .expect("Failed to register clearInterval");
    context
        .register_global_builtin_callable(boa_engine::js_string!("requestAnimationFrame"), 1, request_animation_frame)
        .expect("Failed to register requestAnimationFrame");
    context
        .register_global_builtin_callable(boa_engine::js_string!("cancelAnimationFrame"), 1, cancel_animation_frame)
        .expect("Failed to register cancelAnimationFrame");
}


/// Initialize localStorage
fn init_local_storage(context: &mut Context) {
    // getItem
    let get_item = NativeFunction::from_copy_closure(|_this, args, ctx| {
        let _key = args.get_or_undefined(0).to_string(ctx)?;
        // Return null - real storage would need persistence
        Ok(BoaJsValue::null())
    });

    // setItem
    let set_item = NativeFunction::from_copy_closure(|_this, args, ctx| {
        let key = args.get_or_undefined(0).to_string(ctx)?;
        let value = args.get_or_undefined(1).to_string(ctx)?;
        log::debug!("[JS] localStorage.setItem('{}', '{}')",
            key.to_std_string_escaped(),
            value.to_std_string_escaped());
        Ok(BoaJsValue::undefined())
    });

    // removeItem
    let remove_item = NativeFunction::from_copy_closure(|_this, args, ctx| {
        let _key = args.get_or_undefined(0).to_string(ctx)?;
        Ok(BoaJsValue::undefined())
    });

    // clear
    let clear = NativeFunction::from_copy_closure(|_this, _args, _ctx| {
        Ok(BoaJsValue::undefined())
    });

    let local_storage = ObjectInitializer::new(context)
        .function(get_item, boa_engine::js_string!("getItem"), 1)
        .function(set_item, boa_engine::js_string!("setItem"), 2)
        .function(remove_item, boa_engine::js_string!("removeItem"), 1)
        .function(clear, boa_engine::js_string!("clear"), 0)
        .property(
            boa_engine::js_string!("length"),
            BoaJsValue::from(0),
            Attribute::READONLY,
        )
        .build();

    context
        .register_global_property(boa_engine::js_string!("localStorage"), local_storage.clone(), Attribute::all())
        .expect("Failed to register localStorage");

    // Also register sessionStorage with same implementation
    context
        .register_global_property(boa_engine::js_string!("sessionStorage"), local_storage, Attribute::all())
        .expect("Failed to register sessionStorage");
}

/// Initialize navigator object
fn init_navigator(context: &mut Context) {
    let navigator = ObjectInitializer::new(context)
        .property(
            boa_engine::js_string!("userAgent"),
            BoaJsValue::from(boa_engine::js_string!("Binix/0.1.0 (Linux; Rust)")),
            Attribute::READONLY,
        )
        .property(
            boa_engine::js_string!("appName"),
            BoaJsValue::from(boa_engine::js_string!("Binix")),
            Attribute::READONLY,
        )
        .property(
            boa_engine::js_string!("appVersion"),
            BoaJsValue::from(boa_engine::js_string!("0.1.0")),
            Attribute::READONLY,
        )
        .property(
            boa_engine::js_string!("platform"),
            BoaJsValue::from(boa_engine::js_string!("Linux")),
            Attribute::READONLY,
        )
        .property(
            boa_engine::js_string!("language"),
            BoaJsValue::from(boa_engine::js_string!("en-US")),
            Attribute::READONLY,
        )
        .property(
            boa_engine::js_string!("languages"),
            BoaJsValue::from(boa_engine::js_string!("en-US,en")),
            Attribute::READONLY,
        )
        .property(
            boa_engine::js_string!("onLine"),
            BoaJsValue::from(true),
            Attribute::READONLY,
        )
        .property(
            boa_engine::js_string!("cookieEnabled"),
            BoaJsValue::from(true),
            Attribute::READONLY,
        )
        .build();

    context
        .register_global_property(boa_engine::js_string!("navigator"), navigator, Attribute::all())
        .expect("Failed to register navigator");
}

/// Initialize location object
fn init_location(context: &mut Context, url: &str) {
    // Parse the URL
    let (protocol, host, pathname, search, hash) = parse_url(url);

    let location = ObjectInitializer::new(context)
        .property(
            boa_engine::js_string!("href"),
            BoaJsValue::from(boa_engine::js_string!(url)),
            Attribute::all(),
        )
        .property(
            boa_engine::js_string!("protocol"),
            BoaJsValue::from(boa_engine::js_string!(protocol.as_str())),
            Attribute::all(),
        )
        .property(
            boa_engine::js_string!("host"),
            BoaJsValue::from(boa_engine::js_string!(host.as_str())),
            Attribute::all(),
        )
        .property(
            boa_engine::js_string!("hostname"),
            BoaJsValue::from(boa_engine::js_string!(host.as_str())),
            Attribute::all(),
        )
        .property(
            boa_engine::js_string!("pathname"),
            BoaJsValue::from(boa_engine::js_string!(pathname.as_str())),
            Attribute::all(),
        )
        .property(
            boa_engine::js_string!("search"),
            BoaJsValue::from(boa_engine::js_string!(search.as_str())),
            Attribute::all(),
        )
        .property(
            boa_engine::js_string!("hash"),
            BoaJsValue::from(boa_engine::js_string!(hash.as_str())),
            Attribute::all(),
        )
        .property(
            boa_engine::js_string!("origin"),
            BoaJsValue::from(boa_engine::js_string!(format!("{}://{}", protocol, host).as_str())),
            Attribute::READONLY,
        )
        .build();

    context
        .register_global_property(boa_engine::js_string!("location"), location, Attribute::all())
        .expect("Failed to register location");
}

/// Parse URL into components
fn parse_url(url: &str) -> (String, String, String, String, String) {
    let mut protocol = "https".to_string();
    let mut host = "".to_string();
    let mut pathname = "/".to_string();
    let mut search = "".to_string();
    let mut hash = "".to_string();

    let mut remaining = url;

    // Extract protocol
    if let Some(idx) = remaining.find("://") {
        protocol = remaining[..idx].to_string();
        remaining = &remaining[idx + 3..];
    }

    // Extract hash
    if let Some(idx) = remaining.find('#') {
        hash = remaining[idx..].to_string();
        remaining = &remaining[..idx];
    }

    // Extract search
    if let Some(idx) = remaining.find('?') {
        search = remaining[idx..].to_string();
        remaining = &remaining[..idx];
    }

    // Extract host and pathname
    if let Some(idx) = remaining.find('/') {
        host = remaining[..idx].to_string();
        pathname = remaining[idx..].to_string();
    } else {
        host = remaining.to_string();
    }

    (protocol, host, pathname, search, hash)
}

