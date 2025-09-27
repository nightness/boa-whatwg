//! CDATASection interface unit tests
//! Tests for DOM Level 4 CDATASection implementation

use crate::{js_string, run_test_actions, TestAction, JsValue, JsNativeErrorKind};

#[test]
fn cdata_section_constructor() {
    run_test_actions([
        TestAction::assert_eq("typeof CDATASection", js_string!("function")),
        TestAction::run("var cdata = new CDATASection('Hello CDATA')"),
        TestAction::assert_eq("typeof cdata", js_string!("object")),
        TestAction::assert_eq("cdata !== null", true),
    ]);
}

#[test]
fn cdata_section_constructor_default_data() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection()"),
        TestAction::assert_eq("cdata.data", js_string!("")),
    ]);
}

#[test]
fn cdata_section_data_property() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('Hello CDATA World')"),
        TestAction::assert_eq("cdata.data", js_string!("Hello CDATA World")),
    ]);
}

#[test]
fn cdata_section_data_setter() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('Hello CDATA')"),
        TestAction::run("cdata.data = 'Modified CDATA'"),
        TestAction::assert_eq("cdata.data", js_string!("Modified CDATA")),
    ]);
}

#[test]
fn cdata_section_length_property() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('Hello CDATA')"),
        TestAction::assert_eq("cdata.length", 11), // Length of "Hello CDATA"
    ]);
}

#[test]
fn cdata_section_length_updates() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('test')"),
        TestAction::assert_eq("cdata.length", 4),
        TestAction::run("cdata.data = 'longer test data'"),
        TestAction::assert_eq("cdata.length", 16),
    ]);
}

#[test]
fn cdata_section_substring_data() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('Hello CDATA World')"),
        TestAction::assert_eq("cdata.substringData(0, 5)", js_string!("Hello")),
        TestAction::assert_eq("cdata.substringData(6, 5)", js_string!("CDATA")),
        TestAction::assert_eq("cdata.substringData(12, 5)", js_string!("World")),
    ]);
}

#[test]
fn cdata_section_substring_data_bounds() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('test')"),
        TestAction::assert_native_error(
            "cdata.substringData(10, 5)",
            JsNativeErrorKind::Range,
            "INDEX_SIZE_ERR: offset exceeds data length",
        ),
    ]);
}

#[test]
fn cdata_section_append_data() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('Hello')"),
        TestAction::run("cdata.appendData(' CDATA')"),
        TestAction::assert_eq("cdata.data", js_string!("Hello CDATA")),
        TestAction::assert_eq("cdata.length", 11),
    ]);
}

#[test]
fn cdata_section_insert_data() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('Hello World')"),
        TestAction::run("cdata.insertData(6, 'CDATA ')"),
        TestAction::assert_eq("cdata.data", js_string!("Hello CDATA World")),
    ]);
}

#[test]
fn cdata_section_insert_data_bounds() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('test')"),
        TestAction::assert_native_error(
            "cdata.insertData(10, 'XXX')",
            JsNativeErrorKind::Range,
            "INDEX_SIZE_ERR: offset exceeds data length",
        ),
    ]);
}

#[test]
fn cdata_section_delete_data() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('Hello CDATA World')"),
        TestAction::run("cdata.deleteData(6, 6)"), // Remove "CDATA "
        TestAction::assert_eq("cdata.data", js_string!("Hello World")),
    ]);
}

#[test]
fn cdata_section_delete_data_bounds() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('test')"),
        TestAction::assert_native_error(
            "cdata.deleteData(10, 5)",
            JsNativeErrorKind::Range,
            "INDEX_SIZE_ERR: offset exceeds data length",
        ),
    ]);
}

#[test]
fn cdata_section_replace_data() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('Hello CDATA World')"),
        TestAction::run("cdata.replaceData(6, 5, 'XML')"), // Replace "CDATA" with "XML"
        TestAction::assert_eq("cdata.data", js_string!("Hello XML World")),
    ]);
}

#[test]
fn cdata_section_replace_data_bounds() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('test')"),
        TestAction::assert_native_error(
            "cdata.replaceData(10, 5, 'XXX')",
            JsNativeErrorKind::Range,
            "INDEX_SIZE_ERR: offset exceeds data length",
        ),
    ]);
}

#[test]
fn cdata_section_unicode_support() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('测试CDATA🚀')"),
        TestAction::assert_eq("cdata.length", 8), // 2 Chinese chars + 5 "CDATA" + 1 emoji
        TestAction::assert_eq("cdata.substringData(0, 2)", js_string!("测试")),
        TestAction::assert_eq("cdata.substringData(6, 2)", js_string!("A🚀")),
    ]);
}

#[test]
fn cdata_section_inheritance() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('Hello CDATA')"),

        // Test constructor relationship
        TestAction::assert_eq("cdata.constructor === CDATASection", true),
        TestAction::assert_eq("cdata instanceof CDATASection", true),

        // Test property existence
        TestAction::assert_eq("'data' in cdata", true),
        TestAction::assert_eq("'length' in cdata", true),

        // Test methods exist
        TestAction::assert_eq("typeof cdata.substringData", js_string!("function")),
        TestAction::assert_eq("typeof cdata.appendData", js_string!("function")),
        TestAction::assert_eq("typeof cdata.insertData", js_string!("function")),
        TestAction::assert_eq("typeof cdata.deleteData", js_string!("function")),
        TestAction::assert_eq("typeof cdata.replaceData", js_string!("function")),
    ]);
}

