//! Tests for the Event Web API implementation

use crate::{run_test_actions, TestAction};

#[test]
fn event_constructor() {
    run_test_actions([
        // Event constructor exists and is a function
        TestAction::assert("typeof Event === 'function'"),

        // Can create basic events
        TestAction::run("var evt = new Event('test')"),
        TestAction::assert("evt instanceof Event"),
        TestAction::assert("evt.type === 'test'"),
        TestAction::assert("evt.bubbles === false"),
        TestAction::assert("evt.cancelable === false"),
        TestAction::assert("evt.defaultPrevented === false"),
        TestAction::assert("evt.eventPhase === Event.NONE"),
    ]);
}

#[test]
fn event_constructor_with_options() {
    run_test_actions([
        // Event with options
        TestAction::run("var evt = new Event('custom', { bubbles: true, cancelable: true })"),
        TestAction::assert("evt.type === 'custom'"),
        TestAction::assert("evt.bubbles === true"),
        TestAction::assert("evt.cancelable === true"),
        TestAction::assert("evt.defaultPrevented === false"),
    ]);
}

#[test]
fn event_constants() {
    run_test_actions([
        // Event phase constants
        TestAction::assert("Event.NONE === 0"),
        TestAction::assert("Event.CAPTURING_PHASE === 1"),
        TestAction::assert("Event.AT_TARGET === 2"),
        TestAction::assert("Event.BUBBLING_PHASE === 3"),
    ]);
}

#[test]
fn event_methods() {
    run_test_actions([
        TestAction::run("var evt = new Event('test', { cancelable: true })"),

        // preventDefault
        TestAction::assert("typeof evt.preventDefault === 'function'"),
        TestAction::run("evt.preventDefault()"),
        TestAction::assert("evt.defaultPrevented === true"),

        // stopPropagation
        TestAction::assert("typeof evt.stopPropagation === 'function'"),
        TestAction::run("evt.stopPropagation()"),

        // stopImmediatePropagation
        TestAction::assert("typeof evt.stopImmediatePropagation === 'function'"),
        TestAction::run("evt.stopImmediatePropagation()"),
    ]);
}

#[test]
fn event_init_event() {
    run_test_actions([
        TestAction::run("var evt = new Event('initial')"),
        TestAction::assert("evt.type === 'initial'"),
        TestAction::assert("evt.bubbles === false"),
        TestAction::assert("evt.cancelable === false"),

        // initEvent (legacy method)
        TestAction::run("evt.initEvent('modified', true, true)"),
        TestAction::assert("evt.type === 'modified'"),
        TestAction::assert("evt.bubbles === true"),
        TestAction::assert("evt.cancelable === true"),
    ]);
}

#[test]
fn event_readonly_properties() {
    run_test_actions([
        TestAction::run("var evt = new Event('test')"),

        // Properties should be read-only
        TestAction::assert("evt.type === 'test'"),
        TestAction::assert("evt.bubbles === false"),
        TestAction::assert("evt.cancelable === false"),
        TestAction::assert("typeof evt.timeStamp === 'number'"),
        TestAction::assert("evt.isTrusted === false"),
        TestAction::assert("evt.composed === false"),
    ]);
}