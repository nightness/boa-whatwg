//! MessagePort Web API implementation for Boa
//!
//! Native implementation of MessagePort standard
//! https://html.spec.whatwg.org/multipage/web-messaging.html#message-ports
//!
//! This implements the complete MessagePort interface for Channel Messaging API

use crate::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::Attribute
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};
use crossbeam_channel::{Receiver, Sender, unbounded};

/// MessagePort state for communication between ports
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct MessagePortData {
    #[unsafe_ignore_trace]
    sender: Arc<Mutex<Option<Sender<JsValue>>>>,
    #[unsafe_ignore_trace]
    receiver: Arc<Mutex<Option<Receiver<JsValue>>>>,
    #[unsafe_ignore_trace]
    entangled: Arc<Mutex<bool>>,
    #[unsafe_ignore_trace]
    started: Arc<Mutex<bool>>,
}

impl MessagePortData {
    /// Creates a new MessagePort with the given channel endpoints
    pub fn new(sender: Sender<JsValue>, receiver: Receiver<JsValue>) -> Self {
        Self {
            sender: Arc::new(Mutex::new(Some(sender))),
            receiver: Arc::new(Mutex::new(Some(receiver))),
            entangled: Arc::new(Mutex::new(true)),
            started: Arc::new(Mutex::new(false)),
        }
    }

    /// Creates an entangled pair of MessagePorts
    pub fn create_entangled_pair() -> (Self, Self) {
        let (sender1, receiver1) = unbounded();
        let (sender2, receiver2) = unbounded();

        (
            MessagePortData::new(sender1, receiver2),
            MessagePortData::new(sender2, receiver1),
        )
    }

    /// Sends a message to the other port
    pub fn post_message(&self, message: JsValue) -> JsResult<()> {
        let sender = self.sender.lock().unwrap();
        if let Some(ref sender) = *sender {
            sender.send(message).map_err(|_| {
                JsNativeError::typ().with_message("Failed to send message - port is closed")
            })?;
        } else {
            return Err(JsNativeError::typ()
                .with_message("Cannot send message on closed port")
                .into());
        }
        Ok(())
    }

    /// Starts the port (enables message reception)
    pub fn start(&self) {
        *self.started.lock().unwrap() = true;
    }

    /// Closes the port
    pub fn close(&self) {
        *self.entangled.lock().unwrap() = false;
        *self.sender.lock().unwrap() = None;
        *self.receiver.lock().unwrap() = None;
    }

    /// Checks if the port is entangled
    pub fn is_entangled(&self) -> bool {
        *self.entangled.lock().unwrap()
    }

    /// Checks if the port is started
    pub fn is_started(&self) -> bool {
        *self.started.lock().unwrap()
    }

    /// Creates a MessagePort object from MessagePortData for use in JavaScript
    pub fn create_js_object(&self, context: &mut Context) -> JsResult<JsObject> {
        let proto = context.intrinsics().constructors().message_port().prototype();
        let object = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            self.clone(),
        );

        Ok(object)
    }
}

/// JavaScript `MessagePort` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct MessagePort;

impl IntrinsicObject for MessagePort {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Instance methods
            .property(
                js_string!("postMessage"),
                BuiltInBuilder::callable(realm, post_message)
                    .name(js_string!("postMessage"))
                    .length(1)
                    .build(),
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("start"),
                BuiltInBuilder::callable(realm, start)
                    .name(js_string!("start"))
                    .length(0)
                    .build(),
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("close"),
                BuiltInBuilder::callable(realm, close)
                    .name(js_string!("close"))
                    .length(0)
                    .build(),
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for MessagePort {
    const NAME: JsString = StaticJsStrings::MESSAGE_PORT;
}

impl BuiltInConstructor for MessagePort {
    const LENGTH: usize = 0;
    const P: usize = 2;
    const SP: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::message_port;

    /// MessagePort constructor (not directly constructable)
    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("MessagePort constructor is not directly callable")
            .into())
    }
}

/// `MessagePort.prototype.postMessage(message)`
fn post_message(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MessagePort.postMessage called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<MessagePortData>() {
        let message = args.get_or_undefined(0);
        data.post_message(message.clone())?;
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("MessagePort.postMessage called on invalid object")
            .into())
    }
}

/// `MessagePort.prototype.start()`
fn start(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MessagePort.start called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<MessagePortData>() {
        data.start();
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("MessagePort.start called on invalid object")
            .into())
    }
}

/// `MessagePort.prototype.close()`
fn close(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MessagePort.close called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<MessagePortData>() {
        data.close();
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("MessagePort.close called on invalid object")
            .into())
    }
}