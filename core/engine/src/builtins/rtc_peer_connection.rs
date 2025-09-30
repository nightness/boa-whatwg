//! WHATWG RTCPeerConnection API implementation for WebRTC peer-to-peer networking.
//!
//! Implementation of the RTCPeerConnection interface according to:
//! https://w3c.github.io/webrtc-pc/
//!
//! The RTCPeerConnection interface represents a connection between the local computer
//! and a remote peer. It provides methods to connect to a remote peer, to maintain
//! and monitor the connection, and to close the connection once it's no longer needed.

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
    api::{
        interceptor_registry::register_default_interceptors,
        media_engine::MediaEngine,
        APIBuilder,
        API,
    },
    data_channel::RTCDataChannel,
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration,
        peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription,
        RTCPeerConnection,
    },
};

/// RTCPeerConnection states according to WHATWG specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum RTCPeerConnectionStateEnum {
    New = 0,
    Connecting = 1,
    Connected = 2,
    Disconnected = 3,
    Failed = 4,
    Closed = 5,
}

impl From<RTCPeerConnectionStateEnum> for JsValue {
    fn from(state: RTCPeerConnectionStateEnum) -> Self {
        let state_str = match state {
            RTCPeerConnectionStateEnum::New => "new",
            RTCPeerConnectionStateEnum::Connecting => "connecting",
            RTCPeerConnectionStateEnum::Connected => "connected",
            RTCPeerConnectionStateEnum::Disconnected => "disconnected",
            RTCPeerConnectionStateEnum::Failed => "failed",
            RTCPeerConnectionStateEnum::Closed => "closed",
        };
        JsValue::from(js_string!(state_str))
    }
}

impl From<RTCPeerConnectionState> for RTCPeerConnectionStateEnum {
    fn from(state: RTCPeerConnectionState) -> Self {
        match state {
            RTCPeerConnectionState::New => RTCPeerConnectionStateEnum::New,
            RTCPeerConnectionState::Connecting => RTCPeerConnectionStateEnum::Connecting,
            RTCPeerConnectionState::Connected => RTCPeerConnectionStateEnum::Connected,
            RTCPeerConnectionState::Disconnected => RTCPeerConnectionStateEnum::Disconnected,
            RTCPeerConnectionState::Failed => RTCPeerConnectionStateEnum::Failed,
            RTCPeerConnectionState::Closed => RTCPeerConnectionStateEnum::Closed,
            _ => RTCPeerConnectionStateEnum::New,
        }
    }
}

/// ICE connection states according to WHATWG specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum RTCIceConnectionState {
    New = 0,
    Checking = 1,
    Connected = 2,
    Completed = 3,
    Failed = 4,
    Disconnected = 5,
    Closed = 6,
}

impl From<RTCIceConnectionState> for JsValue {
    fn from(state: RTCIceConnectionState) -> Self {
        let state_str = match state {
            RTCIceConnectionState::New => "new",
            RTCIceConnectionState::Checking => "checking",
            RTCIceConnectionState::Connected => "connected",
            RTCIceConnectionState::Completed => "completed",
            RTCIceConnectionState::Failed => "failed",
            RTCIceConnectionState::Disconnected => "disconnected",
            RTCIceConnectionState::Closed => "closed",
        };
        JsValue::from(js_string!(state_str))
    }
}

/// Signaling states according to WHATWG specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum RTCSignalingState {
    Stable = 0,
    HaveLocalOffer = 1,
    HaveRemoteOffer = 2,
    HaveLocalPranswer = 3,
    HaveRemotePranswer = 4,
    Closed = 5,
}

impl From<RTCSignalingState> for JsValue {
    fn from(state: RTCSignalingState) -> Self {
        let state_str = match state {
            RTCSignalingState::Stable => "stable",
            RTCSignalingState::HaveLocalOffer => "have-local-offer",
            RTCSignalingState::HaveRemoteOffer => "have-remote-offer",
            RTCSignalingState::HaveLocalPranswer => "have-local-pranswer",
            RTCSignalingState::HaveRemotePranswer => "have-remote-pranswer",
            RTCSignalingState::Closed => "closed",
        };
        JsValue::from(js_string!(state_str))
    }
}

