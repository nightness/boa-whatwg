use crate::{run_test_actions, TestAction};

#[test]
fn timers_exist() {
    run_test_actions([
        TestAction::assert("typeof setTimeout === 'function'"),
        TestAction::assert("typeof setInterval === 'function'"),
        TestAction::assert("typeof clearTimeout === 'function'"),
        TestAction::assert("typeof clearInterval === 'function'"),
    ]);
}

#[test]
fn set_timeout_basic() {
    run_test_actions([
        // setTimeout returns a timer ID (number)
        TestAction::assert("typeof setTimeout(function() {}, 100) === 'number'"),
        // setTimeout with 0 delay should still return timer ID
        TestAction::assert("typeof setTimeout(function() {}, 0) === 'number'"),
        // setTimeout without delay should work (defaults to 0)
        TestAction::assert("typeof setTimeout(function() {}) === 'number'"),
    ]);
}

#[test]
fn set_timeout_minimum_delay() {
    run_test_actions([
        // HTML spec requires minimum 4ms delay
        TestAction::run("let id = setTimeout(function() {}, 1)"),
        TestAction::assert("typeof id === 'number'"),
        TestAction::assert("id > 0"),
    ]);
}

#[test]
fn set_interval_basic() {
    run_test_actions([
        // setInterval returns a timer ID (number)
        TestAction::assert("typeof setInterval(function() {}, 100) === 'number'"),
        // setInterval with 0 delay should still return timer ID
        TestAction::assert("typeof setInterval(function() {}, 0) === 'number'"),
        // setInterval without delay should work (defaults to 0)
        TestAction::assert("typeof setInterval(function() {}) === 'number'"),
    ]);
}

#[test]
fn clear_timeout() {
    run_test_actions([
        TestAction::run("let id = setTimeout(function() { throw new Error('Should not execute'); }, 1000)"),
        TestAction::run("clearTimeout(id)"),
        // clearTimeout with invalid ID should not throw
        TestAction::run("clearTimeout(999999)"),
        // clearTimeout with undefined should not throw
        TestAction::run("clearTimeout(undefined)"),
        // clearTimeout with no arguments should not throw
        TestAction::run("clearTimeout()"),
    ]);
}

#[test]
fn clear_interval() {
    run_test_actions([
        TestAction::run("let id = setInterval(function() { throw new Error('Should not execute'); }, 100)"),
        TestAction::run("clearInterval(id)"),
        // clearInterval with invalid ID should not throw
        TestAction::run("clearInterval(999999)"),
        // clearInterval with undefined should not throw
        TestAction::run("clearInterval(undefined)"),
        // clearInterval with no arguments should not throw
        TestAction::run("clearInterval()"),
    ]);
}

#[test]
fn timer_id_increment() {
    run_test_actions([
        TestAction::run("let id1 = setTimeout(function() {}, 100)"),
        TestAction::run("let id2 = setTimeout(function() {}, 100)"),
        TestAction::run("let id3 = setInterval(function() {}, 100)"),
        // Timer IDs should increment
        TestAction::assert("id2 > id1"),
        TestAction::assert("id3 > id2"),
        // Clean up
        TestAction::run("clearTimeout(id1)"),
        TestAction::run("clearTimeout(id2)"),
        TestAction::run("clearInterval(id3)"),
    ]);
}

#[test]
fn timer_arguments_handling() {
    run_test_actions([
        // Non-function callback should not throw (browsers are permissive)
        TestAction::run("let id1 = setTimeout('console.log(\"string callback\")', 100)"),
        TestAction::run("clearTimeout(id1)"),

        // Delay parameter type coercion
        TestAction::run("let id2 = setTimeout(function() {}, '100')"), // string -> number
        TestAction::run("clearTimeout(id2)"),

        TestAction::run("let id3 = setTimeout(function() {}, true)"), // boolean -> number (1)
        TestAction::run("clearTimeout(id3)"),

        TestAction::run("let id4 = setTimeout(function() {}, null)"), // null -> 0
        TestAction::run("clearTimeout(id4)"),
    ]);
}

#[test]
fn empty_arguments() {
    run_test_actions([
        // setTimeout with no arguments should return 0
        TestAction::assert("setTimeout() === 0"),
        // setInterval with no arguments should return 0
        TestAction::assert("setInterval() === 0"),
    ]);
}

#[test]
fn cross_clear() {
    run_test_actions([
        // Should be able to clear timeout with clearInterval and vice versa (browser behavior)
        TestAction::run("let timeoutId = setTimeout(function() {}, 1000)"),
        TestAction::run("let intervalId = setInterval(function() {}, 1000)"),
        TestAction::run("clearInterval(timeoutId)"), // Should not throw
        TestAction::run("clearTimeout(intervalId)"), // Should not throw
    ]);
}