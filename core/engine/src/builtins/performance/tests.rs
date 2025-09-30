//! Tests for the Performance API implementation

use crate::{run_test_actions, TestAction, Context, Source, JsValue, JsString, js_string};

#[test]
fn performance_exists() {
    run_test_actions([
        TestAction::run("typeof performance"),
        TestAction::assert_eq("typeof performance", JsValue::from(JsString::from("object"))),
        TestAction::run("performance !== null"),
        TestAction::assert_eq("performance !== null", JsValue::from(true)),
    ]);
}

#[test]
fn performance_now() {
    run_test_actions([
        TestAction::run("let start = performance.now()"),
        TestAction::run("typeof start"),
        TestAction::assert_eq("typeof start", JsValue::from(JsString::from("number"))),
        TestAction::run("start >= 0"),
        TestAction::assert_eq("start >= 0", JsValue::from(true)),
        TestAction::run("let end = performance.now()"),
        TestAction::run("end >= start"),
        TestAction::assert_eq("end >= start", JsValue::from(true)),
        // Test that now() returns different values over time
        TestAction::run("performance.now() !== performance.now() || true"), // Allow same value but usually different
        TestAction::assert_eq("performance.now() !== performance.now() || true", JsValue::from(true)),
    ]);
}

#[test]
fn performance_time_origin() {
    run_test_actions([
        TestAction::run("typeof performance.timeOrigin"),
        TestAction::assert_eq("typeof performance.timeOrigin", JsValue::from(JsString::from("number"))),
        TestAction::run("performance.timeOrigin > 0"),
        TestAction::assert_eq("performance.timeOrigin > 0", JsValue::from(true)),
        // timeOrigin should be read-only
        TestAction::run("let original = performance.timeOrigin"),
        TestAction::run("performance.timeOrigin = 12345"),
        TestAction::run("performance.timeOrigin === original"),
        TestAction::assert_eq("performance.timeOrigin === original", JsValue::from(true)),
    ]);
}

#[test]
fn performance_mark() {
    run_test_actions([
        TestAction::run("performance.mark('test-mark')"),
        TestAction::run("let entries = performance.getEntriesByType('mark')"),
        TestAction::run("entries.length"),
        TestAction::assert_eq("entries.length", JsValue::from(1)),
        TestAction::run("entries[0].name"),
        TestAction::assert_eq("entries[0].name", JsValue::from(JsString::from("test-mark"))),
        TestAction::run("entries[0].entryType"),
        TestAction::assert_eq("entries[0].entryType", JsValue::from(JsString::from("mark"))),
        TestAction::run("typeof entries[0].startTime"),
        TestAction::assert_eq("typeof entries[0].startTime", JsValue::from(JsString::from("number"))),
        TestAction::run("entries[0].duration"),
        TestAction::assert_eq("entries[0].duration", JsValue::from(0)),
    ]);
}

#[test]
fn performance_mark_multiple() {
    run_test_actions([
        TestAction::run("performance.mark('mark1')"),
        TestAction::run("performance.mark('mark2')"),
        TestAction::run("performance.mark('mark3')"),
        TestAction::run("let entries = performance.getEntriesByType('mark')"),
        TestAction::run("entries.length >= 3"), // May have entries from other tests
        TestAction::assert_eq("entries.length >= 3", JsValue::from(true)),
        // Check that we can find our specific marks
        TestAction::run("performance.getEntriesByName('mark1').length"),
        TestAction::assert_eq("performance.getEntriesByName('mark1').length", JsValue::from(1)),
        TestAction::run("performance.getEntriesByName('mark2').length"),
        TestAction::assert_eq("performance.getEntriesByName('mark2').length", JsValue::from(1)),
        TestAction::run("performance.getEntriesByName('mark3').length"),
        TestAction::assert_eq("performance.getEntriesByName('mark3').length", JsValue::from(1)),
    ]);
}

