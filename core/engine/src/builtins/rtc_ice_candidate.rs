//! WHATWG RTCIceCandidate API implementation for WebRTC ICE candidate handling.
//!
//! Implementation of the RTCIceCandidate interface according to:
//! https://w3c.github.io/webrtc-pc/#rtcicecandidate-interface
//!
//! The RTCIceCandidate interface represents a candidate Interactive Connectivity
//! Establishment (ICE) configuration which may be used to establish an RTCPeerConnection.

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
    ice_transport::ice_candidate::RTCIceCandidate,
    ice_transport::ice_candidate_type::RTCIceCandidateType,
    ice_transport::ice_protocol::RTCIceProtocol,
};

/// RTCIceCandidate initialization dictionary
#[derive(Debug, Clone)]
pub struct RTCIceCandidateInit {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<u16>,
    pub username_fragment: Option<String>,
}

/// JavaScript representation of an RTCIceCandidate
#[derive(Trace, Finalize, JsData)]
pub struct RTCIceCandidateData {
    #[unsafe_ignore_trace]
    ice_candidate: Option<Arc<RTCIceCandidate>>,
    candidate: String,
    sdp_mid: Option<String>,
    sdp_m_line_index: Option<u16>,
    username_fragment: Option<String>,
}

impl std::fmt::Debug for RTCIceCandidateData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RTCIceCandidateData")
            .field("candidate", &self.candidate)
            .field("sdp_mid", &self.sdp_mid)
            .field("sdp_m_line_index", &self.sdp_m_line_index)
            .field("username_fragment", &self.username_fragment)
            .finish()
    }
}

impl RTCIceCandidateData {
    /// Create a new RTCIceCandidateData instance
    pub fn new(candidate_init: RTCIceCandidateInit) -> anyhow::Result<Self> {
        // For now, we'll create a placeholder RTCIceCandidate
        // Real implementation would parse the candidate string and create the actual candidate

        Ok(Self {
            ice_candidate: None, // Will be filled when we have proper candidate parsing
            candidate: candidate_init.candidate,
            sdp_mid: candidate_init.sdp_mid,
            sdp_m_line_index: candidate_init.sdp_mline_index,
            username_fragment: candidate_init.username_fragment,
        })
    }

    /// Create a new empty RTCIceCandidateData instance
    pub fn new_empty() -> Self {
        Self {
            ice_candidate: None,
            candidate: String::new(),
            sdp_mid: None,
            sdp_m_line_index: None,
            username_fragment: None,
        }
    }

    /// Get the ICE candidate reference
    pub fn ice_candidate(&self) -> Option<&Arc<RTCIceCandidate>> {
        self.ice_candidate.as_ref()
    }

    /// Get the candidate string
    pub fn candidate(&self) -> &str {
        &self.candidate
    }

    /// Get the SDP mid
    pub fn sdp_mid(&self) -> Option<&str> {
        self.sdp_mid.as_deref()
    }

    /// Get the SDP m-line index
    pub fn sdp_m_line_index(&self) -> Option<u16> {
        self.sdp_m_line_index
    }

    /// Get the username fragment
    pub fn username_fragment(&self) -> Option<&str> {
        self.username_fragment.as_deref()
    }

    /// Convert to JSON representation
    pub fn to_json(&self) -> String {
        let mut json = String::from("{");

        json.push_str(&format!("\"candidate\":\"{}\"", self.candidate));

        if let Some(sdp_mid) = &self.sdp_mid {
            json.push_str(&format!(",\"sdpMid\":\"{}\"", sdp_mid));
        } else {
            json.push_str(",\"sdpMid\":null");
        }

        if let Some(sdp_m_line_index) = self.sdp_m_line_index {
            json.push_str(&format!(",\"sdpMLineIndex\":{}", sdp_m_line_index));
        } else {
            json.push_str(",\"sdpMLineIndex\":null");
        }

        if let Some(username_fragment) = &self.username_fragment {
            json.push_str(&format!(",\"usernameFragment\":\"{}\"", username_fragment));
        } else {
            json.push_str(",\"usernameFragment\":null");
        }

        json.push('}');
        json
    }
}

/// Global ICE Candidate manager for managing ICE candidates
pub struct IceCandidateManager {
    candidates: Arc<TokioMutex<HashMap<String, Arc<RTCIceCandidate>>>>,
    candidate_counter: AtomicU32,
}

impl std::fmt::Debug for IceCandidateManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IceCandidateManager")
            .field("candidates", &"Arc<TokioMutex<HashMap<String, Arc<RTCIceCandidate>>>>")
            .field("candidate_counter", &self.candidate_counter)
            .finish()
    }
}

impl IceCandidateManager {
    /// Create a new ICE candidate manager
    pub fn new() -> Self {
        Self {
            candidates: Arc::new(TokioMutex::new(HashMap::new())),
            candidate_counter: AtomicU32::new(0),
        }
    }

    /// Generate a unique candidate ID
    pub fn generate_candidate_id(&self) -> String {
        let id = self.candidate_counter.fetch_add(1, Ordering::SeqCst);
        format!("icecandidate_{}", id)
    }

    /// Register an ICE candidate
    pub async fn register_candidate(&self, id: String, candidate: Arc<RTCIceCandidate>) {
        self.candidates.lock().await.insert(id, candidate);
    }

    /// Get an ICE candidate by ID
    pub async fn get_candidate(&self, id: &str) -> Option<Arc<RTCIceCandidate>> {
        self.candidates.lock().await.get(id).cloned()
    }

    /// Remove an ICE candidate
    pub async fn remove_candidate(&self, id: &str) -> Option<Arc<RTCIceCandidate>> {
        self.candidates.lock().await.remove(id)
    }
}

