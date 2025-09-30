use crate::{js_string, run_test_actions, TestAction, JsNativeErrorKind};

#[test]
fn character_data_constructor() {
    run_test_actions([
        TestAction::assert_eq("typeof CharacterData", js_string!("function")),
        // CharacterData is abstract - should not be constructible directly
        TestAction::assert_native_error(
            "new CharacterData()",
            JsNativeErrorKind::Type,
            "Illegal constructor",
        ),
    ]);
}

#[test]
fn character_data_data_property() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello World')"),
        TestAction::assert_eq("text.data", js_string!("Hello World")),
        TestAction::run("text.data = 'Modified'"),
        TestAction::assert_eq("text.data", js_string!("Modified")),
    ]);
}

#[test]
fn character_data_length_property() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello')"),
        TestAction::assert_eq("text.length", 5),
        TestAction::run("text.data = 'Testing 123'"),
        TestAction::assert_eq("text.length", 11),
    ]);
}

#[test]
fn character_data_substring_data() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello World')"),
        TestAction::assert_eq("text.substringData(0, 5)", js_string!("Hello")),
        TestAction::assert_eq("text.substringData(6, 5)", js_string!("World")),
        TestAction::assert_eq("text.substringData(0, 11)", js_string!("Hello World")),
        TestAction::assert_eq("text.substringData(5, 1)", js_string!(" ")),
        // Test edge cases
        TestAction::assert_eq("text.substringData(11, 0)", js_string!("")),
        TestAction::assert_eq("text.substringData(6, 100)", js_string!("World")),
    ]);
}

#[test]
fn character_data_substring_data_errors() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello')"),
        // Offset beyond length should throw
        TestAction::assert_native_error(
            "text.substringData(10, 1)",
            JsNativeErrorKind::Range,
            "Index or size is negative or greater than the allowed amount",
        ),
    ]);
}

#[test]
fn character_data_append_data() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello')"),
        TestAction::run("text.appendData(' World')"),
        TestAction::assert_eq("text.data", js_string!("Hello World")),
        TestAction::assert_eq("text.length", 11),
        TestAction::run("text.appendData('!')"),
        TestAction::assert_eq("text.data", js_string!("Hello World!")),
        TestAction::assert_eq("text.length", 12),
    ]);
}

#[test]
fn character_data_insert_data() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello World')"),
        TestAction::run("text.insertData(5, ' Beautiful')"),
        TestAction::assert_eq("text.data", js_string!("Hello Beautiful World")),
        TestAction::assert_eq("text.length", 21),
        TestAction::run("text.insertData(0, 'Wow! ')"),
        TestAction::assert_eq("text.data", js_string!("Wow! Hello Beautiful World")),
    ]);
}

#[test]
fn character_data_insert_data_errors() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello')"),
        // Offset beyond length should throw
        TestAction::assert_native_error(
            "text.insertData(10, 'test')",
            JsNativeErrorKind::Range,
            "Index or size is negative or greater than the allowed amount",
        ),
    ]);
}

#[test]
fn character_data_delete_data() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello World')"),
        TestAction::run("text.deleteData(5, 6)"), // Delete " World"
        TestAction::assert_eq("text.data", js_string!("Hello")),
        TestAction::assert_eq("text.length", 5),
        TestAction::run("text.deleteData(0, 2)"), // Delete "He"
        TestAction::assert_eq("text.data", js_string!("llo")),
    ]);
}

#[test]
fn character_data_delete_data_edge_cases() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello')"),
        // Delete beyond end should only delete to end
        TestAction::run("text.deleteData(3, 100)"),
        TestAction::assert_eq("text.data", js_string!("Hel")),
        // Delete 0 characters should do nothing
        TestAction::run("text.deleteData(1, 0)"),
        TestAction::assert_eq("text.data", js_string!("Hel")),
    ]);
}

#[test]
fn character_data_delete_data_errors() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello')"),
        // Offset beyond length should throw
        TestAction::assert_native_error(
            "text.deleteData(10, 1)",
            JsNativeErrorKind::Range,
            "Index or size is negative or greater than the allowed amount",
        ),
    ]);
}

#[test]
fn character_data_replace_data() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello World')"),
        TestAction::run("text.replaceData(6, 5, 'JavaScript')"), // Replace "World" with "JavaScript"
        TestAction::assert_eq("text.data", js_string!("Hello JavaScript")),
        TestAction::assert_eq("text.length", 16),
        TestAction::run("text.replaceData(0, 5, 'Hi')"), // Replace "Hello" with "Hi"
        TestAction::assert_eq("text.data", js_string!("Hi JavaScript")),
    ]);
}

#[test]
fn character_data_replace_data_edge_cases() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello')"),
        // Replace beyond end should only replace to end
        TestAction::run("text.replaceData(3, 100, 'p!')"),
        TestAction::assert_eq("text.data", js_string!("Help!")),
        // Replace 0 characters should insert
        TestAction::run("text.replaceData(4, 0, ' me')"),
        TestAction::assert_eq("text.data", js_string!("Help me!")),
    ]);
}

#[test]
fn character_data_replace_data_errors() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello')"),
        // Offset beyond length should throw
        TestAction::assert_native_error(
            "text.replaceData(10, 1, 'test')",
            JsNativeErrorKind::Range,
            "Index or size is negative or greater than the allowed amount",
        ),
    ]);
}

#[test]
fn character_data_unicode_support() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello 🌍')"),
        // Note: JavaScript counts UTF-16 code units, so emoji 🌍 counts as 2 units
        TestAction::assert_eq("text.length", 7),
        TestAction::assert_eq("text.substringData(6, 1)", js_string!("🌍")),
        TestAction::run("text.appendData(' 🚀')"),
        TestAction::assert_eq("text.data", js_string!("Hello 🌍 🚀")),
        // Length should be: "Hello " (6) + "🌍" (1) + " " (1) + "🚀" (1) = 9
        TestAction::assert_eq("text.length", 9),
    ]);
}

#[test]
fn character_data_empty_string() {
    run_test_actions([
        TestAction::run("var text = new Text('')"),
        TestAction::assert_eq("text.data", js_string!("")),
        TestAction::assert_eq("text.length", 0),
        TestAction::assert_eq("text.substringData(0, 0)", js_string!("")),
        TestAction::run("text.appendData('Added')"),
        TestAction::assert_eq("text.data", js_string!("Added")),
        TestAction::assert_eq("text.length", 5),
    ]);
}