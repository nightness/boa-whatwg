use crate::{js_string, run_test_actions, TestAction, JsNativeErrorKind, JsValue};

#[test]
fn event_target_constructor() {
    run_test_actions([
        TestAction::assert_eq("typeof EventTarget", js_string!("function")),
        TestAction::run("var target = new EventTarget()"),
        TestAction::assert_eq("typeof target", js_string!("object")),
        TestAction::assert_eq("target !== null", true),
    ]);
}

#[test]
fn event_target_constructor_requires_new() {
    run_test_actions([
        TestAction::assert_native_error(
            "EventTarget()",
            JsNativeErrorKind::Type,
            "Constructor EventTarget requires 'new'",
        ),
    ]);
}

#[test]
fn event_target_inheritance() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),

        // Test constructor relationship
        TestAction::assert_eq("target.constructor === EventTarget", true),
        TestAction::assert_eq("target instanceof EventTarget", true),

        // Test that it's an object
        TestAction::assert_eq("typeof target", js_string!("object")),
        TestAction::assert_eq("target !== null", true),
    ]);
}

#[test]
fn event_target_methods_exist() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),

        // Test all required methods exist
        TestAction::assert_eq("typeof target.addEventListener", js_string!("function")),
        TestAction::assert_eq("typeof target.removeEventListener", js_string!("function")),
        TestAction::assert_eq("typeof target.dispatchEvent", js_string!("function")),
    ]);
}

#[test]
fn event_target_interface_compliance() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),

        // Test all required interface members exist
        TestAction::assert_eq("'addEventListener' in target", true),
        TestAction::assert_eq("'removeEventListener' in target", true),
        TestAction::assert_eq("'dispatchEvent' in target", true),
    ]);
}

#[test]
fn add_event_listener_basic() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),
        TestAction::run("var called = false"),
        TestAction::run("function handler() { called = true; }"),

        // Add event listener
        TestAction::run("target.addEventListener('test', handler)"),
        TestAction::assert_eq("called", false),

        // Test method returns undefined
        TestAction::assert_eq("target.addEventListener('test2', function(){}) === undefined", true),
    ]);
}

#[test]
fn add_event_listener_with_null_callback() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),

        // Should not throw with null/undefined callbacks (they're just ignored)
        TestAction::run("target.addEventListener('test', null)"),
        TestAction::run("target.addEventListener('test', undefined)"),
        TestAction::assert_eq("true", true), // If we get here, no exception was thrown
    ]);
}

#[test]
fn add_event_listener_with_options_boolean() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),
        TestAction::run("function handler() {}"),

        // Test with boolean capture option
        TestAction::run("target.addEventListener('test', handler, true)"),
        TestAction::run("target.addEventListener('test', handler, false)"),
        TestAction::assert_eq("true", true), // If we get here, no exception was thrown
    ]);
}

#[test]
fn add_event_listener_with_options_object() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),
        TestAction::run("function handler() {}"),

        // Test with options object
        TestAction::run("target.addEventListener('test', handler, {})"),
        TestAction::run("target.addEventListener('test', handler, { capture: true })"),
        TestAction::run("target.addEventListener('test', handler, { once: true })"),
        TestAction::run("target.addEventListener('test', handler, { passive: true })"),
        TestAction::run("target.addEventListener('test', handler, { capture: true, once: true, passive: true })"),
        TestAction::assert_eq("true", true), // If we get here, no exception was thrown
    ]);
}

#[test]
fn remove_event_listener_basic() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),
        TestAction::run("function handler() {}"),

        // Add and remove event listener
        TestAction::run("target.addEventListener('test', handler)"),
        TestAction::run("target.removeEventListener('test', handler)"),

        // Test method returns undefined
        TestAction::assert_eq("target.removeEventListener('test', function(){}) === undefined", true),
    ]);
}

#[test]
fn remove_event_listener_with_options() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),
        TestAction::run("function handler() {}"),

        // Test with options
        TestAction::run("target.addEventListener('test', handler, true)"),
        TestAction::run("target.removeEventListener('test', handler, true)"),
        TestAction::run("target.removeEventListener('test', handler, false)"),
        TestAction::run("target.removeEventListener('test', handler, { capture: true })"),
        TestAction::assert_eq("true", true), // If we get here, no exception was thrown
    ]);
}

