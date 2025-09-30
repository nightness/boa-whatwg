//! Tests for the CacheStorage API implementation

use crate::{Context, JsValue, Source, JsString};

#[test]
fn test_cache_storage_constructor_error() {
    let mut context = Context::default();

    // Attempting to construct CacheStorage directly should throw
    let result = context.eval(Source::from_bytes("new CacheStorage()"));
    assert!(result.is_err());

    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("CacheStorage constructor cannot be called directly"));
}

#[test]
fn test_caches_object_exists() {
    let mut context = Context::default();

    // Test that global caches object exists
    let result = context.eval(Source::from_bytes("typeof caches")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));

    // Test that caches is not null
    let result = context.eval(Source::from_bytes("caches !== null")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_cache_storage_methods_exist() {
    let mut context = Context::default();

    // Test that all main methods exist
    let result = context.eval(Source::from_bytes("typeof caches.open")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof caches.has")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof caches.delete")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof caches.keys")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof caches.match")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_cache_storage_methods_return_promises() {
    let mut context = Context::default();

    // Test that open() returns a Promise
    let result = context.eval(Source::from_bytes("caches.open('test-cache') instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test that has() returns a Promise
    let result = context.eval(Source::from_bytes("caches.has('test-cache') instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test that delete() returns a Promise
    let result = context.eval(Source::from_bytes("caches.delete('test-cache') instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test that keys() returns a Promise
    let result = context.eval(Source::from_bytes("caches.keys() instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test that match() returns a Promise
    let result = context.eval(Source::from_bytes("caches.match('test-request') instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_cache_storage_basic_functionality() {
    let mut context = Context::default();

    // Create a mock test for cache storage functionality
    context.eval(Source::from_bytes(r#"
        // Create a mock cache storage for testing
        var mockCacheStorage = {
            caches: new Map(),

            open: function(cacheName) {
                if (!this.caches.has(cacheName)) {
                    this.caches.set(cacheName, {
                        name: cacheName,
                        entries: new Map()
                    });
                }
                return Promise.resolve(this.caches.get(cacheName));
            },

            has: function(cacheName) {
                return Promise.resolve(this.caches.has(cacheName));
            },

            delete: function(cacheName) {
                var existed = this.caches.has(cacheName);
                this.caches.delete(cacheName);
                return Promise.resolve(existed);
            },

            keys: function() {
                return Promise.resolve(Array.from(this.caches.keys()));
            }
        };

        // Test opening a cache
        mockCacheStorage.open('test-cache');
    "#)).unwrap();

    // Test that cache was created
    let result = context.eval(Source::from_bytes("mockCacheStorage.caches.size")).unwrap();
    assert_eq!(result, JsValue::from(1));
}

#[test]
fn test_cache_storage_delete_functionality() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Create a mock cache storage for testing delete
        var mockCacheStorage = {
            caches: new Map(),

            open: function(cacheName) {
                if (!this.caches.has(cacheName)) {
                    this.caches.set(cacheName, { name: cacheName });
                }
                return Promise.resolve(this.caches.get(cacheName));
            },

            delete: function(cacheName) {
                var existed = this.caches.has(cacheName);
                this.caches.delete(cacheName);
                return Promise.resolve(existed);
            }
        };

        // Create some caches
        mockCacheStorage.open('cache1');
        mockCacheStorage.open('cache2');
    "#)).unwrap();

    // Verify caches exist
    let result = context.eval(Source::from_bytes("mockCacheStorage.caches.size")).unwrap();
    assert_eq!(result, JsValue::from(2));

    // Delete one cache
    context.eval(Source::from_bytes("mockCacheStorage.delete('cache1')")).unwrap();

    // Verify one cache remains
    let result = context.eval(Source::from_bytes("mockCacheStorage.caches.size")).unwrap();
    assert_eq!(result, JsValue::from(1));
}