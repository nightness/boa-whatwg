//! Tests for the Worker builtin

use crate::{Context, JsValue, Source, JsString};

#[test]
fn worker_constructor_exists() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("typeof Worker"));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));
}

#[test]
fn worker_constructor_requires_new() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("Worker('test.js')"));
    assert!(result.is_err());

    // Should throw TypeError when called without 'new'
    if let Err(err) = result {
        let error_string = format!("{}", err);
        assert!(error_string.contains("requires 'new'"));
    }
}

#[test]
fn worker_constructor_with_valid_url() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("new Worker('https://example.com/worker.js')"));
    assert!(result.is_ok());

    let worker = result.unwrap();
    assert!(worker.is_object());
}

#[test]
fn worker_constructor_with_invalid_url() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("new Worker('not-a-valid-url')"));
    assert!(result.is_err());

    if let Err(err) = result {
        let error_string = format!("{}", err);
        assert!(error_string.contains("Invalid Worker script URL"));
    }
}

#[test]
fn worker_has_post_message_method() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worker = new Worker('https://example.com/worker.js'); typeof worker.postMessage"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));
}

#[test]
fn worker_has_terminate_method() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worker = new Worker('https://example.com/worker.js'); typeof worker.terminate"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));
}

#[test]
fn worker_script_url_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worker = new Worker('https://example.com/worker.js'); worker.scriptURL"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("https://example.com/worker.js")));
}

#[test]
fn worker_post_message_basic() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worker = new Worker('https://example.com/worker.js'); worker.postMessage('hello'); 'success'"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("success")));
}

#[test]
fn worker_terminate_basic() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worker = new Worker('https://example.com/worker.js'); worker.terminate(); 'success'"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("success")));
}

#[test]
fn worker_has_onmessage_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worker = new Worker('https://example.com/worker.js'); worker.onmessage"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    // Should be null initially
    assert!(value.is_null());
}

#[test]
fn worker_has_onerror_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worker = new Worker('https://example.com/worker.js'); worker.onerror"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    // Should be null initially
    assert!(value.is_null());
}

#[test]
fn worker_has_onmessageerror_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worker = new Worker('https://example.com/worker.js'); worker.onmessageerror"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    // Should be null initially
    assert!(value.is_null());
}

#[test]
fn worker_can_set_onmessage_handler() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worker = new Worker('https://example.com/worker.js'); worker.onmessage = function() {}; typeof worker.onmessage"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));
}

#[test]
fn worker_can_set_onerror_handler() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worker = new Worker('https://example.com/worker.js'); worker.onerror = function() {}; typeof worker.onerror"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));
}

#[test]
fn worker_name_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("Worker.name"));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("Worker")));
}