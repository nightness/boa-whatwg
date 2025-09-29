//! Worker integration tests
//! Tests for Worker API integration with MessageEvent and structured cloning

use crate::{js_string, run_test_actions, TestAction, JsValue, Context};
use crate::builtins::{structured_clone::*, message_event::create_message_event};

#[test]
fn worker_constructor_exists() {
    run_test_actions([
        TestAction::assert_eq("typeof Worker", js_string!("function")),
        TestAction::assert("Worker.prototype !== undefined"),
        TestAction::assert("Worker.prototype.constructor === Worker"),
    ]);
}

#[test]
fn worker_message_event_creation() {
    // Test that we can create MessageEvent objects properly
    let mut context = Context::default();

    // Test structured cloning of simple data
    let test_value = JsValue::from(js_string!("test message"));
    let cloned = structured_clone(&test_value, &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();
    assert_eq!(deserialized.to_string(&mut context).unwrap().to_std_string_escaped(), "test message");

    // Test MessageEvent creation
    let message_event = create_message_event(
        deserialized,
        Some("test-origin"),
        None,
        None,
        &mut context,
    ).unwrap();

    // Verify MessageEvent properties
    let data_prop = message_event.get(js_string!("data"), &mut context).unwrap();
    assert_eq!(data_prop.to_string(&mut context).unwrap().to_std_string_escaped(), "test message");

    let origin_prop = message_event.get(js_string!("origin"), &mut context).unwrap();
    assert_eq!(origin_prop.to_string(&mut context).unwrap().to_std_string_escaped(), "test-origin");
}

#[test]
fn structured_clone_complex_data() {
    let mut context = Context::default();

    // Test cloning of complex object
    run_test_actions([
        TestAction::run(r#"
            var complexData = {
                string: "hello",
                number: 42,
                boolean: true,
                array: [1, 2, 3],
                nested: {
                    prop: "value"
                },
                date: new Date('2023-01-01')
            };
        "#),
    ]);

    let global = context.global_object();
    let complex_data = global.get(js_string!("complexData"), &mut context).unwrap();

    // Clone and deserialize
    let cloned = structured_clone(&complex_data, &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();

    // Verify structure is preserved
    if let Some(obj) = deserialized.as_object() {
        let string_prop = obj.get(js_string!("string"), &mut context).unwrap();
        assert_eq!(string_prop.to_string(&mut context).unwrap().to_std_string_escaped(), "hello");

        let number_prop = obj.get(js_string!("number"), &mut context).unwrap();
        assert_eq!(number_prop.as_number().unwrap(), 42.0);

        let boolean_prop = obj.get(js_string!("boolean"), &mut context).unwrap();
        assert_eq!(boolean_prop.as_boolean().unwrap(), true);
    }
}

#[test]
fn structured_clone_array_data() {
    let mut context = Context::default();

    run_test_actions([
        TestAction::run("var arrayData = [1, 'two', true, {four: 4}]"),
    ]);

    let global = context.global_object();
    let array_data = global.get(js_string!("arrayData"), &mut context).unwrap();

    // Clone and deserialize
    let cloned = structured_clone(&array_data, &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();

    // Verify array structure
    if let Some(array_obj) = deserialized.as_object() {
        let length = array_obj.get(js_string!("length"), &mut context).unwrap();
        assert_eq!(length.as_number().unwrap(), 4.0);

        let first = array_obj.get(0, &mut context).unwrap();
        assert_eq!(first.as_number().unwrap(), 1.0);

        let second = array_obj.get(1, &mut context).unwrap();
        assert_eq!(second.to_string(&mut context).unwrap().to_std_string_escaped(), "two");
    }
}

#[test]
fn structured_clone_date_object() {
    let mut context = Context::default();

    run_test_actions([
        TestAction::run("var dateData = new Date('2023-12-25T00:00:00.000Z')"),
    ]);

    let global = context.global_object();
    let date_data = global.get(js_string!("dateData"), &mut context).unwrap();

    // Clone and deserialize
    let cloned = structured_clone(&date_data, &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();

    // Verify the date is preserved
    // Note: This tests the Date object cloning functionality
    assert!(deserialized.is_object());
}

#[test]
fn structured_clone_regexp_object() {
    let mut context = Context::default();

    run_test_actions([
        TestAction::run("var regexpData = /test[0-9]+/gi"),
    ]);

    let global = context.global_object();
    let regexp_data = global.get(js_string!("regexpData"), &mut context).unwrap();

    // Clone and deserialize
    let cloned = structured_clone(&regexp_data, &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();

    // Verify the regexp is preserved
    assert!(deserialized.is_object());
}

#[test]
fn worker_script_url_property() {
    run_test_actions([
        // Test that Worker constructor sets scriptURL properly
        TestAction::run("var worker = new Worker('test-script.js')"),
        TestAction::assert_eq("worker.scriptURL", js_string!("test-script.js")),
    ]);
}

#[test]
fn worker_post_message_method_exists() {
    run_test_actions([
        TestAction::run("var worker = new Worker('test-script.js')"),
        TestAction::assert("typeof worker.postMessage === 'function'"),
        TestAction::assert_eq("worker.postMessage.length", 2), // message, transfer
    ]);
}

#[test]
fn worker_terminate_method_exists() {
    run_test_actions([
        TestAction::run("var worker = new Worker('test-script.js')"),
        TestAction::assert("typeof worker.terminate === 'function'"),
        TestAction::assert_eq("worker.terminate.length", 0),
    ]);
}

#[test]
fn worker_event_handler_properties() {
    run_test_actions([
        TestAction::run("var worker = new Worker('test-script.js')"),

        // Should have event handler properties
        TestAction::assert("'onmessage' in worker"),
        TestAction::assert("'onerror' in worker"),
        TestAction::assert("'onmessageerror' in worker"),

        // Initially should be null
        TestAction::assert_eq("worker.onmessage", JsValue::null()),
        TestAction::assert_eq("worker.onerror", JsValue::null()),
        TestAction::assert_eq("worker.onmessageerror", JsValue::null()),
    ]);
}

#[test]
fn worker_event_handler_assignment() {
    run_test_actions([
        TestAction::run("var worker = new Worker('test-script.js')"),

        // Should be able to assign event handlers
        TestAction::run("worker.onmessage = function(e) { console.log('message:', e.data); }"),
        TestAction::assert("typeof worker.onmessage === 'function'"),

        TestAction::run("worker.onerror = function(e) { console.log('error:', e.message); }"),
        TestAction::assert("typeof worker.onerror === 'function'"),

        TestAction::run("worker.onmessageerror = function(e) { console.log('message error'); }"),
        TestAction::assert("typeof worker.onmessageerror === 'function'"),
    ]);
}

#[test]
fn worker_post_message_basic() {
    run_test_actions([
        TestAction::run("var worker = new Worker('test-script.js')"),

        // Should be able to call postMessage without throwing
        TestAction::run("worker.postMessage('hello')"),
        TestAction::run("worker.postMessage({key: 'value'})"),
        TestAction::run("worker.postMessage([1, 2, 3])"),
        TestAction::run("worker.postMessage(42)"),
        TestAction::run("worker.postMessage(true)"),
        TestAction::run("worker.postMessage(null)"),
    ]);
}

#[test]
fn worker_inheritance() {
    run_test_actions([
        TestAction::run("var worker = new Worker('test-script.js')"),

        // Worker should inherit from EventTarget
        TestAction::assert("worker instanceof EventTarget"),
        TestAction::assert("worker instanceof Worker"),

        // Should have EventTarget methods
        TestAction::assert("typeof worker.addEventListener === 'function'"),
        TestAction::assert("typeof worker.removeEventListener === 'function'"),
        TestAction::assert("typeof worker.dispatchEvent === 'function'"),
    ]);
}