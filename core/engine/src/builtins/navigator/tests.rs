//! Tests for the Navigator implementation

use crate::{Context, JsValue, Source, JsString};

#[test]
fn test_navigator_basic() {
    let mut context = Context::default();

    // Test that window.navigator exists
    let result = context.eval(Source::from_bytes("typeof window.navigator")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));

    // Test that window.navigator is not null
    let result = context.eval(Source::from_bytes("window.navigator !== null")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_navigator_properties() {
    let mut context = Context::default();

    // Test userAgent property
    let result = context.eval(Source::from_bytes("typeof window.navigator.userAgent")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("string")));

    let result = context.eval(Source::from_bytes("window.navigator.userAgent.includes('Chrome')")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test platform property
    let result = context.eval(Source::from_bytes("typeof window.navigator.platform")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("string")));

    let result = context.eval(Source::from_bytes("window.navigator.platform")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("MacIntel")));

    // Test language property
    let result = context.eval(Source::from_bytes("typeof window.navigator.language")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("string")));

    let result = context.eval(Source::from_bytes("window.navigator.language")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("en-US")));

    // Test cookieEnabled property
    let result = context.eval(Source::from_bytes("typeof window.navigator.cookieEnabled")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("boolean")));

    let result = context.eval(Source::from_bytes("window.navigator.cookieEnabled")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test onLine property
    let result = context.eval(Source::from_bytes("typeof window.navigator.onLine")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("boolean")));

    let result = context.eval(Source::from_bytes("window.navigator.onLine")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_navigator_locks_property() {
    let mut context = Context::default();

    // Test that navigator.locks exists
    let result = context.eval(Source::from_bytes("typeof window.navigator.locks")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));

    // Test that navigator.locks is not null
    let result = context.eval(Source::from_bytes("window.navigator.locks !== null")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test that navigator.locks has request method
    let result = context.eval(Source::from_bytes("typeof window.navigator.locks.request")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    // Test that navigator.locks has query method
    let result = context.eval(Source::from_bytes("typeof window.navigator.locks.query")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_navigator_readonly_properties() {
    let mut context = Context::default();

    // Test that properties are readonly (attempting to change should not work)
    let result = context.eval(Source::from_bytes(r#"
        var original = window.navigator.userAgent;
        window.navigator.userAgent = "Modified";
        window.navigator.userAgent === original;
    "#)).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes(r#"
        var original = window.navigator.platform;
        window.navigator.platform = "Modified";
        window.navigator.platform === original;
    "#)).unwrap();
    assert_eq!(result, JsValue::from(true));
}