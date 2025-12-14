//! Advanced Web APIs for better site compatibility
//!
//! Implements:
//! - Image constructor
//! - MutationObserver (stub)
//! - performance API
//! - history API
//! - Event/CustomEvent
//! - DOMParser
//! - atob/btoa (base64)

use boa_engine::{
    Context, JsValue as BoaJsValue, NativeFunction, Source,
    object::ObjectInitializer,
    property::Attribute,
    JsArgs,
    js_string,
};

/// Initialize all advanced Web APIs
pub fn init_advanced_apis(context: &mut Context) {
    init_image_constructor(context);
    init_mutation_observer(context);
    init_performance(context);
    init_history(context);
    init_event_classes(context);
    init_base64(context);
    init_misc_globals(context);
}

/// Initialize Image constructor
fn init_image_constructor(context: &mut Context) {
    // Create Image constructor as a proper class
    // Using register_global_class-like pattern for constructor behavior
    let js_code = r#"
        function Image(width, height) {
            this.width = width || 0;
            this.height = height || 0;
            this.src = "";
            this.alt = "";
            this.complete = false;
            this.naturalWidth = 0;
            this.naturalHeight = 0;
            this.onload = null;
            this.onerror = null;
        }
        Image.prototype.toString = function() { return "[object HTMLImageElement]"; };
    "#;

    if let Err(e) = context.eval(Source::from_bytes(js_code)) {
        eprintln!("Failed to initialize Image: {:?}", e);
    }
}

/// Initialize MutationObserver (stub implementation)
fn init_mutation_observer(context: &mut Context) {
    // Define MutationObserver as a JavaScript class
    let js_code = r#"
        function MutationObserver(callback) {
            this._callback = callback;
        }
        MutationObserver.prototype.observe = function(target, options) {};
        MutationObserver.prototype.disconnect = function() {};
        MutationObserver.prototype.takeRecords = function() { return []; };

        // WebKit prefix alias
        var WebKitMutationObserver = MutationObserver;
    "#;

    if let Err(e) = context.eval(Source::from_bytes(js_code)) {
        eprintln!("Failed to initialize MutationObserver: {:?}", e);
    }
}

/// Initialize performance API
fn init_performance(context: &mut Context) {
    use std::time::Instant;

    // Store the start time
    let start_time = Instant::now();

    // performance.now()
    let now_fn = NativeFunction::from_copy_closure(move |_this, _args, _ctx| {
        let elapsed = start_time.elapsed().as_secs_f64() * 1000.0;
        Ok(BoaJsValue::from(elapsed))
    });

    let performance = ObjectInitializer::new(context)
        .function(now_fn, js_string!("now"), 0)
        .property(js_string!("timeOrigin"), BoaJsValue::from(0.0), Attribute::READONLY)
        .build();

    context
        .register_global_property(js_string!("performance"), performance, Attribute::all())
        .expect("Failed to register performance");
}

/// Initialize history API
fn init_history(context: &mut Context) {
    let push_state = NativeFunction::from_copy_closure(|_this, _args, _ctx| {
        Ok(BoaJsValue::undefined())
    });

    let replace_state = NativeFunction::from_copy_closure(|_this, _args, _ctx| {
        Ok(BoaJsValue::undefined())
    });

    let go_fn = NativeFunction::from_copy_closure(|_this, _args, _ctx| {
        Ok(BoaJsValue::undefined())
    });

    let back_fn = NativeFunction::from_copy_closure(|_this, _args, _ctx| {
        Ok(BoaJsValue::undefined())
    });

    let forward_fn = NativeFunction::from_copy_closure(|_this, _args, _ctx| {
        Ok(BoaJsValue::undefined())
    });

    let history = ObjectInitializer::new(context)
        .function(push_state, js_string!("pushState"), 3)
        .function(replace_state, js_string!("replaceState"), 3)
        .function(go_fn, js_string!("go"), 1)
        .function(back_fn, js_string!("back"), 0)
        .function(forward_fn, js_string!("forward"), 0)
        .property(js_string!("length"), BoaJsValue::from(1), Attribute::READONLY)
        .property(js_string!("state"), BoaJsValue::null(), Attribute::all())
        .build();

    context
        .register_global_property(js_string!("history"), history, Attribute::all())
        .expect("Failed to register history");
}

