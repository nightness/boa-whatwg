//! Tests for the Web Storage API implementation (localStorage and sessionStorage)

use boa_engine::{Context, JsValue, Source};

/// Test basic localStorage functionality
#[test]
fn test_local_storage_basic() {
    let mut context = Context::default();

    // Test that localStorage exists
    let result = context.eval(Source::from_bytes("typeof window.localStorage")).unwrap();
    assert_eq!(result, JsValue::from("object"));

    // Test that localStorage has the expected methods and properties
    let result = context.eval(Source::from_bytes("typeof localStorage.getItem")).unwrap();
    assert_eq!(result, JsValue::from("function"));

    let result = context.eval(Source::from_bytes("typeof localStorage.setItem")).unwrap();
    assert_eq!(result, JsValue::from("function"));

    let result = context.eval(Source::from_bytes("typeof localStorage.removeItem")).unwrap();
    assert_eq!(result, JsValue::from("function"));

    let result = context.eval(Source::from_bytes("typeof localStorage.clear")).unwrap();
    assert_eq!(result, JsValue::from("function"));

    let result = context.eval(Source::from_bytes("typeof localStorage.key")).unwrap();
    assert_eq!(result, JsValue::from("function"));

    let result = context.eval(Source::from_bytes("typeof localStorage.length")).unwrap();
    assert_eq!(result, JsValue::from("number"));
}

/// Test basic sessionStorage functionality
#[test]
fn test_session_storage_basic() {
    let mut context = Context::default();

    // Test that sessionStorage exists
    let result = context.eval(Source::from_bytes("typeof window.sessionStorage")).unwrap();
    assert_eq!(result, JsValue::from("object"));

    // Test that sessionStorage has the expected methods and properties
    let result = context.eval(Source::from_bytes("typeof sessionStorage.getItem")).unwrap();
    assert_eq!(result, JsValue::from("function"));

    let result = context.eval(Source::from_bytes("typeof sessionStorage.setItem")).unwrap();
    assert_eq!(result, JsValue::from("function"));

    let result = context.eval(Source::from_bytes("typeof sessionStorage.removeItem")).unwrap();
    assert_eq!(result, JsValue::from("function"));

    let result = context.eval(Source::from_bytes("typeof sessionStorage.clear")).unwrap();
    assert_eq!(result, JsValue::from("function"));

    let result = context.eval(Source::from_bytes("typeof sessionStorage.key")).unwrap();
    assert_eq!(result, JsValue::from("function"));

    let result = context.eval(Source::from_bytes("typeof sessionStorage.length")).unwrap();
    assert_eq!(result, JsValue::from("number"));
}

/// Test localStorage setItem and getItem
#[test]
fn test_local_storage_set_get_item() {
    let mut context = Context::default();

    // Initially empty
    let result = context.eval(Source::from_bytes("localStorage.length")).unwrap();
    assert_eq!(result, JsValue::from(0));

    let result = context.eval(Source::from_bytes("localStorage.getItem('nonexistent')")).unwrap();
    assert!(result.is_null());

    // Set an item
    context.eval(Source::from_bytes("localStorage.setItem('testKey', 'testValue')")).unwrap();

    // Check length increased
    let result = context.eval(Source::from_bytes("localStorage.length")).unwrap();
    assert_eq!(result, JsValue::from(1));

    // Get the item
    let result = context.eval(Source::from_bytes("localStorage.getItem('testKey')")).unwrap();
    assert_eq!(result, JsValue::from("testValue"));

    // Test overwriting
    context.eval(Source::from_bytes("localStorage.setItem('testKey', 'newValue')")).unwrap();
    let result = context.eval(Source::from_bytes("localStorage.getItem('testKey')")).unwrap();
    assert_eq!(result, JsValue::from("newValue"));

    // Length should still be 1
    let result = context.eval(Source::from_bytes("localStorage.length")).unwrap();
    assert_eq!(result, JsValue::from(1));
}

