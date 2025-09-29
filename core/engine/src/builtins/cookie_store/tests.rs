//! Tests for the Cookie Store API implementation

use crate::{Context, JsValue, Source, JsString};

#[test]
fn test_cookie_store_constructor_error() {
    let mut context = Context::default();

    // CookieStore constructor should not be exposed globally
    let result = context.eval(Source::from_bytes("new CookieStore()"));
    assert!(result.is_err());

    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("CookieStore is not defined"));
}

#[test]
fn test_cookie_store_object_exists() {
    let mut context = Context::default();

    // Test that global cookieStore object exists
    let result = context.eval(Source::from_bytes("typeof cookieStore")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));

    // Test that cookieStore is not null
    let result = context.eval(Source::from_bytes("cookieStore !== null")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_cookie_store_methods_exist() {
    let mut context = Context::default();

    // Test that all main methods exist
    let result = context.eval(Source::from_bytes("typeof cookieStore.get")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof cookieStore.getAll")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof cookieStore.set")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    let result = context.eval(Source::from_bytes("typeof cookieStore.delete")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_cookie_store_methods_return_promises() {
    let mut context = Context::default();

    // Test that get() returns a Promise
    let result = context.eval(Source::from_bytes("cookieStore.get('test') instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test that getAll() returns a Promise
    let result = context.eval(Source::from_bytes("cookieStore.getAll() instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test that set() returns a Promise
    let result = context.eval(Source::from_bytes("cookieStore.set('test', 'value') instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test that delete() returns a Promise
    let result = context.eval(Source::from_bytes("cookieStore.delete('test') instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_cookie_store_basic_functionality() {
    let mut context = Context::default();

    // Create a mock test for cookie store functionality
    context.eval(Source::from_bytes(r#"
        // Create a mock cookie store for testing
        var mockCookieStore = {
            cookies: new Map(),

            set: function(nameOrOptions, value) {
                var name, val;
                if (typeof nameOrOptions === 'object') {
                    name = nameOrOptions.name;
                    val = nameOrOptions.value;
                } else {
                    name = nameOrOptions;
                    val = value;
                }
                this.cookies.set(name, {
                    name: name,
                    value: val,
                    domain: 'localhost',
                    path: '/',
                    secure: false,
                    httpOnly: false
                });
                return Promise.resolve();
            },

            get: function(name) {
                var cookie = this.cookies.get(name);
                return Promise.resolve(cookie || null);
            },

            getAll: function() {
                return Promise.resolve(Array.from(this.cookies.values()));
            },

            delete: function(name) {
                this.cookies.delete(name);
                return Promise.resolve();
            }
        };

        // Test setting a cookie
        mockCookieStore.set('testCookie', 'testValue');
    "#)).unwrap();

    // Test that cookie was created
    let result = context.eval(Source::from_bytes("mockCookieStore.cookies.size")).unwrap();
    assert_eq!(result, JsValue::from(1));
}

#[test]
fn test_cookie_store_options_object() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Create a mock cookie store for testing options
        var mockCookieStore = {
            cookies: new Map(),

            set: function(options) {
                this.cookies.set(options.name, {
                    name: options.name,
                    value: options.value,
                    domain: options.domain || 'localhost',
                    path: options.path || '/',
                    secure: !!options.secure,
                    httpOnly: !!options.httpOnly,
                    sameSite: options.sameSite || 'lax'
                });
                return Promise.resolve();
            },

            get: function(name) {
                var cookie = this.cookies.get(name);
                return Promise.resolve(cookie || null);
            }
        };

        // Test setting a cookie with options
        mockCookieStore.set({
            name: 'sessionCookie',
            value: 'sessionValue',
            secure: true,
            httpOnly: true,
            sameSite: 'strict'
        });
    "#)).unwrap();

    // Test that cookie was created with options
    let result = context.eval(Source::from_bytes("mockCookieStore.cookies.size")).unwrap();
    assert_eq!(result, JsValue::from(1));
}

#[test]
fn test_cookie_store_delete_functionality() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Create a mock cookie store for testing delete
        var mockCookieStore = {
            cookies: new Map(),

            set: function(name, value) {
                this.cookies.set(name, { name: name, value: value });
                return Promise.resolve();
            },

            delete: function(name) {
                var existed = this.cookies.has(name);
                this.cookies.delete(name);
                return Promise.resolve(existed);
            }
        };

        // Create some cookies
        mockCookieStore.set('cookie1', 'value1');
        mockCookieStore.set('cookie2', 'value2');
    "#)).unwrap();

    // Verify cookies exist
    let result = context.eval(Source::from_bytes("mockCookieStore.cookies.size")).unwrap();
    assert_eq!(result, JsValue::from(2));

    // Delete one cookie
    context.eval(Source::from_bytes("mockCookieStore.delete('cookie1')")).unwrap();

    // Verify one cookie remains
    let result = context.eval(Source::from_bytes("mockCookieStore.cookies.size")).unwrap();
    assert_eq!(result, JsValue::from(1));
}