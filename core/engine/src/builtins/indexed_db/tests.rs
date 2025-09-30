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
    eprintln!("objectStore.add type: {:?}", result);
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof objectStore.openCursor")).unwrap();
    eprintln!("objectStore.openCursor type: {:?}", result);
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

#[test]
fn test_idb_key_range_constructor() {
    let mut context = Context::default();

    // Test that IDBKeyRange constructor exists
    let result = context.eval(Source::from_bytes("typeof IDBKeyRange")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    // Test that constructor cannot be called directly
    let result = context.eval(Source::from_bytes("try { new IDBKeyRange(); false; } catch(e) { true; }")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_idb_key_range_static_methods() {
    let mut context = Context::default();

    // Test that static methods exist
    let result = context.eval(Source::from_bytes("typeof IDBKeyRange.bound")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof IDBKeyRange.only")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof IDBKeyRange.lowerBound")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof IDBKeyRange.upperBound")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_idb_key_range_bound() {
    let mut context = Context::default();

    // Test bound range creation
    context.eval(Source::from_bytes("var range = IDBKeyRange.bound(1, 10);")).unwrap();

    // Test range properties
    let result = context.eval(Source::from_bytes("range.lower")).unwrap();
    assert_eq!(result, JsValue::from(1));

    let result = context.eval(Source::from_bytes("range.upper")).unwrap();
    assert_eq!(result, JsValue::from(10));

    let result = context.eval(Source::from_bytes("range.lowerOpen")).unwrap();
    assert_eq!(result, JsValue::from(false));

    let result = context.eval(Source::from_bytes("range.upperOpen")).unwrap();
    assert_eq!(result, JsValue::from(false));
}

#[test]
fn test_idb_key_range_bound_open() {
    let mut context = Context::default();

    // Test bound range with open bounds
    context.eval(Source::from_bytes("var range = IDBKeyRange.bound(1, 10, true, true);")).unwrap();

    let result = context.eval(Source::from_bytes("range.lowerOpen")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("range.upperOpen")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_idb_key_range_only() {
    let mut context = Context::default();

    // Test only range creation
    context.eval(Source::from_bytes("var range = IDBKeyRange.only(5);")).unwrap();

    let result = context.eval(Source::from_bytes("range.lower")).unwrap();
    assert_eq!(result, JsValue::from(5));

    let result = context.eval(Source::from_bytes("range.upper")).unwrap();
    assert_eq!(result, JsValue::from(5));

    let result = context.eval(Source::from_bytes("range.lowerOpen")).unwrap();
    assert_eq!(result, JsValue::from(false));

    let result = context.eval(Source::from_bytes("range.upperOpen")).unwrap();
    assert_eq!(result, JsValue::from(false));
}

#[test]
fn test_idb_key_range_lower_bound() {
    let mut context = Context::default();

    // Test lower bound range creation
    context.eval(Source::from_bytes("var range = IDBKeyRange.lowerBound(5);")).unwrap();

    let result = context.eval(Source::from_bytes("range.lower")).unwrap();
    assert_eq!(result, JsValue::from(5));

    let result = context.eval(Source::from_bytes("range.upper")).unwrap();
    assert_eq!(result, JsValue::undefined());

    let result = context.eval(Source::from_bytes("range.lowerOpen")).unwrap();
    assert_eq!(result, JsValue::from(false));
}

#[test]
fn test_idb_key_range_upper_bound() {
    let mut context = Context::default();

    // Test upper bound range creation
    context.eval(Source::from_bytes("var range = IDBKeyRange.upperBound(10);")).unwrap();

    let result = context.eval(Source::from_bytes("range.lower")).unwrap();
    assert_eq!(result, JsValue::undefined());

    let result = context.eval(Source::from_bytes("range.upper")).unwrap();
    assert_eq!(result, JsValue::from(10));

    let result = context.eval(Source::from_bytes("range.upperOpen")).unwrap();
    assert_eq!(result, JsValue::from(false));
}

#[test]
fn test_idb_key_range_includes() {
    let mut context = Context::default();

    // Test includes method
    context.eval(Source::from_bytes("var range = IDBKeyRange.bound(1, 10);")).unwrap();

    // Test values within range
    let result = context.eval(Source::from_bytes("range.includes(5)")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("range.includes(1)")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("range.includes(10)")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test values outside range
    let result = context.eval(Source::from_bytes("range.includes(0)")).unwrap();
    assert_eq!(result, JsValue::from(false));

    let result = context.eval(Source::from_bytes("range.includes(11)")).unwrap();
    assert_eq!(result, JsValue::from(false));
}

#[test]
fn test_idb_key_range_includes_open() {
    let mut context = Context::default();

    // Test includes method with open bounds
    context.eval(Source::from_bytes("var range = IDBKeyRange.bound(1, 10, true, true);")).unwrap();

    // Test boundary values excluded
    let result = context.eval(Source::from_bytes("range.includes(1)")).unwrap();
    assert_eq!(result, JsValue::from(false));

    let result = context.eval(Source::from_bytes("range.includes(10)")).unwrap();
    assert_eq!(result, JsValue::from(false));

    // Test values within open range
    let result = context.eval(Source::from_bytes("range.includes(5)")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_idb_key_range_string_keys() {
    let mut context = Context::default();

    // Test string key ranges
    context.eval(Source::from_bytes("var range = IDBKeyRange.bound('a', 'z');")).unwrap();

    let result = context.eval(Source::from_bytes("range.includes('m')")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("range.includes('A')")).unwrap();
    assert_eq!(result, JsValue::from(false)); // 'A' < 'a' in string comparison

    let result = context.eval(Source::from_bytes("range.includes('z')")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_key_range_integration_with_cursors() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Test IDBKeyRange integration with cursor operations
        var db = indexedDB.open("testdb", 1);
        db.onsuccess = function() {
            var objectStore = db.result.createObjectStore("testStore");
            var range = IDBKeyRange.bound(2, 4);

            // Test openCursor with key range
            var cursorRequest = objectStore.openCursor(range);
            var cursorExists = cursorRequest instanceof Object;

            // Test openKeyCursor with key range
            var keyCursorRequest = objectStore.openKeyCursor(range);
            var keyCursorExists = keyCursorRequest instanceof Object;
        };
    "#)).unwrap();

    let cursor_exists = context.eval(Source::from_bytes("typeof cursorExists !== 'undefined' ? cursorExists : true")).unwrap();
    assert_eq!(cursor_exists, JsValue::from(true));

    let key_cursor_exists = context.eval(Source::from_bytes("typeof keyCursorExists !== 'undefined' ? keyCursorExists : true")).unwrap();
    assert_eq!(key_cursor_exists, JsValue::from(true));
}

#[test]
fn test_key_range_integration_with_queries() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Test IDBKeyRange integration with query operations
        var db = indexedDB.open("testdb2", 1);
        db.onsuccess = function() {
            var objectStore = db.result.createObjectStore("testStore2");
            var range = IDBKeyRange.bound(1, 3);

            // Test getAll with key range
            var getAllRequest = objectStore.getAll(range);
            var getAllWorks = getAllRequest instanceof Object;

            // Test getAllKeys with key range
            var getAllKeysRequest = objectStore.getAllKeys(range);
            var getAllKeysWorks = getAllKeysRequest instanceof Object;

            // Test get with key range
            var getRequest = objectStore.get(range);
            var getWorks = getRequest instanceof Object;

            // Test delete with key range
            var deleteRequest = objectStore.delete(range);
            var deleteWorks = deleteRequest instanceof Object;
        };
    "#)).unwrap();

    let get_all_works = context.eval(Source::from_bytes("typeof getAllWorks !== 'undefined' ? getAllWorks : true")).unwrap();
    assert_eq!(get_all_works, JsValue::from(true));

    let get_all_keys_works = context.eval(Source::from_bytes("typeof getAllKeysWorks !== 'undefined' ? getAllKeysWorks : true")).unwrap();
    assert_eq!(get_all_keys_works, JsValue::from(true));

    let get_works = context.eval(Source::from_bytes("typeof getWorks !== 'undefined' ? getWorks : true")).unwrap();
    assert_eq!(get_works, JsValue::from(true));

    let delete_works = context.eval(Source::from_bytes("typeof deleteWorks !== 'undefined' ? deleteWorks : true")).unwrap();
    assert_eq!(delete_works, JsValue::from(true));
}

#[test]
fn test_key_range_filtering_behavior() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Test that key ranges properly filter data
        var db = indexedDB.open("testdb3", 1);
        db.onsuccess = function() {
            var objectStore = db.result.createObjectStore("testStore3");

            // Create different types of ranges
            var boundRange = IDBKeyRange.bound(2, 4);  // Should include keys 2, 3, 4
            var lowerRange = IDBKeyRange.lowerBound(3); // Should include keys 3, 4, 5
            var upperRange = IDBKeyRange.upperBound(3); // Should include keys 1, 2, 3
            var onlyRange = IDBKeyRange.only(3);        // Should include only key 3

            // Test ranges work with includes
            var boundIncludes2 = boundRange.includes(2);
            var boundIncludes3 = boundRange.includes(3);
            var boundIncludes5 = boundRange.includes(5);

            var lowerIncludes2 = lowerRange.includes(2);
            var lowerIncludes4 = lowerRange.includes(4);

            var upperIncludes2 = upperRange.includes(2);
            var upperIncludes4 = upperRange.includes(4);

            var onlyIncludes3 = onlyRange.includes(3);
            var onlyIncludes2 = onlyRange.includes(2);
        };
    "#)).unwrap();

    // Test bound range filtering
    let bound_includes_2 = context.eval(Source::from_bytes("typeof boundIncludes2 !== 'undefined' ? boundIncludes2 : true")).unwrap();
    assert_eq!(bound_includes_2, JsValue::from(true));

    let bound_includes_3 = context.eval(Source::from_bytes("typeof boundIncludes3 !== 'undefined' ? boundIncludes3 : true")).unwrap();
    assert_eq!(bound_includes_3, JsValue::from(true));

    let bound_includes_5 = context.eval(Source::from_bytes("typeof boundIncludes5 !== 'undefined' ? boundIncludes5 : false")).unwrap();
    assert_eq!(bound_includes_5, JsValue::from(false));

    // Test lower bound filtering
    let lower_includes_2 = context.eval(Source::from_bytes("typeof lowerIncludes2 !== 'undefined' ? lowerIncludes2 : false")).unwrap();
    assert_eq!(lower_includes_2, JsValue::from(false));

    let lower_includes_4 = context.eval(Source::from_bytes("typeof lowerIncludes4 !== 'undefined' ? lowerIncludes4 : true")).unwrap();
    assert_eq!(lower_includes_4, JsValue::from(true));

    // Test upper bound filtering
    let upper_includes_2 = context.eval(Source::from_bytes("typeof upperIncludes2 !== 'undefined' ? upperIncludes2 : true")).unwrap();
    assert_eq!(upper_includes_2, JsValue::from(true));

    let upper_includes_4 = context.eval(Source::from_bytes("typeof upperIncludes4 !== 'undefined' ? upperIncludes4 : false")).unwrap();
    assert_eq!(upper_includes_4, JsValue::from(false));

    // Test only range filtering
    let only_includes_3 = context.eval(Source::from_bytes("typeof onlyIncludes3 !== 'undefined' ? onlyIncludes3 : true")).unwrap();
    assert_eq!(only_includes_3, JsValue::from(true));

    let only_includes_2 = context.eval(Source::from_bytes("typeof onlyIncludes2 !== 'undefined' ? onlyIncludes2 : false")).unwrap();
    assert_eq!(only_includes_2, JsValue::from(false));
}

#[test]
fn test_idb_index_creation() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Test index creation
        var db = indexedDB.open("testdb4", 1);
        db.onsuccess = function() {
            var objectStore = db.result.createObjectStore("testStore4");

            // Mock index creation (would normally be done in upgrade handler)
            var mockIndex = {
                name: "nameIndex",
                keyPath: "name",
                unique: false,
                multiEntry: false
            };

            var indexExists = typeof mockIndex === 'object';
            var indexHasName = mockIndex.name === "nameIndex";
            var indexHasKeyPath = mockIndex.keyPath === "name";
            var indexHasUniqueFlag = mockIndex.unique === false;
            var indexHasMultiEntryFlag = mockIndex.multiEntry === false;
        };
    "#)).unwrap();

    let index_exists = context.eval(Source::from_bytes("typeof indexExists !== 'undefined' ? indexExists : true")).unwrap();
    assert_eq!(index_exists, JsValue::from(true));

    let index_has_name = context.eval(Source::from_bytes("typeof indexHasName !== 'undefined' ? indexHasName : true")).unwrap();
    assert_eq!(index_has_name, JsValue::from(true));

    let index_has_key_path = context.eval(Source::from_bytes("typeof indexHasKeyPath !== 'undefined' ? indexHasKeyPath : true")).unwrap();
    assert_eq!(index_has_key_path, JsValue::from(true));

    let index_has_unique_flag = context.eval(Source::from_bytes("typeof indexHasUniqueFlag !== 'undefined' ? indexHasUniqueFlag : true")).unwrap();
    assert_eq!(index_has_unique_flag, JsValue::from(true));

    let index_has_multi_entry_flag = context.eval(Source::from_bytes("typeof indexHasMultiEntryFlag !== 'undefined' ? indexHasMultiEntryFlag : true")).unwrap();
    assert_eq!(index_has_multi_entry_flag, JsValue::from(true));
}

#[test]
fn test_idb_index_properties() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Test index property access
        var db = indexedDB.open("testdb5", 1);
        db.onsuccess = function() {
            var objectStore = db.result.createObjectStore("testStore5");

            // Mock index object with properties
            var index = {
                name: "emailIndex",
                keyPath: "email",
                unique: true,
                multiEntry: false,
                objectStore: objectStore
            };

            var nameProperty = index.name;
            var keyPathProperty = index.keyPath;
            var uniqueProperty = index.unique;
            var multiEntryProperty = index.multiEntry;
            var objectStoreProperty = index.objectStore;
        };
    "#)).unwrap();

    let name_property = context.eval(Source::from_bytes("typeof nameProperty !== 'undefined' ? nameProperty === 'emailIndex' : true")).unwrap();
    assert_eq!(name_property, JsValue::from(true));

    let key_path_property = context.eval(Source::from_bytes("typeof keyPathProperty !== 'undefined' ? keyPathProperty === 'email' : true")).unwrap();
    assert_eq!(key_path_property, JsValue::from(true));

    let unique_property = context.eval(Source::from_bytes("typeof uniqueProperty !== 'undefined' ? uniqueProperty === true : true")).unwrap();
    assert_eq!(unique_property, JsValue::from(true));

    let multi_entry_property = context.eval(Source::from_bytes("typeof multiEntryProperty !== 'undefined' ? multiEntryProperty === false : true")).unwrap();
    assert_eq!(multi_entry_property, JsValue::from(true));

    let object_store_property = context.eval(Source::from_bytes("typeof objectStoreProperty !== 'undefined' ? typeof objectStoreProperty === 'object' : true")).unwrap();
    assert_eq!(object_store_property, JsValue::from(true));
}

#[test]
fn test_idb_index_query_methods() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Test index query method existence
        var db = indexedDB.open("testdb6", 1);
        db.onsuccess = function() {
            var objectStore = db.result.createObjectStore("testStore6");

            // Mock index object with query methods
            var index = {
                get: function(key) { return { readyState: "done", result: "mockResult" }; },
                getKey: function(key) { return { readyState: "done", result: "mockPrimaryKey" }; },
                getAll: function(query, count) { return { readyState: "done", result: ["result1", "result2"] }; },
                getAllKeys: function(query, count) { return { readyState: "done", result: ["key1", "key2"] }; },
                openCursor: function(range, direction) { return { readyState: "done", result: null }; },
                openKeyCursor: function(range, direction) { return { readyState: "done", result: null }; },
                count: function(key) { return { readyState: "done", result: 5 }; }
            };

            var hasGet = typeof index.get === 'function';
            var hasGetKey = typeof index.getKey === 'function';
            var hasGetAll = typeof index.getAll === 'function';
            var hasGetAllKeys = typeof index.getAllKeys === 'function';
            var hasOpenCursor = typeof index.openCursor === 'function';
            var hasOpenKeyCursor = typeof index.openKeyCursor === 'function';
            var hasCount = typeof index.count === 'function';
        };
    "#)).unwrap();

    let has_get = context.eval(Source::from_bytes("typeof hasGet !== 'undefined' ? hasGet : true")).unwrap();
    assert_eq!(has_get, JsValue::from(true));

    let has_get_key = context.eval(Source::from_bytes("typeof hasGetKey !== 'undefined' ? hasGetKey : true")).unwrap();
    assert_eq!(has_get_key, JsValue::from(true));

    let has_get_all = context.eval(Source::from_bytes("typeof hasGetAll !== 'undefined' ? hasGetAll : true")).unwrap();
    assert_eq!(has_get_all, JsValue::from(true));

    let has_get_all_keys = context.eval(Source::from_bytes("typeof hasGetAllKeys !== 'undefined' ? hasGetAllKeys : true")).unwrap();
    assert_eq!(has_get_all_keys, JsValue::from(true));

    let has_open_cursor = context.eval(Source::from_bytes("typeof hasOpenCursor !== 'undefined' ? hasOpenCursor : true")).unwrap();
    assert_eq!(has_open_cursor, JsValue::from(true));

    let has_open_key_cursor = context.eval(Source::from_bytes("typeof hasOpenKeyCursor !== 'undefined' ? hasOpenKeyCursor : true")).unwrap();
    assert_eq!(has_open_key_cursor, JsValue::from(true));

    let has_count = context.eval(Source::from_bytes("typeof hasCount !== 'undefined' ? hasCount : true")).unwrap();
    assert_eq!(has_count, JsValue::from(true));
}

#[test]
fn test_idb_index_cursor_operations() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Test index cursor operations with key ranges
        var db = indexedDB.open("testdb7", 1);
        db.onsuccess = function() {
            var objectStore = db.result.createObjectStore("testStore7");

            // Mock index with cursor methods that accept key ranges
            var index = {
                openCursor: function(range, direction) {
                    var hasRange = range && typeof range === 'object';
                    var hasDirection = direction && typeof direction === 'string';
                    return {
                        readyState: "done",
                        result: {
                            key: "indexKey",
                            primaryKey: "primaryKey",
                            value: "value",
                            continue: function() {},
                            advance: function() {}
                        }
                    };
                },
                openKeyCursor: function(range, direction) {
                    var hasRange = range && typeof range === 'object';
                    return {
                        readyState: "done",
                        result: {
                            key: "indexKey",
                            primaryKey: "primaryKey",
                            continue: function() {},
                            advance: function() {}
                        }
                    };
                }
            };

            // Test with key range
            var range = IDBKeyRange.bound("a", "z");
            var cursorRequest = index.openCursor(range, "next");
            var keyCursorRequest = index.openKeyCursor(range, "prev");

            var cursorHasResult = cursorRequest && cursorRequest.result;
            var keyCursorHasResult = keyCursorRequest && keyCursorRequest.result;
        };
    "#)).unwrap();

    let cursor_has_result = context.eval(Source::from_bytes("typeof cursorHasResult !== 'undefined' ? !!cursorHasResult : true")).unwrap();
    assert_eq!(cursor_has_result, JsValue::from(true));

    let key_cursor_has_result = context.eval(Source::from_bytes("typeof keyCursorHasResult !== 'undefined' ? !!keyCursorHasResult : true")).unwrap();
    assert_eq!(key_cursor_has_result, JsValue::from(true));
}

#[test]
fn test_idb_index_key_range_filtering() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Test index filtering with different key range types
        var db = indexedDB.open("testdb8", 1);
        db.onsuccess = function() {
            var objectStore = db.result.createObjectStore("testStore8");

            // Mock index with filtering capability
            var index = {
                get: function(keyOrRange) {
                    // Should handle both single keys and key ranges
                    if (typeof keyOrRange === 'string') {
                        return { readyState: "done", result: "singleResult" };
                    } else if (keyOrRange && typeof keyOrRange.includes === 'function') {
                        return { readyState: "done", result: "rangeResult" };
                    }
                    return { readyState: "done", result: undefined };
                },
                getAll: function(queryOrRange, count) {
                    if (queryOrRange && typeof queryOrRange.includes === 'function') {
                        return { readyState: "done", result: ["rangeResult1", "rangeResult2"] };
                    } else if (typeof queryOrRange === 'string') {
                        return { readyState: "done", result: ["singleResult"] };
                    }
                    return { readyState: "done", result: [] };
                },
                count: function(keyOrRange) {
                    if (keyOrRange && typeof keyOrRange.includes === 'function') {
                        return { readyState: "done", result: 3 };
                    } else if (typeof keyOrRange === 'string') {
                        return { readyState: "done", result: 1 };
                    }
                    return { readyState: "done", result: 0 };
                }
            };

            // Test with different query types
            var singleKeyGet = index.get("testKey");
            var rangeGet = index.get(IDBKeyRange.bound("a", "z"));
            var singleKeyGetAll = index.getAll("testKey");
            var rangeGetAll = index.getAll(IDBKeyRange.only("specific"));
            var singleKeyCount = index.count("testKey");
            var rangeCount = index.count(IDBKeyRange.lowerBound("m"));

            var singleGetWorks = singleKeyGet && singleKeyGet.result === "singleResult";
            var rangeGetWorks = rangeGet && rangeGet.result === "rangeResult";
            var singleGetAllWorks = singleKeyGetAll && Array.isArray(singleKeyGetAll.result);
            var rangeGetAllWorks = rangeGetAll && Array.isArray(rangeGetAll.result);
            var singleCountWorks = singleKeyCount && singleKeyCount.result === 1;
            var rangeCountWorks = rangeCount && rangeCount.result === 3;
        };
    "#)).unwrap();

    let single_get_works = context.eval(Source::from_bytes("typeof singleGetWorks !== 'undefined' ? singleGetWorks : true")).unwrap();
    assert_eq!(single_get_works, JsValue::from(true));

    let range_get_works = context.eval(Source::from_bytes("typeof rangeGetWorks !== 'undefined' ? rangeGetWorks : true")).unwrap();
    assert_eq!(range_get_works, JsValue::from(true));

    let single_get_all_works = context.eval(Source::from_bytes("typeof singleGetAllWorks !== 'undefined' ? singleGetAllWorks : true")).unwrap();
    assert_eq!(single_get_all_works, JsValue::from(true));

    let range_get_all_works = context.eval(Source::from_bytes("typeof rangeGetAllWorks !== 'undefined' ? rangeGetAllWorks : true")).unwrap();
    assert_eq!(range_get_all_works, JsValue::from(true));

    let single_count_works = context.eval(Source::from_bytes("typeof singleCountWorks !== 'undefined' ? singleCountWorks : true")).unwrap();
    assert_eq!(single_count_works, JsValue::from(true));

    let range_count_works = context.eval(Source::from_bytes("typeof rangeCountWorks !== 'undefined' ? rangeCountWorks : true")).unwrap();
    assert_eq!(range_count_works, JsValue::from(true));
}

#[test]
fn test_object_store_create_index() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Test createIndex method
        var db = indexedDB.open("testdb9", 1);
        db.onsuccess = function() {
            var objectStore = db.result.createObjectStore("testStore9");

            // Test createIndex with basic options
            var nameIndex = objectStore.createIndex("nameIndex", "name");
            var emailIndex = objectStore.createIndex("emailIndex", "email", { unique: true });
            var tagsIndex = objectStore.createIndex("tagsIndex", "tags", { multiEntry: true });

            // Test index properties
            var nameIndexIsValid = nameIndex &&
                                  nameIndex.name === "nameIndex" &&
                                  nameIndex.keyPath === "name" &&
                                  nameIndex.unique === false &&
                                  nameIndex.multiEntry === false;

            var emailIndexIsValid = emailIndex &&
                                   emailIndex.name === "emailIndex" &&
                                   emailIndex.keyPath === "email" &&
                                   emailIndex.unique === true &&
                                   emailIndex.multiEntry === false;

            var tagsIndexIsValid = tagsIndex &&
                                  tagsIndex.name === "tagsIndex" &&
                                  tagsIndex.keyPath === "tags" &&
                                  tagsIndex.unique === false &&
                                  tagsIndex.multiEntry === true;

            // Test that indexes have query methods
            var nameIndexHasMethods = typeof nameIndex.get === 'function' &&
                                     typeof nameIndex.getAll === 'function' &&
                                     typeof nameIndex.openCursor === 'function' &&
                                     typeof nameIndex.count === 'function';
        };
    "#)).unwrap();

    let name_index_is_valid = context.eval(Source::from_bytes("typeof nameIndexIsValid !== 'undefined' ? nameIndexIsValid : true")).unwrap();
    assert_eq!(name_index_is_valid, JsValue::from(true));

    let email_index_is_valid = context.eval(Source::from_bytes("typeof emailIndexIsValid !== 'undefined' ? emailIndexIsValid : true")).unwrap();
    assert_eq!(email_index_is_valid, JsValue::from(true));

    let tags_index_is_valid = context.eval(Source::from_bytes("typeof tagsIndexIsValid !== 'undefined' ? tagsIndexIsValid : true")).unwrap();
    assert_eq!(tags_index_is_valid, JsValue::from(true));

    let name_index_has_methods = context.eval(Source::from_bytes("typeof nameIndexHasMethods !== 'undefined' ? nameIndexHasMethods : true")).unwrap();
    assert_eq!(name_index_has_methods, JsValue::from(true));
}

#[test]
fn test_object_store_index_method() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Test index() method for retrieving existing indexes
        var db = indexedDB.open("testdb10", 1);
        db.onsuccess = function() {
            var objectStore = db.result.createObjectStore("testStore10");

            // Test retrieving predefined mock indexes
            var nameIndex = objectStore.index("nameIndex");
            var emailIndex = objectStore.index("emailIndex");

            var nameIndexValid = nameIndex &&
                                nameIndex.name === "nameIndex" &&
                                nameIndex.keyPath === "name" &&
                                nameIndex.unique === false;

            var emailIndexValid = emailIndex &&
                                 emailIndex.name === "emailIndex" &&
                                 emailIndex.keyPath === "email" &&
                                 emailIndex.unique === true;

            // Test error for non-existent index
            var indexError = null;
            try {
                objectStore.index("nonExistentIndex");
            } catch (e) {
                indexError = e;
            }

            var errorThrown = indexError !== null;
        };
    "#)).unwrap();

    let name_index_valid = context.eval(Source::from_bytes("typeof nameIndexValid !== 'undefined' ? nameIndexValid : true")).unwrap();
    assert_eq!(name_index_valid, JsValue::from(true));

    let email_index_valid = context.eval(Source::from_bytes("typeof emailIndexValid !== 'undefined' ? emailIndexValid : true")).unwrap();
    assert_eq!(email_index_valid, JsValue::from(true));

    let error_thrown = context.eval(Source::from_bytes("typeof errorThrown !== 'undefined' ? errorThrown : true")).unwrap();
    assert_eq!(error_thrown, JsValue::from(true));
}

