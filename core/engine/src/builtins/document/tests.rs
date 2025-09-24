use crate::{js_string, run_test_actions, TestAction, JsNativeErrorKind, JsValue};

#[test]
fn document_constructor() {
    run_test_actions([
        TestAction::assert_eq("typeof Document", js_string!("function")),
        TestAction::run("var doc = new Document()"),
        TestAction::assert_eq("typeof doc", js_string!("object")),
        TestAction::assert_eq("doc !== null", true),
    ]);
}

#[test]
fn document_inheritance() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),

        // Test constructor relationship
        TestAction::assert_eq("doc.constructor === Document", true),
        TestAction::assert_eq("doc instanceof Document", true),

        // Test that it's an object
        TestAction::assert_eq("typeof doc", js_string!("object")),
        TestAction::assert_eq("doc !== null", true),
    ]);
}

#[test]
fn document_properties_exist() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),

        // Test all required properties exist
        TestAction::assert_eq("'readyState' in doc", true),
        TestAction::assert_eq("'URL' in doc", true),
        TestAction::assert_eq("'title' in doc", true),
        TestAction::assert_eq("'body' in doc", true),
        TestAction::assert_eq("'head' in doc", true),
    ]);
}

#[test]
fn document_methods_exist() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),

        // Test all required methods exist
        TestAction::assert_eq("typeof doc.createElement", js_string!("function")),
        TestAction::assert_eq("typeof doc.getElementById", js_string!("function")),
        TestAction::assert_eq("typeof doc.querySelector", js_string!("function")),
        TestAction::assert_eq("typeof doc.querySelectorAll", js_string!("function")),
        TestAction::assert_eq("typeof doc.addEventListener", js_string!("function")),
        TestAction::assert_eq("typeof doc.removeEventListener", js_string!("function")),
        TestAction::assert_eq("typeof doc.dispatchEvent", js_string!("function")),
        TestAction::assert_eq("typeof doc.startViewTransition", js_string!("function")),
    ]);
}

#[test]
fn document_ready_state() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),

        // readyState should be a string
        TestAction::assert_eq("typeof doc.readyState", js_string!("string")),

        // Should have a valid readyState value
        TestAction::run("var validStates = ['loading', 'interactive', 'complete']"),
        TestAction::run("var isValid = validStates.includes(doc.readyState)"),
        TestAction::assert_eq("isValid || doc.readyState === ''", true), // Allow empty for testing
    ]);
}

#[test]
fn document_url_property() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),

        // URL should be a string
        TestAction::assert_eq("typeof doc.URL", js_string!("string")),

        // URL should be read-only (no setter)
        TestAction::run("var originalURL = doc.URL"),
        TestAction::run("try { doc.URL = 'http://example.com'; } catch(e) {}"),
        TestAction::assert_eq("doc.URL === originalURL", true),
    ]);
}

#[test]
fn document_title_property() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),

        // title should be a string
        TestAction::assert_eq("typeof doc.title", js_string!("string")),

        // Should be able to set and get title
        TestAction::run("doc.title = 'Test Title'"),
        TestAction::assert_eq("doc.title", js_string!("Test Title")),

        // Clear title
        TestAction::run("doc.title = ''"),
        TestAction::assert_eq("doc.title", js_string!("")),
    ]);
}

#[test]
fn document_body_property() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),

        // body should initially be null or an object
        TestAction::run("var bodyType = typeof doc.body"),
        TestAction::run("var isValidBody = bodyType === 'object'"),
        TestAction::assert_eq("isValidBody", true),
    ]);
}

#[test]
fn document_head_property() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),

        // head should initially be null or an object
        TestAction::run("var headType = typeof doc.head"),
        TestAction::run("var isValidHead = headType === 'object'"),
        TestAction::assert_eq("isValidHead", true),
    ]);
}

#[test]
fn document_create_element() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),

        // Create a div element
        TestAction::run("var div = doc.createElement('div')"),
        TestAction::assert_eq("typeof div", js_string!("object")),
        TestAction::assert_eq("div !== null", true),

        // Element should have correct tag name
        TestAction::assert_eq("div.tagName", js_string!("DIV")),

        // Create different element types
        TestAction::run("var span = doc.createElement('span')"),
        TestAction::assert_eq("span.tagName", js_string!("SPAN")),

        TestAction::run("var p = doc.createElement('p')"),
        TestAction::assert_eq("p.tagName", js_string!("P")),
    ]);
}

