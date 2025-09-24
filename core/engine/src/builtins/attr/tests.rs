use crate::{js_string, run_test_actions, TestAction, JsNativeErrorKind, JsValue};

#[test]
fn attr_constructor() {
    run_test_actions([
        TestAction::assert_eq("typeof Attr", js_string!("function")),
        TestAction::run("var attr = new Attr()"),
        TestAction::assert_eq("typeof attr", js_string!("object")),
        TestAction::assert_eq("attr !== null", true),
    ]);
}

#[test]
fn attr_constructor_allows_call_without_new() {
    run_test_actions([
        // DOM constructors typically allow being called without new
        TestAction::run("var attr = Attr()"),
        TestAction::assert_eq("typeof attr", js_string!("object")),
        TestAction::assert_eq("attr !== null", true),
    ]);
}

#[test]
fn attr_inheritance() {
    run_test_actions([
        TestAction::run("var attr = new Attr()"),

        // Test constructor relationship
        TestAction::assert_eq("attr.constructor === Attr", true),
        TestAction::assert_eq("attr instanceof Attr", true),

        // Test that it's an object
        TestAction::assert_eq("typeof attr", js_string!("object")),
        TestAction::assert_eq("attr !== null", true),
    ]);
}

#[test]
fn attr_properties_exist() {
    run_test_actions([
        TestAction::run("var attr = new Attr()"),

        // Test all required properties exist
        TestAction::assert_eq("typeof attr.name", js_string!("string")),
        TestAction::assert_eq("typeof attr.value", js_string!("string")),
        TestAction::assert_eq("attr.ownerElement === null", true),
        TestAction::assert_eq("attr.namespaceURI === null", true),
        TestAction::assert_eq("attr.localName === null", true),
        TestAction::assert_eq("attr.prefix === null", true),
        TestAction::assert_eq("typeof attr.specified", js_string!("boolean")),
    ]);
}

#[test]
fn attr_interface_compliance() {
    run_test_actions([
        TestAction::run("var attr = new Attr()"),

        // Test all required interface members exist
        TestAction::assert_eq("'name' in attr", true),
        TestAction::assert_eq("'value' in attr", true),
        TestAction::assert_eq("'ownerElement' in attr", true),
        TestAction::assert_eq("'namespaceURI' in attr", true),
        TestAction::assert_eq("'localName' in attr", true),
        TestAction::assert_eq("'prefix' in attr", true),
        TestAction::assert_eq("'specified' in attr", true),
    ]);
}

#[test]
fn attr_name_property() {
    run_test_actions([
        TestAction::run("var attr = new Attr()"),

        // DOM standard: name should be a property, not a method
        TestAction::assert_eq("typeof attr.name", js_string!("string")),
        TestAction::assert_eq("attr.name", js_string!("")),
    ]);
}

#[test]
fn attr_value_property() {
    run_test_actions([
        TestAction::run("var attr = new Attr()"),

        // Value should be empty string by default
        TestAction::assert_eq("attr.value", js_string!("")),
        TestAction::assert_eq("typeof attr.value", js_string!("string")),

        // Value should be writable
        TestAction::run("attr.value = 'test-value'"),
        TestAction::assert_eq("attr.value", js_string!("test-value")),
    ]);
}

#[test]
fn attr_owner_element_property() {
    run_test_actions([
        TestAction::run("var attr = new Attr()"),

        // ownerElement should be null by default
        TestAction::assert_eq("attr.ownerElement", JsValue::null()),
        TestAction::assert_eq("attr.ownerElement === null", true),
    ]);
}

#[test]
fn attr_namespace_properties() {
    run_test_actions([
        TestAction::run("var attr = new Attr()"),

        // Namespace properties should be null by default
        TestAction::assert_eq("attr.namespaceURI", JsValue::null()),
        TestAction::assert_eq("attr.localName", JsValue::null()),
        TestAction::assert_eq("attr.prefix", JsValue::null()),

        TestAction::assert_eq("attr.namespaceURI === null", true),
        TestAction::assert_eq("attr.localName === null", true),
        TestAction::assert_eq("attr.prefix === null", true),
    ]);
}

#[test]
fn attr_specified_property() {
    run_test_actions([
        TestAction::run("var attr = new Attr()"),

        // specified should always be true in DOM Level 4
        TestAction::assert_eq("attr.specified", true),
        TestAction::assert_eq("typeof attr.specified", js_string!("boolean")),
    ]);
}

#[test]
fn attr_property_descriptors() {
    run_test_actions([
        TestAction::run("var attr = new Attr()"),

        // Test property descriptors on prototype (where DOM properties live)
        TestAction::run("var nameDesc = Object.getOwnPropertyDescriptor(Attr.prototype, 'name')"),
        TestAction::assert_eq("nameDesc !== undefined", true),
        TestAction::assert_eq("nameDesc.enumerable", true),
        TestAction::assert_eq("nameDesc.configurable", true),
        TestAction::assert_eq("typeof nameDesc.get", js_string!("function")),

        TestAction::run("var valueDesc = Object.getOwnPropertyDescriptor(Attr.prototype, 'value')"),
        TestAction::assert_eq("valueDesc !== undefined", true),
        TestAction::assert_eq("valueDesc.enumerable", true),
        TestAction::assert_eq("valueDesc.configurable", true),
        TestAction::assert_eq("typeof valueDesc.get", js_string!("function")),
        TestAction::assert_eq("typeof valueDesc.set", js_string!("function")),

        TestAction::run("var ownerDesc = Object.getOwnPropertyDescriptor(Attr.prototype, 'ownerElement')"),
        TestAction::assert_eq("ownerDesc !== undefined", true),
        TestAction::assert_eq("ownerDesc.enumerable", true),
        TestAction::assert_eq("ownerDesc.configurable", true),
        TestAction::assert_eq("typeof ownerDesc.get", js_string!("function")),
    ]);
}