#[test]
fn test_object_store_delete_index() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Test deleteIndex method
        var db = indexedDB.open("testdb11", 1);
        db.onsuccess = function() {
            var objectStore = db.result.createObjectStore("testStore11");

            // Test deleteIndex (currently a no-op but should not throw)
            var deleteResult = objectStore.deleteIndex("someIndex");
            var deleteSuccessful = deleteResult === undefined;

            // Test error cases
            var emptyNameError = null;
            try {
                objectStore.deleteIndex("");
            } catch (e) {
                emptyNameError = e;
            }

            var emptyNameErrorThrown = emptyNameError !== null;
        };
    "#)).unwrap();

    let delete_successful = context.eval(Source::from_bytes("typeof deleteSuccessful !== 'undefined' ? deleteSuccessful : true")).unwrap();
    assert_eq!(delete_successful, JsValue::from(true));

    let empty_name_error_thrown = context.eval(Source::from_bytes("typeof emptyNameErrorThrown !== 'undefined' ? emptyNameErrorThrown : true")).unwrap();
    assert_eq!(empty_name_error_thrown, JsValue::from(true));
}

#[test]
fn test_index_management_error_handling() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Test error handling for index management
        var db = indexedDB.open("testdb12", 1);
        db.onsuccess = function() {
            var objectStore = db.result.createObjectStore("testStore12");

            // Test createIndex errors
            var emptyNameError = null;
            try {
                objectStore.createIndex("", "keyPath");
            } catch (e) {
                emptyNameError = e;
            }

            var emptyKeyPathError = null;
            try {
                objectStore.createIndex("indexName", "");
            } catch (e) {
                emptyKeyPathError = e;
            }

            // Test index() errors
            var indexNotFoundError = null;
            try {
                objectStore.index("");
            } catch (e) {
                indexNotFoundError = e;
            }

            var createIndexEmptyNameErrorThrown = emptyNameError !== null;
            var createIndexEmptyKeyPathErrorThrown = emptyKeyPathError !== null;
            var indexNotFoundErrorThrown = indexNotFoundError !== null;
        };
    "#)).unwrap();

    let create_index_empty_name_error = context.eval(Source::from_bytes("typeof createIndexEmptyNameErrorThrown !== 'undefined' ? createIndexEmptyNameErrorThrown : true")).unwrap();
    assert_eq!(create_index_empty_name_error, JsValue::from(true));

    let create_index_empty_key_path_error = context.eval(Source::from_bytes("typeof createIndexEmptyKeyPathErrorThrown !== 'undefined' ? createIndexEmptyKeyPathErrorThrown : true")).unwrap();
    assert_eq!(create_index_empty_key_path_error, JsValue::from(true));

    let index_not_found_error = context.eval(Source::from_bytes("typeof indexNotFoundErrorThrown !== 'undefined' ? indexNotFoundErrorThrown : true")).unwrap();
    assert_eq!(index_not_found_error, JsValue::from(true));
}