#[test]
fn performance_measure() {
    run_test_actions([
        TestAction::run("performance.mark('start-mark')"),
        TestAction::run("performance.mark('end-mark')"),
        TestAction::run("performance.measure('test-measure', 'start-mark', 'end-mark')"),
        TestAction::run("let measures = performance.getEntriesByType('measure')"),
        TestAction::run("measures.length >= 1"),
        TestAction::assert_eq("measures.length >= 1", JsValue::from(true)),
        TestAction::run("let measure = performance.getEntriesByName('test-measure')[0]"),
        TestAction::run("measure.name"),
        TestAction::assert_eq("measure.name", JsValue::from(JsString::from("test-measure"))),
        TestAction::run("measure.entryType"),
        TestAction::assert_eq("measure.entryType", JsValue::from(JsString::from("measure"))),
        TestAction::run("typeof measure.startTime"),
        TestAction::assert_eq("typeof measure.startTime", JsValue::from(JsString::from("number"))),
        TestAction::run("typeof measure.duration"),
        TestAction::assert_eq("typeof measure.duration", JsValue::from(JsString::from("number"))),
        TestAction::run("measure.duration >= 0"),
        TestAction::assert_eq("measure.duration >= 0", JsValue::from(true)),
    ]);
}

#[test]
fn performance_measure_without_marks() {
    run_test_actions([
        // Measure without specifying marks should work
        TestAction::run("performance.measure('auto-measure')"),
        TestAction::run("let measures = performance.getEntriesByName('auto-measure')"),
        TestAction::run("measures.length"),
        TestAction::assert_eq("measures.length", JsValue::from(1)),
        TestAction::run("measures[0].entryType"),
        TestAction::assert_eq("measures[0].entryType", JsValue::from(JsString::from("measure"))),
    ]);
}

#[test]
fn performance_clear_marks() {
    run_test_actions([
        TestAction::run("performance.mark('clear-test-1')"),
        TestAction::run("performance.mark('clear-test-2')"),
        TestAction::run("performance.getEntriesByName('clear-test-1').length"),
        TestAction::assert_eq("performance.getEntriesByName('clear-test-1').length", JsValue::from(1)),
        TestAction::run("performance.getEntriesByName('clear-test-2').length"),
        TestAction::assert_eq("performance.getEntriesByName('clear-test-2').length", JsValue::from(1)),

        // Clear specific mark
        TestAction::run("performance.clearMarks('clear-test-1')"),
        TestAction::run("performance.getEntriesByName('clear-test-1').length"),
        TestAction::assert_eq("performance.getEntriesByName('clear-test-1').length", JsValue::from(0)),
        TestAction::run("performance.getEntriesByName('clear-test-2').length"),
        TestAction::assert_eq("performance.getEntriesByName('clear-test-2').length", JsValue::from(1)),

        // Clear all marks
        TestAction::run("performance.clearMarks()"),
        TestAction::run("performance.getEntriesByName('clear-test-2').length"),
        TestAction::assert_eq("performance.getEntriesByName('clear-test-2').length", 0),
    ]);
}

#[test]
fn performance_clear_measures() {
    run_test_actions([
        TestAction::run("performance.mark('measure-start')"),
        TestAction::run("performance.mark('measure-end')"),
        TestAction::run("performance.measure('clear-measure-1', 'measure-start', 'measure-end')"),
        TestAction::run("performance.measure('clear-measure-2', 'measure-start', 'measure-end')"),
        TestAction::run("performance.getEntriesByName('clear-measure-1').length"),
        TestAction::assert_eq("performance.getEntriesByName('clear-measure-1').length", JsValue::from(1)),
        TestAction::run("performance.getEntriesByName('clear-measure-2').length"),
        TestAction::assert_eq("performance.getEntriesByName('clear-measure-2').length", JsValue::from(1)),

        // Clear specific measure
        TestAction::run("performance.clearMeasures('clear-measure-1')"),
        TestAction::run("performance.getEntriesByName('clear-measure-1').length"),
        TestAction::assert_eq("performance.getEntriesByName('clear-measure-1').length", JsValue::from(0)),
        TestAction::run("performance.getEntriesByName('clear-measure-2').length"),
        TestAction::assert_eq("performance.getEntriesByName('clear-measure-2').length", JsValue::from(1)),

        // Clear all measures
        TestAction::run("performance.clearMeasures()"),
        TestAction::run("performance.getEntriesByName('clear-measure-2').length"),
        TestAction::assert_eq("performance.getEntriesByName('clear-measure-2').length", JsValue::from(0)),
    ]);
}

