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

#[test]
fn test_indexed_db_debug_object_store_methods() {
    let mut context = Context::default();

    // Test that object store methods exist
    context.eval(Source::from_bytes(r#"
        var db = window.indexedDB.open('test-db');
        var transaction = db.result.transaction(['test-store'], 'readonly');
        var objectStore = transaction.objectStore('test-store');
    "#)).unwrap();

    // Check if basic methods exist
    let result = context.eval(Source::from_bytes("typeof objectStore.add")).unwrap();
    println!("objectStore.add type: {:?}", result);
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof objectStore.openCursor")).unwrap();
    println!("objectStore.openCursor type: {:?}", result);
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_indexed_db_cursor_basic() {
    let mut context = Context::default();

    // Test cursor creation through openCursor
    context.eval(Source::from_bytes(r#"
        var db = window.indexedDB.open('test-db');
        var transaction = db.result.transaction(['test-store'], 'readonly');
        var objectStore = transaction.objectStore('test-store');
        var cursor = objectStore.openCursor();
    "#)).unwrap();

    // Test that cursor request is created
    let result = context.eval(Source::from_bytes("typeof cursor")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));

    let result = context.eval(Source::from_bytes("cursor !== null")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_indexed_db_cursor_methods() {
    let mut context = Context::default();

    // Create a cursor and test its methods
    context.eval(Source::from_bytes(r#"
        var db = window.indexedDB.open('test-db');
        var transaction = db.result.transaction(['test-store'], 'readonly');
        var objectStore = transaction.objectStore('test-store');
        var cursorRequest = objectStore.openCursor();
        var cursor = cursorRequest.result;
    "#)).unwrap();

    // Test cursor methods exist
    let result = context.eval(Source::from_bytes("typeof cursor.advance")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof cursor.continue")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof cursor.continuePrimaryKey")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof cursor.delete")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof cursor.update")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_indexed_db_cursor_properties() {
    let mut context = Context::default();

    // Create a cursor and test its properties
    context.eval(Source::from_bytes(r#"
        var db = window.indexedDB.open('test-db');
        var transaction = db.result.transaction(['test-store'], 'readonly');
        var objectStore = transaction.objectStore('test-store');
        var cursorRequest = objectStore.openCursor();
        var cursor = cursorRequest.result;
    "#)).unwrap();

    // Test cursor properties exist
    let result = context.eval(Source::from_bytes("'source' in cursor")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("'direction' in cursor")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("'key' in cursor")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("'primaryKey' in cursor")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("'value' in cursor")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test cursor direction default
    let result = context.eval(Source::from_bytes("cursor.direction")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("next")));
}

#[test]
fn test_indexed_db_cursor_with_direction() {
    let mut context = Context::default();

    // Test cursor creation with direction
    context.eval(Source::from_bytes(r#"
        var db = window.indexedDB.open('test-db');
        var transaction = db.result.transaction(['test-store'], 'readonly');
        var objectStore = transaction.objectStore('test-store');
        var cursorRequest = objectStore.openCursor(null, 'prev');
        var cursor = cursorRequest.result;
    "#)).unwrap();

    // Test cursor direction is set correctly
    let result = context.eval(Source::from_bytes("cursor.direction")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("prev")));
}

#[test]
fn test_indexed_db_key_cursor() {
    let mut context = Context::default();

    // Test key cursor creation
    context.eval(Source::from_bytes(r#"
        var db = window.indexedDB.open('test-db');
        var transaction = db.result.transaction(['test-store'], 'readonly');
        var objectStore = transaction.objectStore('test-store');
        var cursorRequest = objectStore.openKeyCursor();
        var cursor = cursorRequest.result;
    "#)).unwrap();

    // Test that key cursor exists
    let result = context.eval(Source::from_bytes("typeof cursor")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));

    let result = context.eval(Source::from_bytes("cursor !== null")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test key cursor has key but not value (unlike regular cursor with value)
    let result = context.eval(Source::from_bytes("'key' in cursor")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("'primaryKey' in cursor")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_indexed_db_get_all_methods() {
    let mut context = Context::default();

    // Test getAll and getAllKeys methods
    context.eval(Source::from_bytes(r#"
        var db = window.indexedDB.open('test-db');
        var transaction = db.result.transaction(['test-store'], 'readonly');
        var objectStore = transaction.objectStore('test-store');
    "#)).unwrap();

    // Test that getAll method exists
    let result = context.eval(Source::from_bytes("typeof objectStore.getAll")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    // Test that getAllKeys method exists
    let result = context.eval(Source::from_bytes("typeof objectStore.getAllKeys")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    // Test getAll returns a request
    context.eval(Source::from_bytes("var getAllRequest = objectStore.getAll();")).unwrap();
    let result = context.eval(Source::from_bytes("typeof getAllRequest")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));

    // Test getAllKeys returns a request
    context.eval(Source::from_bytes("var getAllKeysRequest = objectStore.getAllKeys();")).unwrap();
    let result = context.eval(Source::from_bytes("typeof getAllKeysRequest")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));
}

#[test]
fn test_indexed_db_cursor_functionality() {
    let mut context = Context::default();

    // Test basic cursor functionality
    context.eval(Source::from_bytes(r#"
        var db = window.indexedDB.open('test-db');
        var transaction = db.result.transaction(['test-store'], 'readonly');
        var objectStore = transaction.objectStore('test-store');
        var cursorRequest = objectStore.openCursor();
        var cursor = cursorRequest.result;
    "#)).unwrap();

    // Test cursor has initial key and value
    let result = context.eval(Source::from_bytes("cursor.key !== null")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("cursor.value !== undefined")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test cursor methods can be called
    let result = context.eval(Source::from_bytes("cursor.advance(1); true")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("cursor.continue(); true")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test cursor delete and update return requests
    context.eval(Source::from_bytes("var deleteRequest = cursor.delete();")).unwrap();
    let result = context.eval(Source::from_bytes("typeof deleteRequest")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));

    context.eval(Source::from_bytes("var updateRequest = cursor.update('new value');")).unwrap();
    let result = context.eval(Source::from_bytes("typeof updateRequest")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));
}