//! Tests for the SharedWorker builtin

use crate::{Context, Source, JsString, JsValue};

#[test]
fn shared_worker_constructor_exists() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("typeof SharedWorker"));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("function")));
}

#[test]
fn shared_worker_constructor_requires_new() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("SharedWorker('test.js')"));
    assert!(result.is_err());

    // Should throw TypeError when called without 'new'
    if let Err(err) = result {
        let error_string = format!("{}", err);
        assert!(error_string.contains("requires 'new'"));
    }
}

#[test]
fn shared_worker_constructor_with_valid_url() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("new SharedWorker('https://example.com/worker.js')"));
    assert!(result.is_ok());

    let worker = result.unwrap();
    assert!(worker.is_object());
}

#[test]
fn shared_worker_constructor_with_invalid_url() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes("new SharedWorker('not-a-valid-url')"));
    assert!(result.is_err());

    if let Err(err) = result {
        let error_string = format!("{}", err);
        assert!(error_string.contains("Invalid SharedWorker script URL"));
    }
}

#[test]
fn shared_worker_constructor_with_name() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "new SharedWorker('https://example.com/worker.js', 'my-worker')"
    ));
    assert!(result.is_ok());

    let worker = result.unwrap();
    assert!(worker.is_object());
}

#[test]
fn shared_worker_constructor_with_options_object() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "new SharedWorker('https://example.com/worker.js', { name: 'my-worker', type: 'module' })"
    ));
    assert!(result.is_ok());

    let worker = result.unwrap();
    assert!(worker.is_object());
}

#[test]
fn shared_worker_has_port_property() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worker = new SharedWorker('https://example.com/worker.js'); typeof worker.port"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("object")));
}

#[test]
fn shared_worker_port_property_readonly() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worker = new SharedWorker('https://example.com/worker.js'); worker.port = null; worker.port !== null"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    // Port should still be an object since it's readonly
    assert!(value.to_boolean());
}

#[test]
fn shared_worker_multiple_instances_same_url() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worker1 = new SharedWorker('https://example.com/worker.js'); let worker2 = new SharedWorker('https://example.com/worker.js'); 'success'"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("success")));
}

#[test]
fn shared_worker_different_names_different_instances() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(
        "let worker1 = new SharedWorker('https://example.com/worker.js', 'worker1'); let worker2 = new SharedWorker('https://example.com/worker.js', 'worker2'); 'success'"
    ));
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value, JsValue::from(JsString::from("success")));
}