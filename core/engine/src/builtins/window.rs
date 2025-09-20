//! Window Web API implementation for Boa
//!
//! Native implementation of Window standard
//! https://html.spec.whatwg.org/#the-window-object

use crate::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::{Attribute, PropertyDescriptorBuilder}
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// JavaScript `Window` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Window;

impl IntrinsicObject for Window {
    fn init(realm: &Realm) {
        let location_func = BuiltInBuilder::callable(realm, get_location)
            .name(js_string!("get location"))
            .build();

        let history_func = BuiltInBuilder::callable(realm, get_history)
            .name(js_string!("get history"))
            .build();

        let document_func = BuiltInBuilder::callable(realm, get_document)
            .name(js_string!("get document"))
            .build();

        let navigator_func = BuiltInBuilder::callable(realm, get_navigator)
            .name(js_string!("get navigator"))
            .build();

        let screen_func = BuiltInBuilder::callable(realm, get_screen)
            .name(js_string!("get screen"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("location"),
                Some(location_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("history"),
                Some(history_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("document"),
                Some(document_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("navigator"),
                Some(navigator_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("screen"),
                Some(screen_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(add_event_listener, js_string!("addEventListener"), 2)
            .method(remove_event_listener, js_string!("removeEventListener"), 2)
            .method(dispatch_event, js_string!("dispatchEvent"), 1)
            .method(match_media, js_string!("matchMedia"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Window {
    const NAME: JsString = StaticJsStrings::WINDOW;
}

impl BuiltInConstructor for Window {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::window;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::window,
            context,
        )?;

        let window_data = WindowData::new();

        let window = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            window_data,
        );

        Ok(window.into())
    }
}

/// Internal data for Window objects
#[derive(Debug, Trace, Finalize, JsData)]
pub struct WindowData {
    #[unsafe_ignore_trace]
    location: Arc<Mutex<JsObject>>,
    #[unsafe_ignore_trace]
    history: Arc<Mutex<JsObject>>,
    #[unsafe_ignore_trace]
    document: Arc<Mutex<JsObject>>,
    #[unsafe_ignore_trace]
    navigator: Arc<Mutex<JsObject>>,
    #[unsafe_ignore_trace]
    event_listeners: Arc<Mutex<HashMap<String, Vec<JsValue>>>>,
    #[unsafe_ignore_trace]
    current_url: Arc<Mutex<String>>,
}

impl WindowData {
    fn new() -> Self {
        Self {
            location: Arc::new(Mutex::new(JsObject::default())),
            history: Arc::new(Mutex::new(JsObject::default())),
            document: Arc::new(Mutex::new(JsObject::default())),
            navigator: Arc::new(Mutex::new(JsObject::default())),
            event_listeners: Arc::new(Mutex::new(HashMap::new())),
            current_url: Arc::new(Mutex::new("about:blank".to_string())),
        }
    }

    pub fn set_location(&self, location: JsObject) {
        *self.location.lock().unwrap() = location;
    }

    pub fn set_history(&self, history: JsObject) {
        *self.history.lock().unwrap() = history;
    }

    pub fn set_document(&self, document: JsObject) {
        *self.document.lock().unwrap() = document;
    }

    pub fn set_navigator(&self, navigator: JsObject) {
        *self.navigator.lock().unwrap() = navigator;
    }

    pub fn get_location(&self) -> JsObject {
        self.location.lock().unwrap().clone()
    }

    pub fn get_history(&self) -> JsObject {
        self.history.lock().unwrap().clone()
    }

    pub fn get_document(&self) -> JsObject {
        self.document.lock().unwrap().clone()
    }

    pub fn get_navigator(&self) -> JsObject {
        self.navigator.lock().unwrap().clone()
    }

    pub fn add_event_listener(&self, event_type: String, listener: JsValue) {
        self.event_listeners.lock().unwrap()
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(listener);
    }

    pub fn remove_event_listener(&self, event_type: &str, listener: &JsValue) {
        if let Some(listeners) = self.event_listeners.lock().unwrap().get_mut(event_type) {
            listeners.retain(|l| !JsValue::same_value(l, listener));
        }
    }

    pub fn get_event_listeners(&self, event_type: &str) -> Vec<JsValue> {
        self.event_listeners.lock().unwrap()
            .get(event_type)
            .cloned()
            .unwrap_or_default()
    }

    pub fn set_current_url(&self, url: String) {
        *self.current_url.lock().unwrap() = url;
    }

    pub fn get_current_url(&self) -> String {
        self.current_url.lock().unwrap().clone()
    }
}

/// `Window.prototype.location` getter
fn get_location(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Window.prototype.location called on non-object")
    })?;

    if let Some(window) = this_obj.downcast_ref::<WindowData>() {
        let location = window.get_location();

        // Initialize location object if empty
        if !location.has_property(js_string!("href"), context)? {
            let current_url = window.get_current_url();

            // Add href property
            location.define_property_or_throw(
                js_string!("href"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(JsString::from(current_url.as_str()))
                    .build(),
                context,
            )?;

            // Add protocol property
            let protocol = if current_url.starts_with("https:") {
                "https:"
            } else if current_url.starts_with("http:") {
                "http:"
            } else {
                "about:"
            };

            location.define_property_or_throw(
                js_string!("protocol"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(JsString::from(protocol))
                    .build(),
                context,
            )?;

            // Add hostname property
            let hostname = if let Some(url_start) = current_url.find("://") {
                let after_protocol = &current_url[url_start + 3..];
                if let Some(slash_pos) = after_protocol.find('/') {
                    &after_protocol[..slash_pos]
                } else {
                    after_protocol
                }
            } else {
                ""
            };

            location.define_property_or_throw(
                js_string!("hostname"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(JsString::from(hostname))
                    .build(),
                context,
            )?;
        }

        Ok(location.into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Window.prototype.location called on non-Window object")
            .into())
    }
}

/// `Window.prototype.history` getter
fn get_history(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Window.prototype.history called on non-object")
    })?;

    if let Some(window) = this_obj.downcast_ref::<WindowData>() {
        let history = window.get_history();

        // Initialize history object if empty
        if !history.has_property(js_string!("length"), context)? {
            // Add length property
            history.define_property_or_throw(
                js_string!("length"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(1)
                    .build(),
                context,
            )?;

            // Add state property
            history.define_property_or_throw(
                js_string!("state"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(JsValue::null())
                    .build(),
                context,
            )?;

            // Add back method
            let back_function = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
                // Implementation would trigger pageswap event and navigate back
                Ok(JsValue::undefined())
            })
            .name(js_string!("back"))
            .build();

            history.define_property_or_throw(
                js_string!("back"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(false)
                    .value(back_function)
                    .build(),
                context,
            )?;

            // Add forward method
            let forward_function = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
                // Implementation would trigger pageswap event and navigate forward
                Ok(JsValue::undefined())
            })
            .name(js_string!("forward"))
            .build();

            history.define_property_or_throw(
                js_string!("forward"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(false)
                    .value(forward_function)
                    .build(),
                context,
            )?;

            // Add go method
            let go_function = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
                let _delta = _args.get_or_undefined(0);
                // Implementation would trigger pageswap event and navigate by delta
                Ok(JsValue::undefined())
            })
            .name(js_string!("go"))
            .build();

            history.define_property_or_throw(
                js_string!("go"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(false)
                    .value(go_function)
                    .build(),
                context,
            )?;

            // Add pushState method
            let push_state_function = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
                let _state = _args.get_or_undefined(0);
                let _title = _args.get_or_undefined(1);
                let _url = _args.get_or_undefined(2);
                // Implementation would trigger pageswap event and push new state
                Ok(JsValue::undefined())
            })
            .name(js_string!("pushState"))
            .build();

            history.define_property_or_throw(
                js_string!("pushState"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(false)
                    .value(push_state_function)
                    .build(),
                context,
            )?;

            // Add replaceState method
            let replace_state_function = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
                let _state = _args.get_or_undefined(0);
                let _title = _args.get_or_undefined(1);
                let _url = _args.get_or_undefined(2);
                // Implementation would trigger pageswap event and replace current state
                Ok(JsValue::undefined())
            })
            .name(js_string!("replaceState"))
            .build();

            history.define_property_or_throw(
                js_string!("replaceState"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(false)
                    .value(replace_state_function)
                    .build(),
                context,
            )?;
        }

        Ok(history.into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Window.prototype.history called on non-Window object")
            .into())
    }
}

/// `Window.prototype.document` getter
fn get_document(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Window.prototype.document called on non-object")
    })?;

    if let Some(window) = this_obj.downcast_ref::<WindowData>() {
        let document = window.get_document();

        // Initialize document if needed
        if !document.has_property(js_string!("readyState"), context)? {
            // Create a new Document instance
            use crate::builtins::document::Document;

            let document_constructor_args: &[JsValue] = &[];
            let new_document = Document::constructor(&JsValue::undefined(), document_constructor_args, context)?;

            if let Some(doc_obj) = new_document.as_object() {
                window.set_document(doc_obj.clone());
                return Ok(new_document);
            }
        }

        Ok(document.into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Window.prototype.document called on non-Window object")
            .into())
    }
}

/// `Window.prototype.navigator` getter
fn get_navigator(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Window.prototype.navigator called on non-object")
    })?;

