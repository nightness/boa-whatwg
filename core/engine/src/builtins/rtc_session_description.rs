//! WHATWG RTCSessionDescription API implementation for WebRTC session description handling.
//!
//! Implementation of the RTCSessionDescription interface according to:
//! https://w3c.github.io/webrtc-pc/#rtcsessiondescription-class
//!
//! The RTCSessionDescription interface represents a session description, which is used
//! to describe an offer or answer for an RTCPeerConnection.

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
    peer_connection::sdp::session_description::RTCSessionDescription as WebRTCSessionDescription,
    peer_connection::sdp::sdp_type::RTCSdpType,
};

/// RTCSessionDescription type enumeration according to WHATWG specification
#[derive(Debug, Clone, PartialEq, Eq, Trace, Finalize)]
#[repr(u32)]
pub enum RTCSessionDescriptionType {
    Offer = 0,
    Answer = 1,
    Pranswer = 2,
    Rollback = 3,
}

impl From<RTCSessionDescriptionType> for JsValue {
    fn from(sdp_type: RTCSessionDescriptionType) -> Self {
        let type_str = match sdp_type {
            RTCSessionDescriptionType::Offer => "offer",
            RTCSessionDescriptionType::Answer => "answer",
            RTCSessionDescriptionType::Pranswer => "pranswer",
            RTCSessionDescriptionType::Rollback => "rollback",
        };
        JsValue::from(js_string!(type_str))
    }
}

impl From<RTCSdpType> for RTCSessionDescriptionType {
    fn from(sdp_type: RTCSdpType) -> Self {
        match sdp_type {
            RTCSdpType::Offer => RTCSessionDescriptionType::Offer,
            RTCSdpType::Answer => RTCSessionDescriptionType::Answer,
            RTCSdpType::Pranswer => RTCSessionDescriptionType::Pranswer,
            RTCSdpType::Rollback => RTCSessionDescriptionType::Rollback,
            RTCSdpType::Unspecified => RTCSessionDescriptionType::Offer, // Default fallback
        }
    }
}

impl From<&str> for RTCSessionDescriptionType {
    fn from(type_str: &str) -> Self {
        match type_str {
            "offer" => RTCSessionDescriptionType::Offer,
            "answer" => RTCSessionDescriptionType::Answer,
            "pranswer" => RTCSessionDescriptionType::Pranswer,
            "rollback" => RTCSessionDescriptionType::Rollback,
            _ => RTCSessionDescriptionType::Offer, // Default fallback
        }
    }
}

/// RTCSessionDescription initialization dictionary
#[derive(Debug, Clone)]
pub struct RTCSessionDescriptionInit {
    pub sdp_type: RTCSessionDescriptionType,
    pub sdp: String,
}

/// JavaScript representation of an RTCSessionDescription
#[derive(Trace, Finalize, JsData)]
pub struct RTCSessionDescriptionData {
    #[unsafe_ignore_trace]
    session_description: Option<Arc<WebRTCSessionDescription>>,
    sdp_type: RTCSessionDescriptionType,
    sdp: String,
}

impl std::fmt::Debug for RTCSessionDescriptionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RTCSessionDescriptionData")
            .field("sdp_type", &self.sdp_type)
            .field("sdp", &self.sdp)
            .finish()
    }
}

impl RTCSessionDescriptionData {
    /// Create a new RTCSessionDescriptionData instance
    pub fn new(description_init: RTCSessionDescriptionInit) -> anyhow::Result<Self> {
        // For now, we'll create a placeholder WebRTCSessionDescription
        // Real implementation would parse the SDP string and create the actual session description

        Ok(Self {
            session_description: None, // Will be filled when we have proper SDP parsing
            sdp_type: description_init.sdp_type,
            sdp: description_init.sdp,
        })
    }

    /// Create a new empty RTCSessionDescriptionData instance
    pub fn new_empty() -> Self {
        Self {
            session_description: None,
            sdp_type: RTCSessionDescriptionType::Offer,
            sdp: String::new(),
        }
    }

    /// Get the session description reference
    pub fn session_description(&self) -> Option<&Arc<WebRTCSessionDescription>> {
        self.session_description.as_ref()
    }

    /// Get the SDP type
    pub fn sdp_type(&self) -> &RTCSessionDescriptionType {
        &self.sdp_type
    }

    /// Get the SDP string
    pub fn sdp(&self) -> &str {
        &self.sdp
    }

    /// Convert to JSON representation
    pub fn to_json(&self) -> String {
        let type_str = match self.sdp_type {
            RTCSessionDescriptionType::Offer => "offer",
            RTCSessionDescriptionType::Answer => "answer",
            RTCSessionDescriptionType::Pranswer => "pranswer",
            RTCSessionDescriptionType::Rollback => "rollback",
        };

        format!(
            "{{\"type\":\"{}\",\"sdp\":\"{}\"}}",
            type_str,
            self.sdp.replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r")
        )
    }
}

