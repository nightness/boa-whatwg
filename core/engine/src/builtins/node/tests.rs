//! Node interface unit tests

use crate::{run_test_actions, JsNativeErrorKind, TestAction, JsValue, js_string};
use boa_macros::js_str;

#[test]
fn node_constructor() {
    run_test_actions([
        // Test Node constructor exists
        TestAction::assert_eq("typeof Node", js_str!("function")),

        // Test Node can be constructed
        TestAction::run("var node = new Node()"),
        TestAction::assert_eq("typeof node", js_str!("object")),
        TestAction::assert("node instanceof Node"),
    ]);
}

#[test]
fn node_constants() {
    run_test_actions([
        // Test all Node type constants
        TestAction::assert_eq("Node.ELEMENT_NODE", 1),
        TestAction::assert_eq("Node.ATTRIBUTE_NODE", 2),
        TestAction::assert_eq("Node.TEXT_NODE", 3),
        TestAction::assert_eq("Node.CDATA_SECTION_NODE", 4),
        TestAction::assert_eq("Node.PROCESSING_INSTRUCTION_NODE", 7),
        TestAction::assert_eq("Node.COMMENT_NODE", 8),
        TestAction::assert_eq("Node.DOCUMENT_NODE", 9),
        TestAction::assert_eq("Node.DOCUMENT_TYPE_NODE", 10),
        TestAction::assert_eq("Node.DOCUMENT_FRAGMENT_NODE", 11),
        TestAction::assert_eq("Node.NOTATION_NODE", 12),
    ]);
}

#[test]
fn node_properties() {
    run_test_actions([
        // Create a Node instance
        TestAction::run("var node = new Node()"),

        // Test nodeType property
        TestAction::assert_eq("node.nodeType()", 0),

        // Test nodeName property
        TestAction::assert_eq("node.nodeName()", js_str!("#node")),

        // Test nodeValue property
        TestAction::assert_eq("node.nodeValue()", JsValue::null()),

        // Test parentNode property
        TestAction::assert_eq("node.parentNode()", JsValue::null()),

        // Test childNodes property
        TestAction::run("var childNodes = node.childNodes()"),
        TestAction::assert_eq("typeof childNodes", js_str!("object")),
        TestAction::assert("Array.isArray(childNodes)"),
        TestAction::assert_eq("childNodes.length", 0),

        // Test firstChild property
        TestAction::assert_eq("node.firstChild()", JsValue::null()),

        // Test lastChild property
        TestAction::assert_eq("node.lastChild()", JsValue::null()),

        // Test previousSibling property
        TestAction::assert_eq("node.previousSibling()", JsValue::null()),

        // Test nextSibling property
        TestAction::assert_eq("node.nextSibling()", JsValue::null()),

        // Test ownerDocument property
        TestAction::assert_eq("node.ownerDocument()", JsValue::null()),
    ]);
}

#[test]
fn node_has_child_nodes() {
    run_test_actions([
        // Create a Node instance
        TestAction::run("var node = new Node()"),

        // Test hasChildNodes method on empty node
        TestAction::assert_eq("node.hasChildNodes()", false),
    ]);
}

#[test]
fn node_clone_node() {
    run_test_actions([
        // Create a Node instance
        TestAction::run("var node = new Node()"),

        // Test cloneNode method (shallow clone)
        TestAction::run("var cloned = node.cloneNode(false)"),
        TestAction::assert_eq("typeof cloned", js_str!("object")),
        TestAction::assert("cloned instanceof Node"),

        // Test cloneNode method (deep clone)
        TestAction::run("var deepCloned = node.cloneNode(true)"),
        TestAction::assert_eq("typeof deepCloned", js_str!("object")),
        TestAction::assert("deepCloned instanceof Node"),

        // Verify cloned node is different object
        TestAction::assert_eq("node.cloneNode() === node", false),

        // Test default parameter (should be false)
        TestAction::run("var defaultCloned = node.cloneNode()"),
        TestAction::assert("defaultCloned instanceof Node"),
    ]);
}

#[test]
fn node_is_same_node() {
    run_test_actions([
        // Create Node instances
        TestAction::run("var node1 = new Node()"),
        TestAction::run("var node2 = new Node()"),

        // Test isSameNode with same node
        TestAction::assert_eq("node1.isSameNode(node1)", true),

        // Test isSameNode with different node
        TestAction::assert_eq("node1.isSameNode(node2)", false),

        // Test isSameNode with null
        TestAction::assert_eq("node1.isSameNode(null)", false),
    ]);
}

#[test]
fn node_is_equal_node() {
    run_test_actions([
        // Create Node instances
        TestAction::run("var node1 = new Node()"),
        TestAction::run("var node2 = new Node()"),

        // Test isEqualNode with same node
        TestAction::assert_eq("node1.isEqualNode(node1)", true),

        // Test isEqualNode with null
        TestAction::assert_eq("node1.isEqualNode(null)", false),

        // Test isEqualNode with equivalent nodes
        TestAction::assert_eq("node1.isEqualNode(node2)", true), // Basic nodes should be equal
    ]);
}

