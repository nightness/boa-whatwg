//! CustomEvent interface unit tests
//! Tests for DOM Level 4 CustomEvent implementation

use crate::{js_string, run_test_actions, TestAction, JsValue, JsNativeErrorKind};

#[test]
fn custom_event_constructor() {
    run_test_actions([
        TestAction::assert_eq("typeof CustomEvent", js_string!("function")),
        TestAction::run("var event = new CustomEvent('test')"),
        TestAction::assert_eq("typeof event", js_string!("object")),
        TestAction::assert_eq("event !== null", true),
    ]);
}

#[test]
fn custom_event_constructor_type_required() {
    run_test_actions([
        TestAction::assert_native_error(
            "new CustomEvent()",
            JsNativeErrorKind::Type,
            "CustomEvent constructor requires type parameter",
        ),
    ]);
}

#[test]
fn custom_event_basic_properties() {
    run_test_actions([
        TestAction::run("var event = new CustomEvent('test-event')"),
        TestAction::assert_eq("event.type", js_string!("test-event")),
        TestAction::assert_eq("event.bubbles", false),
        TestAction::assert_eq("event.cancelable", false),
        TestAction::assert_eq("event.defaultPrevented", false),
        TestAction::assert_eq("event.eventPhase", 0),
        TestAction::assert_eq("event.detail", JsValue::null()),
    ]);
}

#[test]
fn custom_event_with_event_init() {
    run_test_actions([
        TestAction::run("var event = new CustomEvent('test', { bubbles: true, cancelable: true, detail: { key: 'value' } })"),
        TestAction::assert_eq("event.type", js_string!("test")),
        TestAction::assert_eq("event.bubbles", true),
        TestAction::assert_eq("event.cancelable", true),
        TestAction::assert_eq("event.defaultPrevented", false),
        TestAction::assert_eq("typeof event.detail", js_string!("object")),
        TestAction::assert_eq("event.detail.key", js_string!("value")),
    ]);
}

#[test]
fn custom_event_detail_property() {
    run_test_actions([
        // Test with string detail
        TestAction::run("var event1 = new CustomEvent('test', { detail: 'string data' })"),
        TestAction::assert_eq("event1.detail", js_string!("string data")),

        // Test with number detail
        TestAction::run("var event2 = new CustomEvent('test', { detail: 42 })"),
        TestAction::assert_eq("event2.detail", 42),

        // Test with object detail
        TestAction::run("var event3 = new CustomEvent('test', { detail: { a: 1, b: 2 } })"),
        TestAction::assert_eq("event3.detail.a", 1),
        TestAction::assert_eq("event3.detail.b", 2),

        // Test with array detail
        TestAction::run("var event4 = new CustomEvent('test', { detail: [1, 2, 3] })"),
        TestAction::assert_eq("event4.detail[0]", 1),
        TestAction::assert_eq("event4.detail.length", 3),
    ]);
}

#[test]
fn custom_event_prevent_default() {
    run_test_actions([
        // Test with cancelable event
        TestAction::run("var event1 = new CustomEvent('test', { cancelable: true })"),
        TestAction::assert_eq("event1.defaultPrevented", false),
        TestAction::run("event1.preventDefault()"),
        TestAction::assert_eq("event1.defaultPrevented", true),

        // Test with non-cancelable event
        TestAction::run("var event2 = new CustomEvent('test', { cancelable: false })"),
        TestAction::assert_eq("event2.defaultPrevented", false),
        TestAction::run("event2.preventDefault()"),
        TestAction::assert_eq("event2.defaultPrevented", false), // Should remain false
    ]);
}

#[test]
fn custom_event_stop_propagation() {
    run_test_actions([
        TestAction::run("var event = new CustomEvent('test')"),
        // stopPropagation should not throw and should return undefined
        TestAction::assert_eq("event.stopPropagation()", JsValue::undefined()),
        TestAction::assert_eq("event.stopImmediatePropagation()", JsValue::undefined()),
    ]);
}

