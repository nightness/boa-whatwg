//! MessageEvent interface unit tests
//! Tests for HTML5 MessageEvent implementation according to WHATWG specification:
//! https://html.spec.whatwg.org/multipage/comms.html#messageevent

use crate::{js_string, run_test_actions, TestAction, JsValue, JsNativeErrorKind};

#[test]
fn message_event_constructor_exists() {
    run_test_actions([
        TestAction::assert_eq("typeof MessageEvent", js_string!("function")),
        TestAction::assert("MessageEvent.prototype !== undefined"),
        TestAction::assert("MessageEvent.prototype.constructor === MessageEvent"),
    ]);
}

#[test]
fn message_event_constructor_requires_new() {
    run_test_actions([
        TestAction::assert_native_error(
            "MessageEvent('test')",
            JsNativeErrorKind::Type,
            "MessageEvent constructor requires 'new'",
        ),
    ]);
}

#[test]
fn message_event_constructor_basic() {
    run_test_actions([
        TestAction::run("var event = new MessageEvent('message')"),
        TestAction::assert("event instanceof MessageEvent"),
        TestAction::assert("event instanceof Event"),
        TestAction::assert_eq("event.type", js_string!("message")),
        TestAction::assert_eq("event.data", js_string!("")),
        TestAction::assert_eq("event.origin", js_string!("")),
        TestAction::assert_eq("event.lastEventId", js_string!("")),
        TestAction::assert_eq("event.source", JsValue::null()),
        TestAction::assert_eq("event.ports", JsValue::undefined()),
    ]);
}

