//! CustomEvent interface implementation for DOM Level 4
//!
//! The CustomEvent interface represents events initialized by an application for custom purposes.
//! It inherits from Event and adds support for custom data payload.
//! https://dom.spec.whatwg.org/#interface-customevent

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    property::Attribute,
    realm::Realm,
    string::{StaticJsStrings, JsString},
    Context, JsArgs, JsData, JsNativeError, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace, GcRefCell};

/// The CustomEvent data implementation
#[derive(Debug, Trace, Finalize, JsData)]
pub struct CustomEventData {
    /// The event type
    event_type: GcRefCell<String>,
    /// Whether the event bubbles up through the DOM
    bubbles: bool,
    /// Whether the event is cancelable
    cancelable: bool,
    /// Whether the default action has been prevented
    default_prevented: GcRefCell<bool>,
    /// The current event phase
    event_phase: u16,
    /// Custom data associated with the event
    detail: GcRefCell<JsValue>,
    /// Whether the event is currently being dispatched
    is_dispatching: GcRefCell<bool>,
    /// The current target of the event
    current_target: GcRefCell<Option<JsObject>>,
    /// The original target of the event
    target: GcRefCell<Option<JsObject>>,
}

impl CustomEventData {
    /// Create a new CustomEvent with type and optional event init
    pub fn new(event_type: String, bubbles: bool, cancelable: bool, detail: JsValue) -> Self {
        Self {
            event_type: GcRefCell::new(event_type),
            bubbles,
            cancelable,
            default_prevented: GcRefCell::new(false),
            event_phase: 0, // Event.NONE
            detail: GcRefCell::new(detail),
            is_dispatching: GcRefCell::new(false),
            current_target: GcRefCell::new(None),
            target: GcRefCell::new(None),
        }
    }

    /// Get the event type
    pub fn event_type(&self) -> String {
        self.event_type.borrow().clone()
    }

    /// Get whether the event bubbles
    pub fn bubbles(&self) -> bool {
        self.bubbles
    }

    /// Get whether the event is cancelable
    pub fn cancelable(&self) -> bool {
        self.cancelable
    }

    /// Get whether the default action has been prevented
    pub fn default_prevented(&self) -> bool {
        *self.default_prevented.borrow()
    }

    /// Get the current event phase
    pub fn event_phase(&self) -> u16 {
        self.event_phase
    }

    /// Get the custom detail data
    pub fn detail(&self) -> JsValue {
        self.detail.borrow().clone()
    }

    /// Prevent the default action for this event
    pub fn prevent_default(&self) {
        if self.cancelable {
            *self.default_prevented.borrow_mut() = true;
        }
    }

    /// Stop propagation of the event
    pub fn stop_propagation(&self) {
        // Implementation would set internal flag to stop propagation
        // For now, this is a no-op as we don't have full event dispatching
    }

    /// Stop immediate propagation of the event
    pub fn stop_immediate_propagation(&self) {
        // Implementation would set internal flag to stop immediate propagation
        // For now, this is a no-op as we don't have full event dispatching
    }

    /// Initialize the custom event (for compatibility)
    pub fn init_custom_event(&self, event_type: String, bubbles: bool, cancelable: bool, detail: JsValue) {
        if !*self.is_dispatching.borrow() {
            *self.event_type.borrow_mut() = event_type;
            // Note: bubbles and cancelable are readonly after construction in modern specs
            *self.detail.borrow_mut() = detail;
        }
    }
}

/// The `CustomEvent` object
#[derive(Debug, Trace, Finalize)]
pub struct CustomEvent;

impl CustomEvent {
    /// Create a new CustomEvent
    pub fn create(context: &mut Context, event_type: String, bubbles: bool, cancelable: bool, detail: JsValue) -> JsResult<JsObject> {
        let custom_event_data = CustomEventData::new(event_type, bubbles, cancelable, detail);

        let custom_event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().custom_event().prototype(),
            custom_event_data,
        );

