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

/// `Navigator` object that provides information about the user agent and platform.
#[derive(Debug, Clone, Finalize)]
pub struct Navigator {
    user_agent: String,
    platform: String,
    language: String,
    languages: Vec<String>,
    cookie_enabled: bool,
    on_line: bool,
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
            user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            platform: "MacIntel".to_string(),
            language: "en-US".to_string(),
            languages: vec!["en-US".to_string(), "en".to_string()],
            cookie_enabled: true,
            on_line: true,
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

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(js_string!("userAgent"), js_string!("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("platform"), js_string!("MacIntel"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("language"), js_string!("en-US"), Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("cookieEnabled"), true, Attribute::READONLY | Attribute::NON_ENUMERABLE)
            .property(js_string!("onLine"), true, Attribute::READONLY | Attribute::NON_ENUMERABLE)
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

    /// Create a Navigator instance for the global object
    pub fn create_navigator() -> JsObject {
        let navigator = Navigator::new();
        JsObject::from_proto_and_data(None, navigator)
    }
}