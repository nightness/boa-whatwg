//! ProcessingInstruction interface unit tests
//! Tests for DOM Level 4 ProcessingInstruction implementation

use crate::{js_string, run_test_actions, TestAction, JsValue, JsNativeErrorKind};

#[test]
fn processing_instruction_constructor() {
    run_test_actions([
        TestAction::assert_eq("typeof ProcessingInstruction", js_string!("function")),
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'href=\"style.css\"')"),
        TestAction::assert_eq("typeof pi", js_string!("object")),
        TestAction::assert_eq("pi !== null", true),
    ]);
}

#[test]
fn processing_instruction_constructor_target_required() {
    run_test_actions([
        TestAction::assert_native_error(
            "new ProcessingInstruction()",
            JsNativeErrorKind::Type,
            "ProcessingInstruction constructor requires target parameter",
        ),
    ]);
}

#[test]
fn processing_instruction_constructor_default_data() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet')"),
        TestAction::assert_eq("pi.data", js_string!("")),
    ]);
}

#[test]
fn processing_instruction_target_property() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'href=\"style.css\"')"),
        TestAction::assert_eq("pi.target", js_string!("xml-stylesheet")),
    ]);
}

#[test]
fn processing_instruction_target_readonly() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'href=\"style.css\"')"),
        TestAction::run("pi.target = 'new-target'"),
        TestAction::assert_eq("pi.target", js_string!("xml-stylesheet")), // Should remain unchanged
    ]);
}

#[test]
fn processing_instruction_data_property() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'href=\"style.css\"')"),
        TestAction::assert_eq("pi.data", js_string!("href=\"style.css\"")),
    ]);
}

#[test]
fn processing_instruction_data_setter() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'href=\"style.css\"')"),
        TestAction::run("pi.data = 'href=\"new.css\"'"),
        TestAction::assert_eq("pi.data", js_string!("href=\"new.css\"")),
    ]);
}

#[test]
fn processing_instruction_length_property() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'href=\"style.css\"')"),
        TestAction::assert_eq("pi.length", 16), // Length of "href=\"style.css\""
    ]);
}

#[test]
fn processing_instruction_length_updates() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'test')"),
        TestAction::assert_eq("pi.length", 4),
        TestAction::run("pi.data = 'longer test data'"),
        TestAction::assert_eq("pi.length", 16),
    ]);
}

#[test]
fn processing_instruction_substring_data() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'href=\"style.css\"')"),
        TestAction::assert_eq("pi.substringData(5, 5)", js_string!("\"styl")),
    ]);
}

#[test]
fn processing_instruction_substring_data_bounds() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'test')"),
        TestAction::assert_native_error(
            "pi.substringData(10, 5)",
            JsNativeErrorKind::Range,
            "INDEX_SIZE_ERR: offset exceeds data length",
        ),
    ]);
}

#[test]
fn processing_instruction_append_data() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'href=\"style.css\"')"),
        TestAction::run("pi.appendData(' type=\"text/css\"')"),
        TestAction::assert_eq("pi.data", js_string!("href=\"style.css\" type=\"text/css\"")),
    ]);
}

#[test]
fn processing_instruction_insert_data() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'href=\"style.css\"')"),
        TestAction::run("pi.insertData(5, 'XXX')"),
        TestAction::assert_eq("pi.data", js_string!("href=XXX\"style.css\"")),
    ]);
}

#[test]
fn processing_instruction_insert_data_bounds() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'test')"),
        TestAction::assert_native_error(
            "pi.insertData(10, 'XXX')",
            JsNativeErrorKind::Range,
            "INDEX_SIZE_ERR: offset exceeds data length",
        ),
    ]);
}

#[test]
fn processing_instruction_delete_data() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'href=\"style.css\"')"),
        TestAction::run("pi.deleteData(5, 6)"),
        TestAction::assert_eq("pi.data", js_string!("href=.css\"")),
    ]);
}

#[test]
fn processing_instruction_delete_data_bounds() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'test')"),
        TestAction::assert_native_error(
            "pi.deleteData(10, 5)",
            JsNativeErrorKind::Range,
            "INDEX_SIZE_ERR: offset exceeds data length",
        ),
    ]);
}

#[test]
fn processing_instruction_replace_data() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'href=\"style.css\"')"),
        TestAction::run("pi.replaceData(5, 6, '\"new\"')"),
        TestAction::assert_eq("pi.data", js_string!("href=\"new\".css\"")),
    ]);
}

#[test]
fn processing_instruction_replace_data_bounds() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'test')"),
        TestAction::assert_native_error(
            "pi.replaceData(10, 5, 'XXX')",
            JsNativeErrorKind::Range,
            "INDEX_SIZE_ERR: offset exceeds data length",
        ),
    ]);
}