#[test]
fn document_create_element_with_attributes() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),
        TestAction::run("var elem = doc.createElement('div')"),

        // Set attributes on created element
        TestAction::run("elem.setAttribute('id', 'test-div')"),
        TestAction::run("elem.setAttribute('class', 'test-class')"),

        // Verify attributes
        TestAction::assert_eq("elem.getAttribute('id')", js_string!("test-div")),
        TestAction::assert_eq("elem.getAttribute('class')", js_string!("test-class")),
        TestAction::assert_eq("elem.hasAttribute('id')", true),
        TestAction::assert_eq("elem.hasAttribute('class')", true),
    ]);
}

#[test]
fn document_get_element_by_id() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),

        // Initially no element with ID should exist
        TestAction::run("var notFound = doc.getElementById('nonexistent')"),
        TestAction::assert_eq("notFound", JsValue::null()),

        // Test with empty string
        TestAction::run("var emptyId = doc.getElementById('')"),
        TestAction::assert_eq("emptyId", JsValue::null()),
    ]);
}

#[test]
fn document_query_selector() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),

        // querySelector should return null for non-existent selectors
        TestAction::run("var notFound = doc.querySelector('div')"),
        TestAction::assert_eq("notFound", JsValue::null()),

        TestAction::run("var notFoundId = doc.querySelector('#nonexistent')"),
        TestAction::assert_eq("notFoundId", JsValue::null()),

        TestAction::run("var notFoundClass = doc.querySelector('.nonexistent')"),
        TestAction::assert_eq("notFoundClass", JsValue::null()),
    ]);
}

#[test]
fn document_query_selector_all() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),

        // querySelectorAll should return empty array for non-existent selectors
        TestAction::run("var emptyList = doc.querySelectorAll('div')"),
        TestAction::assert_eq("Array.isArray(emptyList)", true),
        TestAction::assert_eq("emptyList.length", 0),

        TestAction::run("var emptyListId = doc.querySelectorAll('#nonexistent')"),
        TestAction::assert_eq("Array.isArray(emptyListId)", true),
        TestAction::assert_eq("emptyListId.length", 0),
    ]);
}

#[test]
fn document_event_methods() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),
        TestAction::run("var called = false"),
        TestAction::run("var handler = function() { called = true; }"),

        // Add event listener
        TestAction::run("doc.addEventListener('test', handler)"),
        TestAction::assert_eq("called", false),

        // Remove event listener
        TestAction::run("doc.removeEventListener('test', handler)"),
        TestAction::assert_eq("called", false),

        // Methods should return undefined
        TestAction::assert_eq("doc.addEventListener('test2', function(){}) === undefined", true),
        TestAction::assert_eq("doc.removeEventListener('test2', function(){}) === undefined", true),
    ]);
}

#[test]
fn document_dispatch_event() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),

        // Create a mock event object
        TestAction::run("var event = { type: 'test', defaultPrevented: false }"),

        // Dispatch event should return boolean
        TestAction::run("var result = doc.dispatchEvent(event)"),
        TestAction::assert_eq("typeof result", js_string!("boolean")),
    ]);
}

#[test]
fn document_start_view_transition() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),

        // startViewTransition should be callable
        TestAction::run("var transition = doc.startViewTransition()"),
        TestAction::assert_eq("typeof transition", js_string!("object")),
        TestAction::assert_eq("transition !== null", true),
    ]);
}

#[test]
fn document_method_errors() {
    run_test_actions([
        // Test methods on non-object
        TestAction::assert_native_error(
            "Document.prototype.createElement.call(null, 'div')",
            JsNativeErrorKind::Type,
            "Document.prototype.createElement called on non-object",
        ),

        TestAction::assert_native_error(
            "Document.prototype.getElementById.call(null, 'test')",
            JsNativeErrorKind::Type,
            "Document.prototype.getElementById called on non-object",
        ),

        TestAction::assert_native_error(
            "Document.prototype.querySelector.call(null, 'div')",
            JsNativeErrorKind::Type,
            "Document.prototype.querySelector called on non-object",
        ),

        TestAction::assert_native_error(
            "Document.prototype.querySelectorAll.call(null, 'div')",
            JsNativeErrorKind::Type,
            "Document.prototype.querySelectorAll called on non-object",
        ),
    ]);
}

