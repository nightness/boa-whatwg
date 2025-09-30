//! Tests for the StorageEvent API implementation

use crate::{Context, JsValue, Source, JsString};

#[test]
fn test_storage_event_constructor() {
    let mut context = Context::default();

    // Test that StorageEvent constructor exists
    let result = context.eval(Source::from_bytes("typeof StorageEvent")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    // Test creating a basic StorageEvent
    let result = context.eval(Source::from_bytes("new StorageEvent('storage')")).unwrap();
    assert!(result.is_object());

    // Test creating a StorageEvent with init data
    let result = context.eval(Source::from_bytes(
        "new StorageEvent('storage', { key: 'testKey', oldValue: 'oldVal', newValue: 'newVal', url: 'http://example.com' })"
    )).unwrap();
    assert!(result.is_object());
}

#[test]
fn test_storage_event_properties() {
    let mut context = Context::default();

    // Create a StorageEvent with all properties
    context.eval(Source::from_bytes(
        "var event = new StorageEvent('storage', { key: 'testKey', oldValue: 'oldVal', newValue: 'newVal', url: 'http://example.com' })"
    )).unwrap();

    // Test key property
    let result = context.eval(Source::from_bytes("event.key")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("testKey")));

    // Test oldValue property
    let result = context.eval(Source::from_bytes("event.oldValue")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("oldVal")));

    // Test newValue property
    let result = context.eval(Source::from_bytes("event.newValue")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("newVal")));

    // Test url property
    let result = context.eval(Source::from_bytes("event.url")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("http://example.com")));

    // Test type property
    let result = context.eval(Source::from_bytes("event.type")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("storage")));

    // Test bubbles property
    let result = context.eval(Source::from_bytes("event.bubbles")).unwrap();
    assert_eq!(result, JsValue::from(false));

    // Test cancelable property
    let result = context.eval(Source::from_bytes("event.cancelable")).unwrap();
    assert_eq!(result, JsValue::from(false));
}

#[test]
fn test_storage_event_null_values() {
    let mut context = Context::default();

    // Create a StorageEvent with null values (like for clear operation)
    context.eval(Source::from_bytes(
        "var event = new StorageEvent('storage', { key: null, oldValue: null, newValue: null, url: 'http://example.com' })"
    )).unwrap();

    // Test null properties
    let result = context.eval(Source::from_bytes("event.key")).unwrap();
    assert!(result.is_null());

    let result = context.eval(Source::from_bytes("event.oldValue")).unwrap();
    assert!(result.is_null());

    let result = context.eval(Source::from_bytes("event.newValue")).unwrap();
    assert!(result.is_null());

    // URL should still be set
    let result = context.eval(Source::from_bytes("event.url")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("http://example.com")));
}

#[test]
fn test_storage_event_init_storage_event() {
    let mut context = Context::default();

    // Create a StorageEvent
    context.eval(Source::from_bytes("var event = new StorageEvent('storage')")).unwrap();

    // Use initStorageEvent to set properties
    context.eval(Source::from_bytes(
        "event.initStorageEvent('storage', false, false, 'initKey', 'initOld', 'initNew', 'http://init.com', null)"
    )).unwrap();

    // Test that properties were set
    let result = context.eval(Source::from_bytes("event.key")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("initKey")));

    let result = context.eval(Source::from_bytes("event.oldValue")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("initOld")));

    let result = context.eval(Source::from_bytes("event.newValue")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("initNew")));

    let result = context.eval(Source::from_bytes("event.url")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("http://init.com")));
}

#[test]
fn test_storage_event_default_values() {
    let mut context = Context::default();

    // Create a StorageEvent with minimal arguments
    context.eval(Source::from_bytes("var event = new StorageEvent('storage')")).unwrap();

    // Test default values
    let result = context.eval(Source::from_bytes("event.key")).unwrap();
    assert!(result.is_null());

    let result = context.eval(Source::from_bytes("event.oldValue")).unwrap();
    assert!(result.is_null());

    let result = context.eval(Source::from_bytes("event.newValue")).unwrap();
    assert!(result.is_null());

    let result = context.eval(Source::from_bytes("event.url")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("about:blank")));

    let result = context.eval(Source::from_bytes("event.storageArea")).unwrap();
    assert!(result.is_null());
}