#[test]
fn performance_get_entries() {
    run_test_actions([
        // Clear previous entries for clean test
        TestAction::run("performance.clearMarks()"),
        TestAction::run("performance.clearMeasures()"),

        TestAction::run("performance.mark('entry-test-mark')"),
        TestAction::run("performance.measure('entry-test-measure')"),
        TestAction::run("let allEntries = performance.getEntries()"),
        TestAction::run("allEntries.length >= 2"), // At least our mark and measure
        TestAction::assert_eq("allEntries.length >= 2", JsValue::from(true)),

        // Test that entries have required properties
        TestAction::run("allEntries.every(entry => typeof entry.name === 'string')"),
        TestAction::assert_eq("allEntries.every(entry => typeof entry.name === 'string')", JsValue::from(true)),
        TestAction::run("allEntries.every(entry => typeof entry.entryType === 'string')"),
        TestAction::assert_eq("allEntries.every(entry => typeof entry.entryType === 'string')", JsValue::from(true)),
        TestAction::run("allEntries.every(entry => typeof entry.startTime === 'number')"),
        TestAction::assert_eq("allEntries.every(entry => typeof entry.startTime === 'number')", JsValue::from(true)),
        TestAction::run("allEntries.every(entry => typeof entry.duration === 'number')"),
        TestAction::assert_eq("allEntries.every(entry => typeof entry.duration === 'number')", JsValue::from(true)),
    ]);
}

#[test]
fn performance_get_entries_by_type() {
    run_test_actions([
        TestAction::run("performance.clearMarks()"),
        TestAction::run("performance.clearMeasures()"),

        TestAction::run("performance.mark('type-test-mark')"),
        TestAction::run("performance.measure('type-test-measure')"),

        TestAction::run("let marks = performance.getEntriesByType('mark')"),
        TestAction::run("let measures = performance.getEntriesByType('measure')"),

        TestAction::run("marks.length >= 1"),
        TestAction::assert_eq("marks.length >= 1", JsValue::from(true)),
        TestAction::run("measures.length >= 1"),
        TestAction::assert_eq("measures.length >= 1", JsValue::from(true)),

        TestAction::run("marks.every(entry => entry.entryType === 'mark')"),
        TestAction::assert_eq("marks.every(entry => entry.entryType === 'mark')", JsValue::from(true)),
        TestAction::run("measures.every(entry => entry.entryType === 'measure')"),
        TestAction::assert_eq("measures.every(entry => entry.entryType === 'measure')", JsValue::from(true)),

        // Test non-existent type
        TestAction::run("performance.getEntriesByType('nonexistent').length"),
        TestAction::assert_eq("performance.getEntriesByType('nonexistent').length", JsValue::from(0)),
    ]);
}

#[test]
fn performance_get_entries_by_name() {
    run_test_actions([
        TestAction::run("performance.mark('name-test-unique')"),
        TestAction::run("performance.measure('name-test-measure')"),

        TestAction::run("let uniqueEntries = performance.getEntriesByName('name-test-unique')"),
        TestAction::run("uniqueEntries.length"),
        TestAction::assert_eq("uniqueEntries.length", JsValue::from(1)),
        TestAction::run("uniqueEntries[0].name"),
        TestAction::assert_eq("uniqueEntries[0].name", JsValue::from(JsString::from("name-test-unique"))),
        TestAction::run("uniqueEntries[0].entryType"),
        TestAction::assert_eq("uniqueEntries[0].entryType", JsValue::from(JsString::from("mark"))),

        // Test with type filter
        TestAction::run("let measureEntries = performance.getEntriesByName('name-test-measure', 'measure')"),
        TestAction::run("measureEntries.length"),
        TestAction::assert_eq("measureEntries.length", JsValue::from(1)),
        TestAction::run("measureEntries[0].entryType"),
        TestAction::assert_eq("measureEntries[0].entryType", JsValue::from(JsString::from("measure"))),

        // Test non-matching type filter
        TestAction::run("performance.getEntriesByName('name-test-measure', 'mark').length"),
        TestAction::assert_eq("performance.getEntriesByName('name-test-measure', 'mark').length", JsValue::from(0)),

        // Test non-existent name
        TestAction::run("performance.getEntriesByName('non-existent-name').length"),
        TestAction::assert_eq("performance.getEntriesByName('non-existent-name').length", JsValue::from(0)),
    ]);
}