    if let Some(window) = this_obj.downcast_ref::<WindowData>() {
        let navigator = window.get_navigator();

        // Initialize navigator object if empty
        if !navigator.has_property(js_string!("userAgent"), context)? {
            // Add userAgent property
            navigator.define_property_or_throw(
                js_string!("userAgent"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(JsString::from("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 Thalora/1.0"))
                    .build(),
                context,
            )?;

            // Add platform property
            navigator.define_property_or_throw(
                js_string!("platform"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(JsString::from("Linux x86_64"))
                    .build(),
                context,
            )?;

            // Add language property
            navigator.define_property_or_throw(
                js_string!("language"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(JsString::from("en-US"))
                    .build(),
                context,
            )?;
        }

        Ok(navigator.into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Window.prototype.navigator called on non-Window object")
            .into())
    }
}

/// `Window.prototype.addEventListener(type, listener)`
fn add_event_listener(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Window.prototype.addEventListener called on non-object")
    })?;

    if let Some(window) = this_obj.downcast_ref::<WindowData>() {
        let event_type = args.get_or_undefined(0).to_string(context)?;
        let listener = args.get_or_undefined(1).clone();

        window.add_event_listener(event_type.to_std_string_escaped(), listener);
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("Window.prototype.addEventListener called on non-Window object")
            .into())
    }
}

