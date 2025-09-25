//! Tests for the Web Locks API implementation

use crate::{Context, JsValue, Source, JsString, js_string};

#[test]
fn test_web_locks_api_basic() {
    let mut context = Context::default();

    // Test that window.navigator.locks exists (window should now be set up by default)
    let result = context.eval(Source::from_bytes("typeof window.navigator.locks")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));

    // Test that window.navigator.locks is not null
    let result = context.eval(Source::from_bytes("window.navigator.locks !== null")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_web_locks_api_methods() {
    let mut context = Context::default();

    // Test that window.navigator.locks.request exists and is a function
    let result = context.eval(Source::from_bytes("typeof window.navigator.locks.request")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    // Test that window.navigator.locks.query exists and is a function
    let result = context.eval(Source::from_bytes("typeof window.navigator.locks.query")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_web_locks_query() {
    let mut context = Context::default();

    // Test that query returns a Promise
    let result = context.eval(Source::from_bytes("window.navigator.locks.query() instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_web_locks_request_basic() {
    let mut context = Context::default();

    // Test that request returns a Promise
    let result = context.eval(Source::from_bytes("window.navigator.locks.request('test-lock', () => 'success') instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_web_locks_request_with_options() {
    let mut context = Context::default();

    // Test that request with options returns a Promise
    let result = context.eval(Source::from_bytes("window.navigator.locks.request('test-lock', {mode: 'shared'}, () => 'success') instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_web_locks_request_if_available() {
    let mut context = Context::default();

    // Test that request with ifAvailable option returns a Promise
    let result = context.eval(Source::from_bytes("window.navigator.locks.request('busy-lock', {ifAvailable: true}, () => 'available') instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_web_locks_error_handling() {
    let mut context = Context::default();

    // Test missing callback throws error
    let result = context.eval(Source::from_bytes("try { window.navigator.locks.request('test'); false; } catch(e) { true; }"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsValue::from(true));

    // Test missing arguments throws error
    let result = context.eval(Source::from_bytes("try { window.navigator.locks.request(); false; } catch(e) { true; }"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsValue::from(true));
}