/// Internal RTCPeerConnection state management
#[derive(Debug, Clone, Trace, Finalize)]
pub struct RTCPeerConnectionData {
    /// Unique identifier for this peer connection
    #[unsafe_ignore_trace]
    id: String,
    /// Current connection state
    #[unsafe_ignore_trace]
    connection_state: Arc<AtomicU32>,
    /// Current ICE connection state
    #[unsafe_ignore_trace]
    ice_connection_state: Arc<AtomicU32>,
    /// Current signaling state
    #[unsafe_ignore_trace]
    signaling_state: Arc<AtomicU32>,
    /// WebRTC runtime for async operations
    #[unsafe_ignore_trace]
    runtime: Arc<tokio::runtime::Runtime>,
}

impl RTCPeerConnectionData {
    /// Create new RTCPeerConnection data
    pub fn new() -> anyhow::Result<Self> {
        let id = format!("rtc_pc_{}", rand::random::<u32>());
        let runtime = tokio::runtime::Runtime::new()?;

        Ok(Self {
            id,
            connection_state: Arc::new(AtomicU32::new(RTCPeerConnectionStateEnum::New as u32)),
            ice_connection_state: Arc::new(AtomicU32::new(RTCIceConnectionState::New as u32)),
            signaling_state: Arc::new(AtomicU32::new(RTCSignalingState::Stable as u32)),
            runtime: Arc::new(runtime),
        })
    }

    /// Get the peer connection ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the current connection state
    pub fn connection_state(&self) -> RTCPeerConnectionStateEnum {
        match self.connection_state.load(Ordering::Relaxed) {
            0 => RTCPeerConnectionStateEnum::New,
            1 => RTCPeerConnectionStateEnum::Connecting,
            2 => RTCPeerConnectionStateEnum::Connected,
            3 => RTCPeerConnectionStateEnum::Disconnected,
            4 => RTCPeerConnectionStateEnum::Failed,
            5 => RTCPeerConnectionStateEnum::Closed,
            _ => RTCPeerConnectionStateEnum::New,
        }
    }

    /// Set the connection state
    pub fn set_connection_state(&self, state: RTCPeerConnectionStateEnum) {
        self.connection_state.store(state as u32, Ordering::Relaxed);
    }

    /// Get the current ICE connection state
    pub fn ice_connection_state(&self) -> RTCIceConnectionState {
        match self.ice_connection_state.load(Ordering::Relaxed) {
            0 => RTCIceConnectionState::New,
            1 => RTCIceConnectionState::Checking,
            2 => RTCIceConnectionState::Connected,
            3 => RTCIceConnectionState::Completed,
            4 => RTCIceConnectionState::Failed,
            5 => RTCIceConnectionState::Disconnected,
            6 => RTCIceConnectionState::Closed,
            _ => RTCIceConnectionState::New,
        }
    }

    /// Set the ICE connection state
    pub fn set_ice_connection_state(&self, state: RTCIceConnectionState) {
        self.ice_connection_state.store(state as u32, Ordering::Relaxed);
    }

    /// Get the current signaling state
    pub fn signaling_state(&self) -> RTCSignalingState {
        match self.signaling_state.load(Ordering::Relaxed) {
            0 => RTCSignalingState::Stable,
            1 => RTCSignalingState::HaveLocalOffer,
            2 => RTCSignalingState::HaveRemoteOffer,
            3 => RTCSignalingState::HaveLocalPranswer,
            4 => RTCSignalingState::HaveRemotePranswer,
            5 => RTCSignalingState::Closed,
            _ => RTCSignalingState::Stable,
        }
    }

    /// Set the signaling state
    pub fn set_signaling_state(&self, state: RTCSignalingState) {
        self.signaling_state.store(state as u32, Ordering::Relaxed);
    }

    /// Get the runtime for async operations
    pub fn runtime(&self) -> &Arc<tokio::runtime::Runtime> {
        &self.runtime
    }
}

/// Global WebRTC manager for managing peer connections
pub struct WebRTCManager {
    peer_connections: Arc<TokioMutex<HashMap<String, Arc<RTCPeerConnection>>>>,
    data_channels: Arc<TokioMutex<HashMap<String, Arc<RTCDataChannel>>>>,
    api: Arc<webrtc::api::API>,
}

impl std::fmt::Debug for WebRTCManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebRTCManager")
            .field("peer_connections", &"Arc<TokioMutex<HashMap<String, Arc<RTCPeerConnection>>>>")
            .field("data_channels", &"Arc<TokioMutex<HashMap<String, Arc<RTCDataChannel>>>>")
            .field("api", &"Arc<webrtc::api::API>")
            .finish()
    }
}

