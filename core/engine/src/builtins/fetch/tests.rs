//! Tests for the Fetch API implementation

use crate::{Context, JsValue, Source, JsString};

#[test]
fn test_fetch_global_exists() {
    let mut context = Context::default();

    // Test that fetch exists globally
    let result = context.eval(Source::from_bytes("typeof fetch")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_fetch_constructor_objects_exist() {
    let mut context = Context::default();

    // Test Request constructor
    let result = context.eval(Source::from_bytes("typeof Request")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    // Test Response constructor
    let result = context.eval(Source::from_bytes("typeof Response")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    // Test Headers constructor
    let result = context.eval(Source::from_bytes("typeof Headers")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_request_constructor() {
    let mut context = Context::default();

    // Test Request constructor with URL
    let result = context.eval(Source::from_bytes("new Request('https://example.com')")).unwrap();
    assert!(result.is_object());
}

#[test]
fn test_response_constructor() {
    let mut context = Context::default();

    // Test Response constructor
    let result = context.eval(Source::from_bytes("new Response('test body')")).unwrap();
    assert!(result.is_object());

    // Test Response with options
    let result = context.eval(Source::from_bytes("new Response('test', { status: 201, statusText: 'Created' })")).unwrap();
    assert!(result.is_object());
}

#[test]
fn test_response_properties() {
    let mut context = Context::default();

    // Test Response properties
    let result = context.eval(Source::from_bytes("let r = new Response('test', { status: 201 }); r.status")).unwrap();
    assert_eq!(result, JsValue::from(201));

    let result = context.eval(Source::from_bytes("let r2 = new Response('test', { status: 201 }); r2.ok")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_response_text_method() {
    let mut context = Context::default();

    // Test Response.prototype.text() returns a Promise
    let result = context.eval(Source::from_bytes("let r = new Response('test'); typeof r.text")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("let r2 = new Response('test'); r2.text() instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_response_json_method() {
    let mut context = Context::default();

    // Test Response.prototype.json() returns a Promise
    let result = context.eval(Source::from_bytes("let r = new Response('{}'); typeof r.json")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("let r2 = new Response('{}'); r2.json() instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_headers_constructor() {
    let mut context = Context::default();

    // Test Headers constructor
    let result = context.eval(Source::from_bytes("new Headers()")).unwrap();
    assert!(result.is_object());
}

#[test]
fn test_fetch_returns_promise() {
    let mut context = Context::default();

    // Test that fetch returns a Promise (even though it will fail due to invalid URL)
    let result = context.eval(Source::from_bytes("fetch('https://httpbin.org/get') instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));
}