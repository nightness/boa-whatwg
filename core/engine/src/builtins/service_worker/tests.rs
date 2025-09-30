//! Tests for the ServiceWorker builtin

use crate::{Context, Source, JsString, JsValue};

#[test]
fn service_worker_constructor_exists() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("typeof ServiceWorker"));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));
}

#[test]
fn service_worker_constructor_requires_new() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("ServiceWorker('test.js')"));
    assert!(result.is_err());

    // Should throw TypeError when called without 'new'
    if let Err(err) = result {
        let error_string = format!("{}", err);
        assert!(error_string.contains("requires 'new'"));
    }
}

#[test]
fn service_worker_constructor_requires_script_url() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("new ServiceWorker()"));
    assert!(result.is_err());

    if let Err(err) = result {
        let error_string = format!("{}", err);
        assert!(error_string.contains("requires a script URL"));
    }
}

#[test]
fn service_worker_constructor_with_valid_url() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("new ServiceWorker('https://example.com/sw.js')"));
    assert!(result.is_ok());

    let worker = result.unwrap();
    assert!(worker.is_object());
}

#[test]
fn service_worker_constructor_with_invalid_url() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("new ServiceWorker('not-a-valid-url')"));
    assert!(result.is_err());

    if let Err(err) = result {
        let error_string = format!("{}", err);
        assert!(error_string.contains("Invalid ServiceWorker script URL"));
    }
}

#[test]
fn service_worker_constructor_with_options() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "new ServiceWorker('https://example.com/sw.js', { scope: '/app/' })"
    ));
    assert!(result.is_ok());

    let worker = result.unwrap();
    assert!(worker.is_object());
}

#[test]
fn service_worker_has_script_url_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let sw = new ServiceWorker('https://example.com/sw.js'); sw.scriptURL"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("https://example.com/sw.js")));
}

#[test]
fn service_worker_script_url_property_readonly() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let sw = new ServiceWorker('https://example.com/sw.js'); sw.scriptURL = 'changed'; sw.scriptURL"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    // scriptURL should still be the original value since it's readonly
    assert_eq!(value, JsValue::from(JsString::from("https://example.com/sw.js")));
}

#[test]
fn service_worker_has_state_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let sw = new ServiceWorker('https://example.com/sw.js'); typeof sw.state"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("string")));
}

#[test]
fn service_worker_initial_state() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let sw = new ServiceWorker('https://example.com/sw.js'); sw.state"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    // Initial state should be "parsed" or "installing"
    let state_str = value.to_string(&mut context).unwrap().to_std_string_escaped();
    assert!(state_str == "parsed" || state_str == "installing");
}

#[test]
fn service_worker_has_post_message_method() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let sw = new ServiceWorker('https://example.com/sw.js'); typeof sw.postMessage"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));
}

#[test]
fn service_worker_post_message_basic() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let sw = new ServiceWorker('https://example.com/sw.js'); sw.postMessage('hello'); 'success'"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("success")));
}

#[test]
fn service_worker_name_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("ServiceWorker.name"));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("ServiceWorker")));
}

#[test]
fn service_worker_length_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("ServiceWorker.length"));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(1));
}