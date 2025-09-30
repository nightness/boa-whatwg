use crate::{run_test_actions, TestAction};

#[test]
fn console_log() {
    run_test_actions([
        TestAction::run("console.log('Hello, World!')"),
        TestAction::run("console.log(42)"),
        TestAction::run("console.log(true, false, null, undefined)"),
        TestAction::run("console.log({name: 'test'})"),
        TestAction::run("console.log([1, 2, 3])"),
    ]);
}

#[test]
fn console_error() {
    run_test_actions([
        TestAction::run("console.error('Error message')"),
        TestAction::run("console.error(new Error('test error'))"),
        TestAction::run("console.error('Multiple', 'arguments', 123)"),
    ]);
}

#[test]
fn console_warn() {
    run_test_actions([
        TestAction::run("console.warn('Warning message')"),
        TestAction::run("console.warn('Deprecated feature')"),
    ]);
}

#[test]
fn console_info() {
    run_test_actions([
        TestAction::run("console.info('Information')"),
        TestAction::run("console.info('System ready')"),
    ]);
}

#[test]
fn console_debug() {
    run_test_actions([
        TestAction::run("console.debug('Debug info')"),
        TestAction::run("console.debug('Variable:', 42)"),
    ]);
}

#[test]
fn console_trace() {
    run_test_actions([
        TestAction::run("console.trace()"),
        TestAction::run("console.trace('Trace with message')"),
    ]);
}

#[test]
fn console_clear() {
    run_test_actions([
        TestAction::run("console.clear()"),
    ]);
}

#[test]
fn console_group() {
    run_test_actions([
        TestAction::run("console.group('Group 1')"),
        TestAction::run("console.log('Inside group')"),
        TestAction::run("console.groupEnd()"),
        TestAction::run("console.groupCollapsed('Collapsed group')"),
        TestAction::run("console.groupEnd()"),
    ]);
}

#[test]
fn console_group_indentation() {
    run_test_actions([
        TestAction::run("console.log('Level 0')"),
        TestAction::run("console.group('Group Level 1')"),
        TestAction::run("console.log('Level 1')"),
        TestAction::run("console.group('Group Level 2')"),
        TestAction::run("console.log('Level 2')"),
        TestAction::run("console.warn('Warning at Level 2')"),
        TestAction::run("console.groupEnd()"),
        TestAction::run("console.log('Back to Level 1')"),
        TestAction::run("console.groupEnd()"),
        TestAction::run("console.log('Back to Level 0')"),
    ]);
}

#[test]
fn console_group_collapsed() {
    run_test_actions([
        TestAction::run("console.groupCollapsed('Collapsed Group')"),
        TestAction::run("console.log('Inside collapsed group')"),
        TestAction::run("console.error('Error in collapsed group')"),
        TestAction::run("console.groupEnd()"),
        TestAction::run("console.log('Outside group')"),
    ]);
}

#[test]
fn console_group_nested() {
    run_test_actions([
        TestAction::run("console.group('Outer')"),
        TestAction::run("console.group('Middle')"),
        TestAction::run("console.group('Inner')"),
        TestAction::run("console.log('Deeply nested')"),
        TestAction::run("console.groupEnd()"), // End inner
        TestAction::run("console.groupEnd()"), // End middle
        TestAction::run("console.groupEnd()"), // End outer
        TestAction::run("console.log('Back to root')"),
    ]);
}

#[test]
fn console_time() {
    run_test_actions([
        TestAction::run("console.time('timer1')"),
        TestAction::run("console.timeLog('timer1', 'checkpoint')"),
        TestAction::run("console.timeEnd('timer1')"),
        TestAction::run("console.time()"), // default timer
        TestAction::run("console.timeEnd()"),
    ]);
}

#[test]
fn console_time_state_management() {
    run_test_actions([
        // Test timer creation and end
        TestAction::run("console.time('test1')"),
        TestAction::run("console.timeEnd('test1')"),

        // Test timer that doesn't exist
        TestAction::run("console.timeEnd('nonexistent')"),

        // Test duplicate timer warning
        TestAction::run("console.time('duplicate')"),
        TestAction::run("console.time('duplicate')"),
        TestAction::run("console.timeEnd('duplicate')"),

        // Test default timer
        TestAction::run("console.time()"),
        TestAction::run("console.timeLog()"),
        TestAction::run("console.timeEnd()"),
    ]);
}

#[test]
fn console_count() {
    run_test_actions([
        TestAction::run("console.count('counter1')"),
        TestAction::run("console.count('counter1')"),
        TestAction::run("console.count('counter2')"),
        TestAction::run("console.countReset('counter1')"),
        TestAction::run("console.count('counter1')"),
        TestAction::run("console.count()"), // default counter
        TestAction::run("console.countReset()"),
    ]);
}

