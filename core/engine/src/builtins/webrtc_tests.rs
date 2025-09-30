//! Comprehensive unit tests for WebRTC APIs in Boa engine
//!
//! Tests cover all implemented WebRTC interfaces:
//! - RTCPeerConnection
//! - RTCDataChannel
//! - RTCIceCandidate
//! - RTCSessionDescription

#[cfg(test)]
mod tests {
    use crate::{Context, JsResult, JsValue};

    /// Test helper to create a new context and evaluate JavaScript code
    fn eval_js(code: &str) -> JsResult<JsValue> {
        let mut context = Context::default();
        context.eval(boa_parser::Source::from_bytes(code))
    }

    /// Test helper to check if a constructor exists
    fn test_constructor_exists(constructor_name: &str) -> JsResult<bool> {
        let code = format!("typeof {} === 'function'", constructor_name);
        let result = eval_js(&code)?;
        Ok(result.to_boolean())
    }

    #[test]
    fn test_rtc_peer_connection_constructor_exists() {
        assert!(test_constructor_exists("RTCPeerConnection").unwrap());
    }

    #[test]
    fn test_rtc_data_channel_constructor_exists() {
        assert!(test_constructor_exists("RTCDataChannel").unwrap());
    }

    #[test]
    fn test_rtc_ice_candidate_constructor_exists() {
        assert!(test_constructor_exists("RTCIceCandidate").unwrap());
    }

    #[test]
    fn test_rtc_session_description_constructor_exists() {
        assert!(test_constructor_exists("RTCSessionDescription").unwrap());
    }

