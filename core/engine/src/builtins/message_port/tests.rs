//! Tests for the MessagePort builtin

use crate::{Context, Source, JsString, JsValue};

#[test]
fn message_port_constructor_not_callable() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("new MessagePort()"));
    assert!(result.is_err());

    // Should throw TypeError when called with 'new'
    if let Err(err) = result {
        let error_string = format!("{}", err);
        assert!(error_string.contains("not directly callable"));
    }
}

#[test]
fn message_port_constructor_without_new() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("MessagePort()"));
    assert!(result.is_err());

    // Should throw TypeError when called without 'new'
    if let Err(err) = result {
        let error_string = format!("{}", err);
        assert!(error_string.contains("not directly callable"));
    }
}

#[test]
fn message_port_constructor_exists() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("typeof MessagePort"));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));
}

#[test]
fn message_port_prototype_methods_exist() {
    let mut context = Context::default();

    // Check that prototype methods exist
    let result = context.eval(Source::from_bytes("typeof MessagePort.prototype.postMessage"));
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof MessagePort.prototype.start"));
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof MessagePort.prototype.close"));
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));
}

#[test]
fn message_port_name_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("MessagePort.name"));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("MessagePort")));
}

#[test]
fn message_port_length_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("MessagePort.length"));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(0));
}