impl WebRTCManager {
    /// Create a new WebRTC manager
    pub fn new() -> anyhow::Result<Self> {
        // Create WebRTC API with proper media engine and interceptors
        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs()?;

        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut media_engine)?;

        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .build();

        Ok(Self {
            peer_connections: Arc::new(TokioMutex::new(HashMap::new())),
            data_channels: Arc::new(TokioMutex::new(HashMap::new())),
            api: Arc::new(api),
        })
    }

    /// Create a new peer connection
    pub async fn create_peer_connection(
        &self,
        id: String,
        config: RTCConfiguration,
    ) -> anyhow::Result<Arc<RTCPeerConnection>> {
        let pc = self.api.new_peer_connection(config).await?;
        let pc_arc = Arc::new(pc);

        self.peer_connections
            .lock()
            .await
            .insert(id, Arc::clone(&pc_arc));

        Ok(pc_arc)
    }

    /// Get a peer connection by ID
    pub async fn get_peer_connection(&self, id: &str) -> Option<Arc<RTCPeerConnection>> {
        self.peer_connections
            .lock()
            .await
            .get(id)
            .cloned()
    }

    /// Remove a peer connection
    pub async fn remove_peer_connection(&self, id: &str) -> Option<Arc<RTCPeerConnection>> {
        self.peer_connections
            .lock()
            .await
            .remove(id)
    }
}

/// RTCPeerConnection builtin object
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct RTCPeerConnectionBuiltin {
    data: RTCPeerConnectionData,
}

