use crate::{js_string, run_test_actions, TestAction, JsNativeErrorKind, JsValue};

#[test]
fn text_constructor() {
    run_test_actions([
        TestAction::assert_eq("typeof Text", js_string!("function")),
        TestAction::run("var text = new Text()"),
        TestAction::assert_eq("typeof text", js_string!("object")),
        TestAction::assert_eq("text.data", js_string!("")),
        TestAction::assert_eq("text.length", 0),
    ]);
}

#[test]
fn text_constructor_with_data() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello World')"),
        TestAction::assert_eq("typeof text", js_string!("object")),
        TestAction::assert_eq("text.data", js_string!("Hello World")),
        TestAction::assert_eq("text.length", 11),
    ]);
}

#[test]
fn text_inherits_character_data_methods() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello World')"),
        // Test CharacterData methods work on Text
        TestAction::assert_eq("text.substringData(0, 5)", js_string!("Hello")),
        TestAction::run("text.appendData('!')"),
        TestAction::assert_eq("text.data", js_string!("Hello World!")),
        TestAction::run("text.insertData(5, ' Beautiful')"),
        TestAction::assert_eq("text.data", js_string!("Hello Beautiful World!")),
        TestAction::run("text.deleteData(5, 10)"), // Delete " Beautiful"
        TestAction::assert_eq("text.data", js_string!("Hello World!")),
        TestAction::run("text.replaceData(11, 1, '?')"), // Replace "!" with "?"
        TestAction::assert_eq("text.data", js_string!("Hello World?")),
    ]);
}

#[test]
fn text_split_text() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello World')"),
        TestAction::run("var newText = text.splitText(6)"),
        TestAction::assert_eq("text.data", js_string!("Hello ")),
        TestAction::assert_eq("text.length", 6),
        TestAction::assert_eq("newText.data", js_string!("World")),
        TestAction::assert_eq("newText.length", 5),
        TestAction::assert_eq("typeof newText", js_string!("object")),
    ]);
}

#[test]
fn text_split_text_edge_cases() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello')"),
        // Split at beginning
        TestAction::run("var newText1 = text.splitText(0)"),
        TestAction::assert_eq("text.data", js_string!("")),
        TestAction::assert_eq("newText1.data", js_string!("Hello")),

        // Split at end
        TestAction::run("var text2 = new Text('World')"),
        TestAction::run("var newText2 = text2.splitText(5)"),
        TestAction::assert_eq("text2.data", js_string!("World")),
        TestAction::assert_eq("newText2.data", js_string!("")),
    ]);
}

#[test]
fn text_split_text_errors() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello')"),
        // Offset beyond length should throw
        TestAction::assert_native_error(
            "text.splitText(10)",
            JsNativeErrorKind::Range,
            "Index or size is negative or greater than the allowed amount",
        ),
    ]);
}

#[test]
fn text_replace_whole_text() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello World')"),
        TestAction::run("var result = text.replaceWholeText('New Content')"),
        TestAction::assert_eq("text.data", js_string!("New Content")),
        TestAction::assert_eq("text.length", 11),
        // Should return the same text node
        TestAction::assert_eq("result === text", true),
    ]);
}

#[test]
fn text_replace_whole_text_empty() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello')"),
        TestAction::run("var result = text.replaceWholeText('')"),
        TestAction::assert_eq("text.data", js_string!("")),
        TestAction::assert_eq("text.length", 0),
        TestAction::assert_eq("result === text", true),
    ]);
}

#[test]
fn text_replace_whole_text_null() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello')"),
        // null should be converted to "null" string
        TestAction::run("var result = text.replaceWholeText(null)"),
        TestAction::assert_eq("text.data", js_string!("null")),
        TestAction::assert_eq("text.length", 4),
    ]);
}

#[test]
fn text_whole_text_property() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello World')"),
        // wholeText should return the same content for a standalone text node
        TestAction::assert_eq("text.wholeText", js_string!("Hello World")),
        TestAction::run("text.data = 'Modified'"),
        TestAction::assert_eq("text.wholeText", js_string!("Modified")),
    ]);
}

#[test]
fn text_assigned_slot_property() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello')"),
        // assignedSlot should be null for unslotted text nodes
        TestAction::assert_eq("text.assignedSlot", JsValue::null()),
    ]);
}

#[test]
fn text_data_property_methods() {
    run_test_actions([
        TestAction::run("var text = new Text('Initial')"),
        // Test getter property
        TestAction::assert_eq("text.data", js_string!("Initial")),
        // Test setter property
        TestAction::run("text.data = 'Updated'"),
        TestAction::assert_eq("text.data", js_string!("Updated")),
        TestAction::assert_eq("text.length", 7),
    ]);
}

#[test]
fn text_length_readonly() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello')"),
        TestAction::assert_eq("text.length", 5),
        // Length should be read-only (attempts to set it should be ignored)
        TestAction::run("text.length = 10"),
        TestAction::assert_eq("text.length", 5),
    ]);
}

#[test]
fn text_unicode_handling() {
    run_test_actions([
        TestAction::run("var text = new Text('🌍🚀🎉')"),
        TestAction::assert_eq("text.length", 3),
        TestAction::assert_eq("text.data", js_string!("🌍🚀🎉")),
        TestAction::run("var newText = text.splitText(1)"),
        TestAction::assert_eq("text.data", js_string!("🌍")),
        TestAction::assert_eq("newText.data", js_string!("🚀🎉")),
    ]);
}

#[test]
fn text_mixed_operations() {
    run_test_actions([
        TestAction::run("var text = new Text('Hello')"),
        TestAction::run("text.appendData(' World')"),
        TestAction::assert_eq("text.data", js_string!("Hello World")),
        TestAction::run("var split = text.splitText(6)"),
        TestAction::assert_eq("text.data", js_string!("Hello ")),
        TestAction::assert_eq("split.data", js_string!("World")),
        TestAction::run("text.replaceWholeText('Goodbye ')"),
        TestAction::assert_eq("text.data", js_string!("Goodbye ")),
        TestAction::assert_eq("text.wholeText", js_string!("Goodbye ")),
    ]);
}

#[test]
fn text_constructor_type_conversion() {
    run_test_actions([
        // Number conversion
        TestAction::run("var text1 = new Text(123)"),
        TestAction::assert_eq("text1.data", js_string!("123")),

        // Boolean conversion
        TestAction::run("var text2 = new Text(true)"),
        TestAction::assert_eq("text2.data", js_string!("true")),

        // Object conversion
        TestAction::run("var text3 = new Text({toString: function() { return 'object'; }})"),
        TestAction::assert_eq("text3.data", js_string!("object")),

        // Null/undefined
        TestAction::run("var text4 = new Text(null)"),
        TestAction::assert_eq("text4.data", js_string!("null")),
    ]);
}

#[test]
fn text_empty_operations() {
    run_test_actions([
        TestAction::run("var text = new Text('')"),
        TestAction::assert_eq("text.length", 0),
        TestAction::run("text.appendData('')"),
        TestAction::assert_eq("text.length", 0),
        TestAction::run("text.insertData(0, '')"),
        TestAction::assert_eq("text.length", 0),
        TestAction::run("text.deleteData(0, 0)"),
        TestAction::assert_eq("text.length", 0),
        TestAction::run("text.replaceData(0, 0, '')"),
        TestAction::assert_eq("text.length", 0),
        TestAction::run("var split = text.splitText(0)"),
        TestAction::assert_eq("text.data", js_string!("")),
        TestAction::assert_eq("split.data", js_string!("")),
    ]);
}