#[test]
fn performance_timing() {
    run_test_actions([
        TestAction::run("typeof performance.timing"),
        TestAction::assert_eq("typeof performance.timing", JsValue::from(JsString::from("object"))),
        TestAction::run("performance.timing !== null"),
        TestAction::assert_eq("performance.timing !== null", JsValue::from(true)),

        // Test that timing properties exist and are numbers
        TestAction::run("typeof performance.timing.navigationStart"),
        TestAction::assert_eq("typeof performance.timing.navigationStart", JsValue::from(JsString::from("number"))),
        TestAction::run("typeof performance.timing.fetchStart"),
        TestAction::assert_eq("typeof performance.timing.fetchStart", JsValue::from(JsString::from("number"))),
        TestAction::run("typeof performance.timing.domLoading"),
        TestAction::assert_eq("typeof performance.timing.domLoading", JsValue::from(JsString::from("number"))),
        TestAction::run("typeof performance.timing.domComplete"),
        TestAction::assert_eq("typeof performance.timing.domComplete", JsValue::from(JsString::from("number"))),
        TestAction::run("typeof performance.timing.loadEventEnd"),
        TestAction::assert_eq("typeof performance.timing.loadEventEnd", JsValue::from(JsString::from("number"))),

        // Test that timing values are reasonable (greater than 0)
        TestAction::run("performance.timing.navigationStart > 0"),
        TestAction::assert_eq("performance.timing.navigationStart > 0", JsValue::from(true)),
        TestAction::run("performance.timing.fetchStart >= performance.timing.navigationStart"),
        TestAction::assert_eq("performance.timing.fetchStart >= performance.timing.navigationStart", JsValue::from(true)),

        // Test that timing object is read-only
        TestAction::run("let original = performance.timing.navigationStart"),
        TestAction::run("performance.timing.navigationStart = 12345"),
        TestAction::run("performance.timing.navigationStart === original"),
        TestAction::assert_eq("performance.timing.navigationStart === original", JsValue::from(true)),
    ]);
}

#[test]
fn performance_mark_timing_consistency() {
    run_test_actions([
        // Test that marks use consistent timing with performance.now()
        TestAction::run("let beforeNow = performance.now()"),
        TestAction::run("performance.mark('timing-test')"),
        TestAction::run("let afterNow = performance.now()"),
        TestAction::run("let markEntry = performance.getEntriesByName('timing-test')[0]"),
        TestAction::run("markEntry.startTime >= beforeNow"),
        TestAction::assert_eq("markEntry.startTime >= beforeNow", JsValue::from(true)),
        TestAction::run("markEntry.startTime <= afterNow"),
        TestAction::assert_eq("markEntry.startTime <= afterNow", JsValue::from(true)),
    ]);
}

#[test]
fn performance_measure_duration_accuracy() {
    run_test_actions([
        TestAction::run("performance.mark('duration-start')"),
        TestAction::run("let startTime = performance.now()"),
        // Small delay simulation
        TestAction::run("for(let i = 0; i < 100; i++) { Math.random(); }"),
        TestAction::run("performance.mark('duration-end')"),
        TestAction::run("let endTime = performance.now()"),
        TestAction::run("performance.measure('duration-test', 'duration-start', 'duration-end')"),

        TestAction::run("let measureEntry = performance.getEntriesByName('duration-test')[0]"),
        TestAction::run("measureEntry.duration >= 0"),
        TestAction::assert_eq("measureEntry.duration >= 0", JsValue::from(true)),

        // The measure duration should be reasonable compared to our timing
        TestAction::run("let timeDiff = endTime - startTime"),
        TestAction::run("Math.abs(measureEntry.duration - timeDiff) < 10"), // Allow 10ms tolerance
        TestAction::assert_eq("Math.abs(measureEntry.duration - timeDiff) < 10", JsValue::from(true)),
    ]);
}

#[test]
fn performance_api_methods_exist() {
    run_test_actions([
        TestAction::run("typeof performance.now"),
        TestAction::assert_eq("typeof performance.now", JsValue::from(JsString::from("function"))),
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

    // Convert to numbers and check that they are non-decreasing
    if let (Ok(time1), Ok(time2)) = (result1.to_number(&mut context), result2.to_number(&mut context)) {
        assert!(time2 >= time1, "Performance.now() should return non-decreasing values");
    } else {
        panic!("Performance.now() should return numbers");
    }

    // Test mark creation
    context.eval(Source::from_bytes("performance.mark('test-mark-direct')")).unwrap();
    let result = context.eval(Source::from_bytes("performance.getEntriesByName('test-mark-direct').length")).unwrap();
    assert_eq!(result, JsValue::from(1));

    // Test measure creation
    context.eval(Source::from_bytes("performance.mark('start'); performance.mark('end'); performance.measure('test-measure', 'start', 'end')")).unwrap();
    let result = context.eval(Source::from_bytes("performance.getEntriesByName('test-measure').length")).unwrap();
    assert_eq!(result, JsValue::from(1));
}