#[test]
fn custom_event_init_custom_event() {
    run_test_actions([
        TestAction::run("var event = new CustomEvent('initial')"),
        TestAction::assert_eq("event.type", js_string!("initial")),
        TestAction::assert_eq("event.detail", JsValue::null()),

        // Initialize with new values
        TestAction::run("event.initCustomEvent('new-type', true, true, 'new-detail')"),
        TestAction::assert_eq("event.type", js_string!("new-type")),
        TestAction::assert_eq("event.detail", js_string!("new-detail")),
    ]);
}

#[test]
fn custom_event_constants() {
    run_test_actions([
        TestAction::assert_eq("CustomEvent.NONE", 0),
        TestAction::assert_eq("CustomEvent.CAPTURING_PHASE", 1),
        TestAction::assert_eq("CustomEvent.AT_TARGET", 2),
        TestAction::assert_eq("CustomEvent.BUBBLING_PHASE", 3),
    ]);
}

#[test]
fn custom_event_inheritance() {
    run_test_actions([
        TestAction::run("var event = new CustomEvent('test')"),

        // Test constructor relationship
        TestAction::assert_eq("event.constructor === CustomEvent", true),
        TestAction::assert_eq("event instanceof CustomEvent", true),

        // Test property existence
        TestAction::assert_eq("'type' in event", true),
        TestAction::assert_eq("'bubbles' in event", true),
        TestAction::assert_eq("'cancelable' in event", true),
        TestAction::assert_eq("'defaultPrevented' in event", true),
        TestAction::assert_eq("'eventPhase' in event", true),
        TestAction::assert_eq("'detail' in event", true),

        // Test methods exist
        TestAction::assert_eq("typeof event.preventDefault", js_string!("function")),
        TestAction::assert_eq("typeof event.stopPropagation", js_string!("function")),
        TestAction::assert_eq("typeof event.stopImmediatePropagation", js_string!("function")),
        TestAction::assert_eq("typeof event.initCustomEvent", js_string!("function")),
    ]);
}

#[test]
fn custom_event_property_descriptors() {
    run_test_actions([
        // Test type property descriptor (read-only)
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(CustomEvent.prototype, 'type').get !== undefined",
            true
        ),
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(CustomEvent.prototype, 'type').set === undefined",
            true
        ),
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(CustomEvent.prototype, 'type').enumerable",
            true
        ),
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(CustomEvent.prototype, 'type').configurable",
            true
        ),

        // Test detail property descriptor (read-only)
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(CustomEvent.prototype, 'detail').get !== undefined",
            true
        ),
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(CustomEvent.prototype, 'detail').set === undefined",
            true
        ),
    ]);
}