/// `Window.prototype.removeEventListener(type, listener)`
fn remove_event_listener(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Window.prototype.removeEventListener called on non-object")
    })?;

    if let Some(window) = this_obj.downcast_ref::<WindowData>() {
        let event_type = args.get_or_undefined(0).to_string(context)?;
        let listener = args.get_or_undefined(1);

        window.remove_event_listener(&event_type.to_std_string_escaped(), listener);
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("Window.prototype.removeEventListener called on non-Window object")
            .into())
    }
}

/// `Window.prototype.dispatchEvent(event)`
fn dispatch_event(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Window.prototype.dispatchEvent called on non-object")
    })?;

    if let Some(window) = this_obj.downcast_ref::<WindowData>() {
        let event = args.get_or_undefined(0);

        // Get event type from event object
        if event.is_object() {
            if let Some(event_obj) = event.as_object() {
                if let Ok(type_val) = event_obj.get(js_string!("type"), context) {
                    let event_type = type_val.to_string(context)?;
                    let listeners = window.get_event_listeners(&event_type.to_std_string_escaped());

                    // Call each listener
                    for listener in listeners {
                        if listener.is_callable() {
                            let _ = listener.as_callable().unwrap().call(
                                &this_obj.clone().into(),
                                &[event.clone()],
                                context,
                            );
                        }
                    }
                }
            }
        }

        Ok(true.into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Window.prototype.dispatchEvent called on non-Window object")
            .into())
    }
}

