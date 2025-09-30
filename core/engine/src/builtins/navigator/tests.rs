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
fn test_navigator_id_properties() {
    let mut context = Context::default();

    // Test appCodeName property
    let result = context.eval(Source::from_bytes("window.navigator.appCodeName")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("Mozilla")));

    // Test appName property
    let result = context.eval(Source::from_bytes("window.navigator.appName")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("Netscape")));

    // Test appVersion property
    let result = context.eval(Source::from_bytes("typeof window.navigator.appVersion")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("string")));

    let result = context.eval(Source::from_bytes("window.navigator.appVersion.includes('Chrome')")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test product property
    let result = context.eval(Source::from_bytes("window.navigator.product")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("Gecko")));

    // Test productSub property
    let result = context.eval(Source::from_bytes("window.navigator.productSub")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("20030107")));

    // Test vendor property
    let result = context.eval(Source::from_bytes("window.navigator.vendor")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("Google Inc.")));

    // Test vendorSub property
    let result = context.eval(Source::from_bytes("window.navigator.vendorSub")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("")));
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
fn test_navigator_language_properties() {
    let mut context = Context::default();

    // Test languages property
    let result = context.eval(Source::from_bytes("typeof window.navigator.languages")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));

    let result = context.eval(Source::from_bytes("Array.isArray(window.navigator.languages)")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("window.navigator.languages.length")).unwrap();
    assert_eq!(result, JsValue::from(2));

    let result = context.eval(Source::from_bytes("window.navigator.languages[0]")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("en-US")));

    let result = context.eval(Source::from_bytes("window.navigator.languages[1]")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("en")));
}

#[test]
fn test_navigator_plugins_properties() {
    let mut context = Context::default();

    // Test plugins property
    let result = context.eval(Source::from_bytes("typeof window.navigator.plugins")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));

    let result = context.eval(Source::from_bytes("Array.isArray(window.navigator.plugins)")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("window.navigator.plugins.length")).unwrap();
    assert_eq!(result, JsValue::from(0)); // Empty for security

    // Test mimeTypes property
    let result = context.eval(Source::from_bytes("typeof window.navigator.mimeTypes")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));

    let result = context.eval(Source::from_bytes("Array.isArray(window.navigator.mimeTypes)")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("window.navigator.mimeTypes.length")).unwrap();
    assert_eq!(result, JsValue::from(0)); // Empty for security

    // Test pdfViewerEnabled property
    let result = context.eval(Source::from_bytes("typeof window.navigator.pdfViewerEnabled")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("boolean")));

    let result = context.eval(Source::from_bytes("window.navigator.pdfViewerEnabled")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test javaEnabled method
    let result = context.eval(Source::from_bytes("typeof window.navigator.javaEnabled")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("window.navigator.javaEnabled()")).unwrap();
    assert_eq!(result, JsValue::from(false)); // Always false for security
}

#[test]
fn test_navigator_protocol_handlers() {
    let mut context = Context::default();

    // Test registerProtocolHandler method exists
    let result = context.eval(Source::from_bytes("typeof window.navigator.registerProtocolHandler")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    // Test unregisterProtocolHandler method exists
    let result = context.eval(Source::from_bytes("typeof window.navigator.unregisterProtocolHandler")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    // Test successful protocol handler registration
    let result = context.eval(Source::from_bytes(
        "window.navigator.registerProtocolHandler('mailto', 'https://example.com/compose?to=%s')"
    ));
    assert!(result.is_ok());

    // Test successful protocol handler unregistration
    let result = context.eval(Source::from_bytes(
        "window.navigator.unregisterProtocolHandler('mailto', 'https://example.com/compose?to=%s')"
    ));
    assert!(result.is_ok());

    // Test error on empty scheme
    let result = context.eval(Source::from_bytes(
        "window.navigator.registerProtocolHandler('', 'https://example.com/%s')"
    ));
    assert!(result.is_err());

    // Test error on URL without %s placeholder
    let result = context.eval(Source::from_bytes(
        "window.navigator.registerProtocolHandler('test', 'https://example.com/')"
    ));
    assert!(result.is_err());
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