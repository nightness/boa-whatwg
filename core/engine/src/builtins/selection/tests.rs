//! Tests for the Selection API implementation

use crate::{run_test_actions, TestAction};

#[test]
fn selection_window_get_selection() {
    run_test_actions([
        TestAction::assert("typeof window !== 'undefined'"),
        TestAction::assert("typeof window.getSelection === 'function'"),
        TestAction::run("var selection = window.getSelection()"),
        TestAction::assert("typeof selection === 'object'"),
        TestAction::assert("selection !== null"),
    ]);
}

#[test]
fn selection_basic_properties() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),
        TestAction::assert("'anchorNode' in selection"),
        TestAction::assert("'anchorOffset' in selection"),
        TestAction::assert("'focusNode' in selection"),
        TestAction::assert("'focusOffset' in selection"),
        TestAction::assert("'isCollapsed' in selection"),
        TestAction::assert("'rangeCount' in selection"),
        TestAction::assert("'type' in selection"),
        TestAction::assert("'direction' in selection"),
    ]);
}

#[test]
fn selection_basic_methods() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),
        TestAction::assert("typeof selection.addRange === 'function'"),
        TestAction::assert("typeof selection.removeAllRanges === 'function'"),
        TestAction::assert("typeof selection.getRangeAt === 'function'"),
        TestAction::assert("typeof selection.getComposedRanges === 'function'"),
        TestAction::assert("typeof selection.setBaseAndExtent === 'function'"),
        TestAction::assert("typeof selection.collapse === 'function'"),
        TestAction::assert("typeof selection.modify === 'function'"),
        TestAction::assert("typeof selection.toString === 'function'"),
    ]);
}

#[test]
fn selection_initial_state() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),
        TestAction::assert("selection.rangeCount === 0"),
        TestAction::assert("selection.isCollapsed === true"),
        TestAction::assert("selection.type === 'None'"),
        TestAction::assert("selection.direction === 'none'"),
        TestAction::assert("selection.anchorNode === null"),
        TestAction::assert("selection.focusNode === null"),
        TestAction::assert("selection.anchorOffset === 0"),
        TestAction::assert("selection.focusOffset === 0"),
    ]);
}

#[test]
fn selection_direction_property() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),
        TestAction::assert("typeof selection.direction === 'string'"),
        TestAction::assert("selection.direction === 'none'"),
        // Valid direction values
        TestAction::assert("['none', 'forward', 'backward'].includes(selection.direction)"),
    ]);
}

#[test]
fn selection_type_property() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),
        TestAction::assert("typeof selection.type === 'string'"),
        TestAction::assert("selection.type === 'None'"),
        // Valid type values
        TestAction::assert("['None', 'Caret', 'Range'].includes(selection.type)"),
    ]);
}

#[test]
fn selection_get_composed_ranges() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),
        TestAction::run("var ranges = selection.getComposedRanges()"),
        TestAction::assert("Array.isArray(ranges)"),
        TestAction::assert("ranges.length === 0"), // Empty selection
        TestAction::assert("typeof ranges === 'object'"),
    ]);
}

#[test]
fn selection_get_composed_ranges_with_param() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),
        TestAction::run("var rangesWithParam = selection.getComposedRanges([])"),
        TestAction::assert("Array.isArray(rangesWithParam)"),
        TestAction::assert("rangesWithParam.length === 0"),
    ]);
}

#[test]
fn selection_modify_method() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),
        TestAction::assert("typeof selection.modify === 'function'"),
        // Test that modify calls don't throw errors
        TestAction::run("selection.modify('move', 'forward', 'character')"),
        TestAction::run("selection.modify('extend', 'backward', 'word')"),
        TestAction::run("selection.modify('move', 'left', 'line')"),
        TestAction::assert("true"), // If we get here, no errors were thrown
    ]);
}

#[test]
fn selection_range_operations() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),
        // Test removeAllRanges
        TestAction::run("selection.removeAllRanges()"),
        TestAction::assert("selection.rangeCount === 0"),

        // Test that getRangeAt throws for invalid index
        TestAction::run("var error = null; try { selection.getRangeAt(0); } catch(e) { error = e; }"),
        TestAction::assert("error !== null"),
    ]);
}

#[test]
fn selection_collapse_operations() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),
        TestAction::assert("typeof selection.collapse === 'function'"),
        TestAction::assert("typeof selection.collapseToStart === 'function'"),
        TestAction::assert("typeof selection.collapseToEnd === 'function'"),

        // Test collapse calls don't throw
        TestAction::run("selection.collapse(null, 0)"),
        TestAction::run("selection.collapseToStart()"),
        TestAction::run("selection.collapseToEnd()"),
        TestAction::assert("true"),
    ]);
}

#[test]
fn selection_extend_operations() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),
        TestAction::assert("typeof selection.extend === 'function'"),
        TestAction::assert("typeof selection.selectAllChildren === 'function'"),
        TestAction::assert("typeof selection.deleteFromDocument === 'function'"),
        TestAction::assert("typeof selection.containsNode === 'function'"),
    ]);
}

#[test]
fn selection_string_conversion() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),
        TestAction::run("var selectionStr = selection.toString()"),
        TestAction::assert("typeof selectionStr === 'string'"),
        TestAction::assert("selectionStr === ''"), // Empty selection
    ]);
}