/// `Window.prototype.matchMedia(mediaQuery)`
fn match_media(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Window.prototype.matchMedia called on non-object")
    })?;

    if let Some(_window) = this_obj.downcast_ref::<WindowData>() {
        let media_query = args.get_or_undefined(0).to_string(context)?;
        let query_str = media_query.to_std_string_escaped();

        // Create MediaQueryList object
        let media_query_list = JsObject::default();

        // Parse and evaluate the media query
        let matches = evaluate_media_query(&query_str);

        // Add properties to MediaQueryList
        media_query_list.define_property_or_throw(
            js_string!("media"),
            PropertyDescriptorBuilder::new()
                .configurable(false)
                .enumerable(true)
                .writable(false)
                .value(media_query)
                .build(),
            context,
        )?;

        media_query_list.define_property_or_throw(
            js_string!("matches"),
            PropertyDescriptorBuilder::new()
                .configurable(false)
                .enumerable(true)
                .writable(false)
                .value(matches)
                .build(),
            context,
        )?;

        // Add addListener method
        let add_listener_func = BuiltInBuilder::callable(context.realm(), media_query_list_add_listener)
            .name(js_string!("addListener"))
            .build();

        media_query_list.define_property_or_throw(
            js_string!("addListener"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(add_listener_func)
                .build(),
            context,
        )?;

        // Add removeListener method
        let remove_listener_func = BuiltInBuilder::callable(context.realm(), media_query_list_remove_listener)
            .name(js_string!("removeListener"))
            .build();

        media_query_list.define_property_or_throw(
            js_string!("removeListener"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(remove_listener_func)
                .build(),
            context,
        )?;

        // Add addEventListener method (newer API)
        let add_event_listener_func = BuiltInBuilder::callable(context.realm(), media_query_list_add_event_listener)
            .name(js_string!("addEventListener"))
            .build();

        media_query_list.define_property_or_throw(
            js_string!("addEventListener"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(add_event_listener_func)
                .build(),
            context,
        )?;

        // Add removeEventListener method
        let remove_event_listener_func = BuiltInBuilder::callable(context.realm(), media_query_list_remove_event_listener)
            .name(js_string!("removeEventListener"))
            .build();

        media_query_list.define_property_or_throw(
            js_string!("removeEventListener"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(remove_event_listener_func)
                .build(),
            context,
        )?;

        Ok(media_query_list.into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Window.prototype.matchMedia called on non-Window object")
            .into())
    }
}

/// Simple media query evaluator
fn evaluate_media_query(query: &str) -> bool {
    // Basic media query parsing and evaluation
    // For now, implement some common patterns

    let query = query.trim();

    // Handle common responsive breakpoints
    if query.contains("max-width: 600px") || query.contains("max-width:600px") {
        // Assume viewport is 1024px (desktop default) for now
        false
    } else if query.contains("min-width: 768px") || query.contains("min-width:768px") {
        true
    } else if query.contains("max-width: 768px") || query.contains("max-width:768px") {
        false
    } else if query.contains("min-width: 1024px") || query.contains("min-width:1024px") {
        true
    } else if query.contains("max-width: 1024px") || query.contains("max-width:1024px") {
        true
    } else if query.contains("orientation: portrait") {
        false // Assume landscape by default
    } else if query.contains("orientation: landscape") {
        true
    } else if query.contains("print") {
        false // Not print media
    } else if query.contains("screen") {
        true // Screen media type
    } else if query == "all" || query.is_empty() {
        true
    } else {
        // Default to true for unrecognized queries
        true
    }
}

// MediaQueryList method implementations
fn media_query_list_add_listener(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let _listener = args.get_or_undefined(0);
    // TODO: Implement listener storage and management
    eprintln!("MediaQueryList addListener called");
    Ok(JsValue::undefined())
}

fn media_query_list_remove_listener(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let _listener = args.get_or_undefined(0);
    // TODO: Implement listener removal
    eprintln!("MediaQueryList removeListener called");
    Ok(JsValue::undefined())
}

fn media_query_list_add_event_listener(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let _event_type = args.get_or_undefined(0);
    let _listener = args.get_or_undefined(1);
    // TODO: Implement event listener storage and management
    eprintln!("MediaQueryList addEventListener called");
    Ok(JsValue::undefined())
}

fn media_query_list_remove_event_listener(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let _event_type = args.get_or_undefined(0);
    let _listener = args.get_or_undefined(1);
    // TODO: Implement event listener removal
    eprintln!("MediaQueryList removeEventListener called");
    Ok(JsValue::undefined())
}

/// `Window.prototype.screen` getter
fn get_screen(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Create Screen object
    let screen = JsObject::default();

    // Default desktop screen dimensions (1920x1080)
    let width = 1920;
    let height = 1080;
    let avail_width = 1920; // Available width (excluding taskbar, etc.)
    let avail_height = 1040; // Available height (excluding taskbar)
    let color_depth = 24; // 24-bit color
    let pixel_depth = 24; // Same as color depth on modern displays

    // Add width property
    screen.define_property_or_throw(
        js_string!("width"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(width)
            .build(),
        context,
    )?;

    // Add height property
    screen.define_property_or_throw(
        js_string!("height"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(height)
            .build(),
        context,
    )?;

    // Add availWidth property
    screen.define_property_or_throw(
        js_string!("availWidth"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(avail_width)
            .build(),
        context,
    )?;

    // Add availHeight property
    screen.define_property_or_throw(
        js_string!("availHeight"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(avail_height)
            .build(),
        context,
    )?;

    // Add colorDepth property
    screen.define_property_or_throw(
        js_string!("colorDepth"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(color_depth)
            .build(),
        context,
    )?;

    // Add pixelDepth property
    screen.define_property_or_throw(
        js_string!("pixelDepth"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(pixel_depth)
            .build(),
        context,
    )?;

    // Create orientation object
    let orientation = JsObject::default();

    // Add orientation properties
    orientation.define_property_or_throw(
        js_string!("angle"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(0) // 0 degrees (landscape)
            .build(),
        context,
    )?;

    orientation.define_property_or_throw(
        js_string!("type"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(js_string!("landscape-primary"))
            .build(),
        context,
    )?;

    // Add lock method to orientation
    let lock_func = BuiltInBuilder::callable(context.realm(), screen_orientation_lock)
        .name(js_string!("lock"))
        .build();

    orientation.define_property_or_throw(
        js_string!("lock"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(lock_func)
            .build(),
        context,
    )?;

    // Add unlock method to orientation
    let unlock_func = BuiltInBuilder::callable(context.realm(), screen_orientation_unlock)
        .name(js_string!("unlock"))
        .build();

    orientation.define_property_or_throw(
        js_string!("unlock"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(unlock_func)
            .build(),
        context,
    )?;

    // Add orientation property to screen
    screen.define_property_or_throw(
        js_string!("orientation"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(orientation)
            .build(),
        context,
    )?;

    Ok(screen.into())
}

// Screen orientation method implementations
fn screen_orientation_lock(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _orientation = args.get_or_undefined(0).to_string(context)?;
    // TODO: Implement actual screen orientation locking
    eprintln!("Screen orientation lock called");

    // Return a resolved Promise for now
    let promise = context.eval(boa_engine::Source::from_bytes("Promise.resolve()"))?;
    Ok(promise)
}

fn screen_orientation_unlock(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // TODO: Implement actual screen orientation unlocking
    eprintln!("Screen orientation unlock called");

    // Return undefined as per spec
    Ok(JsValue::undefined())
}