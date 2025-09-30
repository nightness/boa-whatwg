//! Tests for the Performance API implementation

use crate::{run_test_actions, TestAction, Context, Source, JsValue, JsString, js_string};

#[test]
fn performance_exists() {
    run_test_actions([
        TestAction::run("typeof performance"),
        TestAction::assert_eq("typeof performance", JsValue::from(JsString::from("object"))),
    ]);
}

#[test]
fn performance_now() {
    run_test_actions([
        TestAction::run("typeof performance.now"),
        TestAction::assert_eq("typeof performance.now", JsValue::from(JsString::from("function"))),
        TestAction::run("typeof performance.now()"),
        TestAction::assert_eq("typeof performance.now()", JsValue::from(JsString::from("number"))),
        TestAction::run("performance.now() >= 0"),
        TestAction::assert_eq("performance.now() >= 0", JsValue::from(true)),
    ]);
}

#[test]
fn performance_time_origin() {
    run_test_actions([
        TestAction::run("typeof performance.timeOrigin"),
        TestAction::assert_eq("typeof performance.timeOrigin", JsValue::from(JsString::from("number"))),
        TestAction::run("performance.timeOrigin >= 0"),
        TestAction::assert_eq("performance.timeOrigin >= 0", JsValue::from(true)),
        TestAction::run("const original = performance.timeOrigin; performance.timeOrigin = 123; performance.timeOrigin === original"),
        TestAction::assert_eq("performance.timeOrigin === original", JsValue::from(true)),
    ]);
}

#[test]
fn performance_api_methods_exist() {
    run_test_actions([
        TestAction::run("typeof performance.mark"),
        TestAction::assert_eq("typeof performance.mark", JsValue::from(JsString::from("function"))),
        TestAction::run("typeof performance.measure"),
        TestAction::assert_eq("typeof performance.measure", JsValue::from(JsString::from("function"))),
        TestAction::run("typeof performance.clearMarks"),
        TestAction::assert_eq("typeof performance.clearMarks", JsValue::from(JsString::from("function"))),
        TestAction::run("typeof performance.clearMeasures"),
        TestAction::assert_eq("typeof performance.clearMeasures", JsValue::from(JsString::from("function"))),
        TestAction::run("typeof performance.getEntries"),
        TestAction::assert_eq("typeof performance.getEntries", JsValue::from(JsString::from("function"))),
        TestAction::run("typeof performance.getEntriesByType"),
        TestAction::assert_eq("typeof performance.getEntriesByType", JsValue::from(JsString::from("function"))),
        TestAction::run("typeof performance.getEntriesByName"),
        TestAction::assert_eq("typeof performance.getEntriesByName", JsValue::from(JsString::from("function"))),
    ]);
}

#[test]
fn performance_mark() {
    run_test_actions([
        TestAction::run("const mark = performance.mark('test-mark'); mark.entryType"),
        TestAction::assert_eq("mark.entryType", JsValue::from(JsString::from("mark"))),
        TestAction::run("mark.name"),
        TestAction::assert_eq("mark.name", JsValue::from(JsString::from("test-mark"))),
        TestAction::run("typeof mark.startTime"),
        TestAction::assert_eq("typeof mark.startTime", JsValue::from(JsString::from("number"))),
        TestAction::run("mark.duration"),
        TestAction::assert_eq("mark.duration", JsValue::from(0)),
    ]);
}

#[test]
fn performance_get_entries() {
    run_test_actions([
        TestAction::run("performance.mark('entry-test-1'); performance.mark('entry-test-2'); const entries = performance.getEntries(); entries.length >= 2"),
        TestAction::assert_eq("entries.length >= 2", JsValue::from(true)),
    ]);
}

#[test]
fn performance_get_entries_by_type() {
    run_test_actions([
        TestAction::run("performance.mark('type-test'); const markEntries = performance.getEntriesByType('mark'); markEntries.length >= 1"),
        TestAction::assert_eq("markEntries.length >= 1", JsValue::from(true)),
    ]);
}

#[test]
fn performance_measure() {
    run_test_actions([
        TestAction::run("performance.mark('start'); performance.mark('end'); const measure = performance.measure('test-measure', 'start', 'end'); measure.entryType"),
        TestAction::assert_eq("measure.entryType", JsValue::from(JsString::from("measure"))),
        TestAction::run("measure.name"),
        TestAction::assert_eq("measure.name", JsValue::from(JsString::from("test-measure"))),
        TestAction::run("typeof measure.duration"),
        TestAction::assert_eq("typeof measure.duration", JsValue::from(JsString::from("number"))),
    ]);
}

#[test]
fn performance_clear_marks() {
    run_test_actions([
        TestAction::run("performance.mark('clear-test-1'); performance.mark('clear-test-2'); performance.clearMarks('clear-test-1'); performance.getEntriesByName('clear-test-1').length"),
        TestAction::assert_eq("performance.getEntriesByName('clear-test-1').length", JsValue::from(0)),
        TestAction::run("performance.getEntriesByName('clear-test-2').length"),
        TestAction::assert_eq("performance.getEntriesByName('clear-test-2').length", JsValue::from(1)),
    ]);
}