#[test]
fn message_event_constructor_with_init_dict() {
    run_test_actions([
        TestAction::run(r#"
            var initDict = {
                data: {hello: 'world'},
                origin: 'https://example.com',
                lastEventId: '123',
                source: null,
                ports: [],
                bubbles: true,
                cancelable: true
            };
            var event = new MessageEvent('message', initDict);
        "#),
        TestAction::assert_eq("event.type", js_string!("message")),
        TestAction::assert_eq("event.data.hello", js_string!("world")),
        TestAction::assert_eq("event.origin", js_string!("https://example.com")),
        TestAction::assert_eq("event.lastEventId", js_string!("123")),
        TestAction::assert_eq("event.source", JsValue::null()),
        TestAction::assert("Array.isArray(event.ports)"),
        TestAction::assert_eq("event.bubbles", true),
        TestAction::assert_eq("event.cancelable", true),
    ]);
}

#[test]
fn message_event_constructor_with_complex_data() {
    run_test_actions([
        // Test with various data types
        TestAction::run("var event1 = new MessageEvent('message', { data: 'string data' })"),
        TestAction::assert_eq("event1.data", js_string!("string data")),

        TestAction::run("var event2 = new MessageEvent('message', { data: 42 })"),
        TestAction::assert_eq("event2.data", 42),

        TestAction::run("var event3 = new MessageEvent('message', { data: true })"),
        TestAction::assert_eq("event3.data", true),

        TestAction::run("var event4 = new MessageEvent('message', { data: null })"),
        TestAction::assert_eq("event4.data", JsValue::null()),

        TestAction::run("var event5 = new MessageEvent('message', { data: [1, 2, 3] })"),
        TestAction::assert("Array.isArray(event5.data)"),
        TestAction::assert_eq("event5.data.length", 3),
        TestAction::assert_eq("event5.data[0]", 1),
    ]);
}

#[test]
fn message_event_properties_readonly() {
    run_test_actions([
        TestAction::run(r#"
            var event = new MessageEvent('message', {
                data: 'original',
                origin: 'https://example.com',
                lastEventId: 'abc123'
            });
        "#),
        // Properties should be readonly
        TestAction::run("event.data = 'modified'"),
        TestAction::assert_eq("event.data", js_string!("original")),

        TestAction::run("event.origin = 'https://evil.com'"),
        TestAction::assert_eq("event.origin", js_string!("https://example.com")),

        TestAction::run("event.lastEventId = 'xyz789'"),
        TestAction::assert_eq("event.lastEventId", js_string!("abc123")),
    ]);
}

#[test]
fn message_event_type_conversion() {
    run_test_actions([
        // Type parameter should be converted to string
        TestAction::run("var event1 = new MessageEvent(123)"),
        TestAction::assert_eq("event1.type", js_string!("123")),

        TestAction::run("var event2 = new MessageEvent(true)"),
        TestAction::assert_eq("event2.type", js_string!("true")),

        TestAction::run("var event3 = new MessageEvent(null)"),
        TestAction::assert_eq("event3.type", js_string!("null")),

        TestAction::run("var event4 = new MessageEvent({toString: function() { return 'custom'; }})"),
        TestAction::assert_eq("event4.type", js_string!("custom")),
    ]);
}

#[test]
fn message_event_origin_string_conversion() {
    run_test_actions([
        // Origin should be converted to string
        TestAction::run("var event1 = new MessageEvent('message', { origin: 123 })"),
        TestAction::assert_eq("event1.origin", js_string!("123")),

        TestAction::run("var event2 = new MessageEvent('message', { origin: true })"),
        TestAction::assert_eq("event2.origin", js_string!("true")),

        TestAction::run("var event3 = new MessageEvent('message', { origin: {toString: function() { return 'custom'; }} })"),
        TestAction::assert_eq("event3.origin", js_string!("custom")),
    ]);
}

#[test]
fn message_event_last_event_id_string_conversion() {
    run_test_actions([
        // lastEventId should be converted to string
        TestAction::run("var event1 = new MessageEvent('message', { lastEventId: 123 })"),
        TestAction::assert_eq("event1.lastEventId", js_string!("123")),

        TestAction::run("var event2 = new MessageEvent('message', { lastEventId: true })"),
        TestAction::assert_eq("event2.lastEventId", js_string!("true")),
    ]);
}

#[test]
fn message_event_inherited_event_properties() {
    run_test_actions([
        TestAction::run("var event = new MessageEvent('message')"),

        // Should inherit Event interface properties
        TestAction::assert_eq("event.bubbles", false),
        TestAction::assert_eq("event.cancelable", false),
        TestAction::assert_eq("event.composed", false),
        TestAction::assert_eq("event.defaultPrevented", false),
        TestAction::assert_eq("event.eventPhase", 0),
        TestAction::assert_eq("event.isTrusted", false),
        TestAction::assert_eq("event.target", JsValue::null()),
        TestAction::assert_eq("event.currentTarget", JsValue::null()),
        TestAction::assert("typeof event.timeStamp === 'number'"),
    ]);
}

#[test]
fn message_event_inherited_event_properties_with_options() {
    run_test_actions([
        TestAction::run(r#"
            var event = new MessageEvent('message', {
                bubbles: true,
                cancelable: true
            });
        "#),

        // Event properties should be set from init dict
        TestAction::assert_eq("event.bubbles", true),
        TestAction::assert_eq("event.cancelable", true),
        TestAction::assert_eq("event.composed", false), // Not in init dict, should be false
    ]);
}

#[test]
fn message_event_constructor_length() {
    run_test_actions([
        // Constructor should accept 1 required parameter
        TestAction::assert_eq("MessageEvent.length", 1),
    ]);
}

#[test]
fn message_event_ports_handling() {
    run_test_actions([
        // Test ports property
        TestAction::run("var event1 = new MessageEvent('message')"),
        TestAction::assert_eq("event1.ports", JsValue::undefined()),

        TestAction::run("var event2 = new MessageEvent('message', { ports: [] })"),
        TestAction::assert("Array.isArray(event2.ports)"),
        TestAction::assert_eq("event2.ports.length", 0),

        TestAction::run("var event3 = new MessageEvent('message', { ports: ['port1', 'port2'] })"),
        TestAction::assert("Array.isArray(event3.ports)"),
        TestAction::assert_eq("event3.ports.length", 2),
        TestAction::assert_eq("event3.ports[0]", js_string!("port1")),
        TestAction::assert_eq("event3.ports[1]", js_string!("port2")),
    ]);
}

#[test]
fn message_event_undefined_init_dict() {
    run_test_actions([
        // Should work with undefined init dict
        TestAction::run("var event = new MessageEvent('message', undefined)"),
        TestAction::assert_eq("event.type", js_string!("message")),
        TestAction::assert_eq("event.data", js_string!("")),
        TestAction::assert_eq("event.origin", js_string!("")),
        TestAction::assert_eq("event.lastEventId", js_string!("")),
    ]);
}

#[test]
fn message_event_null_init_dict() {
    run_test_actions([
        // Should work with null init dict
        TestAction::run("var event = new MessageEvent('message', null)"),
        TestAction::assert_eq("event.type", js_string!("message")),
        TestAction::assert_eq("event.data", js_string!("")),
        TestAction::assert_eq("event.origin", js_string!("")),
        TestAction::assert_eq("event.lastEventId", js_string!("")),
    ]);
}

#[test]
fn message_event_partial_init_dict() {
    run_test_actions([
        // Should work with partial init dict
        TestAction::run("var event = new MessageEvent('message', { data: 'test' })"),
        TestAction::assert_eq("event.type", js_string!("message")),
        TestAction::assert_eq("event.data", js_string!("test")),
        TestAction::assert_eq("event.origin", js_string!("")), // Default value
        TestAction::assert_eq("event.lastEventId", js_string!("")), // Default value
    ]);
}

#[test]
fn message_event_source_property() {
    run_test_actions([
        // Test source property
        TestAction::run("var event1 = new MessageEvent('message')"),
        TestAction::assert_eq("event1.source", JsValue::null()),

        TestAction::run("var event2 = new MessageEvent('message', { source: 'test-source' })"),
        TestAction::assert_eq("event2.source", js_string!("test-source")),

        TestAction::run("var obj = {}; var event3 = new MessageEvent('message', { source: obj })"),
        TestAction::assert("event3.source === obj"),
    ]);
}

#[test]
fn message_event_to_string() {
    run_test_actions([
        TestAction::run("var event = new MessageEvent('message')"),
        TestAction::assert("event.toString().includes('[object MessageEvent]') || event.toString() === '[object Object]'"),
    ]);
}