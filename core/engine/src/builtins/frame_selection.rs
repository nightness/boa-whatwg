//! FrameSelection - Internal Selection State Management
//!
//! Separates internal selection state management from the JavaScript DOMSelection API,
//! following Chrome's Blink architecture pattern.
//!
//! This module handles:
//! - Core selection state and transitions
//! - Selection modification logic
//! - Integration with DOM tree and layout
//! - Caret positioning and rendering state

use boa_gc::{Finalize, Trace};
use boa_engine::{JsValue, JsResult};
use std::sync::{Arc, Mutex};

#[cfg(test)]
mod tests;

/// Selection granularity levels (matches Chrome's implementation)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionGranularity {
    Character,
    Word,
    Sentence,
    Line,
    Paragraph,
    Document,
}

/// Selection modification types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionModifyDirection {
    Right,
    Left,
    Forward,
    Backward,
}

/// Selection modification alter types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionModifyAlter {
    Move,
    Extend,
}

/// Selection type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionType {
    None,
    Caret,
    Range,
}

/// Internal selection state in DOM tree
#[derive(Debug, Clone, Trace, Finalize)]
pub struct SelectionInDOMTree {
    anchor_node: Option<JsValue>,
    anchor_offset: u32,
    focus_node: Option<JsValue>,
    focus_offset: u32,
    is_directional: bool,
}

impl SelectionInDOMTree {
    pub fn new() -> Self {
        Self {
            anchor_node: None,
            anchor_offset: 0,
            focus_node: None,
            focus_offset: 0,
            is_directional: false,
        }
    }

    pub fn is_collapsed(&self) -> bool {
        self.anchor_node == self.focus_node && self.anchor_offset == self.focus_offset
    }

    pub fn is_none(&self) -> bool {
        self.anchor_node.is_none() && self.focus_node.is_none()
    }

    pub fn anchor_node(&self) -> Option<&JsValue> {
        self.anchor_node.as_ref()
    }

    pub fn focus_node(&self) -> Option<&JsValue> {
        self.focus_node.as_ref()
    }

    pub fn anchor_offset(&self) -> u32 {
        self.anchor_offset
    }

    pub fn focus_offset(&self) -> u32 {
        self.focus_offset
    }

    pub fn is_directional(&self) -> bool {
        self.is_directional
    }

    pub fn set_selection(&mut self, anchor_node: Option<JsValue>, anchor_offset: u32,
                        focus_node: Option<JsValue>, focus_offset: u32, is_directional: bool) {
        self.anchor_node = anchor_node;
        self.anchor_offset = anchor_offset;
        self.focus_node = focus_node;
        self.focus_offset = focus_offset;
        self.is_directional = is_directional;
    }

    pub fn clear(&mut self) {
        self.anchor_node = None;
        self.anchor_offset = 0;
        self.focus_node = None;
        self.focus_offset = 0;
        self.is_directional = false;
    }
}

/// Selection options builder (inspired by Chrome's SetSelectionOptions::Builder)
#[derive(Debug, Clone)]
pub struct SelectionOptionsBuilder {
    granularity: SelectionGranularity,
    should_clear_typing_style: bool,
    should_close_typing: bool,
    should_shrink_next_tap: bool,
    is_directional: bool,
    do_not_set_focus: bool,
}

impl SelectionOptionsBuilder {
    pub fn new() -> Self {
        Self {
            granularity: SelectionGranularity::Character,
            should_clear_typing_style: false,
            should_close_typing: false,
            should_shrink_next_tap: false,
            is_directional: false,
            do_not_set_focus: false,
        }
    }

    pub fn granularity(mut self, granularity: SelectionGranularity) -> Self {
        self.granularity = granularity;
        self
    }

    pub fn should_clear_typing_style(mut self, should_clear: bool) -> Self {
        self.should_clear_typing_style = should_clear;
        self
    }

    pub fn is_directional(mut self, is_directional: bool) -> Self {
        self.is_directional = is_directional;
        self
    }

    pub fn do_not_set_focus(mut self, do_not_set: bool) -> Self {
        self.do_not_set_focus = do_not_set;
        self
    }

    pub fn build(self) -> SelectionOptions {
        SelectionOptions {
            granularity: self.granularity,
            should_clear_typing_style: self.should_clear_typing_style,
            should_close_typing: self.should_close_typing,
            should_shrink_next_tap: self.should_shrink_next_tap,
            is_directional: self.is_directional,
            do_not_set_focus: self.do_not_set_focus,
        }
    }
}

/// Selection options for complex operations
#[derive(Debug, Clone, Trace, Finalize)]
pub struct SelectionOptions {
    #[unsafe_ignore_trace]
    granularity: SelectionGranularity,
    should_clear_typing_style: bool,
    should_close_typing: bool,
    should_shrink_next_tap: bool,
    is_directional: bool,
    do_not_set_focus: bool,
}

impl SelectionOptions {
    pub fn builder() -> SelectionOptionsBuilder {
        SelectionOptionsBuilder::new()
    }

    pub fn granularity(&self) -> SelectionGranularity {
        self.granularity
    }

    pub fn is_directional(&self) -> bool {
        self.is_directional
    }
}

/// Core FrameSelection - manages internal selection state
#[derive(Debug, Clone, Trace, Finalize)]
pub struct FrameSelection {
    /// Current selection in DOM tree (not traced - manual memory management)
    #[unsafe_ignore_trace]
    selection_in_dom_tree: Arc<Mutex<SelectionInDOMTree>>,

