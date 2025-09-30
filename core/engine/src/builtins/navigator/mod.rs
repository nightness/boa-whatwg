//! Implementation of the Navigator interface.
//!
//! The Navigator interface represents the state and the identity of the user agent.
//! It allows scripts to query it and to register themselves to carry on some activities.
//!
//! More information:
//! - [WHATWG HTML Specification](https://html.spec.whatwg.org/multipage/system-state.html#the-navigator-object)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/Navigator)

use boa_gc::{Finalize, Trace};
use crate::{
    builtins::BuiltInBuilder,
    context::intrinsics::Intrinsics,
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    Context, JsArgs, JsData, JsResult, JsString, JsValue,
};
use crate::builtins::{BuiltInConstructor, BuiltInObject, IntrinsicObject};
use crate::context::intrinsics::StandardConstructor;
use crate::builtins::web_locks::LockManagerObject;
use crate::builtins::service_worker_container::ServiceWorkerContainer;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Protocol handler storage for registerProtocolHandler
type ProtocolHandlers = Arc<Mutex<HashMap<String, String>>>;

/// `Navigator` object that provides information about the user agent and platform.
#[derive(Debug, Clone, Finalize)]
pub struct Navigator {
    // NavigatorID properties
    user_agent: String,
    app_code_name: String,
    app_name: String,
    app_version: String,
    platform: String,
    product: String,
    product_sub: String,
    vendor: String,
    vendor_sub: String,

    // NavigatorLanguage properties
    language: String,
    languages: Vec<String>,

    // NavigatorOnLine properties
    on_line: bool,

    // NavigatorCookies properties
    cookie_enabled: bool,

    // NavigatorPlugins properties (static/empty for security)
    plugins_length: usize,
    mime_types_length: usize,
    java_enabled: bool,
    pdf_viewer_enabled: bool,

    // Protocol handlers storage
    protocol_handlers: ProtocolHandlers,
}

unsafe impl Trace for Navigator {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // No GC'd objects in Navigator, nothing to trace
    }

    unsafe fn trace_non_roots(&self) {
        // No GC'd objects in Navigator, nothing to trace
    }

    fn run_finalizer(&self) {
        // No cleanup needed for Navigator
    }
}

impl JsData for Navigator {}

