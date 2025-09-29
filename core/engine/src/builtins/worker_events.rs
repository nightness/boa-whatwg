//! Worker event system implementation
//!
//! Provides a common event handling system for all Worker APIs
//! Implements event handler properties and event dispatching

use crate::{
    value::JsValue,
    Context, JsResult, JsString, js_string, JsArgs,
    object::JsObject,
    property::{PropertyDescriptorBuilder, Attribute},
    JsNativeError,
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;

/// Event types for Worker APIs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Trace, Finalize)]
pub enum WorkerEventType {
    Message,
    MessageError,
    Error,
}

impl WorkerEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::MessageError => "messageerror",
            Self::Error => "error",
        }
    }

    pub fn handler_property(&self) -> &'static str {
        match self {
            Self::Message => "onmessage",
            Self::MessageError => "onmessageerror",
            Self::Error => "onerror",
        }
    }
}

/// Event handler storage for Worker objects
#[derive(Debug, Trace, Finalize)]
pub struct WorkerEventHandlers {
    handlers: HashMap<WorkerEventType, Option<JsValue>>,
}

impl WorkerEventHandlers {
    pub fn new() -> Self {
        let mut handlers = HashMap::new();
        handlers.insert(WorkerEventType::Message, None);
        handlers.insert(WorkerEventType::MessageError, None);
        handlers.insert(WorkerEventType::Error, None);

        Self { handlers }
    }

    pub fn get_handler(&self, event_type: &WorkerEventType) -> Option<&JsValue> {
        self.handlers.get(event_type).and_then(|h| h.as_ref())
    }

    pub fn set_handler(&mut self, event_type: WorkerEventType, handler: Option<JsValue>) {
        self.handlers.insert(event_type, handler);
    }
}

/// Event object for Worker events
#[derive(Debug, Trace, Finalize)]
pub struct WorkerEvent {
    pub event_type: WorkerEventType,
    pub data: Option<JsValue>,
    pub error: Option<JsValue>,
    pub bubbles: bool,
    pub cancelable: bool,
}

impl WorkerEvent {
    pub fn new_message(data: JsValue) -> Self {
        Self {
            event_type: WorkerEventType::Message,
            data: Some(data),
            error: None,
            bubbles: false,
            cancelable: false,
        }
    }

    pub fn new_error(error: JsValue) -> Self {
        Self {
            event_type: WorkerEventType::Error,
            data: None,
            error: Some(error),
            bubbles: false,
            cancelable: false,
        }
    }

    pub fn new_message_error(error: JsValue) -> Self {
        Self {
            event_type: WorkerEventType::MessageError,
            data: None,
            error: Some(error),
            bubbles: false,
            cancelable: false,
        }
    }
}

/// Add event handler properties to a Worker object
pub fn add_worker_event_handlers(obj: &JsObject, context: &mut Context) -> JsResult<()> {
    // Add onmessage property
    let onmessage_getter = crate::builtins::BuiltInBuilder::callable(context.realm(), get_onmessage)
        .name(js_string!("get onmessage"))
        .build();

    let onmessage_setter = crate::builtins::BuiltInBuilder::callable(context.realm(), set_onmessage)
        .name(js_string!("set onmessage"))
        .build();

    obj.define_property_or_throw(
        js_string!("onmessage"),
        PropertyDescriptorBuilder::new()
            .get(onmessage_getter)
            .set(onmessage_setter)
            .enumerable(true)
            .configurable(true),
        context,
    )?;

    // Add onerror property
    let onerror_getter = crate::builtins::BuiltInBuilder::callable(context.realm(), get_onerror)
        .name(js_string!("get onerror"))
        .build();

    let onerror_setter = crate::builtins::BuiltInBuilder::callable(context.realm(), set_onerror)
        .name(js_string!("set onerror"))
        .build();

    obj.define_property_or_throw(
        js_string!("onerror"),
        PropertyDescriptorBuilder::new()
            .get(onerror_getter)
            .set(onerror_setter)
            .enumerable(true)
            .configurable(true),
        context,
    )?;

    // Add onmessageerror property
    let onmessageerror_getter = crate::builtins::BuiltInBuilder::callable(context.realm(), get_onmessageerror)
        .name(js_string!("get onmessageerror"))
        .build();

    let onmessageerror_setter = crate::builtins::BuiltInBuilder::callable(context.realm(), set_onmessageerror)
        .name(js_string!("set onmessageerror"))
        .build();

    obj.define_property_or_throw(
        js_string!("onmessageerror"),
        PropertyDescriptorBuilder::new()
            .get(onmessageerror_getter)
            .set(onmessageerror_setter)
            .enumerable(true)
            .configurable(true),
        context,
    )?;

    Ok(())
}