#[test]
fn node_contains() {
    run_test_actions([
        // Create Node instances
        TestAction::run("var node1 = new Node()"),
        TestAction::run("var node2 = new Node()"),

        // Test contains with self (should return true)
        TestAction::assert_eq("node1.contains(node1)", true),

        // Test contains with different node (should return false for basic nodes)
        TestAction::assert_eq("node1.contains(node2)", false),
    ]);
}

#[test]
fn node_compare_document_position() {
    run_test_actions([
        // Create Node instances
        TestAction::run("var node1 = new Node()"),
        TestAction::run("var node2 = new Node()"),

        // Test compareDocumentPosition
        TestAction::run("var position = node1.compareDocumentPosition(node2)"),
        TestAction::assert_eq("typeof position", js_str!("number")),

        // Result should be a valid bitmask value (>= 0)
        TestAction::assert("position >= 0"),
    ]);
}

#[test]
fn node_get_root_node() {
    run_test_actions([
        // Create a Node instance
        TestAction::run("var node = new Node()"),

        // Test getRootNode method
        TestAction::run("var root = node.getRootNode()"),
        TestAction::assert_eq("typeof root", js_str!("object")),

        // For a standalone node, it should return itself as root
        TestAction::assert_eq("node.getRootNode() === node", true),
    ]);
}

#[test]
fn node_normalize() {
    run_test_actions([
        // Create a Node instance
        TestAction::run("var node = new Node()"),

        // Test normalize method (should not throw)
        TestAction::run("node.normalize()"),
        TestAction::assert_eq("typeof node.normalize()", js_str!("undefined")),
    ]);
}

#[test]
fn node_lookup_methods() {
    run_test_actions([
        // Create a Node instance
        TestAction::run("var node = new Node()"),

        // Test lookupPrefix method
        TestAction::assert_eq("node.lookupPrefix('http://www.w3.org/1999/xhtml')", JsValue::null()),

        // Test lookupNamespaceURI method
        TestAction::assert_eq("node.lookupNamespaceURI('html')", JsValue::null()),

        // Test isDefaultNamespace method
        TestAction::assert_eq("node.isDefaultNamespace('http://www.w3.org/1999/xhtml')", false),
    ]);
}

#[test]
fn node_error_cases() {
    run_test_actions([
        // Test Node constructor without 'new' should throw
        TestAction::assert_native_error("Node()", JsNativeErrorKind::Type, "Constructor Node requires 'new'"),

        // Test calling methods on non-Node objects should throw
        TestAction::assert_native_error(
            "Node.prototype.nodeType.call({})",
            JsNativeErrorKind::Type,
            "Node.nodeType called on non-Node object"
        ),

        TestAction::assert_native_error(
            "Node.prototype.hasChildNodes.call({})",
            JsNativeErrorKind::Type,
            "Node.hasChildNodes called on non-Node object"
        ),
    ]);
}

#[test]
fn node_inheritance() {
    run_test_actions([
        // Create a Node instance
        TestAction::run("var node = new Node()"),

        // Test instanceof
        TestAction::assert("node instanceof Node"),

        // Test constructor property
        TestAction::assert_eq("node.constructor === Node", true),

        // Test that Node is a function
        TestAction::assert_eq("typeof Node", js_str!("function")),

        // Test Node prototype
        TestAction::assert_eq("typeof Node.prototype", js_str!("object")),
    ]);
}

#[test]
fn node_method_invocation() {
    run_test_actions([
        TestAction::run("var node = new Node()"),

        // Verify all methods can be called
        TestAction::run("node.appendChild(new Node())"),
        TestAction::run("node.insertBefore(new Node(), null)"),
        TestAction::run("node.removeChild(node.firstChild() || new Node())"),
        TestAction::run("node.replaceChild(new Node(), node.firstChild() || new Node())"),
        TestAction::run("node.cloneNode(false)"),
        TestAction::run("node.normalize()"),
        TestAction::run("node.isEqualNode(node)"),
        TestAction::run("node.isSameNode(node)"),
        TestAction::run("node.compareDocumentPosition(new Node())"),
        TestAction::run("node.contains(node)"),
        TestAction::run("node.lookupPrefix(null)"),
        TestAction::run("node.lookupNamespaceURI(null)"),
        TestAction::run("node.isDefaultNamespace(null)"),
        TestAction::run("node.hasChildNodes()"),
        TestAction::run("node.getRootNode()"),
    ]);
}

#[test]
fn node_properties_immutable() {
    run_test_actions([
        TestAction::run("var node = new Node()"),

        // Test that Node constants are immutable
        TestAction::run("var originalValue = Node.ELEMENT_NODE"),
        TestAction::run("Node.ELEMENT_NODE = 999"),
        TestAction::assert_eq("Node.ELEMENT_NODE", 1),
    ]);
}