impl Navigator {
    pub(crate) fn new() -> Self {
        Self {
            // NavigatorID - WHATWG compliant values
            user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            app_code_name: "Mozilla".to_string(),
            app_name: "Netscape".to_string(),
            app_version: "5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            platform: "MacIntel".to_string(),
            product: "Gecko".to_string(),
            product_sub: "20030107".to_string(),
            vendor: "Google Inc.".to_string(),
            vendor_sub: "".to_string(),

            // NavigatorLanguage
            language: "en-US".to_string(),
            languages: vec!["en-US".to_string(), "en".to_string()],

            // NavigatorOnLine
            on_line: true,

            // NavigatorCookies
            cookie_enabled: true,

            // NavigatorPlugins - empty for security/privacy
            plugins_length: 0,
            mime_types_length: 0,
            java_enabled: false,
            pdf_viewer_enabled: true,

            // Protocol handlers
            protocol_handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[cfg(test)]
mod tests;

impl IntrinsicObject for Navigator {
    fn init(realm: &Realm) {
        let locks_getter_func = BuiltInBuilder::callable(realm, Self::locks_getter)
            .name(js_string!("get locks"))
            .build();

        let service_worker_getter_func = BuiltInBuilder::callable(realm, Self::service_worker_getter)
            .name(js_string!("get serviceWorker"))
            .build();

        let languages_getter_func = BuiltInBuilder::callable(realm, Self::languages_getter)
            .name(js_string!("get languages"))
            .build();

        let plugins_getter_func = BuiltInBuilder::callable(realm, Self::plugins_getter)
            .name(js_string!("get plugins"))
            .build();

        let mime_types_getter_func = BuiltInBuilder::callable(realm, Self::mime_types_getter)
            .name(js_string!("get mimeTypes"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // NavigatorID properties
            .property(js_string!("userAgent"), js_string!("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("appCodeName"), js_string!("Mozilla"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("appName"), js_string!("Netscape"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("appVersion"), js_string!("5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("platform"), js_string!("MacIntel"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("product"), js_string!("Gecko"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("productSub"), js_string!("20030107"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("vendor"), js_string!("Google Inc."), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("vendorSub"), js_string!(""), Attribute::READONLY | Attribute::NON_ENUMERABLE)

            // NavigatorLanguage properties
            .property(js_string!("language"), js_string!("en-US"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .accessor(
                js_string!("languages"),
                Some(languages_getter_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )

            // NavigatorOnLine properties
            .property(js_string!("onLine"), true, Attribute::READONLY | Attribute::NON_ENUMERABLE)

            // NavigatorCookies properties
            .property(js_string!("cookieEnabled"), true, Attribute::READONLY | Attribute::NON_ENUMERABLE)

            // NavigatorPlugins properties
            .accessor(
                js_string!("plugins"),
                Some(plugins_getter_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("mimeTypes"),
                Some(mime_types_getter_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .property(js_string!("pdfViewerEnabled"), true, Attribute::READONLY | Attribute::NON_ENUMERABLE)

            // NavigatorContentUtils methods
            .static_method(Self::register_protocol_handler, js_string!("registerProtocolHandler"), 2)
            .static_method(Self::unregister_protocol_handler, js_string!("unregisterProtocolHandler"), 2)
            .static_method(Self::java_enabled, js_string!("javaEnabled"), 0)

            // Web Locks and Service Workers
            .accessor(
                js_string!("locks"),
                Some(locks_getter_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("serviceWorker"),
                Some(service_worker_getter_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Navigator {
    const NAME: JsString = js_string!("Navigator");
}

impl BuiltInConstructor for Navigator {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &StandardConstructor =
        |constructors| constructors.navigator();

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // Navigator constructor is not meant to be called directly
        Ok(JsValue::undefined())
    }
}

// Navigator prototype methods and getters
impl Navigator {
    /// `navigator.locks` getter
    fn locks_getter(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Create and return a LockManager instance
        let lock_manager = LockManagerObject::new();
        let lock_manager_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().lock_manager().prototype()),
            lock_manager
        );
        Ok(JsValue::from(lock_manager_obj))
    }

    /// `navigator.serviceWorker` getter
    fn service_worker_getter(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Create and return a ServiceWorkerContainer instance
        let service_worker_container = ServiceWorkerContainer::create(context)?;
        Ok(JsValue::from(service_worker_container))
    }

    /// `navigator.languages` getter - returns array of language preferences
    fn languages_getter(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let languages = vec![
            JsValue::from(js_string!("en-US")),
            JsValue::from(js_string!("en")),
        ];

        let array = crate::builtins::Array::array_create(languages.len() as u64, None, context)?;

        for (i, lang) in languages.into_iter().enumerate() {
            array.create_data_property_or_throw(i, lang, context)?;
        }

        Ok(JsValue::from(array))
    }

    /// `navigator.plugins` getter - returns empty PluginArray for security
    fn plugins_getter(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Return empty plugin array for security/privacy
        let plugin_array = crate::builtins::Array::array_create(0, None, context)?;

        // Add length property
        plugin_array.create_data_property_or_throw(
            js_string!("length"),
            JsValue::from(0),
            context,
        )?;

        Ok(JsValue::from(plugin_array))
    }

    /// `navigator.mimeTypes` getter - returns empty MimeTypeArray for security
    fn mime_types_getter(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Return empty mime types array for security/privacy
        let mime_array = crate::builtins::Array::array_create(0, None, context)?;

        // Add length property
        mime_array.create_data_property_or_throw(
            js_string!("length"),
            JsValue::from(0),
            context,
        )?;

        Ok(JsValue::from(mime_array))
    }

    /// `navigator.javaEnabled()` method - returns false for security
    fn java_enabled(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // Always return false for security/privacy
        Ok(JsValue::from(false))
    }

    /// `navigator.registerProtocolHandler(scheme, url)` method
    fn register_protocol_handler(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let scheme = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let url = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();

        // Basic validation - scheme should not be empty
        if scheme.is_empty() {
            return Err(crate::JsNativeError::typ()
                .with_message("Failed to execute 'registerProtocolHandler' on 'Navigator': The scheme provided is not valid.")
                .into());
        }

        // URL should contain %s placeholder
        if !url.contains("%s") {
            return Err(crate::JsNativeError::syntax()
                .with_message("Failed to execute 'registerProtocolHandler' on 'Navigator': The url provided does not contain a '%s' token.")
                .into());
        }

        // Store the protocol handler (in a real implementation, this would integrate with the browser)
        eprintln!("Protocol handler registered: {} -> {}", scheme, url);

        Ok(JsValue::undefined())
    }

    /// `navigator.unregisterProtocolHandler(scheme, url)` method
    fn unregister_protocol_handler(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let scheme = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let url = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();

        // Basic validation - scheme should not be empty
        if scheme.is_empty() {
            return Err(crate::JsNativeError::typ()
                .with_message("Failed to execute 'unregisterProtocolHandler' on 'Navigator': The scheme provided is not valid.")
                .into());
        }

        // Store the protocol handler removal (in a real implementation, this would integrate with the browser)
        eprintln!("Protocol handler unregistered: {} -> {}", scheme, url);

        Ok(JsValue::undefined())
    }

    /// Create a Navigator instance for the global object
    pub fn create_navigator() -> JsObject {
        let navigator = Navigator::new();
        JsObject::from_proto_and_data(None, navigator)
    }
}