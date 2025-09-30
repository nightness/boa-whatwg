//! WHATWG RTCDataChannel API implementation for WebRTC data channel messaging.
//!
//! Implementation of the RTCDataChannel interface according to:
//! https://w3c.github.io/webrtc-pc/#rtcdatachannel
//!
//! The RTCDataChannel interface represents a data channel between two peers
//! of an RTCPeerConnection. It provides methods to send data and receive
//! data asynchronously.

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    js_string,
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    Context, JsArgs, JsData, JsResult, JsString, JsValue,
};
use boa_gc::{Finalize, Trace};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    },
};
use tokio::sync::Mutex as TokioMutex;
use webrtc::{
    data_channel::{
        data_channel_message::DataChannelMessage,
        data_channel_state::RTCDataChannelState,
        RTCDataChannel,
    },
};
use bytes::Bytes;

/// RTCDataChannel states according to WHATWG specification
#[derive(Debug, Clone, PartialEq, Eq, Trace, Finalize)]
#[repr(u32)]
pub enum RTCDataChannelStateEnum {
    Connecting = 0,
    Open = 1,
    Closing = 2,
    Closed = 3,
}

impl From<RTCDataChannelStateEnum> for JsValue {
    fn from(state: RTCDataChannelStateEnum) -> Self {
        let state_str = match state {
            RTCDataChannelStateEnum::Connecting => "connecting",
            RTCDataChannelStateEnum::Open => "open",
            RTCDataChannelStateEnum::Closing => "closing",
            RTCDataChannelStateEnum::Closed => "closed",
        };
        JsValue::from(js_string!(state_str))
    }
}

impl From<RTCDataChannelState> for RTCDataChannelStateEnum {
    fn from(state: RTCDataChannelState) -> Self {
        match state {
            RTCDataChannelState::Connecting => RTCDataChannelStateEnum::Connecting,
            RTCDataChannelState::Open => RTCDataChannelStateEnum::Open,
            RTCDataChannelState::Closing => RTCDataChannelStateEnum::Closing,
            RTCDataChannelState::Closed => RTCDataChannelStateEnum::Closed,
            RTCDataChannelState::Unspecified => RTCDataChannelStateEnum::Connecting,
        }
    }
}

/// JavaScript representation of an RTCDataChannel
#[derive(Trace, Finalize, JsData)]
pub struct RTCDataChannelData {
    #[unsafe_ignore_trace]
    channel: Option<Arc<RTCDataChannel>>,
    channel_id: String,
    label: String,
    ordered: bool,
    max_packet_life_time: Option<u16>,
    max_retransmits: Option<u16>,
    protocol: String,
    negotiated: bool,
    id: Option<u16>,
    state: RTCDataChannelStateEnum,
}

impl std::fmt::Debug for RTCDataChannelData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RTCDataChannelData")
            .field("channel_id", &self.channel_id)
            .field("label", &self.label)
            .field("ordered", &self.ordered)
            .field("max_packet_life_time", &self.max_packet_life_time)
            .field("max_retransmits", &self.max_retransmits)
            .field("protocol", &self.protocol)
            .field("negotiated", &self.negotiated)
            .field("id", &self.id)
            .field("state", &self.state)
            .finish()
    }
}

impl RTCDataChannelData {
    /// Create a new RTCDataChannelData instance
    pub fn new(
        channel: Arc<RTCDataChannel>,
        channel_id: String,
        label: String,
    ) -> Self {
        Self {
            channel: Some(channel),
            channel_id,
            label,
            ordered: true,
            max_packet_life_time: None,
            max_retransmits: None,
            protocol: String::new(),
            negotiated: false,
            id: None,
            state: RTCDataChannelStateEnum::Connecting,
        }
    }

    /// Get the data channel reference
    pub fn channel(&self) -> Option<&Arc<RTCDataChannel>> {
        self.channel.as_ref()
    }

    /// Get the channel ID
    pub fn channel_id(&self) -> &str {
        &self.channel_id
    }

    /// Update the data channel state
    pub fn set_state(&mut self, state: RTCDataChannelStateEnum) {
        self.state = state;
    }
}

/// Global DataChannel manager for managing data channels
pub struct DataChannelManager {
    channels: Arc<TokioMutex<HashMap<String, Arc<RTCDataChannel>>>>,
    channel_counter: AtomicU32,
}

impl std::fmt::Debug for DataChannelManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataChannelManager")
            .field("channels", &"Arc<TokioMutex<HashMap<String, Arc<RTCDataChannel>>>>")
            .field("channel_counter", &self.channel_counter)
            .finish()
    }
}

impl DataChannelManager {
    /// Create a new DataChannel manager
    pub fn new() -> Self {
        Self {
            channels: Arc::new(TokioMutex::new(HashMap::new())),
            channel_counter: AtomicU32::new(0),
        }
    }

