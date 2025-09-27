//! Tests for the WebSocketStream implementation

use crate::{Context, Source, JsValue, JsString, js_string};

#[test]
fn test_websocket_stream_constructor_availability() {
    let mut context = Context::default();

    // Test that WebSocketStream constructor is available globally
    let result = context.eval(Source::from_bytes("typeof WebSocketStream")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_websocket_stream_constructor_requires_new() {
    let mut context = Context::default();

    // Test that WebSocketStream constructor requires 'new'
    let result = context.eval(Source::from_bytes("try { WebSocketStream('ws://test'); false; } catch(e) { e.name === 'TypeError'; }"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsValue::from(true));
}

#[test]
fn test_websocket_stream_constructor_requires_url() {
    let mut context = Context::default();

    // Test that WebSocketStream constructor requires a URL argument
    let result = context.eval(Source::from_bytes("try { new WebSocketStream(); false; } catch(e) { e.name === 'TypeError'; }"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsValue::from(true));
}

#[test]
fn test_websocket_stream_constructor_rejects_empty_url() {
    let mut context = Context::default();

    // Test that WebSocketStream constructor rejects empty URL
    let result = context.eval(Source::from_bytes("try { new WebSocketStream(''); false; } catch(e) { e.name === 'TypeError'; }"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsValue::from(true));
}

#[test]
fn test_websocket_stream_instance_creation() {
    let mut context = Context::default();

    // Test that WebSocketStream can be instantiated with a valid URL
    let result = context.eval(Source::from_bytes("new WebSocketStream('ws://localhost:8080')"));
    assert!(result.is_ok());

    // Verify it's an object
    let ws_stream = result.unwrap();
    assert!(ws_stream.is_object());
}

#[test]
fn test_websocket_stream_url_property() {
    let mut context = Context::default();

    // Test that WebSocketStream has a url property that returns the URL
    let result = context.eval(Source::from_bytes("(new WebSocketStream('ws://example.com')).url")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("ws://example.com")));
}

#[test]
fn test_websocket_stream_ready_state_property() {
    let mut context = Context::default();

    // Test that WebSocketStream has a readyState property (should be 0 for CONNECTING initially)
    let result = context.eval(Source::from_bytes("(new WebSocketStream('ws://example.com')).readyState")).unwrap();
    assert_eq!(result, JsValue::from(0)); // CONNECTING = 0
}

#[test]
fn test_websocket_stream_close_method() {
    let mut context = Context::default();

    // Test that WebSocketStream has a close method
    let result = context.eval(Source::from_bytes("typeof (new WebSocketStream('ws://example.com')).close")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_websocket_stream_close_method_execution() {
    let mut context = Context::default();

    // Test that close method can be called and changes readyState
    let result = context.eval(Source::from_bytes(r#"
        let ws = new WebSocketStream('ws://example.com');
        ws.close();
        ws.readyState
    "#)).unwrap();
    assert_eq!(result, JsValue::from(3)); // CLOSED = 3
}

#[test]
fn test_websocket_stream_prototype_constructor() {
    let mut context = Context::default();

    // Test that WebSocketStream prototype is properly set up
    let result = context.eval(Source::from_bytes("WebSocketStream.prototype.constructor === WebSocketStream")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_websocket_stream_instanceof() {
    let mut context = Context::default();

    // Test that instances are properly recognized with instanceof
    let result = context.eval(Source::from_bytes("(new WebSocketStream('ws://test')) instanceof WebSocketStream")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_websocket_stream_with_options() {
    let mut context = Context::default();

    // Test that WebSocketStream accepts options parameter
    let result = context.eval(Source::from_bytes("new WebSocketStream('ws://example.com', { protocols: ['chat'] })"));
    assert!(result.is_ok());
    assert!(result.unwrap().is_object());
}

#[test]
fn test_websocket_stream_method_binding() {
    let mut context = Context::default();

    // Test that methods are properly bound to instances
    // Note: In JavaScript, when you extract a method from an object, it loses its 'this' binding
    // So we need to use call() or bind() to maintain the context
    let result = context.eval(Source::from_bytes(r#"
        let ws = new WebSocketStream('ws://example.com');
        let close = ws.close;
        close.call(ws); // Use call() to provide correct 'this' context
        ws.readyState === 3
    "#)).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_websocket_stream_url_immutable() {
    let mut context = Context::default();

    // Test that url property is read-only (attempting to modify should not work)
    let result = context.eval(Source::from_bytes(r#"
        let ws = new WebSocketStream('ws://example.com');
        let originalUrl = ws.url;
        try { ws.url = 'ws://other.com'; } catch(e) {}
        ws.url === originalUrl
    "#)).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_websocket_stream_ready_state_immutable() {
    let mut context = Context::default();

    // Test that readyState property is read-only
    let result = context.eval(Source::from_bytes(r#"
        let ws = new WebSocketStream('ws://example.com');
        let originalState = ws.readyState;
        try { ws.readyState = 999; } catch(e) {}
        ws.readyState === originalState
    "#)).unwrap();
    assert_eq!(result, JsValue::from(true));
}