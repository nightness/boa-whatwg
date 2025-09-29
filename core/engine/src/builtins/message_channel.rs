//! MessageChannel Web API implementation for Boa
//!
//! Native implementation of MessageChannel standard
//! https://html.spec.whatwg.org/multipage/web-messaging.html#message-channels
//!
//! This implements the complete MessageChannel interface for creating communication channels

#[cfg(test)]
mod tests;

use crate::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder, worker_events},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::Attribute
};
use boa_gc::{Finalize, Trace};
use super::message_port::MessagePortData;

/// JavaScript `MessageChannel` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct MessageChannel;

/// Internal data for MessageChannel instances
#[derive(Debug, Trace, Finalize, JsData)]
struct MessageChannelData {
    port1: JsObject,
    port2: JsObject,
}

impl IntrinsicObject for MessageChannel {
    fn init(realm: &Realm) {
        let port1_func = BuiltInBuilder::callable(realm, get_port1)
            .name(js_string!("get port1"))
            .build();

        let port2_func = BuiltInBuilder::callable(realm, get_port2)
            .name(js_string!("get port2"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Instance properties
            .accessor(
                js_string!("port1"),
                Some(port1_func),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("port2"),
                Some(port2_func),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.constructors().message_channel().constructor()
    }
}

impl BuiltInObject for MessageChannel {
    const NAME: JsString = StaticJsStrings::MESSAGE_CHANNEL;
}

impl BuiltInConstructor for MessageChannel {
    const P: usize = 2; // prototype property capacity
    const SP: usize = 0; // static property capacity
    const LENGTH: usize = 0; // no required parameters

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::message_channel;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Ensure 'new' was used
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("MessageChannel constructor requires 'new'")
                .into());
        }

        // Create entangled MessagePort pair
        let (port1_data, port2_data) = MessagePortData::create_entangled_pair();

        // Create MessagePort objects
        let port1 = create_message_port_object(port1_data, context)?;
        let port2 = create_message_port_object(port2_data, context)?;

        // Create the MessageChannel object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::message_channel, context)?;
        let channel_data = MessageChannelData { port1, port2 };
        let channel_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            channel_data,
        );

        Ok(channel_obj.into())
    }
}

/// Get the port1 property of the MessageChannel
fn get_port1(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MessageChannel port1 getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<MessageChannelData>() {
        Ok(data.port1.clone().into())
    } else {
        Err(JsNativeError::typ()
            .with_message("'this' is not a MessageChannel object")
            .into())
    }
}

/// Get the port2 property of the MessageChannel
fn get_port2(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MessageChannel port2 getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<MessageChannelData>() {
        Ok(data.port2.clone().into())
    } else {
        Err(JsNativeError::typ()
            .with_message("'this' is not a MessageChannel object")
            .into())
    }
}

/// Helper function to create a MessagePort object with proper prototype and methods
fn create_message_port_object(data: MessagePortData, context: &mut Context) -> JsResult<JsObject> {
    // Create the object with MessagePort prototype
    let prototype = context.intrinsics().constructors().object().prototype(); // TODO: Use MessagePort prototype when available
    let port_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        prototype,
        data,
    );

    // Add MessagePort methods
    let post_message_func = BuiltInBuilder::callable(context.realm(), message_port_post_message)
        .name(js_string!("postMessage"))
        .length(1)
        .build();

    let start_func = BuiltInBuilder::callable(context.realm(), message_port_start)
        .name(js_string!("start"))
        .length(0)
        .build();

    let close_func = BuiltInBuilder::callable(context.realm(), message_port_close)
        .name(js_string!("close"))
        .length(0)
        .build();

    // Define methods on the port object using PropertyDescriptorBuilder
    use crate::property::PropertyDescriptorBuilder;

    port_obj.define_property_or_throw(
        js_string!("postMessage"),
        PropertyDescriptorBuilder::new()
            .value(post_message_func)
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    port_obj.define_property_or_throw(
        js_string!("start"),
        PropertyDescriptorBuilder::new()
            .value(start_func)
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    port_obj.define_property_or_throw(
        js_string!("close"),
        PropertyDescriptorBuilder::new()
            .value(close_func)
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Add event handler properties to MessagePort
    worker_events::add_worker_event_handlers(&port_obj, context)?;

    Ok(port_obj)
}

/// MessagePort postMessage implementation
fn message_port_post_message(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let message = args.get_or_undefined(0).clone();

    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MessagePort.prototype.postMessage called on non-object")
    })?;

    let data = this_obj.downcast_ref::<MessagePortData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a MessagePort object")
    })?;

    if !data.is_entangled() {
        return Err(JsNativeError::typ()
            .with_message("Cannot post message on disentangled port")
            .into());
    }

    // TODO: Implement structured cloning for complex objects
    // For now, we'll clone simple values
    data.post_message(message)?;

    Ok(JsValue::undefined())
}

/// MessagePort start implementation
fn message_port_start(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MessagePort.prototype.start called on non-object")
    })?;

    let data = this_obj.downcast_ref::<MessagePortData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a MessagePort object")
    })?;

    data.start();
    Ok(JsValue::undefined())
}

/// MessagePort close implementation
fn message_port_close(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MessagePort.prototype.close called on non-object")
    })?;

    let data = this_obj.downcast_ref::<MessagePortData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a MessagePort object")
    })?;

    data.close();
    Ok(JsValue::undefined())
}