    /// Generate a unique channel ID
    pub fn generate_channel_id(&self) -> String {
        let id = self.channel_counter.fetch_add(1, Ordering::SeqCst);
        format!("datachannel_{}", id)
    }

    /// Register a data channel
    pub async fn register_channel(&self, id: String, channel: Arc<RTCDataChannel>) {
        self.channels.lock().await.insert(id, channel);
    }

    /// Get a data channel by ID
    pub async fn get_channel(&self, id: &str) -> Option<Arc<RTCDataChannel>> {
        self.channels.lock().await.get(id).cloned()
    }

    /// Remove a data channel
    pub async fn remove_channel(&self, id: &str) -> Option<Arc<RTCDataChannel>> {
        self.channels.lock().await.remove(id)
    }
}

impl Default for DataChannelManager {
    fn default() -> Self {
        Self::new()
    }
}

/// The `RTCDataChannel` builtin object
#[derive(Debug, Clone, Trace, Finalize)]
pub struct RTCDataChannelBuiltin;

impl IntrinsicObject for RTCDataChannelBuiltin {
    fn init(realm: &Realm) {

        let get_label = BuiltInBuilder::callable(realm, Self::get_label)
            .name(js_string!("get label"))
            .build();

        let get_ordered = BuiltInBuilder::callable(realm, Self::get_ordered)
            .name(js_string!("get ordered"))
            .build();

        let get_max_packet_life_time = BuiltInBuilder::callable(realm, Self::get_max_packet_life_time)
            .name(js_string!("get maxPacketLifeTime"))
            .build();

        let get_max_retransmits = BuiltInBuilder::callable(realm, Self::get_max_retransmits)
            .name(js_string!("get maxRetransmits"))
            .build();

        let get_protocol = BuiltInBuilder::callable(realm, Self::get_protocol)
            .name(js_string!("get protocol"))
            .build();

        let get_negotiated = BuiltInBuilder::callable(realm, Self::get_negotiated)
            .name(js_string!("get negotiated"))
            .build();

        let get_id = BuiltInBuilder::callable(realm, Self::get_id)
            .name(js_string!("get id"))
            .build();

        let get_ready_state = BuiltInBuilder::callable(realm, Self::get_ready_state)
            .name(js_string!("get readyState"))
            .build();

        let get_buffered_amount = BuiltInBuilder::callable(realm, Self::get_buffered_amount)
            .name(js_string!("get bufferedAmount"))
            .build();

        let get_buffered_amount_low_threshold = BuiltInBuilder::callable(realm, Self::get_buffered_amount_low_threshold)
            .name(js_string!("get bufferedAmountLowThreshold"))
            .build();

        let set_buffered_amount_low_threshold = BuiltInBuilder::callable(realm, Self::set_buffered_amount_low_threshold)
            .name(js_string!("set bufferedAmountLowThreshold"))
            .build();

        let get_binary_type = BuiltInBuilder::callable(realm, Self::get_binary_type)
            .name(js_string!("get binaryType"))
            .build();

        let set_binary_type = BuiltInBuilder::callable(realm, Self::set_binary_type)
            .name(js_string!("set binaryType"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                js_string!("label"),
                get_label,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("ordered"),
                get_ordered,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("maxPacketLifeTime"),
                get_max_packet_life_time,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("maxRetransmits"),
                get_max_retransmits,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("protocol"),
                get_protocol,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("negotiated"),
                get_negotiated,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("id"),
                get_id,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("readyState"),
                get_ready_state,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("bufferedAmount"),
                get_buffered_amount,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("bufferedAmountLowThreshold"),
                get_buffered_amount_low_threshold.clone(),
                Attribute::CONFIGURABLE | Attribute::WRITABLE,
            )
            .property(
                js_string!("binaryType"),
                get_binary_type.clone(),
                Attribute::CONFIGURABLE | Attribute::WRITABLE,
            )
            .method(Self::close, js_string!("close"), 0)
            .method(Self::send, js_string!("send"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for RTCDataChannelBuiltin {
    const NAME: JsString = StaticJsStrings::RTC_DATA_CHANNEL;
}

impl BuiltInConstructor for RTCDataChannelBuiltin {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::rtc_data_channel;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("RTCDataChannel constructor cannot be called directly")
            .into())
    }
}

impl RTCDataChannelBuiltin {
    /// `RTCDataChannel.prototype.label` getter
    pub(crate) fn get_label(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCDataChannelData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on wrong object type")
        })?;

        Ok(JsValue::from(js_string!(data.label.clone())))
    }

    /// `RTCDataChannel.prototype.ordered` getter
    pub(crate) fn get_ordered(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCDataChannelData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on wrong object type")
        })?;

