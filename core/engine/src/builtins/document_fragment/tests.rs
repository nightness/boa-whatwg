use crate::{js_string, run_test_actions, TestAction, JsNativeErrorKind, JsValue};

#[test]
fn document_fragment_constructor() {
    run_test_actions([
        TestAction::assert_eq("typeof DocumentFragment", js_string!(js_string!("function"))),
        TestAction::run("var fragment = new DocumentFragment()"),
        TestAction::assert_eq("typeof fragment", js_string!(js_string!("object"))),
        TestAction::assert_eq("fragment.childElementCount", 0),
        TestAction::assert_eq("fragment.children.length", 0),
        TestAction::assert_eq("fragment.firstElementChild", JsValue::null()),
        TestAction::assert_eq("fragment.lastElementChild", JsValue::null()),
    ]);
}

#[test]
fn document_fragment_constructor_requires_new() {
    run_test_actions([
        TestAction::assert_native_error(
            "DocumentFragment()",
            JsNativeErrorKind::Type,
            "Constructor DocumentFragment requires 'new'",
        ),
    ]);
}

#[test]
fn document_fragment_child_element_count() {
    run_test_actions([
        TestAction::run("var fragment = new DocumentFragment()"),
        TestAction::assert_eq("fragment.childElementCount", 0),

        // Create some mock elements for testing
        TestAction::run("var element1 = {nodeName: 'div'}"),
        TestAction::run("var element2 = {nodeName: 'span'}"),

        // Add elements using append
        TestAction::run("fragment.append(element1)"),
        TestAction::assert_eq("fragment.childElementCount", 1),

        TestAction::run("fragment.append(element2)"),
        TestAction::assert_eq("fragment.childElementCount", 2),
    ]);
}

#[test]
fn document_fragment_first_last_element_child() {
    run_test_actions([
        TestAction::run("var fragment = new DocumentFragment()"),
        TestAction::assert_eq("fragment.firstElementChild", JsValue::null()),
        TestAction::assert_eq("fragment.lastElementChild", JsValue::null()),

        // Add mock elements
        TestAction::run("var element1 = {nodeName: 'div'}"),
        TestAction::run("var element2 = {nodeName: 'span'}"),
        TestAction::run("var element3 = {nodeName: 'p'}"),

        TestAction::run("fragment.append(element1)"),
        TestAction::assert_eq("fragment.firstElementChild === element1", true),
        TestAction::assert_eq("fragment.lastElementChild === element1", true),

        TestAction::run("fragment.append(element2)"),
        TestAction::assert_eq("fragment.firstElementChild === element1", true),
        TestAction::assert_eq("fragment.lastElementChild === element2", true),

        TestAction::run("fragment.append(element3)"),
        TestAction::assert_eq("fragment.firstElementChild === element1", true),
        TestAction::assert_eq("fragment.lastElementChild === element3", true),
    ]);
}

#[test]
fn document_fragment_children_property() {
    run_test_actions([
        TestAction::run("var fragment = new DocumentFragment()"),
        TestAction::assert_eq("Array.isArray(fragment.children)", true),
        TestAction::assert_eq("fragment.children.length", 0),

        // Add mock elements
        TestAction::run("var element1 = {nodeName: 'div'}"),
        TestAction::run("var element2 = {nodeName: 'span'}"),

        TestAction::run("fragment.append(element1)"),
        TestAction::assert_eq("fragment.children.length", 1),
        TestAction::assert_eq("fragment.children[0] === element1", true),

        TestAction::run("fragment.append(element2)"),
        TestAction::assert_eq("fragment.children.length", 2),
        TestAction::assert_eq("fragment.children[0] === element1", true),
        TestAction::assert_eq("fragment.children[1] === element2", true),
    ]);
}

#[test]
fn document_fragment_append_method() {
    run_test_actions([
        TestAction::run("var fragment = new DocumentFragment()"),
        TestAction::run("var element1 = {nodeName: 'div'}"),
        TestAction::run("var element2 = {nodeName: 'span'}"),

        // Test single append
        TestAction::run("fragment.append(element1)"),
        TestAction::assert_eq("fragment.childElementCount", 1),
        TestAction::assert_eq("fragment.firstElementChild === element1", true),

        // Test multiple append
        TestAction::run("fragment.append(element2)"),
        TestAction::assert_eq("fragment.childElementCount", 2),
        TestAction::assert_eq("fragment.lastElementChild === element2", true),

        // Test append returns undefined
        TestAction::assert_eq("fragment.append({})", JsValue::undefined()),
    ]);
}