/// Initialize Event and CustomEvent classes
fn init_event_classes(context: &mut Context) {
    // Define Event classes as JavaScript constructors for proper `new` support
    let js_code = r#"
        // Event constructor
        function Event(type, eventInitDict) {
            this.type = type;
            this.bubbles = (eventInitDict && eventInitDict.bubbles) || false;
            this.cancelable = (eventInitDict && eventInitDict.cancelable) || false;
            this.composed = (eventInitDict && eventInitDict.composed) || false;
            this.defaultPrevented = false;
            this.target = null;
            this.currentTarget = null;
            this.eventPhase = 0;
            this.timeStamp = Date.now();
            this.isTrusted = false;
        }
        Event.prototype.preventDefault = function() { this.defaultPrevented = true; };
        Event.prototype.stopPropagation = function() {};
        Event.prototype.stopImmediatePropagation = function() {};
        Event.prototype.composedPath = function() { return []; };
        Event.AT_TARGET = 2;
        Event.BUBBLING_PHASE = 3;
        Event.CAPTURING_PHASE = 1;
        Event.NONE = 0;

        // CustomEvent constructor
        function CustomEvent(type, eventInitDict) {
            Event.call(this, type, eventInitDict);
            this.detail = (eventInitDict && eventInitDict.detail) || null;
        }
        CustomEvent.prototype = Object.create(Event.prototype);
        CustomEvent.prototype.constructor = CustomEvent;

        // KeyboardEvent
        function KeyboardEvent(type, eventInitDict) {
            Event.call(this, type, eventInitDict);
            this.key = (eventInitDict && eventInitDict.key) || '';
            this.code = (eventInitDict && eventInitDict.code) || '';
            this.ctrlKey = (eventInitDict && eventInitDict.ctrlKey) || false;
            this.shiftKey = (eventInitDict && eventInitDict.shiftKey) || false;
            this.altKey = (eventInitDict && eventInitDict.altKey) || false;
            this.metaKey = (eventInitDict && eventInitDict.metaKey) || false;
            this.repeat = (eventInitDict && eventInitDict.repeat) || false;
        }
        KeyboardEvent.prototype = Object.create(Event.prototype);
        KeyboardEvent.prototype.constructor = KeyboardEvent;

        // MouseEvent
        function MouseEvent(type, eventInitDict) {
            Event.call(this, type, eventInitDict);
            this.clientX = (eventInitDict && eventInitDict.clientX) || 0;
            this.clientY = (eventInitDict && eventInitDict.clientY) || 0;
            this.screenX = (eventInitDict && eventInitDict.screenX) || 0;
            this.screenY = (eventInitDict && eventInitDict.screenY) || 0;
            this.button = (eventInitDict && eventInitDict.button) || 0;
            this.buttons = (eventInitDict && eventInitDict.buttons) || 0;
            this.ctrlKey = (eventInitDict && eventInitDict.ctrlKey) || false;
            this.shiftKey = (eventInitDict && eventInitDict.shiftKey) || false;
            this.altKey = (eventInitDict && eventInitDict.altKey) || false;
            this.metaKey = (eventInitDict && eventInitDict.metaKey) || false;
        }
        MouseEvent.prototype = Object.create(Event.prototype);
        MouseEvent.prototype.constructor = MouseEvent;

        // FocusEvent
        function FocusEvent(type, eventInitDict) {
            Event.call(this, type, eventInitDict);
            this.relatedTarget = (eventInitDict && eventInitDict.relatedTarget) || null;
        }
        FocusEvent.prototype = Object.create(Event.prototype);
        FocusEvent.prototype.constructor = FocusEvent;

        // ErrorEvent
        function ErrorEvent(type, eventInitDict) {
            Event.call(this, type, eventInitDict);
            this.message = (eventInitDict && eventInitDict.message) || '';
            this.filename = (eventInitDict && eventInitDict.filename) || '';
            this.lineno = (eventInitDict && eventInitDict.lineno) || 0;
            this.colno = (eventInitDict && eventInitDict.colno) || 0;
            this.error = (eventInitDict && eventInitDict.error) || null;
        }
        ErrorEvent.prototype = Object.create(Event.prototype);
        ErrorEvent.prototype.constructor = ErrorEvent;

        // ProgressEvent
        function ProgressEvent(type, eventInitDict) {
            Event.call(this, type, eventInitDict);
            this.lengthComputable = (eventInitDict && eventInitDict.lengthComputable) || false;
            this.loaded = (eventInitDict && eventInitDict.loaded) || 0;
            this.total = (eventInitDict && eventInitDict.total) || 0;
        }
        ProgressEvent.prototype = Object.create(Event.prototype);
        ProgressEvent.prototype.constructor = ProgressEvent;

        // MessageEvent
        function MessageEvent(type, eventInitDict) {
            Event.call(this, type, eventInitDict);
            this.data = (eventInitDict && eventInitDict.data) || null;
            this.origin = (eventInitDict && eventInitDict.origin) || '';
            this.source = (eventInitDict && eventInitDict.source) || null;
        }
        MessageEvent.prototype = Object.create(Event.prototype);
        MessageEvent.prototype.constructor = MessageEvent;
    "#;

    if let Err(e) = context.eval(Source::from_bytes(js_code)) {
        eprintln!("Failed to initialize Event classes: {:?}", e);
    }
}