impl Default for IceCandidateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// The `RTCIceCandidate` builtin object
#[derive(Debug, Clone, Trace, Finalize)]
pub struct RTCIceCandidateBuiltin;

impl IntrinsicObject for RTCIceCandidateBuiltin {
    fn init(realm: &Realm) {
        let get_candidate = BuiltInBuilder::callable(realm, Self::get_candidate)
            .name(js_string!("get candidate"))
            .build();

        let get_sdp_mid = BuiltInBuilder::callable(realm, Self::get_sdp_mid)
            .name(js_string!("get sdpMid"))
            .build();

        let get_sdp_m_line_index = BuiltInBuilder::callable(realm, Self::get_sdp_m_line_index)
            .name(js_string!("get sdpMLineIndex"))
            .build();

        let get_username_fragment = BuiltInBuilder::callable(realm, Self::get_username_fragment)
            .name(js_string!("get usernameFragment"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                js_string!("candidate"),
                get_candidate,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("sdpMid"),
                get_sdp_mid,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("sdpMLineIndex"),
                get_sdp_m_line_index,
                Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("usernameFragment"),
                get_username_fragment,
                Attribute::CONFIGURABLE,
            )
            .method(Self::to_json, js_string!("toJSON"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for RTCIceCandidateBuiltin {
    const NAME: JsString = StaticJsStrings::RTC_ICE_CANDIDATE;
}

impl BuiltInConstructor for RTCIceCandidateBuiltin {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::rtc_ice_candidate;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::rtc_ice_candidate,
            context,
        )?;

        let candidate_init = args.get_or_undefined(0);

        let ice_candidate_data = if candidate_init.is_undefined() || candidate_init.is_null() {
            // Create empty RTCIceCandidate if no init provided
            RTCIceCandidateData::new_empty()
        } else {
            // Parse the RTCIceCandidateInit dictionary
            let candidate_obj = candidate_init.as_object().ok_or_else(|| {
                JsNativeError::typ().with_message("RTCIceCandidate constructor requires an object or null")
            })?;

            // Extract candidate string
            let candidate = candidate_obj
                .get(js_string!("candidate"), context)?
                .to_string(context)?
                .to_std_string_escaped();

            // Extract sdpMid (optional)
            let sdp_mid_value = candidate_obj.get(js_string!("sdpMid"), context)?;
            let sdp_mid = if sdp_mid_value.is_null() || sdp_mid_value.is_undefined() {
                None
            } else {
                Some(sdp_mid_value.to_string(context)?.to_std_string_escaped())
            };

            // Extract sdpMLineIndex (optional)
            let sdp_m_line_index_value = candidate_obj.get(js_string!("sdpMLineIndex"), context)?;
            let sdp_m_line_index = if sdp_m_line_index_value.is_null() || sdp_m_line_index_value.is_undefined() {
                None
            } else {
                Some(sdp_m_line_index_value.to_u32(context)? as u16)
            };

            // Extract usernameFragment (optional)
            let username_fragment_value = candidate_obj.get(js_string!("usernameFragment"), context)?;
            let username_fragment = if username_fragment_value.is_null() || username_fragment_value.is_undefined() {
                None
            } else {
                Some(username_fragment_value.to_string(context)?.to_std_string_escaped())
            };

            // Create RTCIceCandidateInit
            let candidate_init = RTCIceCandidateInit {
                candidate: candidate,
                sdp_mid,
                sdp_mline_index: sdp_m_line_index,
                username_fragment,
            };

            // Create the RTCIceCandidateData
            RTCIceCandidateData::new(candidate_init).map_err(|e| {
                JsNativeError::typ().with_message(format!("Failed to create RTCIceCandidate: {}", e))
            })?
        };

        let ice_candidate_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            ice_candidate_data,
        );

        Ok(ice_candidate_obj.into())
    }
}

impl RTCIceCandidateBuiltin {
    /// `RTCIceCandidate.prototype.candidate` getter
    pub(crate) fn get_candidate(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCIceCandidate method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCIceCandidateData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCIceCandidate method called on wrong object type")
        })?;

        Ok(JsValue::from(js_string!(data.candidate().to_string())))
    }

    /// `RTCIceCandidate.prototype.sdpMid` getter
    pub(crate) fn get_sdp_mid(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCIceCandidate method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCIceCandidateData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCIceCandidate method called on wrong object type")
        })?;

        match data.sdp_mid() {
            Some(sdp_mid) => Ok(JsValue::from(js_string!(sdp_mid.to_string()))),
            None => Ok(JsValue::null()),
        }
    }

    /// `RTCIceCandidate.prototype.sdpMLineIndex` getter
    pub(crate) fn get_sdp_m_line_index(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCIceCandidate method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCIceCandidateData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCIceCandidate method called on wrong object type")
        })?;

        match data.sdp_m_line_index() {
            Some(index) => Ok(JsValue::from(index)),
            None => Ok(JsValue::null()),
        }
    }

    /// `RTCIceCandidate.prototype.usernameFragment` getter
    pub(crate) fn get_username_fragment(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCIceCandidate method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCIceCandidateData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCIceCandidate method called on wrong object type")
        })?;

        match data.username_fragment() {
            Some(fragment) => Ok(JsValue::from(js_string!(fragment.to_string()))),
            None => Ok(JsValue::null()),
        }
    }

    /// `RTCIceCandidate.prototype.toJSON()`
    pub(crate) fn to_json(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCIceCandidate method called on non-object")
        })?;

        let data = obj.downcast_ref::<RTCIceCandidateData>().ok_or_else(|| {
            JsNativeError::typ().with_message("RTCIceCandidate method called on wrong object type")
        })?;

        let json_string = data.to_json();
        Ok(JsValue::from(js_string!(json_string)))
    }
}