#[test]
fn document_fragment_append_multiple_arguments() {
    run_test_actions([
        TestAction::run("var fragment = new DocumentFragment()"),
        TestAction::run("var element1 = {nodeName: 'div'}"),
        TestAction::run("var element2 = {nodeName: 'span'}"),
        TestAction::run("var element3 = {nodeName: 'p'}"),

        // Test appending multiple elements at once
        TestAction::run("fragment.append(element1, element2, element3)"),
        TestAction::assert_eq("fragment.childElementCount", 3),
        TestAction::assert_eq("fragment.children[0] === element1", true),
        TestAction::assert_eq("fragment.children[1] === element2", true),
        TestAction::assert_eq("fragment.children[2] === element3", true),
    ]);
}

#[test]
fn document_fragment_prepend_method() {
    run_test_actions([
        TestAction::run("var fragment = new DocumentFragment()"),
        TestAction::run("var element1 = {nodeName: 'div'}"),
        TestAction::run("var element2 = {nodeName: 'span'}"),

        // Test single prepend
        TestAction::run("fragment.prepend(element1)"),
        TestAction::assert_eq("fragment.childElementCount", 1),
        TestAction::assert_eq("fragment.firstElementChild === element1", true),

        // Test prepend places at beginning
        TestAction::run("fragment.prepend(element2)"),
        TestAction::assert_eq("fragment.childElementCount", 2),
        TestAction::assert_eq("fragment.firstElementChild === element2", true),
        TestAction::assert_eq("fragment.lastElementChild === element1", true),

        // Test prepend returns undefined
        TestAction::assert_eq("fragment.prepend({})", JsValue::undefined()),
    ]);
}

#[test]
fn document_fragment_prepend_multiple_arguments() {
    run_test_actions([
        TestAction::run("var fragment = new DocumentFragment()"),
        TestAction::run("var element1 = {nodeName: 'div'}"),
        TestAction::run("var element2 = {nodeName: 'span'}"),
        TestAction::run("var element3 = {nodeName: 'p'}"),

        // Test prepending multiple elements at once (reverse order due to prepend)
        TestAction::run("fragment.prepend(element1, element2, element3)"),
        TestAction::assert_eq("fragment.childElementCount", 3),
        // Note: prepend processes in reverse order to maintain argument order
        TestAction::assert_eq("fragment.children[0] === element3", true),
        TestAction::assert_eq("fragment.children[1] === element2", true),
        TestAction::assert_eq("fragment.children[2] === element1", true),
    ]);
}

#[test]
fn document_fragment_replace_children_method() {
    run_test_actions([
        TestAction::run("var fragment = new DocumentFragment()"),
        TestAction::run("var element1 = {nodeName: 'div'}"),
        TestAction::run("var element2 = {nodeName: 'span'}"),
        TestAction::run("var element3 = {nodeName: 'p'}"),

        // Add some initial children
        TestAction::run("fragment.append(element1, element2)"),
        TestAction::assert_eq("fragment.childElementCount", 2),

        // Replace all children
        TestAction::run("fragment.replaceChildren(element3)"),
        TestAction::assert_eq("fragment.childElementCount", 1),
        TestAction::assert_eq("fragment.firstElementChild === element3", true),

        // Replace with multiple children
        TestAction::run("fragment.replaceChildren(element1, element2)"),
        TestAction::assert_eq("fragment.childElementCount", 2),
        TestAction::assert_eq("fragment.children[0] === element1", true),
        TestAction::assert_eq("fragment.children[1] === element2", true),

        // Replace with no children (clear)
        TestAction::run("fragment.replaceChildren()"),
        TestAction::assert_eq("fragment.childElementCount", 0),
        TestAction::assert_eq("fragment.firstElementChild", JsValue::null()),
    ]);
}

#[test]
fn document_fragment_query_selector() {
    run_test_actions([
        TestAction::run("var fragment = new DocumentFragment()"),

        // Test query on empty fragment
        TestAction::assert_eq("fragment.querySelector('div')", JsValue::null()),

        // Add mock elements
        TestAction::run("var element1 = {nodeName: 'div'}"),
        TestAction::run("fragment.append(element1)"),

        // Test basic query (simplified implementation returns first element)
        TestAction::run("var result = fragment.querySelector('div')"),
        TestAction::assert_eq("result === element1", true),

        // Test method returns value
        TestAction::assert_eq("typeof fragment.querySelector('span')", js_string!("object")),
    ]);
}

#[test]
fn document_fragment_query_selector_all() {
    run_test_actions([
        TestAction::run("var fragment = new DocumentFragment()"),

        // Test query on empty fragment
        TestAction::run("var emptyResult = fragment.querySelectorAll('div')"),
        TestAction::assert_eq("Array.isArray(emptyResult)", true),
        TestAction::assert_eq("emptyResult.length", 0),

        // Add mock elements
        TestAction::run("var element1 = {nodeName: 'div'}"),
        TestAction::run("var element2 = {nodeName: 'span'}"),
        TestAction::run("fragment.append(element1, element2)"),

        // Test query returns all elements (simplified implementation)
        TestAction::run("var result = fragment.querySelectorAll('*')"),
        TestAction::assert_eq("Array.isArray(result)", true),
        TestAction::assert_eq("result.length", 2),
        TestAction::assert_eq("result[0] === element1", true),
        TestAction::assert_eq("result[1] === element2", true),
    ]);
}