#[test]
fn performance_clear_measures() {
    run_test_actions([
        TestAction::run("performance.mark('measure-start'); performance.measure('clear-measure-1', 'measure-start'); performance.measure('clear-measure-2', 'measure-start'); performance.clearMeasures('clear-measure-1'); performance.getEntriesByName('clear-measure-1').length"),
        TestAction::assert_eq("performance.getEntriesByName('clear-measure-1').length", JsValue::from(0)),
        TestAction::run("performance.getEntriesByName('clear-measure-2').length"),
        TestAction::assert_eq("performance.getEntriesByName('clear-measure-2').length", JsValue::from(1)),
    ]);
}

#[test]
fn performance_get_entries_by_name() {
    run_test_actions([
        TestAction::run("performance.mark('name-test'); performance.mark('name-test'); const nameEntries = performance.getEntriesByName('name-test'); nameEntries.length"),
        TestAction::assert_eq("nameEntries.length", JsValue::from(2)),
        TestAction::run("performance.mark('other-start'); performance.measure('name-test', 'other-start'); const mixedEntries = performance.getEntriesByName('name-test'); mixedEntries.length"),
        TestAction::assert_eq("mixedEntries.length", JsValue::from(3)),
        TestAction::run("const measureEntries = performance.getEntriesByName('name-test', 'measure'); measureEntries.length"),
        TestAction::assert_eq("measureEntries.length", JsValue::from(1)),
    ]);
}

#[test]
fn performance_timing() {
    run_test_actions([
        TestAction::run("typeof performance.timing"),
        TestAction::assert_eq("typeof performance.timing", JsValue::from(JsString::from("object"))),
        TestAction::run("typeof performance.timing.navigationStart"),
        TestAction::assert_eq("typeof performance.timing.navigationStart", JsValue::from(JsString::from("number"))),
        TestAction::run("typeof performance.timing.loadEventEnd"),
        TestAction::assert_eq("typeof performance.timing.loadEventEnd", JsValue::from(JsString::from("number"))),
        TestAction::run("performance.timing.navigationStart > 0"),
        TestAction::assert_eq("performance.timing.navigationStart > 0", JsValue::from(true)),
    ]);
}

#[test]
fn performance_mark_timing_consistency() {
    run_test_actions([
        TestAction::run("const mark1 = performance.mark('timing-test-1'); const mark2 = performance.mark('timing-test-2'); mark2.startTime >= mark1.startTime"),
        TestAction::assert_eq("mark2.startTime >= mark1.startTime", JsValue::from(true)),
    ]);
}

#[test]
fn performance_mark_multiple() {
    run_test_actions([
        TestAction::run("performance.mark('multi-1'); performance.mark('multi-2'); performance.mark('multi-3'); const entries = performance.getEntriesByType('mark'); entries.length >= 3"),
        TestAction::assert_eq("entries.length >= 3", JsValue::from(true)),
    ]);
}

#[test]
fn performance_measure_duration_accuracy() {
    run_test_actions([
        TestAction::run("performance.mark('duration-start'); performance.mark('duration-end'); const measure = performance.measure('duration-test', 'duration-start', 'duration-end'); measure.duration >= 0"),
        TestAction::assert_eq("measure.duration >= 0", JsValue::from(true)),
    ]);
}

#[test]
fn performance_measure_without_marks() {
    run_test_actions([
        TestAction::run("const measure = performance.measure('no-marks-test'); measure.entryType"),
        TestAction::assert_eq("measure.entryType", JsValue::from(JsString::from("measure"))),
        TestAction::run("measure.startTime >= 0"),
        TestAction::assert_eq("measure.startTime >= 0", JsValue::from(true)),
        TestAction::run("measure.duration >= 0"),
        TestAction::assert_eq("measure.duration >= 0", JsValue::from(true)),
    ]);
}

// Test using the Context API directly for more detailed testing
#[test]
fn performance_direct_context_test() {
    let mut context = Context::default();

    // Test what's actually in the global object
    let global_obj = context.global_object();
    let performance_prop = global_obj.get(js_string!("performance"), &mut context);
    eprintln!("Global performance property result: {:?}", performance_prop);

    // Compare with console for reference
    let console_prop = global_obj.get(js_string!("console"), &mut context);
    eprintln!("Global console property result: {:?}", console_prop);

    // Test that performance object exists
    let result = context.eval(Source::from_bytes("typeof performance"));
    eprintln!("typeof performance result: {:?}", result);

    // Also check console for comparison
    let console_result = context.eval(Source::from_bytes("typeof console"));
    eprintln!("typeof console result: {:?}", console_result);

    assert!(result.is_ok(), "Performance access should not error: {:?}", result);
    assert_eq!(result.unwrap(), JsValue::from(JsString::from("object")));

    // Test performance.now() returns a number
    let result = context.eval(Source::from_bytes("typeof performance.now()")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("number")));

    // Test performance.now() returns increasing values
    let result1 = context.eval(Source::from_bytes("performance.now()")).unwrap();
    let result2 = context.eval(Source::from_bytes("performance.now()")).unwrap();

    if let (Some(n1), Some(n2)) = (result1.as_number(), result2.as_number()) {
        assert!(n2 >= n1, "performance.now() should return non-decreasing values");
    }
}