/// Test sessionStorage setItem and getItem
#[test]
fn test_session_storage_set_get_item() {
    let mut context = Context::default();

    // Initially empty
    let result = context.eval(Source::from_bytes("sessionStorage.length")).unwrap();
    assert_eq!(result, JsValue::from(0));

    let result = context.eval(Source::from_bytes("sessionStorage.getItem('nonexistent')")).unwrap();
    assert!(result.is_null());

    // Set an item
    context.eval(Source::from_bytes("sessionStorage.setItem('sessionKey', 'sessionValue')")).unwrap();

    // Check length increased
    let result = context.eval(Source::from_bytes("sessionStorage.length")).unwrap();
    assert_eq!(result, JsValue::from(1));

    // Get the item
    let result = context.eval(Source::from_bytes("sessionStorage.getItem('sessionKey')")).unwrap();
    assert_eq!(result, JsValue::from("sessionValue"));
}

/// Test localStorage removeItem
#[test]
fn test_local_storage_remove_item() {
    let mut context = Context::default();

    // Set some items
    context.eval(Source::from_bytes("localStorage.setItem('key1', 'value1')")).unwrap();
    context.eval(Source::from_bytes("localStorage.setItem('key2', 'value2')")).unwrap();

    // Verify length
    let result = context.eval(Source::from_bytes("localStorage.length")).unwrap();
    assert_eq!(result, JsValue::from(2));

    // Remove one item
    context.eval(Source::from_bytes("localStorage.removeItem('key1')")).unwrap();

    // Check length decreased
    let result = context.eval(Source::from_bytes("localStorage.length")).unwrap();
    assert_eq!(result, JsValue::from(1));

    // Check item was removed
    let result = context.eval(Source::from_bytes("localStorage.getItem('key1')")).unwrap();
    assert!(result.is_null());

    // Check other item still exists
    let result = context.eval(Source::from_bytes("localStorage.getItem('key2')")).unwrap();
    assert_eq!(result, JsValue::from("value2"));

    // Remove non-existent item (should not error)
    context.eval(Source::from_bytes("localStorage.removeItem('nonexistent')")).unwrap();

    // Length should still be 1
    let result = context.eval(Source::from_bytes("localStorage.length")).unwrap();
    assert_eq!(result, JsValue::from(1));
}

/// Test localStorage clear
#[test]
fn test_local_storage_clear() {
    let mut context = Context::default();

    // Set some items
    context.eval(Source::from_bytes("localStorage.setItem('key1', 'value1')")).unwrap();
    context.eval(Source::from_bytes("localStorage.setItem('key2', 'value2')")).unwrap();
    context.eval(Source::from_bytes("localStorage.setItem('key3', 'value3')")).unwrap();

    // Verify length
    let result = context.eval(Source::from_bytes("localStorage.length")).unwrap();
    assert_eq!(result, JsValue::from(3));

    // Clear all items
    context.eval(Source::from_bytes("localStorage.clear()")).unwrap();

    // Check length is 0
    let result = context.eval(Source::from_bytes("localStorage.length")).unwrap();
    assert_eq!(result, JsValue::from(0));

    // Check all items were removed
    let result = context.eval(Source::from_bytes("localStorage.getItem('key1')")).unwrap();
    assert!(result.is_null());

    let result = context.eval(Source::from_bytes("localStorage.getItem('key2')")).unwrap();
    assert!(result.is_null());

    let result = context.eval(Source::from_bytes("localStorage.getItem('key3')")).unwrap();
    assert!(result.is_null());
}

/// Test localStorage key method
#[test]
fn test_local_storage_key() {
    let mut context = Context::default();

    // Initially no keys
    let result = context.eval(Source::from_bytes("localStorage.key(0)")).unwrap();
    assert!(result.is_null());

    // Set some items
    context.eval(Source::from_bytes("localStorage.setItem('firstKey', 'value1')")).unwrap();
    context.eval(Source::from_bytes("localStorage.setItem('secondKey', 'value2')")).unwrap();

    // Get keys (order may vary in HashMap)
    let result1 = context.eval(Source::from_bytes("localStorage.key(0)")).unwrap();
    let result2 = context.eval(Source::from_bytes("localStorage.key(1)")).unwrap();

    // Both results should be strings
    assert!(result1.is_string());
    assert!(result2.is_string());

    // Convert to strings for comparison
    let key1 = result1.to_string(&mut context).unwrap().to_std_string_escaped();
    let key2 = result2.to_string(&mut context).unwrap().to_std_string_escaped();

    // Should contain both keys (in some order)
    assert!(key1 == "firstKey" || key1 == "secondKey");
    assert!(key2 == "firstKey" || key2 == "secondKey");
    assert_ne!(key1, key2);

    // Out of bounds should return null
    let result = context.eval(Source::from_bytes("localStorage.key(2)")).unwrap();
    assert!(result.is_null());

    let result = context.eval(Source::from_bytes("localStorage.key(100)")).unwrap();
    assert!(result.is_null());
}