/// Initialize base64 encoding/decoding (atob/btoa)
fn init_base64(context: &mut Context) {
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    // btoa - encode string to base64
    let btoa_fn = NativeFunction::from_copy_closure(|_this, args, ctx| {
        let input = args.get_or_undefined(0).to_string(ctx)?;
        let input_str = input.to_std_string_escaped();
        let encoded = STANDARD.encode(input_str.as_bytes());
        Ok(BoaJsValue::from(js_string!(encoded.as_str())))
    });

    // atob - decode base64 to string
    let atob_fn = NativeFunction::from_copy_closure(|_this, args, ctx| {
        let input = args.get_or_undefined(0).to_string(ctx)?;
        let input_str = input.to_std_string_escaped();
        match STANDARD.decode(&input_str) {
            Ok(decoded) => {
                let s = String::from_utf8_lossy(&decoded).to_string();
                Ok(BoaJsValue::from(js_string!(s.as_str())))
            }
            Err(_) => Ok(BoaJsValue::from(js_string!("")))
        }
    });

    context
        .register_global_builtin_callable(js_string!("btoa"), 1, btoa_fn)
        .expect("Failed to register btoa");
    context
        .register_global_builtin_callable(js_string!("atob"), 1, atob_fn)
        .expect("Failed to register atob");
}

