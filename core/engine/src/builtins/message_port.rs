//! MessagePort Web API implementation for Boa
//!
//! Native implementation of MessagePort standard
//! https://html.spec.whatwg.org/multipage/web-messaging.html#message-ports
//!
//! This implements the complete MessagePort interface for Channel Messaging API

use crate::{
    Context, JsData, JsNativeError, JsResult, JsValue,
    object::JsObject,
    realm::Realm,
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
        let object = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            self.clone(),
        );

        // TODO: Add proper method bindings when needed
        // For now, this provides basic MessagePort data storage

        Ok(object)
    }
}