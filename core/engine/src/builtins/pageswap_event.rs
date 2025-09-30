//! PageSwapEvent Web API implementation for Boa
//!
//! Native implementation of PageSwapEvent standard (Chrome 124+)
//! https://wicg.github.io/navigation-api/#pageswapevent

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
use std::sync::{Arc, Mutex};

/// JavaScript `PageSwapEvent` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct PageSwapEvent;

impl IntrinsicObject for PageSwapEvent {
    fn init(realm: &Realm) {
        let activation_func = BuiltInBuilder::callable(realm, get_activation)
            .name(js_string!("get activation"))
            .build();

        let view_transition_func = BuiltInBuilder::callable(realm, get_view_transition)
            .name(js_string!("get viewTransition"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("activation"),
                Some(activation_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("viewTransition"),
                Some(view_transition_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for PageSwapEvent {
    const NAME: JsString = StaticJsStrings::PAGESWAP_EVENT;
}

impl BuiltInConstructor for PageSwapEvent {
    const LENGTH: usize = 1;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::pageswap_event;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::pageswap_event,
            context,
        )?;

        let event_type = args.get_or_undefined(0);
        let event_init = args.get(1).cloned().unwrap_or(JsValue::undefined());

        // Validate event type
        let type_string = event_type.to_string(context)?;

        let pageswap_data = PageSwapEventData::new(
            type_string.to_std_string_escaped(),
            event_init,
        );

        let pageswap_event = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            pageswap_data,
        );

        Ok(pageswap_event.into())
    }
}

/// Internal data for PageSwapEvent objects
#[derive(Debug, Trace, Finalize, JsData)]
pub struct PageSwapEventData {
    #[unsafe_ignore_trace]
    event_type: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    activation: Arc<Mutex<Option<JsValue>>>,
    #[unsafe_ignore_trace]
    view_transition: Arc<Mutex<Option<JsValue>>>,
    #[unsafe_ignore_trace]
    bubbles: Arc<Mutex<bool>>,
    #[unsafe_ignore_trace]
    cancelable: Arc<Mutex<bool>>,
}

impl PageSwapEventData {
    fn new(event_type: String, _event_init: JsValue) -> Self {
        let activation = None;
        let view_transition = None;
        let bubbles = false;
        let cancelable = false;

        // Parse event_init if it's an object
        // Note: In a real implementation, we would properly parse the event_init dict
        // For now, we just create default values

        Self {
            event_type: Arc::new(Mutex::new(event_type)),
            activation: Arc::new(Mutex::new(activation)),
            view_transition: Arc::new(Mutex::new(view_transition)),
            bubbles: Arc::new(Mutex::new(bubbles)),
            cancelable: Arc::new(Mutex::new(cancelable)),
        }
    }

    pub fn get_type(&self) -> String {
        self.event_type.lock().unwrap().clone()
    }

    pub fn get_activation(&self) -> Option<JsValue> {
        self.activation.lock().unwrap().clone()
    }

    pub fn get_view_transition(&self) -> Option<JsValue> {
        self.view_transition.lock().unwrap().clone()
    }

    pub fn set_activation(&self, activation: Option<JsValue>) {
        *self.activation.lock().unwrap() = activation;
    }

    pub fn set_view_transition(&self, view_transition: Option<JsValue>) {
        *self.view_transition.lock().unwrap() = view_transition;
    }

    pub fn is_bubbles(&self) -> bool {
        *self.bubbles.lock().unwrap()
    }

    pub fn is_cancelable(&self) -> bool {
        *self.cancelable.lock().unwrap()
    }
}

/// `PageSwapEvent.prototype.activation` getter
fn get_activation(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("PageSwapEvent.prototype.activation called on non-object")
    })?;

    if let Some(pageswap_event) = this_obj.downcast_ref::<PageSwapEventData>() {
        Ok(pageswap_event.get_activation().unwrap_or(JsValue::null()))
    } else {
        Err(JsNativeError::typ()
            .with_message("PageSwapEvent.prototype.activation called on non-PageSwapEvent object")
            .into())
    }
}

/// `PageSwapEvent.prototype.viewTransition` getter
fn get_view_transition(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("PageSwapEvent.prototype.viewTransition called on non-object")
    })?;

    if let Some(pageswap_event) = this_obj.downcast_ref::<PageSwapEventData>() {
        Ok(pageswap_event.get_view_transition().unwrap_or(JsValue::null()))
    } else {
        Err(JsNativeError::typ()
            .with_message("PageSwapEvent.prototype.viewTransition called on non-PageSwapEvent object")
            .into())
    }
}

/// Navigation activation entry for PageSwap events
#[derive(Debug, Trace, Finalize, JsData)]
pub struct NavigationActivationData {
    #[unsafe_ignore_trace]
    entry: Arc<Mutex<JsValue>>,
    #[unsafe_ignore_trace]
    from: Arc<Mutex<Option<JsValue>>>,
    #[unsafe_ignore_trace]
    navigation_type: Arc<Mutex<String>>,
}

impl NavigationActivationData {
    pub fn new(entry: JsValue, from: Option<JsValue>, navigation_type: String) -> Self {
        Self {
            entry: Arc::new(Mutex::new(entry)),
            from: Arc::new(Mutex::new(from)),
            navigation_type: Arc::new(Mutex::new(navigation_type)),
        }
    }

    pub fn get_entry(&self) -> JsValue {
        self.entry.lock().unwrap().clone()
    }

    pub fn get_from(&self) -> Option<JsValue> {
        self.from.lock().unwrap().clone()
    }

    pub fn get_navigation_type(&self) -> String {
        self.navigation_type.lock().unwrap().clone()
    }
}

/// Create a PageSwap event for navigation
pub fn create_pageswap_event(
    context: &mut Context,
    activation: Option<JsValue>,
    view_transition: Option<JsValue>,
) -> JsResult<JsValue> {
    // Create event init dictionary
    let event_init = JsObject::default();

    if let Some(activation_val) = activation {
        event_init.define_property_or_throw(
            js_string!("activation"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(activation_val)
                .build(),
            context,
        )?;
    }

    if let Some(view_transition_val) = view_transition {
        event_init.define_property_or_throw(
            js_string!("viewTransition"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(view_transition_val)
                .build(),
            context,
        )?;
    }

    // Create PageSwapEvent
    let args = [JsValue::from(js_string!("pageswap")), event_init.into()];
    PageSwapEvent::constructor(&JsValue::undefined(), &args, context)
}

/// Dispatch a pageswap event on the window object
pub fn dispatch_pageswap_event(
    context: &mut Context,
    window_obj: &JsObject,
    activation: Option<JsValue>,
    view_transition: Option<JsValue>,
) -> JsResult<()> {
    // Create the pageswap event
    let pageswap_event = create_pageswap_event(context, activation, view_transition)?;

    // Get event listeners for 'pageswap' from window
    if let Ok(listeners_val) = window_obj.get(js_string!("__pageswap_listeners"), context) {
        if listeners_val.is_object() {
            if let Some(listeners_obj) = listeners_val.as_object() {
                // Get the array of listeners
                if let Ok(length_val) = listeners_obj.get(js_string!("length"), context) {
                    if let Some(length) = length_val.as_number() {
                        let len = length as usize;

                        // Call each listener
                        for i in 0..len {
                            if let Ok(listener) = listeners_obj.get(i, context) {
                                if listener.is_callable() {
                                    let _ = listener.as_callable().unwrap().call(
                                        &window_obj.clone().into(),
                                        &[pageswap_event.clone()],
                                        context,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}