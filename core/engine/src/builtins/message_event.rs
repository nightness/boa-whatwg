//! MessageEvent implementation for Boa
//!
//! Implements the MessageEvent interface as defined in:
//! https://html.spec.whatwg.org/multipage/comms.html#messageevent

#[cfg(test)]
mod tests;

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, js_string,
};
use boa_gc::{Finalize, Trace};

/// JavaScript `MessageEvent` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct MessageEvent;

impl IntrinsicObject for MessageEvent {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                js_string!("data"),
                js_string!(""),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("origin"),
                js_string!(""),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("lastEventId"),
                js_string!(""),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("source"),
                JsValue::null(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("ports"),
                JsValue::undefined(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for MessageEvent {
    const NAME: JsString = StaticJsStrings::MESSAGE_EVENT;
}

impl BuiltInConstructor for MessageEvent {
    const LENGTH: usize = 1;
    const P: usize = 1;
    const SP: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::message_event;

    /// `new MessageEvent(type, eventInitDict)`
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("MessageEvent constructor requires 'new'")
                .into());
        }

        let type_arg = args.get_or_undefined(0);
        let event_init_dict = args.get_or_undefined(1);

        // Convert type to string
        let event_type = type_arg.to_string(context)?;

        // Create the MessageEvent object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::message_event, context)?;
        let message_event_data = MessageEventData::new(event_type.to_std_string_escaped());
        let message_event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            message_event_data,
        );

        // Parse eventInitDict if provided
        if !event_init_dict.is_undefined() {
            if let Some(init_obj) = event_init_dict.as_object() {
                // Set data property
                if let Ok(data_val) = init_obj.get(js_string!("data"), context) {
                    message_event_obj.set(js_string!("data"), data_val, false, context)?;
                }

                // Set origin property
                if let Ok(origin_val) = init_obj.get(js_string!("origin"), context) {
                    let origin_str = origin_val.to_string(context)?;
                    message_event_obj.set(js_string!("origin"), origin_str, false, context)?;
                }

                // Set lastEventId property
                if let Ok(last_event_id_val) = init_obj.get(js_string!("lastEventId"), context) {
                    let last_event_id_str = last_event_id_val.to_string(context)?;
                    message_event_obj.set(js_string!("lastEventId"), last_event_id_str, false, context)?;
                }

                // Set source property
                if let Ok(source_val) = init_obj.get(js_string!("source"), context) {
                    message_event_obj.set(js_string!("source"), source_val, false, context)?;
                }

                // Set ports property
                if let Ok(ports_val) = init_obj.get(js_string!("ports"), context) {
                    message_event_obj.set(js_string!("ports"), ports_val, false, context)?;
                }

                // Set bubbles property (inherited from Event)
                if let Ok(bubbles_val) = init_obj.get(js_string!("bubbles"), context) {
                    let bubbles = bubbles_val.to_boolean();
                    message_event_obj.set(js_string!("bubbles"), bubbles, false, context)?;
                }

                // Set cancelable property (inherited from Event)
                if let Ok(cancelable_val) = init_obj.get(js_string!("cancelable"), context) {
                    let cancelable = cancelable_val.to_boolean();
                    message_event_obj.set(js_string!("cancelable"), cancelable, false, context)?;
                }
            }
        }

        // Set the type property
        message_event_obj.set(js_string!("type"), event_type, false, context)?;

        // Set default values for Event interface properties
        message_event_obj.set(js_string!("bubbles"), false, false, context)?;
        message_event_obj.set(js_string!("cancelable"), false, false, context)?;
        message_event_obj.set(js_string!("composed"), false, false, context)?;
        message_event_obj.set(js_string!("defaultPrevented"), false, false, context)?;
        message_event_obj.set(js_string!("eventPhase"), 0, false, context)?;
        message_event_obj.set(js_string!("isTrusted"), false, false, context)?;
        message_event_obj.set(js_string!("target"), JsValue::null(), false, context)?;
        message_event_obj.set(js_string!("currentTarget"), JsValue::null(), false, context)?;
        message_event_obj.set(js_string!("timeStamp"), context.clock().now().millis_since_epoch(), false, context)?;

        Ok(message_event_obj.into())
    }
}

/// Internal data for MessageEvent instances
#[derive(Debug, Trace, Finalize, JsData)]
struct MessageEventData {
    #[unsafe_ignore_trace]
    event_type: String,
}

impl MessageEventData {
    fn new(event_type: String) -> Self {
        Self { event_type }
    }
}

/// Create a MessageEvent from structured clone data
pub fn create_message_event(
    data: JsValue,
    origin: Option<&str>,
    source: Option<JsValue>,
    ports: Option<JsValue>,
    context: &mut Context,
) -> JsResult<JsObject> {
    let message_event_constructor = context.intrinsics().constructors().message_event().constructor();

    // Create the event
    let message_event = message_event_constructor.construct(
        &[js_string!("message").into()],
        Some(&message_event_constructor),
        context,
    )?;

    // Set the data
    message_event.set(js_string!("data"), data, false, context)?;

    // Set origin if provided
    if let Some(origin_str) = origin {
        message_event.set(js_string!("origin"), js_string!(origin_str), false, context)?;
    }

    // Set source if provided
    if let Some(source_val) = source {
        message_event.set(js_string!("source"), source_val, false, context)?;
    }

    // Set ports if provided
    if let Some(ports_val) = ports {
        message_event.set(js_string!("ports"), ports_val, false, context)?;
    }

    Ok(message_event)
}