/// Global Session Description manager for managing session descriptions
pub struct SessionDescriptionManager {
    descriptions: Arc<TokioMutex<HashMap<String, Arc<WebRTCSessionDescription>>>>,
    description_counter: AtomicU32,
}

impl std::fmt::Debug for SessionDescriptionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionDescriptionManager")
            .field("descriptions", &"Arc<TokioMutex<HashMap<String, Arc<WebRTCSessionDescription>>>>")
            .field("description_counter", &self.description_counter)
            .finish()
    }
}

impl SessionDescriptionManager {
    /// Create a new session description manager
    pub fn new() -> Self {
        Self {
            descriptions: Arc::new(TokioMutex::new(HashMap::new())),
            description_counter: AtomicU32::new(0),
        }
    }

    /// Generate a unique description ID
    pub fn generate_description_id(&self) -> String {
        let id = self.description_counter.fetch_add(1, Ordering::SeqCst);
        format!("sessiondescription_{}", id)
    }

    /// Register a session description
    pub async fn register_description(&self, id: String, description: Arc<WebRTCSessionDescription>) {
        self.descriptions.lock().await.insert(id, description);
    }

    /// Get a session description by ID
    pub async fn get_description(&self, id: &str) -> Option<Arc<WebRTCSessionDescription>> {
        self.descriptions.lock().await.get(id).cloned()
    }

    /// Remove a session description
    pub async fn remove_description(&self, id: &str) -> Option<Arc<WebRTCSessionDescription>> {
        self.descriptions.lock().await.remove(id)
    }
}

impl Default for SessionDescriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// The `RTCSessionDescription` builtin object
#[derive(Debug, Clone, Trace, Finalize)]
pub struct RTCSessionDescriptionBuiltin;

impl IntrinsicObject for RTCSessionDescriptionBuiltin {
    fn init(realm: &Realm) {
        let get_type = BuiltInBuilder::callable(realm, Self::get_type)
            .name(js_string!("get type"))
            .build();

        let get_sdp = BuiltInBuilder::callable(realm, Self::get_sdp)
            .name(js_string!("get sdp"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                js_string!("type"),
                get_type,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("sdp"),
                get_sdp,
                Attribute::CONFIGURABLE,
            )
            .method(Self::to_json, js_string!("toJSON"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for RTCSessionDescriptionBuiltin {
    const NAME: JsString = StaticJsStrings::RTC_SESSION_DESCRIPTION;
}

impl BuiltInConstructor for RTCSessionDescriptionBuiltin {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::rtc_session_description;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::rtc_session_description,
            context,
        )?;

        let description_init = args.get_or_undefined(0);

        let session_description_data = if description_init.is_undefined() || description_init.is_null() {
            // Create empty RTCSessionDescription if no init provided
            RTCSessionDescriptionData::new_empty()
        } else {
            // Parse the RTCSessionDescriptionInit dictionary
            let description_obj = description_init.as_object().ok_or_else(|| {
                JsNativeError::typ().with_message("RTCSessionDescription constructor requires an object or null")
            })?;

            // Extract type (required)
            let type_value = description_obj.get(js_string!("type"), context)?;
            let sdp_type_str = type_value.to_string(context)?.to_std_string_escaped();
            let sdp_type = RTCSessionDescriptionType::from(sdp_type_str.as_str());

            // Extract sdp (required)
            let sdp_value = description_obj.get(js_string!("sdp"), context)?;
            let sdp = sdp_value.to_string(context)?.to_std_string_escaped();

            // Create RTCSessionDescriptionInit
            let description_init = RTCSessionDescriptionInit {
                sdp_type,
                sdp,
            };

            // Create the RTCSessionDescriptionData
            RTCSessionDescriptionData::new(description_init).map_err(|e| {
                JsNativeError::typ().with_message(format!("Failed to create RTCSessionDescription: {}", e))
            })?
        };

        let session_description_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            session_description_data,
        );

        Ok(session_description_obj.into())
    }
}

impl RTCSessionDescriptionBuiltin {
    /// `RTCSessionDescription.prototype.type` getter
    pub(crate) fn get_type(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCSessionDescription method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCSessionDescriptionData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCSessionDescription method called on wrong object type")
        })?;

        Ok(data.sdp_type.clone().into())
    }

    /// `RTCSessionDescription.prototype.sdp` getter
    pub(crate) fn get_sdp(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCSessionDescription method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCSessionDescriptionData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCSessionDescription method called on wrong object type")
        })?;

        Ok(JsValue::from(js_string!(data.sdp().to_string())))
    }

    /// `RTCSessionDescription.prototype.toJSON()`
    pub(crate) fn to_json(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCSessionDescription method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCSessionDescriptionData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCSessionDescription method called on wrong object type")
        })?;

        let json_string = data.to_json();
        Ok(JsValue::from(js_string!(json_string)))
    }
}