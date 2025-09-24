use crate::{js_string, run_test_actions, TestAction, JsValue};

#[test]
fn comment_constructor() {
    run_test_actions([
        TestAction::assert_eq("typeof Comment", js_string!("function")),
        TestAction::run("var comment = new Comment()"),
        TestAction::assert_eq("typeof comment", js_string!("object")),
        TestAction::assert_eq("comment !== null", true),
    ]);
}

#[test]
fn comment_constructor_with_data() {
    run_test_actions([
        TestAction::run("var comment = new Comment('Hello World')"),
        TestAction::assert_eq("comment.data", js_string!("Hello World")),
        TestAction::assert_eq("comment.length", 11),
    ]);
}

#[test]
fn comment_inheritance() {
    run_test_actions([
        TestAction::run("var comment = new Comment()"),

        // Test constructor relationship
        TestAction::assert_eq("comment.constructor === Comment", true),
        TestAction::assert_eq("comment instanceof Comment", true),

        // Test that it's an object
        TestAction::assert_eq("typeof comment", js_string!("object")),
        TestAction::assert_eq("comment !== null", true),
    ]);
}

#[test]
fn comment_data_property() {
    run_test_actions([
        TestAction::run("var comment = new Comment()"),

        // Default data should be empty string
        TestAction::assert_eq("comment.data", js_string!("")),
        TestAction::assert_eq("typeof comment.data", js_string!("string")),

        // Test setting data
        TestAction::run("comment.data = 'Test comment'"),
        TestAction::assert_eq("comment.data", js_string!("Test comment")),
    ]);
}

#[test]
fn comment_length_property() {
    run_test_actions([
        TestAction::run("var comment = new Comment('Hello')"),
        TestAction::assert_eq("comment.length", 5),
        TestAction::assert_eq("typeof comment.length", js_string!("number")),

        // Length should update when data changes
        TestAction::run("comment.data = 'Hi'"),
        TestAction::assert_eq("comment.length", 2),

        // Test with empty string
        TestAction::run("comment.data = ''"),
        TestAction::assert_eq("comment.length", 0),
    ]);
}

#[test]
fn comment_substring_data() {
    run_test_actions([
        TestAction::run("var comment = new Comment('Hello World')"),

        // Basic substring
        TestAction::assert_eq("comment.substringData(0, 5)", js_string!("Hello")),
        TestAction::assert_eq("comment.substringData(6, 5)", js_string!("World")),
        TestAction::assert_eq("comment.substringData(0, 11)", js_string!("Hello World")),

        // Partial substring
        TestAction::assert_eq("comment.substringData(1, 3)", js_string!("ell")),

        // Substring with count exceeding length
        TestAction::assert_eq("comment.substringData(6, 100)", js_string!("World")),
    ]);
}

#[test]
fn comment_append_data() {
    run_test_actions([
        TestAction::run("var comment = new Comment('Hello')"),
        TestAction::assert_eq("comment.data", js_string!("Hello")),

        // Append data
        TestAction::run("comment.appendData(' World')"),
        TestAction::assert_eq("comment.data", js_string!("Hello World")),
        TestAction::assert_eq("comment.length", 11),

        // Append empty string
        TestAction::run("comment.appendData('')"),
        TestAction::assert_eq("comment.data", js_string!("Hello World")),
    ]);
}

#[test]
fn comment_insert_data() {
    run_test_actions([
        TestAction::run("var comment = new Comment('Hello World')"),

        // Insert at beginning
        TestAction::run("comment.insertData(0, 'Hi, ')"),
        TestAction::assert_eq("comment.data", js_string!("Hi, Hello World")),

        // Insert in middle
        TestAction::run("comment = new Comment('Hello World')"),
        TestAction::run("comment.insertData(6, 'Beautiful ')"),
        TestAction::assert_eq("comment.data", js_string!("Hello Beautiful World")),

        // Insert at end
        TestAction::run("comment = new Comment('Hello')"),
        TestAction::run("comment.insertData(5, ' World')"),
        TestAction::assert_eq("comment.data", js_string!("Hello World")),
    ]);
}

#[test]
fn comment_delete_data() {
    run_test_actions([
        TestAction::run("var comment = new Comment('Hello World')"),

        // Delete from beginning
        TestAction::run("comment.deleteData(0, 6)"),
        TestAction::assert_eq("comment.data", js_string!("World")),

        // Delete from middle
        TestAction::run("comment = new Comment('Hello Beautiful World')"),
        TestAction::run("comment.deleteData(6, 10)"),
        TestAction::assert_eq("comment.data", js_string!("Hello World")),

        // Delete from end
        TestAction::run("comment = new Comment('Hello World')"),
        TestAction::run("comment.deleteData(5, 6)"),
        TestAction::assert_eq("comment.data", js_string!("Hello")),
    ]);
}

#[test]
fn comment_replace_data() {
    run_test_actions([
        TestAction::run("var comment = new Comment('Hello World')"),

        // Replace at beginning
        TestAction::run("comment.replaceData(0, 5, 'Hi')"),
        TestAction::assert_eq("comment.data", js_string!("Hi World")),

        // Replace in middle
        TestAction::run("comment = new Comment('Hello World')"),
        TestAction::run("comment.replaceData(6, 5, 'JavaScript')"),
        TestAction::assert_eq("comment.data", js_string!("Hello JavaScript")),

        // Replace at end
        TestAction::run("comment = new Comment('Hello World')"),
        TestAction::run("comment.replaceData(6, 5, 'Boa')"),
        TestAction::assert_eq("comment.data", js_string!("Hello Boa")),
    ]);
}

