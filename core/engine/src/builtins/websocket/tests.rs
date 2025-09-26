//! Tests for the WebSocket API implementation

use crate::{Context, JsValue, Source, JsString};

#[test]
fn test_websocket_constructor_exists() {
    let mut context = Context::default();

    // Test that WebSocket constructor exists
    let result = context.eval(Source::from_bytes("typeof WebSocket")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_websocket_constructor_requires_new() {
    let mut context = Context::default();

    // Test that WebSocket constructor requires 'new'
    let result = context.eval(Source::from_bytes("try { WebSocket('ws://test'); false; } catch(e) { e.name === 'TypeError'; }"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsValue::from(true));
}

#[test]
fn test_websocket_constructor_requires_url() {
    let mut context = Context::default();

    // Test that WebSocket constructor requires a URL argument
    let result = context.eval(Source::from_bytes("try { new WebSocket(); false; } catch(e) { e.name === 'SyntaxError'; }"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsValue::from(true));
}

#[test]
fn test_websocket_invalid_url() {
    let mut context = Context::default();

    // Test that invalid URLs throw errors
    let result = context.eval(Source::from_bytes("try { new WebSocket('invalid-url'); false; } catch(e) { e.name === 'SyntaxError'; }"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsValue::from(true));
}

#[test]
fn test_websocket_invalid_scheme() {
    let mut context = Context::default();

    // Test that non-WebSocket schemes throw errors
    let result = context.eval(Source::from_bytes("try { new WebSocket('http://example.com'); false; } catch(e) { e.name === 'SyntaxError'; }"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsValue::from(true));
}

#[test]
fn test_websocket_valid_creation() {
    let mut context = Context::default();

    // Test that WebSocket can be created with valid URL
    let result = context.eval(Source::from_bytes("new WebSocket('ws://localhost:8080')"));
    assert!(result.is_ok());
    assert!(result.unwrap().is_object());
}

#[test]
fn test_websocket_url_property() {
    let mut context = Context::default();

    // Test that WebSocket has url property
    let result = context.eval(Source::from_bytes("(new WebSocket('ws://example.com')).url")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("ws://example.com")));
}

#[test]
fn test_websocket_ready_state_property() {
    let mut context = Context::default();

    // Test that WebSocket has readyState property (should be 0 for CONNECTING initially)
    let result = context.eval(Source::from_bytes("(new WebSocket('ws://example.com')).readyState")).unwrap();
    assert_eq!(result, JsValue::from(0)); // CONNECTING = 0
}

#[test]
fn test_websocket_constants() {
    let mut context = Context::default();

    // Test WebSocket constants
    let result = context.eval(Source::from_bytes("WebSocket.CONNECTING")).unwrap();
    assert_eq!(result, JsValue::from(0));

    let result = context.eval(Source::from_bytes("WebSocket.OPEN")).unwrap();
    assert_eq!(result, JsValue::from(1));

    let result = context.eval(Source::from_bytes("WebSocket.CLOSING")).unwrap();
    assert_eq!(result, JsValue::from(2));

    let result = context.eval(Source::from_bytes("WebSocket.CLOSED")).unwrap();
    assert_eq!(result, JsValue::from(3));
}

#[test]
fn test_websocket_methods_exist() {
    let mut context = Context::default();

    // Test that WebSocket has send and close methods
    let result = context.eval(Source::from_bytes("typeof (new WebSocket('ws://example.com')).send")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof (new WebSocket('ws://example.com')).close")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_websocket_properties() {
    let mut context = Context::default();

    // Test other WebSocket properties
    let result = context.eval(Source::from_bytes("typeof (new WebSocket('ws://example.com')).bufferedAmount")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("number")));

    let result = context.eval(Source::from_bytes("typeof (new WebSocket('ws://example.com')).extensions")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("string")));

    let result = context.eval(Source::from_bytes("typeof (new WebSocket('ws://example.com')).protocol")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("string")));

    let result = context.eval(Source::from_bytes("typeof (new WebSocket('ws://example.com')).binaryType")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("string")));
}

#[test]
fn test_websocket_instanceof() {
    let mut context = Context::default();

    // Test that instances are properly recognized with instanceof
    let result = context.eval(Source::from_bytes("(new WebSocket('ws://test')) instanceof WebSocket")).unwrap();
    assert_eq!(result, JsValue::from(true));
}