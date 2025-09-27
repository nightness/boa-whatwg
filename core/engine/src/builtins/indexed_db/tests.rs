//! Tests for the IndexedDB API implementation

use crate::{Context, JsValue, Source, JsString};

#[test]
fn test_indexed_db_api_basic() {
    let mut context = Context::default();

    // Test that window.indexedDB exists
    let result = context.eval(Source::from_bytes("typeof window.indexedDB")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));

    // Test that window.indexedDB is not null
    let result = context.eval(Source::from_bytes("window.indexedDB !== null")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_indexed_db_api_methods() {
    let mut context = Context::default();

    // Test that all main methods exist
    let result = context.eval(Source::from_bytes("typeof window.indexedDB.open")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof window.indexedDB.deleteDatabase")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof window.indexedDB.databases")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof window.indexedDB.cmp")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_indexed_db_open() {
    let mut context = Context::default();

    // Test that open returns an object (IDBRequest)
    let result = context.eval(Source::from_bytes("typeof window.indexedDB.open('test-db')")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));

    // Test that open result is not null
    let result = context.eval(Source::from_bytes("window.indexedDB.open('test-db') !== null")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_indexed_db_open_with_version() {
    let mut context = Context::default();

    // Test that open with version works
    let result = context.eval(Source::from_bytes("typeof window.indexedDB.open('test-db', 2)")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));

    let result = context.eval(Source::from_bytes("window.indexedDB.open('test-db', 2) !== null")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_indexed_db_delete_database() {
    let mut context = Context::default();

    // Test that deleteDatabase returns an object (IDBRequest)
    let result = context.eval(Source::from_bytes("typeof window.indexedDB.deleteDatabase('test-db')")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));

    let result = context.eval(Source::from_bytes("window.indexedDB.deleteDatabase('test-db') !== null")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_indexed_db_databases() {
    let mut context = Context::default();

    // Test that databases returns a Promise
    let result = context.eval(Source::from_bytes("window.indexedDB.databases() instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_indexed_db_cmp() {
    let mut context = Context::default();

    // Test number comparison
    let result = context.eval(Source::from_bytes("window.indexedDB.cmp(1, 2)")).unwrap();
    assert_eq!(result, JsValue::from(-1));

    let result = context.eval(Source::from_bytes("window.indexedDB.cmp(2, 1)")).unwrap();
    assert_eq!(result, JsValue::from(1));

    let result = context.eval(Source::from_bytes("window.indexedDB.cmp(1, 1)")).unwrap();
    assert_eq!(result, JsValue::from(0));
}

#[test]
fn test_indexed_db_cmp_strings() {
    let mut context = Context::default();

    // Test string comparison
    let result = context.eval(Source::from_bytes("window.indexedDB.cmp('a', 'b')")).unwrap();
    assert_eq!(result, JsValue::from(-1));

    let result = context.eval(Source::from_bytes("window.indexedDB.cmp('b', 'a')")).unwrap();
    assert_eq!(result, JsValue::from(1));

    let result = context.eval(Source::from_bytes("window.indexedDB.cmp('test', 'test')")).unwrap();
    assert_eq!(result, JsValue::from(0));
}

#[test]
fn test_indexed_db_request_properties() {
    let mut context = Context::default();

    // Test that request has expected properties
    let result = context.eval(Source::from_bytes("'readyState' in window.indexedDB.open('test-db')")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("'onsuccess' in window.indexedDB.open('test-db')")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("'onerror' in window.indexedDB.open('test-db')")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test readyState type
    let result = context.eval(Source::from_bytes("typeof window.indexedDB.open('test-db').readyState")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("string")));
}

#[test]
fn test_indexed_db_error_handling() {
    let mut context = Context::default();

    // Test error handling for missing arguments
    let result = context.eval(Source::from_bytes("try { window.indexedDB.open(); false; } catch(e) { true; }"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsValue::from(true));

    let result = context.eval(Source::from_bytes("try { window.indexedDB.deleteDatabase(); false; } catch(e) { true; }"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsValue::from(true));

    let result = context.eval(Source::from_bytes("try { window.indexedDB.cmp(); false; } catch(e) { true; }"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsValue::from(true));
}