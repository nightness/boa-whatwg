//! Tests for the StorageManager API implementation

use crate::{Context, JsValue, Source, JsString};

#[test]
fn test_storage_manager_exists() {
    let mut context = Context::default();

    // Test that navigator.storage exists
    let result = context.eval(Source::from_bytes("typeof navigator.storage")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));

    // Test that navigator.storage is not null
    let result = context.eval(Source::from_bytes("navigator.storage !== null")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_storage_manager_methods() {
    let mut context = Context::default();

    // Test that all main methods exist
    let result = context.eval(Source::from_bytes("typeof navigator.storage.estimate")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof navigator.storage.persist")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof navigator.storage.persisted")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof navigator.storage.getDirectory")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_storage_estimate_returns_promise() {
    let mut context = Context::default();

    // Test that estimate() returns a Promise
    let result = context.eval(Source::from_bytes("navigator.storage.estimate() instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_storage_persist_returns_promise() {
    let mut context = Context::default();

    // Test that persist() returns a Promise
    let result = context.eval(Source::from_bytes("navigator.storage.persist() instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_storage_persisted_returns_promise() {
    let mut context = Context::default();

    // Test that persisted() returns a Promise
    let result = context.eval(Source::from_bytes("navigator.storage.persisted() instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_storage_get_directory_returns_promise() {
    let mut context = Context::default();

    // Test that getDirectory() returns a Promise
    let result = context.eval(Source::from_bytes("navigator.storage.getDirectory() instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_storage_manager_constructor_error() {
    let mut context = Context::default();

    // Attempting to construct StorageManager directly should throw
    let result = context.eval(Source::from_bytes("new StorageManager()"));
    assert!(result.is_err());

    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("StorageManager constructor cannot be called directly"));
}