//! AbortController Web API implementation for Boa
//!
//! Native implementation of AbortController standard
//! https://dom.spec.whatwg.org/#interface-abortcontroller

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
use std::sync::Arc;

/// JavaScript `AbortController` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct AbortController;

impl IntrinsicObject for AbortController {
    fn init(realm: &Realm) {
        let signal_func = BuiltInBuilder::callable(realm, get_signal)
            .name(js_string!("get signal"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("signal"),
                Some(signal_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(abort, js_string!("abort"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for AbortController {
    const NAME: JsString = StaticJsStrings::ABORT_CONTROLLER;
}

impl BuiltInConstructor for AbortController {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::abort_controller;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::abort_controller,
            context,
        )?;

        let abort_controller_data = AbortControllerData::new();

        let abort_controller = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            abort_controller_data,
        );

        Ok(abort_controller.into())
    }
}

/// Internal data for AbortController objects
#[derive(Debug, Trace, Finalize, JsData)]
pub struct AbortControllerData {
    #[unsafe_ignore_trace]
    signal: Arc<std::sync::Mutex<AbortSignalState>>,
}

#[derive(Debug)]
struct AbortSignalState {
    aborted: bool,
    reason: Option<JsValue>,
}

impl AbortControllerData {
    fn new() -> Self {
        Self {
            signal: Arc::new(std::sync::Mutex::new(AbortSignalState {
                aborted: false,
                reason: None,
            })),
        }
    }

    fn abort(&self, reason: Option<JsValue>) {
        let mut signal = self.signal.lock().unwrap();
        if !signal.aborted {
            signal.aborted = true;
            signal.reason = reason;
            // In a full implementation, this would trigger abort events
        }
    }

    fn is_aborted(&self) -> bool {
        self.signal.lock().unwrap().aborted
    }
}

/// `AbortController.prototype.signal` getter
fn get_signal(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("AbortController.prototype.signal called on non-object")
    })?;

    if let Some(abort_controller) = this_obj.downcast_ref::<AbortControllerData>() {
        // Create AbortSignal object (simplified - real implementation would be more complex)
        let signal_obj = JsObject::default();

        // Add aborted property
        let aborted = abort_controller.is_aborted();
        signal_obj.define_property_or_throw(
            js_string!("aborted"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(false)
                .value(aborted)
                .build(),
            context,
        )?;

        Ok(signal_obj.into())
    } else {
        Err(JsNativeError::typ()
            .with_message("AbortController.prototype.signal called on non-AbortController object")
            .into())
    }
}

/// `AbortController.prototype.abort(reason)`
fn abort(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("AbortController.prototype.abort called on non-object")
    })?;

    if let Some(abort_controller) = this_obj.downcast_ref::<AbortControllerData>() {
        let reason = args.get_or_undefined(0).clone();
        abort_controller.abort(Some(reason));
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("AbortController.prototype.abort called on non-AbortController object")
            .into())
    }
}