#[test]
fn attr_property_enumeration() {
    run_test_actions([
        TestAction::run("var attr = new Attr()"),

        // DOM properties are on prototype, not instance - use for...in enumeration
        TestAction::run("var props = []; for (var prop in attr) { props.push(prop); }"),
        TestAction::assert_eq("props.includes('name')", true),
        TestAction::assert_eq("props.includes('value')", true),
        TestAction::assert_eq("props.includes('ownerElement')", true),
        TestAction::assert_eq("props.includes('namespaceURI')", true),
        TestAction::assert_eq("props.includes('localName')", true),
        TestAction::assert_eq("props.includes('prefix')", true),
        TestAction::assert_eq("props.includes('specified')", true),

        // Verify properties are accessible but not own properties
        TestAction::assert_eq("attr.hasOwnProperty('name')", false),
        TestAction::assert_eq("'name' in attr", true),
    ]);
}

#[test]
fn attr_readonly_properties() {
    run_test_actions([
        TestAction::run("var attr = new Attr()"),

        // Name should be read-only
        TestAction::run("try { attr.name = 'new-name'; } catch(e) {}"),
        TestAction::assert_eq("attr.name", js_string!("")),

        // ownerElement should be read-only
        TestAction::run("try { attr.ownerElement = {}; } catch(e) {}"),
        TestAction::assert_eq("attr.ownerElement", JsValue::null()),

        // namespace properties should be read-only
        TestAction::run("try { attr.namespaceURI = 'test'; } catch(e) {}"),
        TestAction::assert_eq("attr.namespaceURI", JsValue::null()),

        TestAction::run("try { attr.localName = 'test'; } catch(e) {}"),
        TestAction::assert_eq("attr.localName", JsValue::null()),

        TestAction::run("try { attr.prefix = 'test'; } catch(e) {}"),
        TestAction::assert_eq("attr.prefix", JsValue::null()),

        // specified should be read-only
        TestAction::run("try { attr.specified = false; } catch(e) {}"),
        TestAction::assert_eq("attr.specified", true),
    ]);
}

#[test]
fn attr_value_setting() {
    run_test_actions([
        TestAction::run("var attr = new Attr()"),

        // Test setting different value types
        TestAction::run("attr.value = 'string-value'"),
        TestAction::assert_eq("attr.value", js_string!("string-value")),

        TestAction::run("attr.value = 123"),
        TestAction::assert_eq("attr.value", js_string!("123")),

        TestAction::run("attr.value = true"),
        TestAction::assert_eq("attr.value", js_string!("true")),

        TestAction::run("attr.value = null"),
        TestAction::assert_eq("attr.value", js_string!("null")),

        TestAction::run("attr.value = undefined"),
        TestAction::assert_eq("attr.value", js_string!("undefined")),

        // Reset to empty
        TestAction::run("attr.value = ''"),
        TestAction::assert_eq("attr.value", js_string!("")),
    ]);
}

#[test]
fn attr_property_errors() {
    run_test_actions([
        TestAction::run("var obj = {}"),

        // Test property access on non-Attr object
        TestAction::assert_native_error(
            "Object.getOwnPropertyDescriptor(Attr.prototype, 'name').get.call(obj)",
            JsNativeErrorKind::Type,
            "Attr.prototype.name called on non-Attr object",
        ),

        TestAction::assert_native_error(
            "Object.getOwnPropertyDescriptor(Attr.prototype, 'value').get.call(obj)",
            JsNativeErrorKind::Type,
            "Attr.prototype.value called on non-Attr object",
        ),

        TestAction::assert_native_error(
            "Object.getOwnPropertyDescriptor(Attr.prototype, 'ownerElement').get.call(obj)",
            JsNativeErrorKind::Type,
            "Attr.prototype.ownerElement called on non-Attr object",
        ),
    ]);
}

#[test]
fn attr_interface_completeness() {
    run_test_actions([
        TestAction::run("var attr = new Attr()"),

        // Test that all DOM Level 4 Attr interface properties are present
        TestAction::assert_eq("typeof attr.name", js_string!("string")),
        TestAction::assert_eq("typeof attr.value", js_string!("string")),
        TestAction::assert_eq("attr.ownerElement === null", true),
        TestAction::assert_eq("attr.namespaceURI === null", true),
        TestAction::assert_eq("attr.localName === null", true),
        TestAction::assert_eq("attr.prefix === null", true),
        TestAction::assert_eq("attr.specified === true", true),

        // Test constructor and prototype chain
        TestAction::assert_eq("attr.constructor === Attr", true),
        TestAction::assert_eq("attr instanceof Attr", true),
        TestAction::assert_eq("Object.getPrototypeOf(attr) === Attr.prototype", true),
    ]);
}

#[test]
fn attr_edge_cases() {
    run_test_actions([
        TestAction::run("var attr = new Attr()"),

        // Test edge cases for value setting
        TestAction::run("attr.value = ''"),
        TestAction::assert_eq("attr.value", js_string!("")),

        TestAction::run("attr.value = '   '"),
        TestAction::assert_eq("attr.value", js_string!("   ")),

        TestAction::run("attr.value = 'special chars: \\\"\\n\\t'"),
        TestAction::assert_eq("attr.value", js_string!("special chars: \"\n\t")),

        // Test with very long strings
        TestAction::run("attr.value = 'a'.repeat(1000)"),
        TestAction::assert_eq("attr.value.length", 1000),
    ]);
}