        Ok(custom_event_obj)
    }

    /// Get the type property
    fn get_type(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CustomEvent.prototype.type called on non-object")
        })?;

        if let Some(custom_event_data) = this_obj.downcast_ref::<CustomEventData>() {
            Ok(JsString::from(custom_event_data.event_type()).into())
        } else {
            Err(JsNativeError::typ()
                .with_message("CustomEvent.prototype.type called on non-CustomEvent object")
                .into())
        }
    }

    /// Get the bubbles property
    fn get_bubbles(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CustomEvent.prototype.bubbles called on non-object")
        })?;

        if let Some(custom_event_data) = this_obj.downcast_ref::<CustomEventData>() {
            Ok(JsValue::new(custom_event_data.bubbles()))
        } else {
            Err(JsNativeError::typ()
                .with_message("CustomEvent.prototype.bubbles called on non-CustomEvent object")
                .into())
        }
    }

    /// Get the cancelable property
    fn get_cancelable(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CustomEvent.prototype.cancelable called on non-object")
        })?;

        if let Some(custom_event_data) = this_obj.downcast_ref::<CustomEventData>() {
            Ok(JsValue::new(custom_event_data.cancelable()))
        } else {
            Err(JsNativeError::typ()
                .with_message("CustomEvent.prototype.cancelable called on non-CustomEvent object")
                .into())
        }
    }

    /// Get the defaultPrevented property
    fn get_default_prevented(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CustomEvent.prototype.defaultPrevented called on non-object")
        })?;

        if let Some(custom_event_data) = this_obj.downcast_ref::<CustomEventData>() {
            Ok(JsValue::new(custom_event_data.default_prevented()))
        } else {
            Err(JsNativeError::typ()
                .with_message("CustomEvent.prototype.defaultPrevented called on non-CustomEvent object")
                .into())
        }
    }

    /// Get the eventPhase property
    fn get_event_phase(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CustomEvent.prototype.eventPhase called on non-object")
        })?;

        if let Some(custom_event_data) = this_obj.downcast_ref::<CustomEventData>() {
            Ok(JsValue::new(custom_event_data.event_phase()))
        } else {
            Err(JsNativeError::typ()
                .with_message("CustomEvent.prototype.eventPhase called on non-CustomEvent object")
                .into())
        }
    }

    /// Get the detail property
    fn get_detail(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CustomEvent.prototype.detail called on non-object")
        })?;

        if let Some(custom_event_data) = this_obj.downcast_ref::<CustomEventData>() {
            Ok(custom_event_data.detail())
        } else {
            Err(JsNativeError::typ()
                .with_message("CustomEvent.prototype.detail called on non-CustomEvent object")
                .into())
        }
    }

    /// preventDefault method
    fn prevent_default(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CustomEvent.prototype.preventDefault called on non-object")
        })?;

        if let Some(custom_event_data) = this_obj.downcast_ref::<CustomEventData>() {
            custom_event_data.prevent_default();
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("CustomEvent.prototype.preventDefault called on non-CustomEvent object")
                .into())
        }
    }

    /// stopPropagation method
    fn stop_propagation(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CustomEvent.prototype.stopPropagation called on non-object")
        })?;

        if let Some(custom_event_data) = this_obj.downcast_ref::<CustomEventData>() {
            custom_event_data.stop_propagation();
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("CustomEvent.prototype.stopPropagation called on non-CustomEvent object")
                .into())
        }
    }

    /// stopImmediatePropagation method
    fn stop_immediate_propagation(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CustomEvent.prototype.stopImmediatePropagation called on non-object")
        })?;

        if let Some(custom_event_data) = this_obj.downcast_ref::<CustomEventData>() {
            custom_event_data.stop_immediate_propagation();
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("CustomEvent.prototype.stopImmediatePropagation called on non-CustomEvent object")
                .into())
        }
    }

    /// initCustomEvent method (deprecated but included for compatibility)
    fn init_custom_event(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CustomEvent.prototype.initCustomEvent called on non-object")
        })?;

        if let Some(custom_event_data) = this_obj.downcast_ref::<CustomEventData>() {
            let event_type = args.get_or_undefined(0).to_string(context)?.to_std_string().unwrap_or_default();
            let bubbles = args.get_or_undefined(1).to_boolean();
            let cancelable = args.get_or_undefined(2).to_boolean();
            let detail = args.get_or_undefined(3).clone();

            custom_event_data.init_custom_event(event_type, bubbles, cancelable, detail);
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("CustomEvent.prototype.initCustomEvent called on non-CustomEvent object")
                .into())
        }
    }
}