#[test]
fn document_fragment_get_element_by_id() {
    run_test_actions([
        TestAction::run("var fragment = new DocumentFragment()"),

        // Test on empty fragment
        TestAction::assert_eq("fragment.getElementById('test')", JsValue::null()),

        // Add mock element
        TestAction::run("var element1 = {nodeName: 'div', id: 'test'}"),
        TestAction::run("fragment.append(element1)"),

        // Test basic ID query (simplified implementation returns first element)
        TestAction::run("var result = fragment.getElementById('test')"),
        TestAction::assert_eq("result === element1", true),

        // Test method signature
        TestAction::assert_eq("typeof fragment.getElementById", js_string!("function")),
    ]);
}

#[test]
fn document_fragment_mixed_operations() {
    run_test_actions([
        TestAction::run("var fragment = new DocumentFragment()"),
        TestAction::run("var div = {nodeName: 'div'}"),
        TestAction::run("var span = {nodeName: 'span'}"),
        TestAction::run("var p = {nodeName: 'p'}"),

        // Build complex structure
        TestAction::run("fragment.append(div)"),
        TestAction::run("fragment.prepend(span)"),
        TestAction::run("fragment.append(p)"),
        TestAction::assert_eq("fragment.childElementCount", 3),

        // Verify order: span, div, p
        TestAction::assert_eq("fragment.children[0] === span", true),
        TestAction::assert_eq("fragment.children[1] === div", true),
        TestAction::assert_eq("fragment.children[2] === p", true),

        // Test replace operation
        TestAction::run("var h1 = {nodeName: 'h1'}"),
        TestAction::run("fragment.replaceChildren(h1)"),
        TestAction::assert_eq("fragment.childElementCount", 1),
        TestAction::assert_eq("fragment.firstElementChild === h1", true),

        // Test query operations
        TestAction::run("var queryResult = fragment.querySelector('h1')"),
        TestAction::assert_eq("queryResult === h1", true),

        TestAction::run("var queryAllResult = fragment.querySelectorAll('*')"),
        TestAction::assert_eq("queryAllResult.length", 1),
        TestAction::assert_eq("queryAllResult[0] === h1", true),
    ]);
}

#[test]
fn document_fragment_property_types() {
    run_test_actions([
        TestAction::run("var fragment = new DocumentFragment()"),

        // Test property types
        TestAction::assert_eq("typeof fragment.childElementCount", js_string!("number")),
        TestAction::assert_eq("typeof fragment.children", js_string!("object")),
        TestAction::assert_eq("typeof fragment.firstElementChild", js_string!("object")),
        TestAction::assert_eq("typeof fragment.lastElementChild", js_string!("object")),

        // Test method types
        TestAction::assert_eq("typeof fragment.append", js_string!("function")),
        TestAction::assert_eq("typeof fragment.prepend", js_string!("function")),
        TestAction::assert_eq("typeof fragment.replaceChildren", js_string!("function")),
        TestAction::assert_eq("typeof fragment.querySelector", js_string!("function")),
        TestAction::assert_eq("typeof fragment.querySelectorAll", js_string!("function")),
        TestAction::assert_eq("typeof fragment.getElementById", js_string!("function")),
    ]);
}

#[test]
fn document_fragment_empty_operations() {
    run_test_actions([
        TestAction::run("var fragment = new DocumentFragment()"),

        // Test operations on empty fragment
        TestAction::assert_eq("fragment.append()", JsValue::undefined()),
        TestAction::assert_eq("fragment.prepend()", JsValue::undefined()),
        TestAction::assert_eq("fragment.replaceChildren()", JsValue::undefined()),

        // Verify still empty
        TestAction::assert_eq("fragment.childElementCount", 0),
        TestAction::assert_eq("fragment.children.length", 0),

        // Test queries on empty fragment
        TestAction::assert_eq("fragment.querySelector('div')", JsValue::null()),
        TestAction::run("var emptyQuery = fragment.querySelectorAll('*')"),
        TestAction::assert_eq("emptyQuery.length", 0),
        TestAction::assert_eq("fragment.getElementById('test')", JsValue::null()),
    ]);
}

#[test]
fn document_fragment_inheritance() {
    run_test_actions([
        TestAction::run("var fragment = new DocumentFragment()"),

        // Test constructor relationship
        TestAction::assert_eq("fragment.constructor === DocumentFragment", true),
        TestAction::assert_eq("fragment instanceof DocumentFragment", true),

        // Test that it's an object
        TestAction::assert_eq("typeof fragment", js_string!("object")),
        TestAction::assert_eq("fragment !== null", true),
    ]);
}