impl IntrinsicObject for RTCPeerConnectionBuiltin {
    fn init(realm: &Realm) {
        // Create getter functions
        let connection_state_func = BuiltInBuilder::callable(realm, get_connection_state)
            .name(js_string!("get connectionState"))
            .build();
        let ice_connection_state_func = BuiltInBuilder::callable(realm, get_ice_connection_state)
            .name(js_string!("get iceConnectionState"))
            .build();
        let signaling_state_func = BuiltInBuilder::callable(realm, get_signaling_state)
            .name(js_string!("get signalingState"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Instance properties as accessors
            .accessor(
                js_string!("connectionState"),
                Some(connection_state_func),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("iceConnectionState"),
                Some(ice_connection_state_func),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("signalingState"),
                Some(signaling_state_func),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            // Methods
            .method(Self::create_offer, js_string!("createOffer"), 0)
            .method(Self::create_answer, js_string!("createAnswer"), 0)
            .method(Self::set_local_description, js_string!("setLocalDescription"), 1)
            .method(Self::set_remote_description, js_string!("setRemoteDescription"), 1)
            .method(Self::add_ice_candidate, js_string!("addIceCandidate"), 1)
            .method(Self::create_data_channel, js_string!("createDataChannel"), 1)
            .method(Self::close, js_string!("close"), 0)
            // Event handlers
            .property(
                js_string!("onconnectionstatechange"),
                JsValue::null(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .property(
                js_string!("oniceconnectionstatechange"),
                JsValue::null(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .property(
                js_string!("onsignalingstatechange"),
                JsValue::null(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .property(
                js_string!("onicecandidate"),
                JsValue::null(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .property(
                js_string!("ondatachannel"),
                JsValue::null(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for RTCPeerConnectionBuiltin {
    const NAME: JsString = StaticJsStrings::RTC_PEER_CONNECTION;
}

impl BuiltInConstructor for RTCPeerConnectionBuiltin {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::rtc_peer_connection;

    /// Constructor for RTCPeerConnection
    ///
    /// `new RTCPeerConnection([configuration])`
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Parse RTCConfiguration from args
        let _config = if !args.is_empty() && args[0].is_object() {
            // TODO: Parse ice servers from JS config object
            RTCConfiguration {
                ice_servers: vec![RTCIceServer {
                    urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                    ..Default::default()
                }],
                ..Default::default()
            }
        } else {
            RTCConfiguration::default()
        };

        // 2. Create the RTCPeerConnection object
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::rtc_peer_connection,
            context,
        )?;

        let data = RTCPeerConnectionData::new()
            .map_err(|e| JsNativeError::error().with_message(format!("Failed to create RTCPeerConnection: {}", e)))?;

        let rtc_peer_connection = RTCPeerConnectionBuiltin { data };

        let object = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            rtc_peer_connection,
        );

        Ok(object.into())
    }
}

impl RTCPeerConnectionBuiltin {
    /// Create an offer
    fn create_offer(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(_rtc_pc) = object.downcast_ref::<RTCPeerConnectionBuiltin>() {
                // TODO: Implement real offer creation
                // For now, return a promise that resolves to a mock SDP offer
                let offer_obj = JsObject::default();
                offer_obj.set(
                    js_string!("type"),
                    JsValue::from(js_string!("offer")),
                    false,
                    _context,
                )?;
                offer_obj.set(
                    js_string!("sdp"),
                    JsValue::from(js_string!("v=0\r\no=- 4611731400430051336 2 IN IP4 127.0.0.1\r\n")),
                    false,
                    _context,
                )?;

                return Ok(offer_obj.into());
            }
        }
        Ok(JsValue::undefined())
    }

    /// Create an answer
    fn create_answer(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(_rtc_pc) = object.downcast_ref::<RTCPeerConnectionBuiltin>() {
                // TODO: Implement real answer creation
                let answer_obj = JsObject::default();
                answer_obj.set(
                    js_string!("type"),
                    JsValue::from(js_string!("answer")),
                    false,
                    _context,
                )?;
                answer_obj.set(
                    js_string!("sdp"),
                    JsValue::from(js_string!("v=0\r\no=- 4611731400430051336 2 IN IP4 127.0.0.1\r\n")),
                    false,
                    _context,
                )?;

                return Ok(answer_obj.into());
            }
        }
        Ok(JsValue::undefined())
    }

    /// Set local description
    fn set_local_description(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(rtc_pc) = object.downcast_ref::<RTCPeerConnectionBuiltin>() {
                // TODO: Implement real local description setting
                rtc_pc.data.set_signaling_state(RTCSignalingState::HaveLocalOffer);
                return Ok(JsValue::undefined());
            }
        }
        Ok(JsValue::undefined())
    }

    /// Set remote description
    fn set_remote_description(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(rtc_pc) = object.downcast_ref::<RTCPeerConnectionBuiltin>() {
                // TODO: Implement real remote description setting
                rtc_pc.data.set_signaling_state(RTCSignalingState::HaveRemoteOffer);
                return Ok(JsValue::undefined());
            }
        }
        Ok(JsValue::undefined())
    }

    /// Add ICE candidate
    fn add_ice_candidate(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(_rtc_pc) = object.downcast_ref::<RTCPeerConnectionBuiltin>() {
                // TODO: Implement real ICE candidate addition
                return Ok(JsValue::undefined());
            }
        }
        Ok(JsValue::undefined())
    }

    /// Create data channel
    fn create_data_channel(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(_rtc_pc) = object.downcast_ref::<RTCPeerConnectionBuiltin>() {
                let label = args.get_or_undefined(0).to_string(_context)?;

                // TODO: Implement real data channel creation
                let data_channel_obj = JsObject::default();
                data_channel_obj.set(
                    js_string!("label"),
                    JsValue::from(label),
                    false,
                    _context,
                )?;
                data_channel_obj.set(
                    js_string!("readyState"),
                    JsValue::from(js_string!("connecting")),
                    false,
                    _context,
                )?;

                return Ok(data_channel_obj.into());
            }
        }
        Ok(JsValue::undefined())
    }

    /// Close the peer connection
    fn close(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(rtc_pc) = object.downcast_ref::<RTCPeerConnectionBuiltin>() {
                rtc_pc.data.set_connection_state(RTCPeerConnectionStateEnum::Closed);
                rtc_pc.data.set_ice_connection_state(RTCIceConnectionState::Closed);
                rtc_pc.data.set_signaling_state(RTCSignalingState::Closed);
            }
        }
        Ok(JsValue::undefined())
    }
}

/// Get the connectionState property of RTCPeerConnection
fn get_connection_state(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(object) = this.as_object() {
        if let Some(rtc_pc) = object.downcast_ref::<RTCPeerConnectionBuiltin>() {
            return Ok(rtc_pc.data.connection_state().into());
        }
    }
    Ok(JsValue::undefined())
}

/// Get the iceConnectionState property of RTCPeerConnection
fn get_ice_connection_state(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(object) = this.as_object() {
        if let Some(rtc_pc) = object.downcast_ref::<RTCPeerConnectionBuiltin>() {
            return Ok(rtc_pc.data.ice_connection_state().into());
        }
    }
    Ok(JsValue::undefined())
}

/// Get the signalingState property of RTCPeerConnection
fn get_signaling_state(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(object) = this.as_object() {
        if let Some(rtc_pc) = object.downcast_ref::<RTCPeerConnectionBuiltin>() {
            return Ok(rtc_pc.data.signaling_state().into());
        }
    }
    Ok(JsValue::undefined())
}