#[test]
fn dispatch_event_requires_event_object() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),

        // Test that dispatchEvent requires an Event object
        TestAction::assert_native_error(
            "target.dispatchEvent(null)",
            JsNativeErrorKind::Type,
            "EventTarget.dispatchEvent requires an Event object",
        ),
        TestAction::assert_native_error(
            "target.dispatchEvent(undefined)",
            JsNativeErrorKind::Type,
            "EventTarget.dispatchEvent requires an Event object",
        ),
        TestAction::assert_native_error(
            "target.dispatchEvent('string')",
            JsNativeErrorKind::Type,
            "EventTarget.dispatchEvent requires an Event object",
        ),
    ]);
}

#[test]
fn dispatch_event_basic() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),

        // Create a mock event object with type property
        TestAction::run("var event = { type: 'test', defaultPrevented: false }"),

        // Test dispatch returns boolean
        TestAction::run("var result = target.dispatchEvent(event)"),
        TestAction::assert_eq("typeof result", js_string!("boolean")),
        TestAction::assert_eq("result", true), // No preventDefault called, so should return true
    ]);
}

#[test]
fn event_listener_execution() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),
        TestAction::run("var called = false"),
        TestAction::run("var receivedEvent = null"),
        TestAction::run("function handler(event) { called = true; receivedEvent = event; }"),

        // Add listener and dispatch event
        TestAction::run("target.addEventListener('test', handler)"),
        TestAction::run("var event = { type: 'test', defaultPrevented: false }"),
        TestAction::run("target.dispatchEvent(event)"),

        // Check that handler was called
        TestAction::assert_eq("called", true),
        TestAction::assert_eq("receivedEvent === event", true),
    ]);
}

#[test]
fn event_listener_multiple_handlers() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),
        TestAction::run("var count = 0"),
        TestAction::run("function handler1() { count++; }"),
        TestAction::run("function handler2() { count++; }"),
        TestAction::run("function handler3() { count++; }"),

        // Add multiple listeners for same event
        TestAction::run("target.addEventListener('test', handler1)"),
        TestAction::run("target.addEventListener('test', handler2)"),
        TestAction::run("target.addEventListener('test', handler3)"),

        // Dispatch event
        TestAction::run("var event = { type: 'test', defaultPrevented: false }"),
        TestAction::run("target.dispatchEvent(event)"),

        // All three handlers should have been called
        TestAction::assert_eq("count", 3),
    ]);
}

#[test]
fn event_listener_once_option() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),
        TestAction::run("var count = 0"),
        TestAction::run("function handler() { count++; }"),

        // Add listener with once: true
        TestAction::run("target.addEventListener('test', handler, { once: true })"),

        // Dispatch event multiple times
        TestAction::run("var event = { type: 'test', defaultPrevented: false }"),
        TestAction::run("target.dispatchEvent(event)"),
        TestAction::assert_eq("count", 1),

        TestAction::run("target.dispatchEvent(event)"),
        TestAction::assert_eq("count", 1), // Should still be 1, not 2

        TestAction::run("target.dispatchEvent(event)"),
        TestAction::assert_eq("count", 1), // Should still be 1, not 3
    ]);
}

#[test]
fn event_listener_removal() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),
        TestAction::run("var count = 0"),
        TestAction::run("function handler() { count++; }"),

        // Add listener
        TestAction::run("target.addEventListener('test', handler)"),

        // Dispatch event
        TestAction::run("var event = { type: 'test', defaultPrevented: false }"),
        TestAction::run("target.dispatchEvent(event)"),
        TestAction::assert_eq("count", 1),

        // Remove listener
        TestAction::run("target.removeEventListener('test', handler)"),

        // Dispatch event again
        TestAction::run("target.dispatchEvent(event)"),
        TestAction::assert_eq("count", 1), // Should still be 1, not 2
    ]);
}

#[test]
fn event_listener_capture_flag_matching() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),
        TestAction::run("var count = 0"),
        TestAction::run("function handler() { count++; }"),

        // Add listener with capture: true
        TestAction::run("target.addEventListener('test', handler, true)"),

        // Try to remove with capture: false (should not remove)
        TestAction::run("target.removeEventListener('test', handler, false)"),

        // Dispatch event - should still call handler
        TestAction::run("var event = { type: 'test', defaultPrevented: false }"),
        TestAction::run("target.dispatchEvent(event)"),
        TestAction::assert_eq("count", 1),

        // Remove with correct capture flag
        TestAction::run("target.removeEventListener('test', handler, true)"),

        // Dispatch event again - should not call handler
        TestAction::run("target.dispatchEvent(event)"),
        TestAction::assert_eq("count", 1),
    ]);
}