/// Initialize miscellaneous globals
fn init_misc_globals(context: &mut Context) {
    // Define all observers and classes as JavaScript constructors
    let js_classes = r#"
        // ResizeObserver
        function ResizeObserver(callback) {
            this._callback = callback;
        }
        ResizeObserver.prototype.observe = function(target, options) {};
        ResizeObserver.prototype.unobserve = function(target) {};
        ResizeObserver.prototype.disconnect = function() {};

        // IntersectionObserver
        function IntersectionObserver(callback, options) {
            this._callback = callback;
            this._options = options || {};
        }
        IntersectionObserver.prototype.observe = function(target) {};
        IntersectionObserver.prototype.unobserve = function(target) {};
        IntersectionObserver.prototype.disconnect = function() {};
        IntersectionObserver.prototype.takeRecords = function() { return []; };

        // PerformanceObserver
        function PerformanceObserver(callback) {
            this._callback = callback;
        }
        PerformanceObserver.prototype.observe = function(options) {};
        PerformanceObserver.prototype.disconnect = function() {};
        PerformanceObserver.prototype.takeRecords = function() { return []; };

        // ReportingObserver
        function ReportingObserver(callback, options) {
            this._callback = callback;
        }
        ReportingObserver.prototype.observe = function() {};
        ReportingObserver.prototype.disconnect = function() {};
        ReportingObserver.prototype.takeRecords = function() { return []; };

        // AbortController
        function AbortController() {
            this.signal = { aborted: false, reason: undefined };
        }
        AbortController.prototype.abort = function(reason) {
            this.signal.aborted = true;
            this.signal.reason = reason;
        };

        // AbortSignal
        function AbortSignal() {
            this.aborted = false;
            this.reason = undefined;
        }
        AbortSignal.timeout = function(ms) { return new AbortSignal(); };

        // DOMParser
        function DOMParser() {}
        DOMParser.prototype.parseFromString = function(str, type) {
            return { documentElement: null };
        };

        // XMLSerializer
        function XMLSerializer() {}
        XMLSerializer.prototype.serializeToString = function(node) { return ""; };

        // Worker (stub)
        function Worker(url) {
            this._url = url;
            this.onmessage = null;
            this.onerror = null;
        }
        Worker.prototype.postMessage = function(msg) {};
        Worker.prototype.terminate = function() {};

        // MessageChannel
        function MessageChannel() {
            this.port1 = { postMessage: function() {}, onmessage: null };
            this.port2 = { postMessage: function() {}, onmessage: null };
        }

        // Blob
        function Blob(parts, options) {
            this.size = 0;
            this.type = (options && options.type) || "";
        }
        Blob.prototype.text = function() { return Promise.resolve(""); };
        Blob.prototype.arrayBuffer = function() { return Promise.resolve(new ArrayBuffer(0)); };
        Blob.prototype.slice = function() { return new Blob(); };

        // File
        function File(parts, name, options) {
            this.name = name;
            this.size = 0;
            this.type = (options && options.type) || "";
            this.lastModified = Date.now();
        }

        // FileReader
        function FileReader() {
            this.result = null;
            this.readyState = 0;
            this.onload = null;
            this.onerror = null;
        }
        FileReader.prototype.readAsText = function(blob) {};
        FileReader.prototype.readAsDataURL = function(blob) {};
        FileReader.prototype.readAsArrayBuffer = function(blob) {};
        FileReader.prototype.abort = function() {};

        // FormData
        function FormData(form) {
            this._data = {};
        }
        FormData.prototype.append = function(key, value) { this._data[key] = value; };
        FormData.prototype.get = function(key) { return this._data[key] || null; };
        FormData.prototype.set = function(key, value) { this._data[key] = value; };
        FormData.prototype.delete = function(key) { delete this._data[key]; };
        FormData.prototype.has = function(key) { return key in this._data; };

        // URL constructor
        function URL(url, base) {
            var fullUrl = url;
            if (base) {
                // Simple base URL handling
                if (!url.match(/^[a-z]+:/i)) {
                    fullUrl = base.replace(/\/[^\/]*$/, '/') + url;
                }
            }
            this.href = fullUrl;

            // Parse URL
            var match = fullUrl.match(/^([a-z]+):\/\/([^\/\?#]+)(\/[^\?#]*)?(\?[^#]*)?(#.*)?$/i);
            if (match) {
                this.protocol = (match[1] || 'https') + ':';
                this.host = match[2] || '';
                this.hostname = this.host.split(':')[0];
                this.port = this.host.split(':')[1] || '';
                this.pathname = match[3] || '/';
                this.search = match[4] || '';
                this.hash = match[5] || '';
            } else {
                this.protocol = '';
                this.host = '';
                this.hostname = '';
                this.port = '';
                this.pathname = fullUrl;
                this.search = '';
                this.hash = '';
            }
            this.origin = this.protocol + '//' + this.host;
            this.searchParams = new URLSearchParams(this.search.slice(1));
        }
        URL.prototype.toString = function() { return this.href; };
        URL.createObjectURL = function(obj) { return 'blob:' + Math.random(); };
        URL.revokeObjectURL = function(url) {};

        // URLSearchParams
        function URLSearchParams(init) {
            this._params = {};
            if (typeof init === 'string') {
                var pairs = init.split('&');
                for (var i = 0; i < pairs.length; i++) {
                    var pair = pairs[i].split('=');
                    if (pair[0]) {
                        this._params[decodeURIComponent(pair[0])] = decodeURIComponent(pair[1] || '');
                    }
                }
            }
        }
        URLSearchParams.prototype.get = function(key) { return this._params[key] || null; };
        URLSearchParams.prototype.set = function(key, value) { this._params[key] = value; };
        URLSearchParams.prototype.has = function(key) { return key in this._params; };
        URLSearchParams.prototype.delete = function(key) { delete this._params[key]; };
        URLSearchParams.prototype.append = function(key, value) { this._params[key] = value; };
        URLSearchParams.prototype.toString = function() {
            var parts = [];
            for (var key in this._params) {
                parts.push(encodeURIComponent(key) + '=' + encodeURIComponent(this._params[key]));
            }
            return parts.join('&');
        };

        // TextEncoder
        function TextEncoder() {
            this.encoding = 'utf-8';
        }
        TextEncoder.prototype.encode = function(str) {
            var arr = [];
            for (var i = 0; i < str.length; i++) {
                arr.push(str.charCodeAt(i));
            }
            return new Uint8Array(arr);
        };

        // TextDecoder
        function TextDecoder(encoding) {
            this.encoding = encoding || 'utf-8';
        }
        TextDecoder.prototype.decode = function(arr) {
            if (!arr) return '';
            var str = '';
            for (var i = 0; i < arr.length; i++) {
                str += String.fromCharCode(arr[i]);
            }
            return str;
        };

        // Headers class for fetch
        function Headers(init) {
            this._headers = {};
            if (init) {
                for (var key in init) {
                    this._headers[key.toLowerCase()] = init[key];
                }
            }
        }
        Headers.prototype.get = function(name) { return this._headers[name.toLowerCase()] || null; };
        Headers.prototype.set = function(name, value) { this._headers[name.toLowerCase()] = value; };
        Headers.prototype.has = function(name) { return name.toLowerCase() in this._headers; };
        Headers.prototype.delete = function(name) { delete this._headers[name.toLowerCase()]; };
        Headers.prototype.append = function(name, value) { this._headers[name.toLowerCase()] = value; };

        // Request class for fetch
        function Request(input, init) {
            this.url = typeof input === 'string' ? input : input.url;
            this.method = (init && init.method) || 'GET';
            this.headers = new Headers((init && init.headers) || {});
            this.body = (init && init.body) || null;
            this.mode = (init && init.mode) || 'cors';
            this.credentials = (init && init.credentials) || 'same-origin';
        }
        Request.prototype.clone = function() { return new Request(this.url, this); };

        // Response class for fetch
        function Response(body, init) {
            this.body = body;
            this.status = (init && init.status) || 200;
            this.statusText = (init && init.statusText) || 'OK';
            this.ok = this.status >= 200 && this.status < 300;
            this.headers = new Headers((init && init.headers) || {});
            this.url = '';
        }
        Response.prototype.text = function() { return Promise.resolve(this.body || ''); };
        Response.prototype.json = function() {
            var self = this;
            return Promise.resolve().then(function() { return JSON.parse(self.body || '{}'); });
        };
        Response.prototype.clone = function() { return new Response(this.body, this); };
        Response.error = function() { return new Response(null, { status: 0, statusText: '' }); };
        Response.redirect = function(url, status) { return new Response(null, { status: status || 302 }); };
    "#;

    if let Err(e) = context.eval(Source::from_bytes(js_classes)) {
        eprintln!("Failed to initialize misc globals: {:?}", e);
    }

    // NOTE: URL, URLSearchParams, TextEncoder, TextDecoder are defined in the JS classes above

    // Crypto.getRandomValues (basic)
    let get_random_values = NativeFunction::from_copy_closure(|_this, args, _ctx| {
        // Just return the input array unchanged for now
        Ok(args.get_or_undefined(0).clone())
    });

    let crypto = ObjectInitializer::new(context)
        .function(get_random_values, js_string!("getRandomValues"), 1)
        .build();

    context
        .register_global_property(js_string!("crypto"), crypto, Attribute::all())
        .expect("Failed to register crypto");
}



