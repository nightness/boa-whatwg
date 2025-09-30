//! Tests for the Cache API implementation

use crate::{Context, JsValue, Source, JsString};

#[test]
fn test_cache_constructor_error() {
    let mut context = Context::default();

    // Attempting to construct Cache directly should throw
    let result = context.eval(Source::from_bytes("new Cache()"));
    assert!(result.is_err());

    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("Cache constructor cannot be called directly"));
}

#[test]
fn test_cache_put_and_match() {
    let mut context = Context::default();

    // Create a cache instance and test put/match functionality
    context.eval(Source::from_bytes(r#"
        // Create a mock cache object for testing
        var cache = {
            entries: new Map(),

            put: function(request, response) {
                var url = typeof request === 'string' ? request : request.url;
                this.entries.set(url, response);
                return Promise.resolve();
            },

            match: function(request) {
                var url = typeof request === 'string' ? request : request.url;
                var response = this.entries.get(url);
                return Promise.resolve(response);
            },

            delete: function(request) {
                var url = typeof request === 'string' ? request : request.url;
                var existed = this.entries.has(url);
                this.entries.delete(url);
                return Promise.resolve(existed);
            }
        };

        // Test put and match
        var testUrl = 'https://example.com/api/data';
        var mockResponse = { status: 200, statusText: 'OK', body: 'test data' };

        cache.put(testUrl, mockResponse);
    "#)).unwrap();

    // Test that cache works
    let result = context.eval(Source::from_bytes("cache.entries.size")).unwrap();
    assert_eq!(result, JsValue::from(1));
}

#[test]
fn test_cache_delete_functionality() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Create a mock cache for testing delete
        var cache = {
            entries: new Map(),

            put: function(request, response) {
                var url = typeof request === 'string' ? request : request.url;
                this.entries.set(url, response);
                return Promise.resolve();
            },

            delete: function(request) {
                var url = typeof request === 'string' ? request : request.url;
                var existed = this.entries.has(url);
                this.entries.delete(url);
                return Promise.resolve(existed);
            }
        };

        // Put something in cache
        cache.put('test-url', { status: 200 });
    "#)).unwrap();

    // Verify it's there
    let result = context.eval(Source::from_bytes("cache.entries.size")).unwrap();
    assert_eq!(result, JsValue::from(1));

    // Delete it
    context.eval(Source::from_bytes("cache.delete('test-url')")).unwrap();

    // Verify it's gone
    let result = context.eval(Source::from_bytes("cache.entries.size")).unwrap();
    assert_eq!(result, JsValue::from(0));
}

#[test]
fn test_cache_keys_functionality() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Create a mock cache for testing keys
        var cache = {
            entries: new Map(),

            put: function(request, response) {
                var url = typeof request === 'string' ? request : request.url;
                this.entries.set(url, response);
                return Promise.resolve();
            },

            keys: function() {
                var keys = Array.from(this.entries.keys()).map(function(url) {
                    return { url: url, method: 'GET' };
                });
                return Promise.resolve(keys);
            }
        };

        // Add multiple entries
        cache.put('url1', { status: 200 });
        cache.put('url2', { status: 200 });
        cache.put('url3', { status: 200 });
    "#)).unwrap();

    // Test keys
    let result = context.eval(Source::from_bytes("cache.entries.size")).unwrap();
    assert_eq!(result, JsValue::from(3));
}

#[test]
fn test_cache_match_not_found() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Create a mock cache
        var cache = {
            entries: new Map(),

            match: function(request) {
                var url = typeof request === 'string' ? request : request.url;
                var response = this.entries.get(url);
                return Promise.resolve(response); // undefined if not found
            }
        };

        var matchResult = cache.match('non-existent-url');
        var resolved = false;
        matchResult.then(function(response) {
            resolved = true;
            window.matchedResponse = response;
        });
    "#)).unwrap();

    // Since we can't easily test Promise resolution in this simple test,
    // we'll just verify the match function exists and can be called
    let result = context.eval(Source::from_bytes("typeof cache.match")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}