#[test]
fn console_count_state_management() {
    run_test_actions([
        // Test counter increment
        TestAction::run("console.count('test')"), // Should show 1
        TestAction::run("console.count('test')"), // Should show 2
        TestAction::run("console.count('test')"), // Should show 3

        // Test different counters
        TestAction::run("console.count('other')"), // Should show 1
        TestAction::run("console.count('test')"),  // Should show 4

        // Test counter reset
        TestAction::run("console.countReset('test')"),
        TestAction::run("console.count('test')"), // Should show 1 again

        // Test reset nonexistent counter
        TestAction::run("console.countReset('nonexistent')"),

        // Test default counter
        TestAction::run("console.count()"), // Should show default: 1
        TestAction::run("console.count()"), // Should show default: 2
        TestAction::run("console.countReset()"),
        TestAction::run("console.count()"), // Should show default: 1
    ]);
}

#[test]
fn console_assert() {
    run_test_actions([
        TestAction::run("console.assert(true, 'This should not print')"),
        TestAction::run("console.assert(false, 'This should print')"),
        TestAction::run("console.assert(1 === 1, 'Math works')"),
        TestAction::run("console.assert(1 === 2, 'Math is broken')"),
        TestAction::run("console.assert(false)"), // Default message
    ]);
}

#[test]
fn console_table() {
    run_test_actions([
        TestAction::run("console.table([1, 2, 3])"),
        TestAction::run("console.table({a: 1, b: 2})"),
        TestAction::run("console.table([{name: 'John', age: 30}, {name: 'Jane', age: 25}])"),
    ]);
}

#[test]
fn console_dir() {
    run_test_actions([
        TestAction::run("console.dir({nested: {object: true}})"),
        TestAction::run("console.dir(function test() {})"),
        TestAction::run("console.dirxml('<div>HTML</div>')"),
    ]);
}

#[test]
fn console_dir_enhanced() {
    run_test_actions([
        TestAction::run("console.dir({})"), // Empty object
        TestAction::run("console.dir([])"), // Empty array
        TestAction::run("console.dir(42)"), // Number
        TestAction::run("console.dir('string')"), // String
        TestAction::run("console.dir(true)"), // Boolean
        TestAction::run("console.dir(null)"), // Null
        TestAction::run("console.dir(undefined)"), // Undefined
        TestAction::run("console.dir({a: 1, b: 2})"), // Object with properties
    ]);
}

#[test]
fn console_table_enhanced() {
    run_test_actions([
        TestAction::run("console.table([1, 2, 3])"), // Array
        TestAction::run("console.table({a: 1, b: 2})"), // Object
        TestAction::run("console.table([{name: 'Alice', age: 30}, {name: 'Bob', age: 25}])"), // Array of objects
        TestAction::run("console.table('not tabular')"), // Non-tabular data
        TestAction::run("console.table(null)"), // Null data
    ]);
}

#[test]
fn console_trace_enhanced() {
    run_test_actions([
        TestAction::run("console.trace()"), // No message
        TestAction::run("console.trace('Custom trace message')"), // With message
        TestAction::run("console.trace('Multiple', 'arguments', 123)"), // Multiple args
        // Test trace with grouping
        TestAction::run("console.group('Test Group')"),
        TestAction::run("console.trace('Trace inside group')"),
        TestAction::run("console.groupEnd()"),
    ]);
}

#[test]
fn console_exists() {
    run_test_actions([
        TestAction::assert("typeof console === 'function'"),
        TestAction::assert("typeof console.log === 'function'"),
        TestAction::assert("typeof console.error === 'function'"),
        TestAction::assert("typeof console.warn === 'function'"),
        TestAction::assert("typeof console.info === 'function'"),
        TestAction::assert("typeof console.debug === 'function'"),
        TestAction::assert("typeof console.trace === 'function'"),
        TestAction::assert("typeof console.clear === 'function'"),
        TestAction::assert("typeof console.group === 'function'"),
        TestAction::assert("typeof console.groupCollapsed === 'function'"),
        TestAction::assert("typeof console.groupEnd === 'function'"),
        TestAction::assert("typeof console.time === 'function'"),
        TestAction::assert("typeof console.timeEnd === 'function'"),
        TestAction::assert("typeof console.timeLog === 'function'"),
        TestAction::assert("typeof console.count === 'function'"),
        TestAction::assert("typeof console.countReset === 'function'"),
        TestAction::assert("typeof console.assert === 'function'"),
        TestAction::assert("typeof console.table === 'function'"),
        TestAction::assert("typeof console.dir === 'function'"),
        TestAction::assert("typeof console.dirxml === 'function'"),
    ]);
}