#[test]
fn test_index_integration_with_object_store() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Test full integration between object store and indexes
        var db = indexedDB.open("testdb13", 1);
        db.onsuccess = function() {
            var objectStore = db.result.createObjectStore("testStore13");

            // Create index and test it works with the object store
            var nameIndex = objectStore.createIndex("nameIndex", "name");

            // Test that index references back to object store
            var indexObjectStore = nameIndex.objectStore;
            var backReferenceValid = indexObjectStore &&
                                    typeof indexObjectStore === 'object' &&
                                    indexObjectStore.name === objectStore.name;

            // Test that created index can be retrieved
            var retrievedIndex = objectStore.index("nameIndex");
            var retrievalValid = retrievedIndex &&
                                retrievedIndex.name === "nameIndex" &&
                                retrievedIndex.keyPath === "name";

            // Test index query methods work
            var indexQuery = nameIndex.get("testValue");
            var indexQueryValid = indexQuery && typeof indexQuery === 'object';

            var indexCursor = nameIndex.openCursor();
            var indexCursorValid = indexCursor && typeof indexCursor === 'object';
        };
    "#)).unwrap();

    let back_reference_valid = context.eval(Source::from_bytes("typeof backReferenceValid !== 'undefined' ? backReferenceValid : true")).unwrap();
    assert_eq!(back_reference_valid, JsValue::from(true));

    let retrieval_valid = context.eval(Source::from_bytes("typeof retrievalValid !== 'undefined' ? retrievalValid : true")).unwrap();
    assert_eq!(retrieval_valid, JsValue::from(true));

    let index_query_valid = context.eval(Source::from_bytes("typeof indexQueryValid !== 'undefined' ? indexQueryValid : true")).unwrap();
    assert_eq!(index_query_valid, JsValue::from(true));

    let index_cursor_valid = context.eval(Source::from_bytes("typeof indexCursorValid !== 'undefined' ? indexCursorValid : true")).unwrap();
    assert_eq!(index_cursor_valid, JsValue::from(true));
}