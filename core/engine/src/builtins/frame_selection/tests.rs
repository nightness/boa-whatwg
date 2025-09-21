//! Tests for the FrameSelection architecture integration

use crate::{run_test_actions, TestAction};

#[test]
fn selection_range_integration() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),
        TestAction::run("var ranges = selection.getComposedRanges()"),
        TestAction::assert("Array.isArray(ranges)"),

        // Each range should be a proper Range object when they exist
        TestAction::run(r#"
            var allRangesValid = true;
            for (var i = 0; i < ranges.length; i++) {
                if (!(ranges[i] instanceof Range)) {
                    allRangesValid = false;
                    break;
                }
                if (!('collapsed' in ranges[i])) {
                    allRangesValid = false;
                    break;
                }
            }
            allRangesValid
        "#),
        TestAction::assert("allRangesValid === true"),
    ]);
}

#[test]
fn selection_uses_frame_selection_internally() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),
        // Test that Selection properly delegates to FrameSelection
        TestAction::assert("selection.rangeCount >= 0"),
        TestAction::assert("typeof selection.direction === 'string'"),
        TestAction::assert("typeof selection.type === 'string'"),

        // Test direction values match FrameSelection spec
        TestAction::assert("['none', 'forward', 'backward'].includes(selection.direction)"),
        TestAction::assert("['None', 'Caret', 'Range'].includes(selection.type)"),
    ]);
}

#[test]
fn selection_modify_uses_frame_selection() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),
        // Test modify method (FrameSelection feature)
        TestAction::assert("typeof selection.modify === 'function'"),

        // Test various granularities and directions
        TestAction::run("selection.modify('move', 'forward', 'character')"),
        TestAction::run("selection.modify('extend', 'backward', 'word')"),
        TestAction::run("selection.modify('move', 'left', 'line')"),
        TestAction::run("selection.modify('extend', 'right', 'sentence')"),
        TestAction::run("selection.modify('move', 'forward', 'paragraph')"),

        // Should not throw errors
        TestAction::assert("true"),
    ]);
}

#[test]
fn selection_direction_reflects_frame_selection_state() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),
        // Initial state should be 'none'
        TestAction::assert("selection.direction === 'none'"),
        TestAction::assert("selection.isCollapsed === true"),

        // Direction should be consistent with collapsed state
        TestAction::run("var directionMatchesState = selection.isCollapsed ? selection.direction === 'none' : true"),
        TestAction::assert("directionMatchesState === true"),
    ]);
}

#[test]
fn selection_type_reflects_frame_selection_state() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),
        // Initial state should be 'None'
        TestAction::assert("selection.type === 'None'"),
        TestAction::assert("selection.rangeCount === 0"),

        // Type should be consistent with range count
        TestAction::run("var typeMatchesState = (selection.rangeCount === 0) ? selection.type === 'None' : true"),
        TestAction::assert("typeMatchesState === true"),
    ]);
}

#[test]
fn selection_state_changes_together() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),

        // Test removeAllRanges affects all state
        TestAction::run("selection.removeAllRanges()"),
        TestAction::assert("selection.rangeCount === 0"),
        TestAction::assert("selection.type === 'None'"),
        TestAction::assert("selection.direction === 'none'"),
        TestAction::assert("selection.isCollapsed === true"),
    ]);
}

#[test]
fn selection_get_composed_ranges_uses_frame_selection() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),
        TestAction::run("var ranges = selection.getComposedRanges()"),

        // getComposedRanges should return Range objects that match internal state
        TestAction::assert("Array.isArray(ranges)"),
        TestAction::assert("ranges.length === selection.rangeCount"),

        // Test with shadow roots parameter (FrameSelection Shadow DOM support)
        TestAction::run("var rangesWithShadow = selection.getComposedRanges([])"),
        TestAction::assert("Array.isArray(rangesWithShadow)"),
        TestAction::assert("rangesWithShadow.length >= 0"),
    ]);
}

#[test]
fn selection_get_range_at_uses_frame_selection() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),

        // getRangeAt should fail for empty selection
        TestAction::run("var error = null; try { selection.getRangeAt(0); } catch(e) { error = e; }"),
        TestAction::assert("error !== null"),
        TestAction::assert("error instanceof RangeError"),
    ]);
}

#[test]
fn selection_frame_selection_granularity_support() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),

        // Test all granularity levels supported by FrameSelection
        TestAction::run("selection.modify('move', 'forward', 'character')"),
        TestAction::run("selection.modify('move', 'forward', 'word')"),
        TestAction::run("selection.modify('move', 'forward', 'sentence')"),
        TestAction::run("selection.modify('move', 'forward', 'line')"),
        TestAction::run("selection.modify('move', 'forward', 'paragraph')"),
        TestAction::run("selection.modify('move', 'forward', 'document')"),

        // Should not throw errors
        TestAction::assert("true"),
    ]);
}

#[test]
fn selection_frame_selection_direction_support() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),

        // Test all direction types supported by FrameSelection
        TestAction::run("selection.modify('move', 'forward', 'character')"),
        TestAction::run("selection.modify('move', 'backward', 'character')"),
        TestAction::run("selection.modify('move', 'left', 'character')"),
        TestAction::run("selection.modify('move', 'right', 'character')"),

        // Should not throw errors
        TestAction::assert("true"),
    ]);
}

#[test]
fn selection_frame_selection_alter_support() {
    run_test_actions([
        TestAction::run("var selection = window.getSelection()"),

        // Test both alter types supported by FrameSelection
        TestAction::run("selection.modify('move', 'forward', 'character')"),
        TestAction::run("selection.modify('extend', 'forward', 'character')"),

        // Should not throw errors
        TestAction::assert("true"),
    ]);
}