    #[test]
    fn test_rtc_peer_connection_basic_creation() {
        let result = eval_js("
            let pc = new RTCPeerConnection();
            typeof pc === 'object' && pc.constructor.name === 'RTCPeerConnection'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_peer_connection_with_configuration() {
        let result = eval_js("
            let pc = new RTCPeerConnection({
                iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
            });
            typeof pc === 'object' && pc.constructor.name === 'RTCPeerConnection'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_peer_connection_connection_state() {
        let result = eval_js("
            let pc = new RTCPeerConnection();
            typeof pc.connectionState === 'string'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_peer_connection_ice_connection_state() {
        let result = eval_js("
            let pc = new RTCPeerConnection();
            typeof pc.iceConnectionState === 'string'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_peer_connection_ice_gathering_state() {
        let result = eval_js("
            let pc = new RTCPeerConnection();
            typeof pc.iceGatheringState === 'string'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_peer_connection_signaling_state() {
        let result = eval_js("
            let pc = new RTCPeerConnection();
            typeof pc.signalingState === 'string'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_peer_connection_create_offer_method() {
        let result = eval_js("
            let pc = new RTCPeerConnection();
            typeof pc.createOffer === 'function'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_peer_connection_create_answer_method() {
        let result = eval_js("
            let pc = new RTCPeerConnection();
            typeof pc.createAnswer === 'function'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_peer_connection_set_local_description_method() {
        let result = eval_js("
            let pc = new RTCPeerConnection();
            typeof pc.setLocalDescription === 'function'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_peer_connection_set_remote_description_method() {
        let result = eval_js("
            let pc = new RTCPeerConnection();
            typeof pc.setRemoteDescription === 'function'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_peer_connection_add_ice_candidate_method() {
        let result = eval_js("
            let pc = new RTCPeerConnection();
            typeof pc.addIceCandidate === 'function'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_peer_connection_close_method() {
        let result = eval_js("
            let pc = new RTCPeerConnection();
            typeof pc.close === 'function'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_ice_candidate_empty_constructor() {
        let result = eval_js("
            let candidate = new RTCIceCandidate();
            typeof candidate === 'object' && candidate.constructor.name === 'RTCIceCandidate'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_ice_candidate_with_init() {
        let result = eval_js("
            let candidate = new RTCIceCandidate({
                candidate: 'candidate:1 1 UDP 2130706431 192.168.1.100 54400 typ host',
                sdpMid: '0',
                sdpMLineIndex: 0,
                usernameFragment: 'username'
            });
            typeof candidate === 'object' && candidate.constructor.name === 'RTCIceCandidate'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_ice_candidate_properties() {
        let result = eval_js("
            let candidate = new RTCIceCandidate({
                candidate: 'candidate:1 1 UDP 2130706431 192.168.1.100 54400 typ host',
                sdpMid: '0',
                sdpMLineIndex: 0,
                usernameFragment: 'username'
            });
            candidate.candidate === 'candidate:1 1 UDP 2130706431 192.168.1.100 54400 typ host' &&
            candidate.sdpMid === '0' &&
            candidate.sdpMLineIndex === 0 &&
            candidate.usernameFragment === 'username'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_ice_candidate_to_json() {
        let result = eval_js("
            let candidate = new RTCIceCandidate({
                candidate: 'test-candidate',
                sdpMid: '0',
                sdpMLineIndex: 0,
                usernameFragment: 'username'
            });
            typeof candidate.toJSON === 'function' &&
            typeof candidate.toJSON() === 'string'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_ice_candidate_null_properties() {
        let result = eval_js("
            let candidate = new RTCIceCandidate();
            candidate.candidate === '' &&
            candidate.sdpMid === null &&
            candidate.sdpMLineIndex === null &&
            candidate.usernameFragment === null
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_session_description_empty_constructor() {
        let result = eval_js("
            let desc = new RTCSessionDescription();
            typeof desc === 'object' && desc.constructor.name === 'RTCSessionDescription'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_session_description_with_init() {
        let result = eval_js("
            let desc = new RTCSessionDescription({
                type: 'offer',
                sdp: 'v=0\\r\\no=- 123456 2 IN IP4 127.0.0.1\\r\\n'
            });
            typeof desc === 'object' && desc.constructor.name === 'RTCSessionDescription'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_session_description_properties() {
        let result = eval_js("
            let desc = new RTCSessionDescription({
                type: 'offer',
                sdp: 'v=0\\r\\no=- 123456 2 IN IP4 127.0.0.1\\r\\n'
            });
            desc.type === 'offer' &&
            desc.sdp === 'v=0\\r\\no=- 123456 2 IN IP4 127.0.0.1\\r\\n'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_session_description_types() {
        let result = eval_js("
            let offer = new RTCSessionDescription({ type: 'offer', sdp: 'test' });
            let answer = new RTCSessionDescription({ type: 'answer', sdp: 'test' });
            let pranswer = new RTCSessionDescription({ type: 'pranswer', sdp: 'test' });
            let rollback = new RTCSessionDescription({ type: 'rollback', sdp: 'test' });

            offer.type === 'offer' &&
            answer.type === 'answer' &&
            pranswer.type === 'pranswer' &&
            rollback.type === 'rollback'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_session_description_to_json() {
        let result = eval_js("
            let desc = new RTCSessionDescription({
                type: 'offer',
                sdp: 'test-sdp'
            });
            typeof desc.toJSON === 'function' &&
            typeof desc.toJSON() === 'string'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_data_channel_constructor_throws() {
        // RTCDataChannel constructor should not be callable directly
        let result = eval_js("
            try {
                new RTCDataChannel();
                false; // Should not reach here
            } catch (e) {
                e instanceof TypeError;
            }
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_webrtc_apis_global_availability() {
        let result = eval_js("
            typeof RTCPeerConnection === 'function' &&
            typeof RTCDataChannel === 'function' &&
            typeof RTCIceCandidate === 'function' &&
            typeof RTCSessionDescription === 'function'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_rtc_peer_connection_state_initial_values() {
        let result = eval_js("
            let pc = new RTCPeerConnection();
            pc.connectionState === 'new' &&
            pc.iceConnectionState === 'new' &&
            pc.iceGatheringState === 'new' &&
            pc.signalingState === 'stable'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_webrtc_error_handling() {
        // Test that invalid parameters throw appropriate errors
        let result = eval_js("
            try {
                new RTCIceCandidate('invalid');
                false;
            } catch (e) {
                e instanceof TypeError;
            }
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_webrtc_property_descriptors() {
        // Test that properties are properly configured as readonly
        let result = eval_js("
            let candidate = new RTCIceCandidate({
                candidate: 'test',
                sdpMid: '0'
            });

            let desc = Object.getOwnPropertyDescriptor(Object.getPrototypeOf(candidate), 'candidate');
            desc && desc.get && !desc.set
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_webrtc_inheritance() {
        // Test that WebRTC objects properly inherit from Object
        let result = eval_js("
            let pc = new RTCPeerConnection();
            let candidate = new RTCIceCandidate();
            let desc = new RTCSessionDescription();

            pc instanceof Object &&
            candidate instanceof Object &&
            desc instanceof Object
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_webrtc_json_serialization() {
        let result = eval_js("
            let candidate = new RTCIceCandidate({
                candidate: 'test-candidate',
                sdpMid: '0',
                sdpMLineIndex: 0
            });

            let desc = new RTCSessionDescription({
                type: 'offer',
                sdp: 'test-sdp'
            });

            let candidateJson = JSON.parse(candidate.toJSON());
            let descJson = JSON.parse(desc.toJSON());

            candidateJson.candidate === 'test-candidate' &&
            candidateJson.sdpMid === '0' &&
            candidateJson.sdpMLineIndex === 0 &&
            descJson.type === 'offer' &&
            descJson.sdp === 'test-sdp'
        ");
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_webrtc_comprehensive_api_coverage() {
        // Test that all major WebRTC APIs are available and functional
        let result = eval_js("
            // Test RTCPeerConnection creation and basic methods
            let pc = new RTCPeerConnection();
            let hasBasicMethods = typeof pc.createOffer === 'function' &&
                                 typeof pc.createAnswer === 'function' &&
                                 typeof pc.setLocalDescription === 'function' &&
                                 typeof pc.setRemoteDescription === 'function' &&
                                 typeof pc.addIceCandidate === 'function' &&
                                 typeof pc.close === 'function';

            // Test RTCIceCandidate creation and properties
            let candidate = new RTCIceCandidate({
                candidate: 'candidate:1 1 UDP 2130706431 192.168.1.100 54400 typ host',
                sdpMid: '0',
                sdpMLineIndex: 0
            });
            let hasIceCandidate = candidate.candidate.includes('192.168.1.100') &&
                                 candidate.sdpMid === '0' &&
                                 typeof candidate.toJSON === 'function';

            // Test RTCSessionDescription creation and properties
            let desc = new RTCSessionDescription({
                type: 'offer',
                sdp: 'v=0\\r\\no=- 123456 2 IN IP4 127.0.0.1\\r\\n'
            });
            let hasSessionDesc = desc.type === 'offer' &&
                                desc.sdp.includes('127.0.0.1') &&
                                typeof desc.toJSON === 'function';

            // Test RTCDataChannel exists (even if constructor throws)
            let hasDataChannel = typeof RTCDataChannel === 'function';

            hasBasicMethods && hasIceCandidate && hasSessionDesc && hasDataChannel
        ");
        assert!(result.unwrap().to_boolean());
    }
}