#[test]
fn custom_event_property_enumeration() {
    run_test_actions([
        TestAction::run("var event = new CustomEvent('test')"),
        TestAction::run(r#"
            var props = [];
            for (var prop in event) {
                if (['type', 'bubbles', 'cancelable', 'defaultPrevented', 'eventPhase', 'detail'].includes(prop)) {
                    props.push(prop);
                }
            }
            props.sort();
        "#),
        TestAction::assert_eq("props.length >= 6", true), // Should find all properties
    ]);
}

#[test]
fn custom_event_method_binding() {
    run_test_actions([
        TestAction::run("var event = new CustomEvent('test', { cancelable: true })"),

        // Should work when called on the event object
        TestAction::run("event.preventDefault()"),
        TestAction::assert_eq("event.defaultPrevented", true),

        // Should fail when called on wrong object type
        TestAction::run("var obj = {}"),
        TestAction::assert_native_error(
            "event.preventDefault.call(obj)",
            JsNativeErrorKind::Type,
            "CustomEvent.prototype.preventDefault called on non-CustomEvent object",
        ),
    ]);
}

#[test]
fn custom_event_readonly_properties() {
    run_test_actions([
        TestAction::run("var event = new CustomEvent('test', { bubbles: true, cancelable: true })"),

        // Try to modify read-only properties (should have no effect)
        TestAction::run("event.type = 'modified'"),
        TestAction::assert_eq("event.type", js_string!("test")),

        TestAction::run("event.bubbles = false"),
        TestAction::assert_eq("event.bubbles", true),

        TestAction::run("event.cancelable = false"),
        TestAction::assert_eq("event.cancelable", true),
    ]);
}

#[test]
fn custom_event_complex_detail() {
    run_test_actions([
        TestAction::run(r#"
            var complexDetail = {
                nested: {
                    array: [1, 2, 3],
                    string: "test",
                    boolean: true
                },
                func: function() { return "hello"; },
                date: new Date(2025, 0, 1)
            };
            var event = new CustomEvent('complex', { detail: complexDetail });
        "#),
        TestAction::assert_eq("event.detail.nested.array[1]", 2),
        TestAction::assert_eq("event.detail.nested.string", js_string!("test")),
        TestAction::assert_eq("event.detail.nested.boolean", true),
        TestAction::assert_eq("typeof event.detail.func", js_string!("function")),
        TestAction::assert_eq("event.detail.func()", js_string!("hello")),
    ]);
}

#[test]
fn custom_event_type_coercion() {
    run_test_actions([
        // Test type coercion for event type
        TestAction::run("var event1 = new CustomEvent(123)"),
        TestAction::assert_eq("event1.type", js_string!("123")),

        TestAction::run("var event2 = new CustomEvent(true)"),
        TestAction::assert_eq("event2.type", js_string!("true")),

        TestAction::run("var event3 = new CustomEvent(null)"),
        TestAction::assert_eq("event3.type", js_string!("null")),
    ]);
}

#[test]
fn custom_event_init_options_validation() {
    run_test_actions([
        // Test with non-object init parameter (should use defaults)
        TestAction::run("var event1 = new CustomEvent('test', 'not-an-object')"),
        TestAction::assert_eq("event1.bubbles", false),
        TestAction::assert_eq("event1.cancelable", false),
        TestAction::assert_eq("event1.detail", JsValue::null()),

        // Test with null init parameter
        TestAction::run("var event2 = new CustomEvent('test', null)"),
        TestAction::assert_eq("event2.bubbles", false),
        TestAction::assert_eq("event2.cancelable", false),
        TestAction::assert_eq("event2.detail", JsValue::null()),

        // Test with partial init object
        TestAction::run("var event3 = new CustomEvent('test', { bubbles: true })"),
        TestAction::assert_eq("event3.bubbles", true),
        TestAction::assert_eq("event3.cancelable", false), // Should default to false
        TestAction::assert_eq("event3.detail", JsValue::null()), // Should default to null
    ]);
}

#[test]
fn custom_event_empty_type() {
    run_test_actions([
        TestAction::run("var event = new CustomEvent('')"),
        TestAction::assert_eq("event.type", js_string!("")),
    ]);
}

#[test]
fn custom_event_special_characters_type() {
    run_test_actions([
        TestAction::run("var event = new CustomEvent('test:event-name_123')"),
        TestAction::assert_eq("event.type", js_string!("test:event-name_123")),
    ]);
}

#[test]
fn custom_event_method_return_values() {
    run_test_actions([
        TestAction::run("var event = new CustomEvent('test', { cancelable: true })"),

        // Test that methods return undefined
        TestAction::assert_eq("event.preventDefault()", JsValue::undefined()),
        TestAction::assert_eq("event.stopPropagation()", JsValue::undefined()),
        TestAction::assert_eq("event.stopImmediatePropagation()", JsValue::undefined()),
        TestAction::assert_eq("event.initCustomEvent('new', true, true, null)", JsValue::undefined()),
    ]);
}