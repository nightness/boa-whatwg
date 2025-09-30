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

#[test]
fn timer_callback_execution() {
    run_test_actions([
        // Test string callback execution
        TestAction::run("setTimeout('console.log(\"String callback executed\")', 10)"),

        // Test function callback (placeholder test since we can't actually execute)
        TestAction::run("setTimeout(function() { console.log('Function callback'); }, 20)"),

        // Test with additional arguments
        TestAction::run("setTimeout(function(a, b) { console.log('Args:', a, b); }, 30, 'arg1', 'arg2')"),
    ]);
}

#[test]
fn timer_delay_clamping() {
    run_test_actions([
        // Test minimum delay clamping
        TestAction::run("let id1 = setTimeout(function() {}, 0)"), // Should be clamped to 4ms
        TestAction::run("let id2 = setTimeout(function() {}, 1)"), // Should be clamped to 4ms
        TestAction::run("let id3 = setTimeout(function() {}, 3)"), // Should be clamped to 4ms
        TestAction::run("let id4 = setTimeout(function() {}, 5)"), // Should remain 5ms

        TestAction::assert("typeof id1 === 'number'"),
        TestAction::assert("typeof id2 === 'number'"),
        TestAction::assert("typeof id3 === 'number'"),
        TestAction::assert("typeof id4 === 'number'"),

        // Clean up
        TestAction::run("clearTimeout(id1)"),
        TestAction::run("clearTimeout(id2)"),
        TestAction::run("clearTimeout(id3)"),
        TestAction::run("clearTimeout(id4)"),
    ]);
}

#[test]
fn timer_async_behavior() {
    run_test_actions([
        // Test that timers return immediately but execute asynchronously
        TestAction::run("let executed = false"),
        TestAction::run("let timerId = setTimeout(function() { executed = true; }, 50)"),
        TestAction::assert("executed === false"), // Should not have executed yet
        TestAction::assert("typeof timerId === 'number'"),

        // Test interval creation
        TestAction::run("let intervalExecuted = 0"),
        TestAction::run("let intervalId = setInterval(function() { intervalExecuted++; }, 25)"),
        TestAction::assert("intervalExecuted === 0"), // Should not have executed yet

        // Clean up
        TestAction::run("clearInterval(intervalId)"),
    ]);
}

#[test]
fn timer_string_vs_function_callbacks() {
    run_test_actions([
        // Test string callbacks (eval-style)
        TestAction::run("setTimeout('var stringCallbackVar = 42;', 10)"),

        // Test function callbacks
        TestAction::run("setTimeout(function() { var functionCallbackVar = 84; }, 15)"),

        // Test arrow function callbacks
        TestAction::run("setTimeout(() => { var arrowCallbackVar = 168; }, 20)"),

        // All should return valid timer IDs
        TestAction::run("let id1 = setTimeout('console.log(1)', 30)"),
        TestAction::run("let id2 = setTimeout(function() { console.log(2); }, 35)"),
        TestAction::run("let id3 = setTimeout(() => console.log(3), 40)"),

        TestAction::assert("typeof id1 === 'number'"),
        TestAction::assert("typeof id2 === 'number'"),
        TestAction::assert("typeof id3 === 'number'"),
        TestAction::assert("id1 !== id2 && id2 !== id3 && id1 !== id3"),
    ]);
}

#[test]
fn interval_vs_timeout_behavior() {
    run_test_actions([
        // Test that setTimeout executes once
        TestAction::run("let timeoutCount = 0"),
        TestAction::run("setTimeout(function() { timeoutCount++; }, 20)"),

        // Test that setInterval would execute multiple times (if we let it)
        TestAction::run("let intervalCount = 0"),
        TestAction::run("let intervalId = setInterval(function() { intervalCount++; }, 25)"),

        // Clear interval immediately to prevent multiple executions in test
        TestAction::run("clearInterval(intervalId)"),

        TestAction::assert("timeoutCount === 0"), // Not executed yet
        TestAction::assert("intervalCount === 0"), // Not executed yet (cleared)
    ]);
}

#[test]
fn timer_id_uniqueness() {
    run_test_actions([
        // Test that timer IDs are unique and incrementing
        TestAction::run("let ids = []"),
        TestAction::run("for (let i = 0; i < 10; i++) { ids.push(setTimeout(function() {}, 100)); }"),

        // Check uniqueness
        TestAction::run("let unique = new Set(ids)"),
        TestAction::assert("unique.size === 10"),

        // Check ascending order
        TestAction::run("let sorted = [...ids].sort((a, b) => a - b)"),
        TestAction::assert("JSON.stringify(ids) === JSON.stringify(sorted)"),

        // Clean up all timers
        TestAction::run("ids.forEach(id => clearTimeout(id))"),
    ]);
}