#[test]
fn comment_methods_exist() {
    run_test_actions([
        TestAction::run("var comment = new Comment()"),

        // Check that all methods exist and are functions
        TestAction::assert_eq("typeof comment.substringData", js_string!("function")),
        TestAction::assert_eq("typeof comment.appendData", js_string!("function")),
        TestAction::assert_eq("typeof comment.insertData", js_string!("function")),
        TestAction::assert_eq("typeof comment.deleteData", js_string!("function")),
        TestAction::assert_eq("typeof comment.replaceData", js_string!("function")),
    ]);
}

#[test]
fn comment_properties_exist() {
    run_test_actions([
        TestAction::run("var comment = new Comment()"),

        // Test all required properties exist
        TestAction::assert_eq("typeof comment.data", js_string!("string")),
        TestAction::assert_eq("typeof comment.length", js_string!("number")),
    ]);
}

#[test]
fn comment_interface_compliance() {
    run_test_actions([
        TestAction::run("var comment = new Comment()"),

        // Test all required interface members exist
        TestAction::assert_eq("'data' in comment", true),
        TestAction::assert_eq("'length' in comment", true),
        TestAction::assert_eq("'substringData' in comment", true),
        TestAction::assert_eq("'appendData' in comment", true),
        TestAction::assert_eq("'insertData' in comment", true),
        TestAction::assert_eq("'deleteData' in comment", true),
        TestAction::assert_eq("'replaceData' in comment", true),
    ]);
}

#[test]
fn comment_property_descriptors() {
    run_test_actions([
        TestAction::run("var comment = new Comment()"),

        // Test property descriptors on prototype (where DOM properties live)
        TestAction::run("var dataDesc = Object.getOwnPropertyDescriptor(Comment.prototype, 'data')"),
        TestAction::assert_eq("dataDesc !== undefined", true),
        TestAction::assert_eq("dataDesc.enumerable", true),
        TestAction::assert_eq("dataDesc.configurable", true),
        TestAction::assert_eq("typeof dataDesc.get", js_string!("function")),
        TestAction::assert_eq("typeof dataDesc.set", js_string!("function")),

        TestAction::run("var lengthDesc = Object.getOwnPropertyDescriptor(Comment.prototype, 'length')"),
        TestAction::assert_eq("lengthDesc !== undefined", true),
        TestAction::assert_eq("lengthDesc.enumerable", true),
        TestAction::assert_eq("lengthDesc.configurable", true),
        TestAction::assert_eq("typeof lengthDesc.get", js_string!("function")),
    ]);
}

#[test]
fn comment_property_enumeration() {
    run_test_actions([
        TestAction::run("var comment = new Comment()"),

        // DOM properties are on prototype, not instance - use for...in enumeration
        TestAction::run("var props = []; for (var prop in comment) { props.push(prop); }"),
        TestAction::assert_eq("props.includes('data')", true),
        TestAction::assert_eq("props.includes('length')", true),

        // Verify properties are accessible but not own properties
        TestAction::assert_eq("comment.hasOwnProperty('data')", false),
        TestAction::assert_eq("'data' in comment", true),
    ]);
}

#[test]
fn comment_edge_cases() {
    run_test_actions([
        TestAction::run("var comment = new Comment()"),

        // Test with empty string
        TestAction::run("comment.data = ''"),
        TestAction::assert_eq("comment.data", js_string!("")),
        TestAction::assert_eq("comment.length", 0),

        // Test with whitespace
        TestAction::run("comment.data = '   '"),
        TestAction::assert_eq("comment.data", js_string!("   ")),
        TestAction::assert_eq("comment.length", 3),

        // Test with special characters
        TestAction::run("comment.data = 'special chars: \\\"\\n\\t'"),
        TestAction::assert_eq("comment.data", js_string!("special chars: \"\n\t")),

        // Test with very long strings
        TestAction::run("comment.data = 'a'.repeat(1000)"),
        TestAction::assert_eq("comment.data.length", 1000),
    ]);
}

#[test]
fn comment_unicode_support() {
    run_test_actions([
        TestAction::run("var comment = new Comment('Hello 世界 🌍')"),
        TestAction::assert_eq("comment.length", 10), // Unicode characters counted correctly
        TestAction::assert_eq("comment.substringData(6, 2)", js_string!("世界")),
        TestAction::assert_eq("comment.substringData(9, 1)", js_string!("🌍")),
    ]);
}

#[test]
fn comment_method_chaining() {
    run_test_actions([
        TestAction::run("var comment = new Comment('Hello World')"),

        // Methods should return undefined (not chainable)
        TestAction::assert_eq("comment.appendData(' Test')", JsValue::undefined()),
        TestAction::assert_eq("comment.data", js_string!("Hello World Test")),

        TestAction::assert_eq("comment.insertData(0, 'Hi, ')", JsValue::undefined()),
        TestAction::assert_eq("comment.data", js_string!("Hi, Hello World Test")),
    ]);
}