impl IntrinsicObject for CustomEvent {
    fn init(realm: &Realm) {
        let type_getter = BuiltInBuilder::callable(realm, Self::get_type)
            .name(js_string!("get type"))
            .build();

        let bubbles_getter = BuiltInBuilder::callable(realm, Self::get_bubbles)
            .name(js_string!("get bubbles"))
            .build();

        let cancelable_getter = BuiltInBuilder::callable(realm, Self::get_cancelable)
            .name(js_string!("get cancelable"))
            .build();

        let default_prevented_getter = BuiltInBuilder::callable(realm, Self::get_default_prevented)
            .name(js_string!("get defaultPrevented"))
            .build();

        let event_phase_getter = BuiltInBuilder::callable(realm, Self::get_event_phase)
            .name(js_string!("get eventPhase"))
            .build();

        let detail_getter = BuiltInBuilder::callable(realm, Self::get_detail)
            .name(js_string!("get detail"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("type"),
                Some(type_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("bubbles"),
                Some(bubbles_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("cancelable"),
                Some(cancelable_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("defaultPrevented"),
                Some(default_prevented_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("eventPhase"),
                Some(event_phase_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("detail"),
                Some(detail_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .method(Self::prevent_default, js_string!("preventDefault"), 0)
            .method(Self::stop_propagation, js_string!("stopPropagation"), 0)
            .method(Self::stop_immediate_propagation, js_string!("stopImmediatePropagation"), 0)
            .method(Self::init_custom_event, js_string!("initCustomEvent"), 4)
            // Event phase constants
            .static_property(js_string!("NONE"), 0, Attribute::all())
            .static_property(js_string!("CAPTURING_PHASE"), 1, Attribute::all())
            .static_property(js_string!("AT_TARGET"), 2, Attribute::all())
            .static_property(js_string!("BUBBLING_PHASE"), 3, Attribute::all())
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for CustomEvent {
    const NAME: JsString = js_string!("CustomEvent");
}

impl BuiltInConstructor for CustomEvent {
    const LENGTH: usize = 1;
    const P: usize = 0;
    const SP: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::custom_event;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::custom_event,
            context,
        )?;

        // CustomEvent requires type parameter
        let type_arg = args.get_or_undefined(0);
        let event_type = if type_arg.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("CustomEvent constructor requires type parameter")
                .into());
        } else {
            type_arg.to_string(context)?.to_std_string().unwrap_or_default()
        };

        // Get optional event init dictionary
        let event_init = args.get_or_undefined(1);
        let (bubbles, cancelable, detail) = if event_init.is_object() {
            let init_obj = event_init.as_object().unwrap();

            let bubbles = init_obj.get(js_string!("bubbles"), context)
                .unwrap_or(JsValue::undefined())
                .to_boolean();

            let cancelable = init_obj.get(js_string!("cancelable"), context)
                .unwrap_or(JsValue::undefined())
                .to_boolean();

            let detail_val = init_obj.get(js_string!("detail"), context)
                .unwrap_or(JsValue::undefined());
            let detail = if detail_val.is_undefined() { JsValue::null() } else { detail_val };

            (bubbles, cancelable, detail)
        } else {
            (false, false, JsValue::null())
        };

        let custom_event_data = CustomEventData::new(event_type, bubbles, cancelable, detail);

        let custom_event_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            custom_event_data,
        );

        Ok(custom_event_obj.into())
    }
}