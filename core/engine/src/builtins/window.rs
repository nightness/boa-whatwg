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

        let chrome_func = BuiltInBuilder::callable(realm, get_chrome)
            .name(js_string!("get chrome"))
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
            .accessor(
                js_string!("chrome"),
                Some(chrome_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("innerWidth"),
                1366, // Standard desktop width
                Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("innerHeight"),
                768, // Standard desktop height
                Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .method(add_event_listener, js_string!("addEventListener"), 2)
            .method(remove_event_listener, js_string!("removeEventListener"), 2)
            .method(dispatch_event, js_string!("dispatchEvent"), 1)
            .method(match_media, js_string!("matchMedia"), 1)
            .build();

        // Google 2025 bot detection bypass APIs DISABLED to prevent stack overflow
        // TODO: Re-implement without causing infinite recursion
        eprintln!("🚀 Window initialization completed (bot detection APIs disabled)");
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
            // Add userAgent property - use Windows Chrome to avoid Linux bot detection
            navigator.define_property_or_throw(
                js_string!("userAgent"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(JsString::from("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"))
                    .build(),
                context,
            )?;

            // Add platform property - Windows to match userAgent
            navigator.define_property_or_throw(
                js_string!("platform"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(JsString::from("Win32"))
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

            // Add languages array property
            use crate::builtins::array::Array;
            let languages_array = Array::create_array_from_list([
                JsString::from("en-US").into(),
                JsString::from("en").into(),
            ], context);
            navigator.define_property_or_throw(
                js_string!("languages"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(languages_array)
                    .build(),
                context,
            )?;

            // Add webdriver property (should be false for legitimate browsers)
            navigator.define_property_or_throw(
                js_string!("webdriver"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(false)
                    .value(false)
                    .build(),
                context,
            )?;

            // Add plugins array (fake some common plugins)
            let plugins_array = Array::create_array_from_list([
                // Create fake PDF Viewer plugin
                create_fake_plugin(context, "PDF Viewer", "Portable Document Format", "pdf")?,
                // Create fake Chrome PDF Plugin
                create_fake_plugin(context, "Chrome PDF Plugin", "Portable Document Format", "pdf")?,
                // Create fake Chromium PDF Plugin
                create_fake_plugin(context, "Chromium PDF Plugin", "Portable Document Format", "pdf")?,
                // Create fake Microsoft Edge PDF Plugin
                create_fake_plugin(context, "Microsoft Edge PDF Plugin", "Portable Document Format", "pdf")?,
                // Create fake WebKit built-in PDF
                create_fake_plugin(context, "WebKit built-in PDF", "Portable Document Format", "pdf")?,
            ], context);

            // Add length property to plugins array
            plugins_array.define_property_or_throw(
                js_string!("length"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(false)
                    .writable(false)
                    .value(5)
                    .build(),
                context,
            )?;

            navigator.define_property_or_throw(
                js_string!("plugins"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(plugins_array)
                    .build(),
                context,
            )?;

            // Add mimeTypes array (related to plugins)
            let mime_types_array = Array::create_array_from_list([
                create_fake_mime_type(context, "application/pdf", "pdf")?,
                create_fake_mime_type(context, "text/pdf", "pdf")?,
            ], context);

            mime_types_array.define_property_or_throw(
                js_string!("length"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(false)
                    .writable(false)
                    .value(2)
                    .build(),
                context,
            )?;

            navigator.define_property_or_throw(
                js_string!("mimeTypes"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(mime_types_array)
                    .build(),
                context,
            )?;

            // Add cookieEnabled property
            navigator.define_property_or_throw(
                js_string!("cookieEnabled"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(true)
                    .build(),
                context,
            )?;

            // Add doNotTrack property
            navigator.define_property_or_throw(
                js_string!("doNotTrack"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(JsValue::null())
                    .build(),
                context,
            )?;

            // Add hardwareConcurrency property (fake CPU core count)
            navigator.define_property_or_throw(
                js_string!("hardwareConcurrency"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(8) // Fake 8 CPU cores
                    .build(),
                context,
            )?;

            // Add maxTouchPoints property
            navigator.define_property_or_throw(
                js_string!("maxTouchPoints"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(0) // Desktop - no touch
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

/// Enhanced media query evaluator with better parsing
fn evaluate_media_query(query: &str) -> bool {
    let query = query.trim();

    // Default viewport dimensions (can be made configurable later)
    let viewport_width = 1366.0; // Common desktop width
    let viewport_height = 768.0;  // Common desktop height
    let pixel_density = 1.0;

    // Handle empty/all queries
    if query.is_empty() || query == "all" {
        return true;
    }

    // Handle media types
    if query == "screen" {
        return true;
    }
    if query == "print" || query == "speech" || query == "braille" {
        return false;
    }

    // Parse complex queries with logical operators
    if query.contains(" and ") {
        return query.split(" and ")
            .all(|part| evaluate_single_media_feature(part.trim(), viewport_width, viewport_height, pixel_density));
    }

    if query.contains(" or ") || query.contains(", ") {
        // Handle both comma-separated and " or " separated queries
        let parts = if query.contains(", ") {
            query.split(", ").collect::<Vec<_>>()
        } else {
            query.split(" or ").collect::<Vec<_>>()
        };

        return parts.iter()
            .map(|part| part.trim())
            .any(|part| {
                if part.contains(" and ") {
                    part.split(" and ")
                        .all(|subpart| evaluate_single_media_feature(subpart.trim(), viewport_width, viewport_height, pixel_density))
                } else {
                    evaluate_single_media_feature(part, viewport_width, viewport_height, pixel_density)
                }
            });
    }

    // Single media feature
    evaluate_single_media_feature(query, viewport_width, viewport_height, pixel_density)
}

fn evaluate_single_media_feature(feature: &str, width: f64, height: f64, density: f64) -> bool {
    let feature = feature.trim();

    // Remove parentheses if present
    let feature = if feature.starts_with('(') && feature.ends_with(')') {
        &feature[1..feature.len()-1]
    } else {
        feature
    };

    // Width queries
    if let Some(value) = extract_pixel_value(feature, "max-width") {
        return width <= value;
    }
    if let Some(value) = extract_pixel_value(feature, "min-width") {
        return width >= value;
    }
    if let Some(value) = extract_pixel_value(feature, "width") {
        return width == value;
    }

    // Height queries
    if let Some(value) = extract_pixel_value(feature, "max-height") {
        return height <= value;
    }
    if let Some(value) = extract_pixel_value(feature, "min-height") {
        return height >= value;
    }
    if let Some(value) = extract_pixel_value(feature, "height") {
        return height == value;
    }

    // Device pixel ratio
    if let Some(value) = extract_float_value(feature, "max-device-pixel-ratio") {
        return density <= value;
    }
    if let Some(value) = extract_float_value(feature, "min-device-pixel-ratio") {
        return density >= value;
    }
    if let Some(value) = extract_float_value(feature, "-webkit-max-device-pixel-ratio") {
        return density <= value;
    }
    if let Some(value) = extract_float_value(feature, "-webkit-min-device-pixel-ratio") {
        return density >= value;
    }

    // Orientation
    if feature.contains("orientation: landscape") || feature.contains("orientation:landscape") {
        return width > height;
    }
    if feature.contains("orientation: portrait") || feature.contains("orientation:portrait") {
        return height > width;
    }

    // Media types
    if feature == "screen" {
        return true;
    }
    if feature == "print" || feature == "speech" || feature == "braille" {
        return false;
    }

    // Color capabilities
    if feature.contains("color") && !feature.contains(":") {
        return true; // Assume color display
    }
    if let Some(value) = extract_numeric_value(feature, "min-color") {
        return value <= 8; // 8-bit color depth
    }

    // Default to true for unrecognized features to be permissive
    true
}

fn extract_pixel_value(feature: &str, property: &str) -> Option<f64> {
    let pattern = format!("{}:", property);
    if let Some(start) = feature.find(&pattern) {
        let value_part = &feature[start + pattern.len()..];
        let value_part = value_part.trim();

        // Handle px values
        if value_part.ends_with("px") {
            if let Ok(value) = value_part[..value_part.len()-2].trim().parse::<f64>() {
                return Some(value);
            }
        }

        // Handle em values (assume 16px = 1em)
        if value_part.ends_with("em") {
            if let Ok(value) = value_part[..value_part.len()-2].trim().parse::<f64>() {
                return Some(value * 16.0);
            }
        }

        // Handle rem values (assume 16px = 1rem)
        if value_part.ends_with("rem") {
            if let Ok(value) = value_part[..value_part.len()-3].trim().parse::<f64>() {
                return Some(value * 16.0);
            }
        }

        // Handle unitless values (assume px)
        if let Ok(value) = value_part.parse::<f64>() {
            return Some(value);
        }
    }
    None
}

fn extract_float_value(feature: &str, property: &str) -> Option<f64> {
    let pattern = format!("{}:", property);
    if let Some(start) = feature.find(&pattern) {
        let value_part = &feature[start + pattern.len()..];
        if let Ok(value) = value_part.trim().parse::<f64>() {
            return Some(value);
        }
    }
    None
}

fn extract_numeric_value(feature: &str, property: &str) -> Option<u32> {
    let pattern = format!("{}:", property);
    if let Some(start) = feature.find(&pattern) {
        let value_part = &feature[start + pattern.len()..];
        if let Ok(value) = value_part.trim().parse::<u32>() {
            return Some(value);
        }
    }
    None
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
    eprintln!("✅ Creating Screen object for window.screen");

    // Check if we already have a screen object in global scope to avoid creating duplicate
    if let Ok(existing_screen) = context.global_object().get(js_string!("screen"), context) {
        if !existing_screen.is_undefined() {
            return Ok(existing_screen);
        }
    }

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

    // Also register screen as a global variable (not just window.screen)
    // This ensures both window.screen and global screen work correctly
    context.global_object().set(
        js_string!("screen"),
        screen.clone(),
        false,
        context,
    )?;

    eprintln!("✅ Screen object registered both as window.screen and global screen");

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

/// Helper function to create fake plugin objects
fn create_fake_plugin(context: &mut Context, name: &str, description: &str, suffix: &str) -> JsResult<JsValue> {
    let plugin = JsObject::default();

    // Add name property
    plugin.define_property_or_throw(
        js_string!("name"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(JsString::from(name))
            .build(),
        context,
    )?;

    // Add description property
    plugin.define_property_or_throw(
        js_string!("description"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(JsString::from(description))
            .build(),
        context,
    )?;

    // Add filename property
    plugin.define_property_or_throw(
        js_string!("filename"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(JsString::from(format!("internal-{}-viewer", suffix)))
            .build(),
        context,
    )?;

    // Add length property
    plugin.define_property_or_throw(
        js_string!("length"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(false)
            .writable(false)
            .value(1)
            .build(),
        context,
    )?;

    Ok(plugin.into())
}

/// Helper function to create fake MIME type objects
fn create_fake_mime_type(context: &mut Context, type_name: &str, suffix: &str) -> JsResult<JsValue> {
    let mime_type = JsObject::default();

    // Add type property
    mime_type.define_property_or_throw(
        js_string!("type"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(JsString::from(type_name))
            .build(),
        context,
    )?;

    // Add suffixes property
    mime_type.define_property_or_throw(
        js_string!("suffixes"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(JsString::from(suffix))
            .build(),
        context,
    )?;

    // Add description property
    mime_type.define_property_or_throw(
        js_string!("description"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(JsString::from("Portable Document Format"))
            .build(),
        context,
    )?;

    // Add enabledPlugin property (reference back to plugin)
    let fake_plugin = create_fake_plugin(context, "PDF Viewer", "Portable Document Format", suffix)?;
    mime_type.define_property_or_throw(
        js_string!("enabledPlugin"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(fake_plugin)
            .build(),
        context,
    )?;

    Ok(mime_type.into())
}

/// `Window.prototype.chrome` getter - Chrome-specific APIs
fn get_chrome(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Create Chrome object with common Chrome-specific APIs
    let chrome = JsObject::default();

    // Add runtime object (Chrome extension API)
    let runtime = JsObject::default();

    // Add onConnect property to runtime (for Chrome extensions)
    runtime.define_property_or_throw(
        js_string!("onConnect"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(JsValue::undefined())
            .build(),
        context,
    )?;

    // Add getManifest method to runtime
    let get_manifest_func = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
        // Return undefined since we're not a Chrome extension
        Ok(JsValue::undefined())
    })
    .name(js_string!("getManifest"))
    .build();

    runtime.define_property_or_throw(
        js_string!("getManifest"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(get_manifest_func)
            .build(),
        context,
    )?;

    chrome.define_property_or_throw(
        js_string!("runtime"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(runtime)
            .build(),
        context,
    )?;

    // Add app object (Chrome Apps API)
    let app = JsObject::default();

    // Add isInstalled property
    app.define_property_or_throw(
        js_string!("isInstalled"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(false)
            .build(),
        context,
    )?;

    chrome.define_property_or_throw(
        js_string!("app"),
        PropertyDescriptorBuilder::new()
            .configurable(false)
            .enumerable(true)
            .writable(false)
            .value(app)
            .build(),
        context,
    )?;

    // Add csi method (Chrome Speed Index)
    let csi_func = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
        // Return empty object for CSI data
        Ok(JsValue::undefined())
    })
    .name(js_string!("csi"))
    .build();

    chrome.define_property_or_throw(
        js_string!("csi"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(csi_func)
            .build(),
        context,
    )?;

    // Add loadTimes method (deprecated but still checked)
    let load_times_func = BuiltInBuilder::callable(context.realm(), |_this, _args, context| {
        // Return realistic Chrome loadTimes object
        let load_times = JsObject::default();

        load_times.define_property_or_throw(
            js_string!("requestTime"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(1577836800.0) // Unix timestamp
                .build(),
            context,
        )?;

        load_times.define_property_or_throw(
            js_string!("startLoadTime"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(1577836800.1)
                .build(),
            context,
        )?;

        load_times.define_property_or_throw(
            js_string!("commitLoadTime"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(1577836800.2)
                .build(),
            context,
        )?;

        load_times.define_property_or_throw(
            js_string!("finishDocumentLoadTime"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(1577836800.3)
                .build(),
            context,
        )?;

        load_times.define_property_or_throw(
            js_string!("finishLoadTime"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(1577836800.4)
                .build(),
            context,
        )?;

        load_times.define_property_or_throw(
            js_string!("firstPaintTime"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(1577836800.5)
                .build(),
            context,
        )?;

        load_times.define_property_or_throw(
            js_string!("firstPaintAfterLoadTime"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(1577836800.6)
                .build(),
            context,
        )?;

        load_times.define_property_or_throw(
            js_string!("navigationType"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(JsString::from("navigate"))
                .build(),
            context,
        )?;

        Ok(load_times.into())
    })
    .name(js_string!("loadTimes"))
    .build();

    chrome.define_property_or_throw(
        js_string!("loadTimes"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(true)
            .value(load_times_func)
            .build(),
        context,
    )?;

    Ok(chrome.into())
}