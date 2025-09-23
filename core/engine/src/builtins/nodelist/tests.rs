use crate::{js_string, run_test_actions, TestAction, JsNativeErrorKind, JsValue};

#[test]
fn nodelist_constructor() {
    run_test_actions([
        TestAction::assert_eq("typeof NodeList", js_string!("function")),
        TestAction::run("var list = new NodeList()"),
        TestAction::assert_eq("typeof list", js_string!("object")),
        TestAction::assert_eq("list.length", 0),
        TestAction::assert_eq("list.item(0)", JsValue::null()),
    ]);
}

#[test]
fn nodelist_constructor_requires_new() {
    run_test_actions([
        TestAction::assert_native_error(
            "NodeList()",
            JsNativeErrorKind::Type,
            "Constructor NodeList requires 'new'",
        ),
    ]);
}

#[test]
fn nodelist_length_property() {
    run_test_actions([
        TestAction::run("var list = new NodeList()"),
        TestAction::assert_eq("list.length", 0),
        TestAction::assert_eq("typeof list.length", js_string!("number")),
    ]);
}

#[test]
fn nodelist_item_method() {
    run_test_actions([
        TestAction::run("var list = new NodeList()"),

        // Test with empty list
        TestAction::assert_eq("list.item(0)", JsValue::null()),
        TestAction::assert_eq("list.item(1)", JsValue::null()),
        TestAction::assert_eq("list.item(-1)", JsValue::null()),

        // Test method exists
        TestAction::assert_eq("typeof list.item", js_string!("function")),
    ]);
}

#[test]
fn nodelist_for_each_method() {
    run_test_actions([
        TestAction::run("var list = new NodeList()"),
        TestAction::run("var callCount = 0"),
        TestAction::run("var callback = function() { callCount++; }"),

        // Test forEach on empty list
        TestAction::run("list.forEach(callback)"),
        TestAction::assert_eq("callCount", 0),

        // Test method exists
        TestAction::assert_eq("typeof list.forEach", js_string!("function")),
    ]);
}

#[test]
fn nodelist_for_each_requires_callback() {
    run_test_actions([
        TestAction::run("var list = new NodeList()"),
        TestAction::assert_native_error(
            "list.forEach()",
            JsNativeErrorKind::Type,
            "NodeList.forEach callback is not callable",
        ),
        TestAction::assert_native_error(
            "list.forEach(null)",
            JsNativeErrorKind::Type,
            "NodeList.forEach callback is not callable",
        ),
        TestAction::assert_native_error(
            "list.forEach('not a function')",
            JsNativeErrorKind::Type,
            "NodeList.forEach callback is not callable",
        ),
    ]);
}

#[test]
fn nodelist_keys_method() {
    run_test_actions([
        TestAction::run("var list = new NodeList()"),

        // Test keys on empty list
        TestAction::run("var keys = list.keys()"),
        TestAction::assert_eq("Array.isArray(keys)", true),
        TestAction::assert_eq("keys.length", 0),

        // Test method exists
        TestAction::assert_eq("typeof list.keys", js_string!("function")),
    ]);
}

#[test]
fn nodelist_values_method() {
    run_test_actions([
        TestAction::run("var list = new NodeList()"),

        // Test values on empty list
        TestAction::run("var values = list.values()"),
        TestAction::assert_eq("Array.isArray(values)", true),
        TestAction::assert_eq("values.length", 0),

        // Test method exists
        TestAction::assert_eq("typeof list.values", js_string!("function")),
    ]);
}

#[test]
fn nodelist_entries_method() {
    run_test_actions([
        TestAction::run("var list = new NodeList()"),

        // Test entries on empty list
        TestAction::run("var entries = list.entries()"),
        TestAction::assert_eq("Array.isArray(entries)", true),
        TestAction::assert_eq("entries.length", 0),

        // Test method exists
        TestAction::assert_eq("typeof list.entries", js_string!("function")),
    ]);
}

#[test]
fn nodelist_inheritance() {
    run_test_actions([
        TestAction::run("var list = new NodeList()"),

        // Test constructor relationship
        TestAction::assert_eq("list.constructor === NodeList", true),
        TestAction::assert_eq("list instanceof NodeList", true),

        // Test that it's an object
        TestAction::assert_eq("typeof list", js_string!("object")),
        TestAction::assert_eq("list !== null", true),
    ]);
}