    /// Selection granularity for modifications
    #[unsafe_ignore_trace]
    granularity: SelectionGranularity,

    /// Whether selection is focused
    is_focused: bool,

    /// Whether caret is visible
    is_caret_visible: bool,

    /// Selection change listeners (function callbacks)
    #[unsafe_ignore_trace]
    change_listeners: Arc<Mutex<Vec<JsValue>>>,
}

impl FrameSelection {
    pub fn new() -> Self {
        Self {
            selection_in_dom_tree: Arc::new(Mutex::new(SelectionInDOMTree::new())),
            granularity: SelectionGranularity::Character,
            is_focused: false,
            is_caret_visible: true,
            change_listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get current selection in DOM tree (thread-safe)
    pub fn selection_in_dom_tree(&self) -> Arc<Mutex<SelectionInDOMTree>> {
        self.selection_in_dom_tree.clone()
    }

    /// Set selection with options (Chrome-style API)
    pub fn set_selection(&mut self, anchor_node: Option<JsValue>, anchor_offset: u32,
                        focus_node: Option<JsValue>, focus_offset: u32,
                        options: SelectionOptions) -> JsResult<()> {
        let mut selection = self.selection_in_dom_tree.lock().unwrap();

        // Store old selection for change detection
        let old_selection = selection.clone();

        // Apply new selection
        selection.set_selection(
            anchor_node,
            anchor_offset,
            focus_node,
            focus_offset,
            options.is_directional(),
        );

        // Update internal state
        self.granularity = options.granularity();

        // Notify change if selection actually changed
        if !self.selections_equal(&old_selection, &selection) {
            drop(selection); // Release lock before notify
            self.notify_selection_changed();
        }

        println!("FrameSelection: Selection updated with granularity {:?}", self.granularity);
        Ok(())
    }

    /// Clear current selection
    pub fn clear(&mut self) -> JsResult<()> {
        let mut selection = self.selection_in_dom_tree.lock().unwrap();
        let was_empty = selection.is_none();

        selection.clear();

        if !was_empty {
            drop(selection); // Release lock before notify
            self.notify_selection_changed();
        }

        println!("FrameSelection: Selection cleared");
        Ok(())
    }

    /// Modify selection (Chrome's modify method equivalent)
    pub fn modify(&mut self, alter: SelectionModifyAlter, direction: SelectionModifyDirection,
                  granularity: SelectionGranularity) -> JsResult<bool> {

        self.granularity = granularity;

        // TODO: Implement actual text modification logic based on DOM tree traversal
        // For now, this is a stub that demonstrates the API structure

        println!("FrameSelection: Modify selection - alter: {:?}, direction: {:?}, granularity: {:?}",
                alter, direction, granularity);

        // Simulate selection modification
        match alter {
            SelectionModifyAlter::Move => {
                // Move caret/selection
                self.notify_selection_changed();
                Ok(true)
            },
            SelectionModifyAlter::Extend => {
                // Extend current selection
                self.notify_selection_changed();
                Ok(true)
            }
        }
    }

    /// Get selection type
    pub fn get_selection_type(&self) -> SelectionType {
        let selection = self.selection_in_dom_tree.lock().unwrap();

        if selection.is_none() {
            SelectionType::None
        } else if selection.is_collapsed() {
            SelectionType::Caret
        } else {
            SelectionType::Range
        }
    }

    /// Check if selection is focused
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        if self.is_focused != focused {
            self.is_focused = focused;
            println!("FrameSelection: Focus state changed to {}", focused);
        }
    }

    /// Check if caret is visible
    pub fn is_caret_visible(&self) -> bool {
        self.is_caret_visible
    }

    /// Set caret visibility
    pub fn set_caret_visible(&mut self, visible: bool) {
        if self.is_caret_visible != visible {
            self.is_caret_visible = visible;
            println!("FrameSelection: Caret visibility changed to {}", visible);
        }
    }

    /// Add selection change listener
    pub fn add_change_listener(&mut self, listener: JsValue) {
        let mut listeners = self.change_listeners.lock().unwrap();
        listeners.push(listener);
        println!("FrameSelection: Added selection change listener");
    }

    /// Remove selection change listener
    pub fn remove_change_listener(&mut self, listener: &JsValue) {
        let mut listeners = self.change_listeners.lock().unwrap();
        listeners.retain(|l| l != listener);
        println!("FrameSelection: Removed selection change listener");
    }

    /// Notify all listeners of selection change
    fn notify_selection_changed(&self) {
        let listeners = self.change_listeners.lock().unwrap();
        if !listeners.is_empty() {
            println!("FrameSelection: Notifying {} selection change listeners", listeners.len());
            // TODO: Actually dispatch events to listeners
            // This would require access to the JavaScript context
        }
    }

    /// Compare two selections for equality
    fn selections_equal(&self, a: &SelectionInDOMTree, b: &SelectionInDOMTree) -> bool {
        a.anchor_node() == b.anchor_node() &&
        a.focus_node() == b.focus_node() &&
        a.anchor_offset() == b.anchor_offset() &&
        a.focus_offset() == b.focus_offset() &&
        a.is_directional() == b.is_directional()
    }

    /// Get current granularity
    pub fn granularity(&self) -> SelectionGranularity {
        self.granularity
    }

    /// Set granularity
    pub fn set_granularity(&mut self, granularity: SelectionGranularity) {
        self.granularity = granularity;
        println!("FrameSelection: Granularity changed to {:?}", granularity);
    }
}

impl Default for FrameSelection {
    fn default() -> Self {
        Self::new()
    }
}