#[test]
fn processing_instruction_unicode_support() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', '测试数据🚀')"),
        TestAction::assert_eq("pi.length", 5), // 4 Chinese chars + 1 emoji
        TestAction::assert_eq("pi.substringData(0, 2)", js_string!("测试")),
    ]);
}

#[test]
fn processing_instruction_inheritance() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'href=\"style.css\"')"),

        // Test constructor relationship
        TestAction::assert_eq("pi.constructor === ProcessingInstruction", true),
        TestAction::assert_eq("pi instanceof ProcessingInstruction", true),

        // Test property existence and descriptors
        TestAction::assert_eq("'target' in pi", true),
        TestAction::assert_eq("'data' in pi", true),
        TestAction::assert_eq("'length' in pi", true),

        // Test methods exist
        TestAction::assert_eq("typeof pi.substringData", js_string!("function")),
        TestAction::assert_eq("typeof pi.appendData", js_string!("function")),
        TestAction::assert_eq("typeof pi.insertData", js_string!("function")),
        TestAction::assert_eq("typeof pi.deleteData", js_string!("function")),
        TestAction::assert_eq("typeof pi.replaceData", js_string!("function")),
    ]);
}

#[test]
fn processing_instruction_property_descriptors() {
    run_test_actions([
        // Test target property descriptor (should be read-only)
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(ProcessingInstruction.prototype, 'target').get !== undefined",
            true
        ),
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(ProcessingInstruction.prototype, 'target').set === undefined",
            true
        ),
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(ProcessingInstruction.prototype, 'target').enumerable",
            true
        ),
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(ProcessingInstruction.prototype, 'target').configurable",
            true
        ),

        // Test data property descriptor (should have both getter and setter)
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(ProcessingInstruction.prototype, 'data').get !== undefined",
            true
        ),
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(ProcessingInstruction.prototype, 'data').set !== undefined",
            true
        ),
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(ProcessingInstruction.prototype, 'data').enumerable",
            true
        ),
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(ProcessingInstruction.prototype, 'data').configurable",
            true
        ),

        // Test length property descriptor (should be read-only)
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(ProcessingInstruction.prototype, 'length').get !== undefined",
            true
        ),
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(ProcessingInstruction.prototype, 'length').set === undefined",
            true
        ),
    ]);
}

#[test]
fn processing_instruction_property_enumeration() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'href=\"style.css\"')"),
        TestAction::run(r#"
            var props = [];
            for (var prop in pi) {
                if (prop === 'target' || prop === 'data' || prop === 'length') {
                    props.push(prop);
                }
            }
            props.sort();
        "#),
        TestAction::assert_eq("props.length >= 3", true), // Should find all three properties
    ]);
}

#[test]
fn processing_instruction_method_binding() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'test')"),

        // Should work when called on the pi object
        TestAction::run("pi.appendData('XXX')"),
        TestAction::assert_eq("pi.data", js_string!("testXXX")),

        // Should fail when called on wrong object type
        TestAction::run("var obj = {}"),
        TestAction::assert_native_error(
            "pi.appendData.call(obj, 'XXX')",
            JsNativeErrorKind::Type,
            "ProcessingInstruction.prototype.appendData called on non-ProcessingInstruction object",
        ),
    ]);
}

#[test]
fn processing_instruction_empty_target_edge_case() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('', 'data')"),
        TestAction::assert_eq("pi.target", js_string!("")),
        TestAction::assert_eq("pi.data", js_string!("data")),
    ]);
}

#[test]
fn processing_instruction_special_characters() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'href=\"file with spaces.css\"')"),
        TestAction::assert_eq("pi.data", js_string!("href=\"file with spaces.css\"")),
        TestAction::assert_eq("pi.length", 27),
    ]);
}

#[test]
fn processing_instruction_large_data_manipulation() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('xml-stylesheet', 'a'.repeat(1000))"),
        TestAction::assert_eq("pi.length", 1000),
        TestAction::run("pi.insertData(500, 'INSERTED')"),
        TestAction::assert_eq("pi.length", 1008),
        TestAction::assert_eq("pi.substringData(498, 12)", js_string!("aaINSERTEDaa")),
    ]);
}

#[test]
fn processing_instruction_zero_length_operations() {
    run_test_actions([
        TestAction::run("var pi = new ProcessingInstruction('target', 'test')"),

        // Zero-length substring should work
        TestAction::assert_eq("pi.substringData(0, 0)", js_string!("")),
        TestAction::assert_eq("pi.substringData(2, 0)", js_string!("")),

        // Zero-length delete should be no-op
        TestAction::run("pi.deleteData(2, 0)"),
        TestAction::assert_eq("pi.data", js_string!("test")),

        // Zero-length replace should be insert
        TestAction::run("pi.replaceData(2, 0, 'INSERT')"),
        TestAction::assert_eq("pi.data", js_string!("teINSERTst")),
    ]);
}