        Ok(JsValue::from(data.ordered))
    }

    /// `RTCDataChannel.prototype.maxPacketLifeTime` getter
    pub(crate) fn get_max_packet_life_time(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCDataChannelData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on wrong object type")
        })?;

        match data.max_packet_life_time {
            Some(time) => Ok(JsValue::from(time)),
            None => Ok(JsValue::null()),
        }
    }

    /// `RTCDataChannel.prototype.maxRetransmits` getter
    pub(crate) fn get_max_retransmits(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCDataChannelData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on wrong object type")
        })?;

        match data.max_retransmits {
            Some(retransmits) => Ok(JsValue::from(retransmits)),
            None => Ok(JsValue::null()),
        }
    }

    /// `RTCDataChannel.prototype.protocol` getter
    pub(crate) fn get_protocol(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCDataChannelData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on wrong object type")
        })?;

        Ok(JsValue::from(js_string!(data.protocol.clone())))
    }

    /// `RTCDataChannel.prototype.negotiated` getter
    pub(crate) fn get_negotiated(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCDataChannelData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on wrong object type")
        })?;

        Ok(JsValue::from(data.negotiated))
    }

    /// `RTCDataChannel.prototype.id` getter
    pub(crate) fn get_id(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCDataChannelData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on wrong object type")
        })?;

        match data.id {
            Some(id) => Ok(JsValue::from(id)),
            None => Ok(JsValue::null()),
        }
    }

    /// `RTCDataChannel.prototype.readyState` getter
    pub(crate) fn get_ready_state(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCDataChannelData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on wrong object type")
        })?;

        Ok(data.state.clone().into())
    }

    /// `RTCDataChannel.prototype.bufferedAmount` getter
    pub(crate) fn get_buffered_amount(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCDataChannelData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on wrong object type")
        })?;

        // For now, return 0 as a placeholder
        // Real implementation would get the actual buffered amount from the WebRTC channel
        Ok(JsValue::from(0))
    }

    /// `RTCDataChannel.prototype.bufferedAmountLowThreshold` getter
    pub(crate) fn get_buffered_amount_low_threshold(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on non-object")
        })?;

        let _data = obj.downcast_ref::<RTCDataChannelData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on wrong object type")
        })?;

        // For now, return 0 as a placeholder
        // Real implementation would store and return the actual threshold
        Ok(JsValue::from(0))
    }

    /// `RTCDataChannel.prototype.bufferedAmountLowThreshold` setter
    pub(crate) fn set_buffered_amount_low_threshold(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on non-object")
        })?;

        let _data = obj.downcast_ref::<RTCDataChannelData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on wrong object type")
        })?;

        let _threshold = args.get_or_undefined(0).to_u32(_context)?;

        // For now, this is a no-op
        // Real implementation would store the threshold and use it for events

        Ok(JsValue::undefined())
    }

    /// `RTCDataChannel.prototype.binaryType` getter
    pub(crate) fn get_binary_type(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on non-object")
        })?;

        let _data = obj.downcast_ref::<RTCDataChannelData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on wrong object type")
        })?;

        // Default binary type is "blob"
        Ok(JsValue::from(js_string!("blob")))
    }

    /// `RTCDataChannel.prototype.binaryType` setter
    pub(crate) fn set_binary_type(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on non-object")
        })?;

        let _data = obj.downcast_ref::<RTCDataChannelData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on wrong object type")
        })?;

        let binary_type = args.get_or_undefined(0).to_string(context)?;

        // Validate binary type
        if binary_type.to_std_string_escaped() != "blob" && binary_type.to_std_string_escaped() != "arraybuffer" {
            return Err(JsNativeError::typ()
                .with_message("Invalid binaryType - must be 'blob' or 'arraybuffer'")
                .into());
        }

        // For now, this is a no-op
        // Real implementation would store the binary type preference

        Ok(JsValue::undefined())
    }

    /// `RTCDataChannel.prototype.close()`
    pub(crate) fn close(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCDataChannelData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on wrong object type")
        })?;

        if let Some(channel) = data.channel() {
            // Close the data channel
            // Real implementation would use: channel.close().await;
            // For now, just log the operation
            println!("Closing data channel: {}", data.channel_id());
        }

        Ok(JsValue::undefined())
    }

    /// `RTCDataChannel.prototype.send()`
    pub(crate) fn send(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCDataChannelData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCDataChannel method called on wrong object type")
        })?;

        let message_data = args.get_or_undefined(0);

        if let Some(channel) = data.channel() {
            // Convert JavaScript value to bytes
            let bytes = if message_data.is_string() {
                let string_data = message_data.to_string(context)?;
                Bytes::from(string_data.to_std_string_escaped().into_bytes())
            } else {
                // For now, convert other types to string representation
                let string_data = message_data.to_string(context)?;
                Bytes::from(string_data.to_std_string_escaped().into_bytes())
            };

            // Create data channel message
            let message = DataChannelMessage {
                data: bytes,
                is_string: message_data.is_string(),
            };

            // Send the message
            // Real implementation would use: channel.send(&message).await;
            // For now, just log the operation
            println!("Sending message on data channel {}: {} bytes",
                     data.channel_id(), message.data.len());
        }

        Ok(JsValue::undefined())
    }
}