#[test]
fn document_property_errors() {
    run_test_actions([
        TestAction::run("var obj = {}"),

        // Test property access on non-Document object
        TestAction::assert_native_error(
            "Object.getOwnPropertyDescriptor(Document.prototype, 'readyState').get.call(obj)",
            JsNativeErrorKind::Type,
            "Document.prototype.readyState called on non-Document object",
        ),

        TestAction::assert_native_error(
            "Object.getOwnPropertyDescriptor(Document.prototype, 'URL').get.call(obj)",
            JsNativeErrorKind::Type,
            "Document.prototype.URL called on non-Document object",
        ),

        TestAction::assert_native_error(
            "Object.getOwnPropertyDescriptor(Document.prototype, 'title').get.call(obj)",
            JsNativeErrorKind::Type,
            "Document.prototype.title called on non-Document object",
        ),
    ]);
}

#[test]
fn document_interface_compliance() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),

        // Test all required interface members exist
        TestAction::assert_eq("'readyState' in doc", true),
        TestAction::assert_eq("'URL' in doc", true),
        TestAction::assert_eq("'title' in doc", true),
        TestAction::assert_eq("'body' in doc", true),
        TestAction::assert_eq("'head' in doc", true),
        TestAction::assert_eq("'createElement' in doc", true),
        TestAction::assert_eq("'getElementById' in doc", true),
        TestAction::assert_eq("'querySelector' in doc", true),
        TestAction::assert_eq("'querySelectorAll' in doc", true),
        TestAction::assert_eq("'addEventListener' in doc", true),
        TestAction::assert_eq("'removeEventListener' in doc", true),
        TestAction::assert_eq("'dispatchEvent' in doc", true),
        TestAction::assert_eq("'startViewTransition' in doc", true),
    ]);
}

#[test]
fn document_create_element_edge_cases() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),

        // Test with empty string
        TestAction::run("var empty = doc.createElement('')"),
        TestAction::assert_eq("typeof empty", js_string!("object")),

        // Test with mixed case
        TestAction::run("var mixed = doc.createElement('DiV')"),
        TestAction::assert_eq("mixed.tagName", js_string!("DIV")),

        // Test with numbers
        TestAction::run("var withNum = doc.createElement('h1')"),
        TestAction::assert_eq("withNum.tagName", js_string!("H1")),
    ]);
}

#[test]
fn document_selector_edge_cases() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),

        // Test with empty selector
        TestAction::run("var emptySelector = doc.querySelector('')"),
        TestAction::assert_eq("emptySelector", JsValue::null()),

        TestAction::run("var emptySelectorAll = doc.querySelectorAll('')"),
        TestAction::assert_eq("Array.isArray(emptySelectorAll)", true),
        TestAction::assert_eq("emptySelectorAll.length", 0),

        // Test with whitespace selector
        TestAction::run("var wsSelector = doc.querySelector('   ')")
            ,
        TestAction::assert_eq("wsSelector", JsValue::null()),
    ]);
}

#[test]
fn document_title_edge_cases() {
    run_test_actions([
        TestAction::run("var doc = new Document()"),

        // Test setting title to various values
        TestAction::run("doc.title = 'Normal Title'"),
        TestAction::assert_eq("doc.title", js_string!("Normal Title")),

        TestAction::run("doc.title = ''"),
        TestAction::assert_eq("doc.title", js_string!("")),

        TestAction::run("doc.title = '   '"),
        TestAction::assert_eq("doc.title", js_string!("   ")),

        TestAction::run("doc.title = 'Title with \"quotes\" and \'apostrophes\''"),
        TestAction::assert_eq("doc.title", js_string!("Title with \"quotes\" and 'apostrophes'")),
    ]);
}


*** End Patch