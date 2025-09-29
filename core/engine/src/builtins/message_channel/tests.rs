//! Tests for the MessageChannel builtin

use crate::{Context, Source, JsString, JsValue};

#[test]
fn message_channel_constructor_exists() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("typeof MessageChannel"));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));
}

#[test]
fn message_channel_constructor_requires_new() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("MessageChannel()"));
    assert!(result.is_err());

    // Should throw TypeError when called without 'new'
    if let Err(err) = result {
        let error_string = format!("{}", err);
        assert!(error_string.contains("requires 'new'"));
    }
}

#[test]
fn message_channel_constructor_basic() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("new MessageChannel()"));
    assert!(result.is_ok());

    let channel = result.unwrap();
    assert!(channel.is_object());
}

#[test]
fn message_channel_has_port1_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let channel = new MessageChannel(); typeof channel.port1"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("object")));
}

#[test]
fn message_channel_has_port2_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let channel = new MessageChannel(); typeof channel.port2"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("object")));
}

#[test]
fn message_channel_ports_are_different() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let channel = new MessageChannel(); channel.port1 !== channel.port2"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert!(value.to_boolean());
}

#[test]
fn message_channel_ports_have_post_message() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let channel = new MessageChannel(); typeof channel.port1.postMessage"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes(
        "let channel = new MessageChannel(); typeof channel.port2.postMessage"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));
}

#[test]
fn message_channel_ports_have_start() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let channel = new MessageChannel(); typeof channel.port1.start"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));
}

#[test]
fn message_channel_ports_have_close() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let channel = new MessageChannel(); typeof channel.port1.close"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));
}

#[test]
fn message_channel_port_post_message_basic() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let channel = new MessageChannel(); channel.port1.postMessage('hello'); 'success'"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("success")));
}

#[test]
fn message_channel_port_start_basic() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let channel = new MessageChannel(); channel.port1.start(); 'success'"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("success")));
}

#[test]
fn message_channel_port_close_basic() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let channel = new MessageChannel(); channel.port1.close(); 'success'"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("success")));
}

#[test]
fn message_channel_ports_readonly() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let channel = new MessageChannel(); let orig = channel.port1; channel.port1 = null; channel.port1 === orig"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    // port1 should still be the original value since it's readonly
    assert!(value.to_boolean());
}

#[test]
fn message_channel_name_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("MessageChannel.name"));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("MessageChannel")));
}

#[test]
fn message_channel_length_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("MessageChannel.length"));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(0));
}