/// Dispatch an event to a Worker object
pub fn dispatch_worker_event(obj: &JsObject, event: WorkerEvent, context: &mut Context) -> JsResult<()> {
    // Get the appropriate event handler
    if let Some(handler) = get_event_handler_for_object(obj, &event.event_type) {
        if handler.is_callable() {
            // Create event object
            let event_obj = create_event_object(&event, context)?;

            // Call the handler with the event object
            if let Some(handler_obj) = handler.as_callable() {
                let _ = handler_obj.call(&JsValue::from(obj.clone()), &[event_obj], context);
            }
        }
    }

    Ok(())
}

/// Get event handler from object's event handlers storage
fn get_event_handler_for_object(obj: &JsObject, event_type: &WorkerEventType) -> Option<JsValue> {
    // Try to get the event handlers from the object
    // This is a simplified implementation - in a real implementation we'd store this in the object's data
    match obj.get(js_string!(event_type.handler_property()), &mut Context::default()) {
        Ok(value) if !value.is_null_or_undefined() => Some(value),
        _ => None,
    }
}

/// Create an event object from WorkerEvent
fn create_event_object(event: &WorkerEvent, context: &mut Context) -> JsResult<JsValue> {
    let event_obj = JsObject::with_object_proto(context.intrinsics());

    // Set event type
    event_obj.set(
        js_string!("type"),
        js_string!(event.event_type.as_str()),
        false,
        context,
    )?;

    // Set data if present
    if let Some(ref data) = event.data {
        event_obj.set(js_string!("data"), data.clone(), false, context)?;
    }

    // Set error if present
    if let Some(ref error) = event.error {
        event_obj.set(js_string!("error"), error.clone(), false, context)?;
    }

    // Set event properties
    event_obj.set(js_string!("bubbles"), event.bubbles, false, context)?;
    event_obj.set(js_string!("cancelable"), event.cancelable, false, context)?;

    Ok(JsValue::from(event_obj))
}

// Event handler getter/setter functions

fn get_onmessage(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("onmessage getter called on non-object")
    })?;

    // Get the stored handler value
    match this_obj.get(js_string!("__onmessage_handler"), _context) {
        Ok(value) => Ok(value),
        Err(_) => Ok(JsValue::null()),
    }
}

fn set_onmessage(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("onmessage setter called on non-object")
    })?;

    let handler = args.get_or_undefined(0);

    // Store the handler value
    this_obj.set(js_string!("__onmessage_handler"), handler.clone(), false, context)?;

    Ok(JsValue::undefined())
}

fn get_onerror(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("onerror getter called on non-object")
    })?;

    match this_obj.get(js_string!("__onerror_handler"), _context) {
        Ok(value) => Ok(value),
        Err(_) => Ok(JsValue::null()),
    }
}

fn set_onerror(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("onerror setter called on non-object")
    })?;

    let handler = args.get_or_undefined(0);
    this_obj.set(js_string!("__onerror_handler"), handler.clone(), false, context)?;

    Ok(JsValue::undefined())
}

fn get_onmessageerror(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("onmessageerror getter called on non-object")
    })?;

    match this_obj.get(js_string!("__onmessageerror_handler"), _context) {
        Ok(value) => Ok(value),
        Err(_) => Ok(JsValue::null()),
    }
}

fn set_onmessageerror(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("onmessageerror setter called on non-object")
    })?;

    let handler = args.get_or_undefined(0);
    this_obj.set(js_string!("__onmessageerror_handler"), handler.clone(), false, context)?;

    Ok(JsValue::undefined())
}