/// Test Storage constructor cannot be called directly
#[test]
fn test_storage_constructor_error() {
    let mut context = Context::default();

    // Attempting to construct Storage directly should throw
    let result = context.eval(Source::from_bytes("new Storage()"));
    assert!(result.is_err());

    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("Storage constructor cannot be called directly"));
}

/// Test that localStorage and sessionStorage are separate
#[test]
fn test_storage_separation() {
    let mut context = Context::default();

    // Set item in localStorage
    context.eval(Source::from_bytes("localStorage.setItem('sharedKey', 'localValue')")).unwrap();

    // Set item in sessionStorage with same key
    context.eval(Source::from_bytes("sessionStorage.setItem('sharedKey', 'sessionValue')")).unwrap();

    // Values should be separate
    let local_result = context.eval(Source::from_bytes("localStorage.getItem('sharedKey')")).unwrap();
    assert_eq!(local_result, JsValue::from("localValue"));

    let session_result = context.eval(Source::from_bytes("sessionStorage.getItem('sharedKey')")).unwrap();
    assert_eq!(session_result, JsValue::from("sessionValue"));

    // Lengths should be independent
    let local_length = context.eval(Source::from_bytes("localStorage.length")).unwrap();
    assert_eq!(local_length, JsValue::from(1));

    let session_length = context.eval(Source::from_bytes("sessionStorage.length")).unwrap();
    assert_eq!(session_length, JsValue::from(1));

    // Clear localStorage should not affect sessionStorage
    context.eval(Source::from_bytes("localStorage.clear()")).unwrap();

    let local_length = context.eval(Source::from_bytes("localStorage.length")).unwrap();
    assert_eq!(local_length, JsValue::from(0));

    let session_length = context.eval(Source::from_bytes("sessionStorage.length")).unwrap();
    assert_eq!(session_length, JsValue::from(1));

    let session_result = context.eval(Source::from_bytes("sessionStorage.getItem('sharedKey')")).unwrap();
    assert_eq!(session_result, JsValue::from("sessionValue"));
}

/// Test storing various data types (they should all be converted to strings)
#[test]
fn test_storage_data_types() {
    let mut context = Context::default();

    // Test number
    context.eval(Source::from_bytes("localStorage.setItem('number', 42)")).unwrap();
    let result = context.eval(Source::from_bytes("localStorage.getItem('number')")).unwrap();
    assert_eq!(result, JsValue::from("42"));

    // Test boolean
    context.eval(Source::from_bytes("localStorage.setItem('boolean', true)")).unwrap();
    let result = context.eval(Source::from_bytes("localStorage.getItem('boolean')")).unwrap();
    assert_eq!(result, JsValue::from("true"));

    // Test object (should become string representation)
    context.eval(Source::from_bytes("localStorage.setItem('object', {a: 1})")).unwrap();
    let result = context.eval(Source::from_bytes("localStorage.getItem('object')")).unwrap();
    assert_eq!(result, JsValue::from("[object Object]"));

    // Test array (should become string representation)
    context.eval(Source::from_bytes("localStorage.setItem('array', [1, 2, 3])")).unwrap();
    let result = context.eval(Source::from_bytes("localStorage.getItem('array')")).unwrap();
    assert_eq!(result, JsValue::from("1,2,3"));

    // Test null
    context.eval(Source::from_bytes("localStorage.setItem('null', null)")).unwrap();
    let result = context.eval(Source::from_bytes("localStorage.getItem('null')")).unwrap();
    assert_eq!(result, JsValue::from("null"));

    // Test undefined
    context.eval(Source::from_bytes("localStorage.setItem('undefined', undefined)")).unwrap();
    let result = context.eval(Source::from_bytes("localStorage.getItem('undefined')")).unwrap();
    assert_eq!(result, JsValue::from("undefined"));
}