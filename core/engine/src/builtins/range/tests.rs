//! Tests for the Range API implementation

use crate::{run_test_actions, TestAction};

#[test]
fn range_constructor_available() {
    run_test_actions([
        TestAction::assert("typeof Range === 'function'"),
    ]);
}

#[test]
fn range_constructor_new() {
    run_test_actions([
        TestAction::run("var range = new Range()"),
        TestAction::assert("range instanceof Range"),
        TestAction::assert("typeof range === 'object'"),
        TestAction::assert("range !== null"),
    ]);
}

#[test]
fn range_constructor_requires_new() {
    run_test_actions([
        // Test that Range constructor requires 'new'
        TestAction::run("var error = null; try { Range(); } catch(e) { error = e; }"),
        TestAction::assert("error !== null"),
        TestAction::assert("error instanceof TypeError"),
    ]);
}

#[test]
fn range_basic_properties() {
    run_test_actions([
        TestAction::run("var range = new Range()"),
        TestAction::assert("'startContainer' in range"),
        TestAction::assert("'startOffset' in range"),
        TestAction::assert("'endContainer' in range"),
        TestAction::assert("'endOffset' in range"),
        TestAction::assert("'collapsed' in range"),
        TestAction::assert("'commonAncestorContainer' in range"),
    ]);
}

#[test]
fn range_constants() {
    run_test_actions([
        TestAction::assert("Range.START_TO_START === 0"),
        TestAction::assert("Range.START_TO_END === 1"),
        TestAction::assert("Range.END_TO_END === 2"),
        TestAction::assert("Range.END_TO_START === 3"),
        TestAction::assert("typeof Range.START_TO_START === 'number'"),
        TestAction::assert("typeof Range.START_TO_END === 'number'"),
        TestAction::assert("typeof Range.END_TO_END === 'number'"),
        TestAction::assert("typeof Range.END_TO_START === 'number'"),
    ]);
}

#[test]
fn range_initial_state() {
    run_test_actions([
        TestAction::run("var range = new Range()"),
        TestAction::assert("range.collapsed === true"),
        TestAction::assert("range.startContainer === null"),
        TestAction::assert("range.endContainer === null"),
        TestAction::assert("range.startOffset === 0"),
        TestAction::assert("range.endOffset === 0"),
        TestAction::assert("range.commonAncestorContainer === null"),
    ]);
}

#[test]
fn range_boundary_methods() {
    run_test_actions([
        TestAction::run("var range = new Range()"),
        TestAction::assert("typeof range.setStart === 'function'"),
        TestAction::assert("typeof range.setEnd === 'function'"),
        TestAction::assert("typeof range.setStartBefore === 'function'"),
        TestAction::assert("typeof range.setStartAfter === 'function'"),
        TestAction::assert("typeof range.setEndBefore === 'function'"),
        TestAction::assert("typeof range.setEndAfter === 'function'"),
    ]);
}

#[test]
fn range_selection_methods() {
    run_test_actions([
        TestAction::run("var range = new Range()"),
        TestAction::assert("typeof range.selectNode === 'function'"),
        TestAction::assert("typeof range.selectNodeContents === 'function'"),
        TestAction::assert("typeof range.compareBoundaryPoints === 'function'"),
        TestAction::assert("typeof range.collapse === 'function'"),
    ]);
}

#[test]
fn range_content_methods() {
    run_test_actions([
        TestAction::run("var range = new Range()"),
        TestAction::assert("typeof range.deleteContents === 'function'"),
        TestAction::assert("typeof range.extractContents === 'function'"),
        TestAction::assert("typeof range.cloneContents === 'function'"),
        TestAction::assert("typeof range.insertNode === 'function'"),
        TestAction::assert("typeof range.surroundContents === 'function'"),
    ]);
}

#[test]
fn range_utility_methods() {
    run_test_actions([
        TestAction::run("var range = new Range()"),
        TestAction::assert("typeof range.cloneRange === 'function'"),
        TestAction::assert("typeof range.toString === 'function'"),
        TestAction::assert("typeof range.detach === 'function'"),
    ]);
}

#[test]
fn range_clone_functionality() {
    run_test_actions([
        TestAction::run("var range1 = new Range()"),
        TestAction::run("var range2 = range1.cloneRange()"),
        TestAction::assert("range2 instanceof Range"),
        TestAction::assert("range1 !== range2"), // Different objects
        TestAction::assert("range1.collapsed === range2.collapsed"),
        TestAction::assert("range1.startOffset === range2.startOffset"),
        TestAction::assert("range1.endOffset === range2.endOffset"),
    ]);
}

#[test]
fn range_collapse_to_start() {
    run_test_actions([
        TestAction::run("var range = new Range()"),
        TestAction::run("range.collapse(true)"),
        TestAction::assert("range.collapsed === true"),
    ]);
}

#[test]
fn range_collapse_to_end() {
    run_test_actions([
        TestAction::run("var range = new Range()"),
        TestAction::run("range.collapse(false)"),
        TestAction::assert("range.collapsed === true"), // Still collapsed since no content
    ]);
}

#[test]
fn range_to_string_functionality() {
    run_test_actions([
        TestAction::run("var range = new Range()"),
        TestAction::run("var str = range.toString()"),
        TestAction::assert("typeof str === 'string'"),
        TestAction::assert("str === ''"), // Empty range
    ]);
}

#[test]
fn range_detach_functionality() {
    run_test_actions([
        TestAction::run("var range = new Range()"),
        TestAction::run("range.detach()"),
        // Detach should not throw errors
        TestAction::assert("true"),
    ]);
}

#[test]
fn range_compare_boundary_points_functionality() {
    run_test_actions([
        TestAction::run("var range1 = new Range()"),
        TestAction::run("var range2 = new Range()"),
        // Test comparison with same ranges
        TestAction::run("var result = range1.compareBoundaryPoints(Range.START_TO_START, range2)"),
        TestAction::assert("typeof result === 'number'"),
        TestAction::assert("result === 0"), // Same ranges should be equal
    ]);
}

#[test]
fn range_boundary_operations() {
    run_test_actions([
        TestAction::run("var range = new Range()"),
        // Test setStart/setEnd with null node (should not throw)
        TestAction::run("range.setStart(null, 0)"),
        TestAction::run("range.setEnd(null, 0)"),
        TestAction::assert("range.collapsed === true"),
    ]);
}

#[test]
fn range_select_operations() {
    run_test_actions([
        TestAction::run("var range = new Range()"),
        // Test selectNode and selectNodeContents with null (should not throw)
        TestAction::run("range.selectNode(null)"),
        TestAction::run("range.selectNodeContents(null)"),
        TestAction::assert("true"), // If we get here, no errors were thrown
    ]);
}

#[test]
fn range_content_operations() {
    run_test_actions([
        TestAction::run("var range = new Range()"),
        // Test content operations on empty range
        TestAction::run("range.deleteContents()"),
        TestAction::run("var extracted = range.extractContents()"),
        TestAction::run("var cloned = range.cloneContents()"),
        TestAction::assert("extracted === null"),
        TestAction::assert("cloned === null"),
    ]);
}

#[test]
fn range_insertion_operations() {
    run_test_actions([
        TestAction::run("var range = new Range()"),
        // Test insertNode and surroundContents (should not throw)
        TestAction::run("range.insertNode(null)"),
        TestAction::run("range.surroundContents(null)"),
        TestAction::assert("true"), // If we get here, no errors were thrown
    ]);
}