#[test]
fn cdata_section_property_descriptors() {
    run_test_actions([
        // Test data property descriptor (should have both getter and setter)
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(CDATASection.prototype, 'data').get !== undefined",
            true
        ),
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(CDATASection.prototype, 'data').set !== undefined",
            true
        ),
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(CDATASection.prototype, 'data').enumerable",
            true
        ),
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(CDATASection.prototype, 'data').configurable",
            true
        ),

        // Test length property descriptor (should be read-only)
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(CDATASection.prototype, 'length').get !== undefined",
            true
        ),
        TestAction::assert_eq(
            "Object.getOwnPropertyDescriptor(CDATASection.prototype, 'length').set === undefined",
            true
        ),
    ]);
}

#[test]
fn cdata_section_property_enumeration() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('Hello CDATA')"),
        TestAction::run(r#"
            var props = [];
            for (var prop in cdata) {
                if (prop === 'data' || prop === 'length') {
                    props.push(prop);
                }
            }
            props.sort();
        "#),
        TestAction::assert_eq("props.length >= 2", true), // Should find both properties
    ]);
}

#[test]
fn cdata_section_method_binding() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('test')"),

        // Should work when called on the cdata object
        TestAction::run("cdata.appendData('XXX')"),
        TestAction::assert_eq("cdata.data", js_string!("testXXX")),

        // Should fail when called on wrong object type
        TestAction::run("var obj = {}"),
        TestAction::assert_native_error(
            "cdata.appendData.call(obj, 'XXX')",
            JsNativeErrorKind::Type,
            "CDATASection.prototype.appendData called on non-CDATASection object",
        ),
    ]);
}

#[test]
fn cdata_section_empty_data_edge_case() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('')"),
        TestAction::assert_eq("cdata.data", js_string!("")),
        TestAction::assert_eq("cdata.length", 0),
        TestAction::run("cdata.appendData('test')"),
        TestAction::assert_eq("cdata.data", js_string!("test")),
    ]);
}

#[test]
fn cdata_section_xml_characters() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('<![CDATA[This is CDATA content]]>')"),
        TestAction::assert_eq("cdata.data", js_string!("<![CDATA[This is CDATA content]]>")),
        TestAction::assert_eq("cdata.length", 33),
    ]);
}

#[test]
fn cdata_section_large_data_manipulation() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('x'.repeat(1000))"),
        TestAction::assert_eq("cdata.length", 1000),
        TestAction::run("cdata.insertData(500, 'INSERTED')"),
        TestAction::assert_eq("cdata.length", 1008),
        TestAction::assert_eq("cdata.substringData(498, 12)", js_string!("xxINSERTEDxx")),
    ]);
}

#[test]
fn cdata_section_zero_length_operations() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('test')"),

        // Zero-length substring should work
        TestAction::assert_eq("cdata.substringData(0, 0)", js_string!("")),
        TestAction::assert_eq("cdata.substringData(2, 0)", js_string!("")),

        // Zero-length delete should be no-op
        TestAction::run("cdata.deleteData(2, 0)"),
        TestAction::assert_eq("cdata.data", js_string!("test")),

        // Zero-length replace should be insert
        TestAction::run("cdata.replaceData(2, 0, 'INSERT')"),
        TestAction::assert_eq("cdata.data", js_string!("teINSERTst")),
    ]);
}

#[test]
fn cdata_section_special_xml_content() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('< > & \" \\' <tag>content</tag>')"),
        TestAction::assert_eq("cdata.data", js_string!("< > & \" ' <tag>content</tag>")),
        TestAction::assert_eq("cdata.length", 28),
    ]);
}

#[test]
fn cdata_section_newline_handling() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('line1\\nline2\\r\\nline3')"),
        TestAction::assert_eq("cdata.length", 18),
        TestAction::assert_eq("cdata.substringData(0, 5)", js_string!("line1")),
        TestAction::assert_eq("cdata.substringData(6, 5)", js_string!("line2")),
    ]);
}

#[test]
fn cdata_section_boundary_conditions() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('test')"),

        // Insert at start
        TestAction::run("cdata.insertData(0, 'START')"),
        TestAction::assert_eq("cdata.data", js_string!("STARTtest")),

        // Insert at end
        TestAction::run("cdata.insertData(9, 'END')"),
        TestAction::assert_eq("cdata.data", js_string!("STARTtestEND")),

        // Delete from start
        TestAction::run("cdata.deleteData(0, 5)"),
        TestAction::assert_eq("cdata.data", js_string!("testEND")),

        // Delete to end
        TestAction::run("cdata.deleteData(4, 10)"),
        TestAction::assert_eq("cdata.data", js_string!("test")),
    ]);
}

#[test]
fn cdata_section_method_return_values() {
    run_test_actions([
        TestAction::run("var cdata = new CDATASection('test')"),

        // Test that methods return undefined
        TestAction::assert_eq("cdata.appendData('X')", JsValue::undefined()),
        TestAction::assert_eq("cdata.insertData(0, 'Y')", JsValue::undefined()),
        TestAction::assert_eq("cdata.deleteData(0, 1)", JsValue::undefined()),
        TestAction::assert_eq("cdata.replaceData(0, 1, 'Z')", JsValue::undefined()),

        // substringData should return string
        TestAction::assert_eq("typeof cdata.substringData(0, 1)", js_string!("string")),
    ]);
}