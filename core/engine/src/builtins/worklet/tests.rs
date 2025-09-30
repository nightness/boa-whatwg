//! Tests for the Worklet builtin

use crate::{Context, Source, JsString, JsValue};

#[test]
fn worklet_constructor_exists() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("typeof Worklet"));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));
}

#[test]
fn worklet_constructor_requires_new() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("Worklet()"));
    assert!(result.is_err());

    // Should throw TypeError when called without 'new'
    if let Err(err) = result {
        let error_string = format!("{}", err);
        assert!(error_string.contains("requires 'new'"));
    }
}

#[test]
fn worklet_constructor_basic() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("new Worklet()"));
    assert!(result.is_ok());

    let worklet = result.unwrap();
    assert!(worklet.is_object());
}

#[test]
fn worklet_has_add_module_method() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worklet = new Worklet(); typeof worklet.addModule"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));
}

#[test]
fn worklet_add_module_requires_url() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worklet = new Worklet(); worklet.addModule()"
    ));
    assert!(result.is_err());

    if let Err(err) = result {
        let error_string = format!("{}", err);
        assert!(error_string.contains("requires a module URL"));
    }
}

#[test]
fn worklet_add_module_with_valid_url() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worklet = new Worklet(); worklet.addModule('https://example.com/worklet.js'); 'success'"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("success")));
}

#[test]
fn worklet_add_module_with_invalid_url() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worklet = new Worklet(); worklet.addModule('not-a-valid-url')"
    ));
    assert!(result.is_err());

    if let Err(err) = result {
        let error_string = format!("{}", err);
        assert!(error_string.contains("Invalid module URL"));
    }
}

#[test]
fn worklet_add_module_multiple_calls() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worklet = new Worklet(); worklet.addModule('https://example.com/worklet1.js'); worklet.addModule('https://example.com/worklet2.js'); 'success'"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("success")));
}

#[test]
fn worklet_name_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("Worklet.name"));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("Worklet")));
}

#[test]
fn worklet_length_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("Worklet.length"));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(0));
}

#[test]
fn worklet_instances_are_independent() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worklet1 = new Worklet(); let worklet2 = new Worklet(); worklet1 !== worklet2"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert!(value.to_boolean());
}