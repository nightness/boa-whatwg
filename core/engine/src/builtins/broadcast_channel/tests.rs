//! Tests for the BroadcastChannel builtin

use crate::{Context, Source, JsString, JsValue};

#[test]
fn broadcast_channel_constructor_exists() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("typeof BroadcastChannel"));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));
}

#[test]
fn broadcast_channel_constructor_requires_new() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("BroadcastChannel('test')"));
    assert!(result.is_err());

    // Should throw TypeError when called without 'new'
    if let Err(err) = result {
        let error_string = format!("{}", err);
        assert!(error_string.contains("requires 'new'"));
    }
}

#[test]
fn broadcast_channel_constructor_with_name() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("new BroadcastChannel('test-channel')"));
    assert!(result.is_ok());

    let channel = result.unwrap();
    assert!(channel.is_object());
}

#[test]
fn broadcast_channel_has_name_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let channel = new BroadcastChannel('my-channel'); channel.name"
    ));
    assert!(result.is_ok());

    let name = result.unwrap();
    assert_eq!(name, JsValue::from(JsString::from("my-channel")));
}

#[test]
fn broadcast_channel_name_is_readonly() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let channel = new BroadcastChannel('test'); channel.name = 'changed'; channel.name"
    ));
    assert!(result.is_ok());

    let name = result.unwrap();
    // Name should still be 'test' since it's readonly
    assert_eq!(name, JsValue::from(JsString::from("test")));
}

#[test]
fn broadcast_channel_has_post_message_method() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let channel = new BroadcastChannel('test'); typeof channel.postMessage"
    ));
    assert!(result.is_ok());

    let method_type = result.unwrap();
    assert_eq!(method_type, JsValue::from(JsString::from("function")));
}

#[test]
fn broadcast_channel_has_close_method() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let channel = new BroadcastChannel('test'); typeof channel.close"
    ));
    assert!(result.is_ok());

    let method_type = result.unwrap();
    assert_eq!(method_type, JsValue::from(JsString::from("function")));
}

#[test]
fn broadcast_channel_has_onmessage_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let channel = new BroadcastChannel('test'); channel.onmessage"
    ));
    assert!(result.is_ok());

    let onmessage = result.unwrap();
    // Should be null initially
    assert!(onmessage.is_null());
}

#[test]
fn broadcast_channel_post_message_basic() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let channel = new BroadcastChannel('test'); channel.postMessage('hello'); 'success'"
    ));
    assert!(result.is_ok());

    let success = result.unwrap();
    assert_eq!(success, JsValue::from(JsString::from("success")));
}

#[test]
fn broadcast_channel_close_basic() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let channel = new BroadcastChannel('test'); channel.close(); 'success'"
    ));
    assert!(result.is_ok());

    let success = result.unwrap();
    assert_eq!(success, JsValue::from(JsString::from("success")));
}

#[test]
fn broadcast_channel_post_message_after_close() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let channel = new BroadcastChannel('test'); channel.close(); channel.postMessage('test');"
    ));
    assert!(result.is_err());

    // Should throw error when posting to closed channel
    if let Err(err) = result {
        let error_string = format!("{}", err);
        assert!(error_string.contains("closed"));
    }
}

#[test]
fn broadcast_channel_multiple_instances_same_name() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let ch1 = new BroadcastChannel('shared'); let ch2 = new BroadcastChannel('shared'); 'success'"
    ));
    assert!(result.is_ok());

    let success = result.unwrap();
    assert_eq!(success, JsValue::from(JsString::from("success")));
}

#[test]
fn broadcast_channel_message_broadcasting() {
    let mut context = Context::default();

    // This test sets up message broadcasting between channels
    let result = context.eval(Source::from_bytes(
        r#"
        let received = null;
        let ch1 = new BroadcastChannel('test');
        let ch2 = new BroadcastChannel('test');

        ch2.onmessage = function(event) {
            received = event.data;
        };

        ch1.postMessage('hello world');

        // In a real implementation, the message would be delivered asynchronously
        // For now, we just test that the setup works
        'broadcast-test-complete'
        "#
    ));
    assert!(result.is_ok());

    let result_val = result.unwrap();
    assert_eq!(result_val, JsValue::from(JsString::from("broadcast-test-complete")));
}