#[test]
fn event_listener_different_event_types() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),
        TestAction::run("var testCount = 0"),
        TestAction::run("var clickCount = 0"),
        TestAction::run("function testHandler() { testCount++; }"),
        TestAction::run("function clickHandler() { clickCount++; }"),

        // Add listeners for different event types
        TestAction::run("target.addEventListener('test', testHandler)"),
        TestAction::run("target.addEventListener('click', clickHandler)"),

        // Dispatch test event
        TestAction::run("var testEvent = { type: 'test', defaultPrevented: false }"),
        TestAction::run("target.dispatchEvent(testEvent)"),
        TestAction::assert_eq("testCount", 1),
        TestAction::assert_eq("clickCount", 0),

        // Dispatch click event
        TestAction::run("var clickEvent = { type: 'click', defaultPrevented: false }"),
        TestAction::run("target.dispatchEvent(clickEvent)"),
        TestAction::assert_eq("testCount", 1),
        TestAction::assert_eq("clickCount", 1),
    ]);
}

#[test]
fn event_target_method_errors() {
    run_test_actions([
        // Test addEventListener on non-object
        TestAction::assert_native_error(
            "EventTarget.prototype.addEventListener.call(null, 'test', function(){})",
            JsNativeErrorKind::Type,
            "EventTarget.addEventListener called on non-object",
        ),

        // Test removeEventListener on non-object
        TestAction::assert_native_error(
            "EventTarget.prototype.removeEventListener.call(null, 'test', function(){})",
            JsNativeErrorKind::Type,
            "EventTarget.removeEventListener called on non-object",
        ),

        // Test dispatchEvent on non-object
        TestAction::assert_native_error(
            "EventTarget.prototype.dispatchEvent.call(null, {})",
            JsNativeErrorKind::Type,
            "EventTarget.dispatchEvent called on non-object",
        ),
    ]);
}

#[test]
fn event_target_on_non_event_target_object() {
    run_test_actions([
        TestAction::run("var obj = {}"),

        // Test addEventListener on non-EventTarget object
        TestAction::assert_native_error(
            "EventTarget.prototype.addEventListener.call(obj, 'test', function(){})",
            JsNativeErrorKind::Type,
            "EventTarget.addEventListener called on non-EventTarget object",
        ),

        // Test removeEventListener on non-EventTarget object
        TestAction::assert_native_error(
            "EventTarget.prototype.removeEventListener.call(obj, 'test', function(){})",
            JsNativeErrorKind::Type,
            "EventTarget.removeEventListener called on non-EventTarget object",
        ),

        // Test dispatchEvent on non-EventTarget object
        TestAction::assert_native_error(
            "EventTarget.prototype.dispatchEvent.call(obj, {})",
            JsNativeErrorKind::Type,
            "EventTarget.dispatchEvent called on non-EventTarget object",
        ),
    ]);
}

#[test]
fn event_target_empty_operations() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),

        // Test operations with no listeners
        TestAction::run("var event = { type: 'test', defaultPrevented: false }"),
        TestAction::run("var result = target.dispatchEvent(event)"),
        TestAction::assert_eq("result", true),

        // Remove non-existent listener (should not throw)
        TestAction::run("target.removeEventListener('test', function(){})"),
        TestAction::assert_eq("true", true),
    ]);
}

#[test]
fn event_listener_error_handling() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),
        TestAction::run("var called = false"),
        TestAction::run("function throwingHandler() { throw new Error('test error'); }"),
        TestAction::run("function normalHandler() { called = true; }"),

        // Add both handlers
        TestAction::run("target.addEventListener('test', throwingHandler)"),
        TestAction::run("target.addEventListener('test', normalHandler)"),

        // Dispatch event - errors in handlers should not stop other handlers
        TestAction::run("var event = { type: 'test', defaultPrevented: false }"),
        TestAction::run("target.dispatchEvent(event)"),

        // Normal handler should still have been called despite error in first handler
        TestAction::assert_eq("called", true),
    ]);
}

#[test]
fn event_target_edge_cases() {
    run_test_actions([
        TestAction::run("var target = new EventTarget()"),

        // Test with weird event type names
        TestAction::run("target.addEventListener('', function(){})"),
        TestAction::run("target.addEventListener('123', function(){})"),
        TestAction::run("target.addEventListener('test-event-name', function(){})"),

        // Test with event objects missing properties
        TestAction::run("var result1 = target.dispatchEvent({})"), // No type property
        TestAction::assert_eq("typeof result1", js_string!("boolean")),

        TestAction::run("var result2 = target.dispatchEvent({ type: null })"),
        TestAction::assert_eq("typeof result2", js_string!("boolean")),

        TestAction::assert_eq("true", true), // If we get here, no exception was thrown
    ]);
}