#[test]
fn nodelist_property_types() {
    run_test_actions([
        TestAction::run("var list = new NodeList()"),

        // Test property types
        TestAction::assert_eq("typeof list.length", js_string!("number")),

        // Test method types
        TestAction::assert_eq("typeof list.item", js_string!("function")),
        TestAction::assert_eq("typeof list.forEach", js_string!("function")),
        TestAction::assert_eq("typeof list.keys", js_string!("function")),
        TestAction::assert_eq("typeof list.values", js_string!("function")),
        TestAction::assert_eq("typeof list.entries", js_string!("function")),
    ]);
}

#[test]
fn nodelist_empty_operations() {
    run_test_actions([
        TestAction::run("var list = new NodeList()"),

        // Test operations on empty list
        TestAction::assert_eq("list.length", 0),
        TestAction::assert_eq("list.item(0)", JsValue::null()),
        TestAction::assert_eq("list.item(999)", JsValue::null()),

        // Test iterators on empty list
        TestAction::run("var keys = list.keys()"),
        TestAction::assert_eq("keys.length", 0),

        TestAction::run("var values = list.values()"),
        TestAction::assert_eq("values.length", 0),

        TestAction::run("var entries = list.entries()"),
        TestAction::assert_eq("entries.length", 0),
    ]);
}

#[test]
fn nodelist_method_return_values() {
    run_test_actions([
        TestAction::run("var list = new NodeList()"),
        TestAction::run("var callback = function() {}"),

        // Test method return values
        TestAction::assert_eq("list.forEach(callback)", JsValue::undefined()),
        TestAction::assert_eq("typeof list.keys()", js_string!("object")),
        TestAction::assert_eq("typeof list.values()", js_string!("object")),
        TestAction::assert_eq("typeof list.entries()", js_string!("object")),
    ]);
}

// Note: Tests for NodeList with actual nodes would require Node objects
// These tests focus on the empty NodeList behavior and interface compliance
#[test]
fn nodelist_interface_compliance() {
    run_test_actions([
        TestAction::run("var list = new NodeList()"),

        // Test all required interface members exist
        TestAction::assert_eq("'length' in list", true),
        TestAction::assert_eq("'item' in list", true),
        TestAction::assert_eq("'forEach' in list", true),
        TestAction::assert_eq("'keys' in list", true),
        TestAction::assert_eq("'values' in list", true),
        TestAction::assert_eq("'entries' in list", true),

        // Test length property is present (but typically not enumerable like real DOM)
        TestAction::run("var keys = Object.keys(list)"),
        TestAction::assert_eq("keys.length", 0), // DOM properties are typically non-enumerable
    ]);
}

#[test]
fn nodelist_edge_cases() {
    run_test_actions([
        TestAction::run("var list = new NodeList()"),

        // Test edge case indices for item()
        TestAction::assert_eq("list.item(0)", JsValue::null()),
        TestAction::assert_eq("list.item(-1)", JsValue::null()),
        TestAction::assert_eq("list.item(Number.MAX_SAFE_INTEGER)", JsValue::null()),
        TestAction::assert_eq("list.item(Infinity)", JsValue::null()),
        TestAction::assert_eq("list.item(-Infinity)", JsValue::null()),
        TestAction::assert_eq("list.item(NaN)", JsValue::null()),

        // Test forEach with different callback patterns
        TestAction::run("var results = []"),
        TestAction::run("list.forEach(function(node, index, list) { results.push([node, index, list]); })"),
        TestAction::assert_eq("results.length", 0),
    ]);
}

#[test]
fn nodelist_foreach_context() {
    run_test_actions([
        TestAction::run("var list = new NodeList()"),
        TestAction::run("var context = { test: 'value' }"),
        TestAction::run("var capturedThis"),

        // Test forEach thisArg parameter
        TestAction::run("list.forEach(function() { capturedThis = this; }, context)"),
        // With empty list, callback won't be called, so capturedThis should remain undefined
        TestAction::assert_eq("capturedThis", JsValue::undefined()),
    ]);
}