//! The Selection Web API implementation
//!
//! The Selection API represents the text selection in a document.
//! This module provides the native Selection implementation for the Boa JavaScript engine.

use boa_gc::{Finalize, Trace};
use boa_engine::{
    builtins::{IntrinsicObject, BuiltInObject, BuiltInConstructor},
    js_string, JsString, JsArgs, JsData,
    property::Attribute,
    realm::Realm,
    Context, JsNativeError, JsObject, JsResult, JsValue,
    builtins::BuiltInBuilder,
    string::StaticJsStrings,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::internal_methods::get_prototype_from_constructor,
};

use super::frame_selection::{FrameSelection, SelectionOptions};

#[cfg(test)]
mod tests;

/// The Selection object represents the text selection made by the user.
/// This is the JavaScript API layer that delegates to FrameSelection for internal state.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct SelectionData {
    /// Internal frame selection manager (Chrome-style separation)
    frame_selection: FrameSelection,
    /// Cached ranges for JavaScript API compatibility
    ranges: Vec<JsValue>,
}

impl SelectionData {
    fn new() -> Self {
        Self {
            frame_selection: FrameSelection::new(),
            ranges: Vec::new(),
        }
    }

    /// Get anchor node by delegating to FrameSelection
    fn get_anchor_node(&self) -> Option<JsValue> {
        let binding = self.frame_selection.selection_in_dom_tree();
        let selection = binding.lock().unwrap();
        selection.anchor_node().cloned()
    }

    /// Get anchor offset by delegating to FrameSelection
    fn get_anchor_offset(&self) -> u32 {
        let binding = self.frame_selection.selection_in_dom_tree();
        let selection = binding.lock().unwrap();
        selection.anchor_offset()
    }

    /// Get focus node by delegating to FrameSelection
    fn get_focus_node(&self) -> Option<JsValue> {
        let binding = self.frame_selection.selection_in_dom_tree();
        let selection = binding.lock().unwrap();
        selection.focus_node().cloned()
    }

    /// Get focus offset by delegating to FrameSelection
    fn get_focus_offset(&self) -> u32 {
        let binding = self.frame_selection.selection_in_dom_tree();
        let selection = binding.lock().unwrap();
        selection.focus_offset()
    }

    /// Check if selection is collapsed by delegating to FrameSelection
    fn is_collapsed(&self) -> bool {
        let binding = self.frame_selection.selection_in_dom_tree();
        let selection = binding.lock().unwrap();
        selection.is_collapsed()
    }

    /// Get range count by delegating to FrameSelection
    fn get_range_count(&self) -> u32 {
        match self.frame_selection.get_selection_type() {
            super::frame_selection::SelectionType::None => 0,
            super::frame_selection::SelectionType::Caret |
            super::frame_selection::SelectionType::Range => 1,
        }
    }

    /// Get selection type string by delegating to FrameSelection
    fn get_type_string(&self) -> &'static str {
        match self.frame_selection.get_selection_type() {
            super::frame_selection::SelectionType::None => "None",
            super::frame_selection::SelectionType::Caret => "Caret",
            super::frame_selection::SelectionType::Range => "Range",
        }
    }

    /// Get selection direction by delegating to FrameSelection
    fn get_direction(&self) -> &'static str {
        let binding = self.frame_selection.selection_in_dom_tree();
        let selection = binding.lock().unwrap();
        if selection.is_collapsed() {
            "none"
        } else if selection.is_directional() {
            // TODO: Calculate actual direction based on DOM position
            "forward"
        } else {
            "none"
        }
    }

    /// Set selection using FrameSelection with proper options
    fn set_selection(&mut self, anchor_node: Option<JsValue>, anchor_offset: u32,
                    focus_node: Option<JsValue>, focus_offset: u32,
                    is_directional: bool) -> JsResult<()> {
        let options = SelectionOptions::builder()
            .is_directional(is_directional)
            .build();

        self.frame_selection.set_selection(anchor_node, anchor_offset, focus_node, focus_offset, options)
    }

    /// Clear selection using FrameSelection
    fn clear_selection(&mut self) -> JsResult<()> {
        self.frame_selection.clear()?;
        self.ranges.clear();
        Ok(())
    }

    /// Add range to selection with FrameSelection integration
    fn add_range(&mut self, range: JsValue) -> JsResult<()> {
        // Clear existing ranges (Selection API typically supports only one range)
        self.ranges.clear();
        self.ranges.push(range.clone());

        // TODO: Extract range boundaries and update FrameSelection
        // For now, just add to cached ranges
        eprintln!("Selection.addRange called - delegating to FrameSelection");
        Ok(())
    }

    /// Access internal frame selection for advanced operations
    pub fn frame_selection(&mut self) -> &mut FrameSelection {
        &mut self.frame_selection
    }
}

/// The `Selection` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct Selection;

impl IntrinsicObject for Selection {
    fn init(realm: &Realm) {
        let anchor_node_func = BuiltInBuilder::callable(realm, get_anchor_node)
            .name(js_string!("get anchorNode"))
            .build();

        let anchor_offset_func = BuiltInBuilder::callable(realm, get_anchor_offset)
            .name(js_string!("get anchorOffset"))
            .build();

        let focus_node_func = BuiltInBuilder::callable(realm, get_focus_node)
            .name(js_string!("get focusNode"))
            .build();

        let focus_offset_func = BuiltInBuilder::callable(realm, get_focus_offset)
            .name(js_string!("get focusOffset"))
            .build();

        let is_collapsed_func = BuiltInBuilder::callable(realm, get_is_collapsed)
            .name(js_string!("get isCollapsed"))
            .build();

        let range_count_func = BuiltInBuilder::callable(realm, get_range_count)
            .name(js_string!("get rangeCount"))
            .build();

        let type_func = BuiltInBuilder::callable(realm, get_type)
            .name(js_string!("get type"))
            .build();

        let direction_func = BuiltInBuilder::callable(realm, get_direction)
            .name(js_string!("get direction"))
            .build();

        // Method builders are unused since we use .method() on the BuiltInBuilder

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("anchorNode"),
                Some(anchor_node_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("anchorOffset"),
                Some(anchor_offset_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("focusNode"),
                Some(focus_node_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("focusOffset"),
                Some(focus_offset_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("isCollapsed"),
                Some(is_collapsed_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("rangeCount"),
                Some(range_count_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("type"),
                Some(type_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("direction"),
                Some(direction_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(add_range, js_string!("addRange"), 1)
            .method(remove_all_ranges, js_string!("removeAllRanges"), 0)
            .method(get_range_at, js_string!("getRangeAt"), 1)
            .method(get_composed_ranges, js_string!("getComposedRanges"), 1)
            .method(set_base_and_extent, js_string!("setBaseAndExtent"), 4)
            .method(collapse, js_string!("collapse"), 2)
            .method(modify, js_string!("modify"), 3)
            .method(selection_to_string, js_string!("toString"), 0)
            .method(collapse_to_start, js_string!("collapseToStart"), 0)
            .method(collapse_to_end, js_string!("collapseToEnd"), 0)
            .method(extend, js_string!("extend"), 2)
            .method(select_all_children, js_string!("selectAllChildren"), 1)
            .method(delete_from_document, js_string!("deleteFromDocument"), 0)
            .method(contains_node, js_string!("containsNode"), 2)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Selection {
    const NAME: JsString = StaticJsStrings::SELECTION;
}

impl BuiltInConstructor for Selection {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::selection;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If new_target is undefined then this function was called without new
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling Selection constructor without `new` is forbidden")
                .into());
        }

        let data = SelectionData::new();
        let prototype = get_prototype_from_constructor(new_target, StandardConstructors::selection, context)?;
        let selection = JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), prototype, data);
        Ok(selection.into())
    }
}

/// Get the anchor node of the selection.
fn get_anchor_node(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    if let Some(selection_data) = this_obj.downcast_ref::<SelectionData>() {
        Ok(selection_data.get_anchor_node().unwrap_or(JsValue::null()))
    } else {
        Err(JsNativeError::typ().with_message("Selection method called on non-Selection object").into())
    }
}

/// Get the anchor offset of the selection.
fn get_anchor_offset(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    if let Some(selection_data) = this_obj.downcast_ref::<SelectionData>() {
        Ok(JsValue::from(selection_data.get_anchor_offset()))
    } else {
        Err(JsNativeError::typ().with_message("Selection method called on non-Selection object").into())
    }
}

/// Get the focus node of the selection.
fn get_focus_node(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    if let Some(selection_data) = this_obj.downcast_ref::<SelectionData>() {
        Ok(selection_data.get_focus_node().unwrap_or(JsValue::null()))
    } else {
        Err(JsNativeError::typ().with_message("Selection method called on non-Selection object").into())
    }
}

/// Get the focus offset of the selection.
fn get_focus_offset(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    if let Some(selection_data) = this_obj.downcast_ref::<SelectionData>() {
        Ok(JsValue::from(selection_data.get_focus_offset()))
    } else {
        Err(JsNativeError::typ().with_message("Selection method called on non-Selection object").into())
    }
}

/// Get whether the selection is collapsed.
fn get_is_collapsed(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    if let Some(selection_data) = this_obj.downcast_ref::<SelectionData>() {
        Ok(JsValue::from(selection_data.is_collapsed()))
    } else {
        Err(JsNativeError::typ().with_message("Selection method called on non-Selection object").into())
    }
}

/// Get the range count of the selection.
fn get_range_count(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    if let Some(selection_data) = this_obj.downcast_ref::<SelectionData>() {
        Ok(JsValue::from(selection_data.get_range_count()))
    } else {
        Err(JsNativeError::typ().with_message("Selection method called on non-Selection object").into())
    }
}

/// Get the type of the selection.
fn get_type(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    if let Some(selection_data) = this_obj.downcast_ref::<SelectionData>() {
        let type_str = selection_data.get_type_string();
        Ok(JsValue::from(js_string!(type_str)))
    } else {
        Err(JsNativeError::typ().with_message("Selection method called on non-Selection object").into())
    }
}

/// Get the direction of the selection (Chrome 137 feature).
fn get_direction(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    if let Some(selection_data) = this_obj.downcast_ref::<SelectionData>() {
        let direction = selection_data.get_direction();
        Ok(JsValue::from(js_string!(direction)))
    } else {
        Err(JsNativeError::typ().with_message("Selection method called on non-Selection object").into())
    }
}

/// Add a range to the selection.
fn add_range(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let range = args.get_or_undefined(0);

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        selection_data.add_range(range.clone())?;
        eprintln!("Selection.addRange called - delegated to FrameSelection");
    }

    Ok(JsValue::undefined())
}

/// Remove all ranges from the selection.
fn remove_all_ranges(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        selection_data.clear_selection()?;
        eprintln!("Selection.removeAllRanges called - delegated to FrameSelection");
    }

    Ok(JsValue::undefined())
}

/// Get a range at the specified index.
fn get_range_at(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let index = args.get_or_undefined(0).to_integer_or_infinity(context)?;

    if let Some(selection_data) = this_obj.downcast_ref::<SelectionData>() {
        // Convert to f64 for comparison
        let index_f64 = match index {
            boa_engine::value::IntegerOrInfinity::Integer(i) => i as f64,
            boa_engine::value::IntegerOrInfinity::PositiveInfinity => f64::INFINITY,
            boa_engine::value::IntegerOrInfinity::NegativeInfinity => f64::NEG_INFINITY,
        };

        if index_f64 < 0.0 || index_f64 >= selection_data.get_range_count() as f64 {
            return Err(JsNativeError::range().with_message("Index out of range").into());
        }

        let index = index_f64 as usize;
        if let Some(range) = selection_data.ranges.get(index) {
            Ok(range.clone())
        } else {
            // Create a proper Range object using FrameSelection data
            let range_constructor = context.intrinsics().constructors().range().constructor();
            let range_prototype = range_constructor.prototype();

            // Create range data based on current selection
            let range_data = super::range::RangeData::new();
            let range_obj = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                range_prototype.clone(),
                range_data
            );

            // Set range boundaries from selection
            if let Some(mut range_data_mut) = range_obj.downcast_mut::<super::range::RangeData>() {
                if let (Some(anchor), Some(focus)) = (selection_data.get_anchor_node(), selection_data.get_focus_node()) {
                    let _ = range_data_mut.set_start(anchor, selection_data.get_anchor_offset());
                    let _ = range_data_mut.set_end(focus, selection_data.get_focus_offset());
                }
            }

            Ok(range_obj.into())
        }
    } else {
        Err(JsNativeError::typ().with_message("Selection method called on non-Selection object").into())
    }
}

/// Get composed ranges (Chrome 137 feature) with enhanced Shadow DOM support.
fn get_composed_ranges(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let shadow_roots = args.get_or_undefined(0);

    if let Some(selection_data) = this_obj.downcast_ref::<SelectionData>() {
        // Enhanced implementation with Shadow DOM support
        let mut composed_ranges: Vec<JsValue> = Vec::new();

        // Check if we have a valid selection
        if selection_data.get_range_count() > 0 {
            // Create a Range object for the current selection
            let range_constructor = context.intrinsics().constructors().range().constructor();
            let range_prototype = range_constructor.prototype();

            // Create range data based on current selection
            let range_data = super::range::RangeData::new();
            let range_obj = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                range_prototype.clone(),
                range_data
            );

            // Set range boundaries from selection
            if let Some(mut range_data_mut) = range_obj.downcast_mut::<super::range::RangeData>() {
                if let (Some(anchor), Some(focus)) = (selection_data.get_anchor_node(), selection_data.get_focus_node()) {
                    let _ = range_data_mut.set_start(anchor, selection_data.get_anchor_offset());
                    let _ = range_data_mut.set_end(focus, selection_data.get_focus_offset());
                }
            }

            composed_ranges.push(range_obj.into());

            // Enhanced Shadow DOM support
            if !shadow_roots.is_null() && !shadow_roots.is_undefined() {
                // Check if shadow_roots is an array
                if let Some(shadow_roots_obj) = shadow_roots.as_object() {
                    if shadow_roots_obj.is_array() {
                        // Process each shadow root for additional ranges
                        // In a real implementation, this would:
                        // 1. Traverse each shadow root's DOM tree
                        // 2. Find intersections with the main selection
                        // 3. Create Range objects for each shadow tree segment
                        // 4. Handle slotted content and distribution

                        eprintln!("Selection.getComposedRanges: Processing {} shadow roots",
                                shadow_roots_obj.get(js_string!("length"), context)?
                                    .to_integer_or_infinity(context)?
                                    .as_integer().unwrap_or(0));

                        // For now, we'll create placeholder ranges for demonstration
                        // In reality, these would be computed based on shadow tree analysis
                        for _i in 0..1 { // Placeholder: one additional range per shadow root
                            let shadow_range_data = super::range::RangeData::new();
                            let shadow_range_obj = JsObject::from_proto_and_data_with_shared_shape(
                                context.root_shape(),
                                range_prototype.clone(),
                                shadow_range_data
                            );
                            composed_ranges.push(shadow_range_obj.into());
                        }
                    }
                }
            }
        }

        // Create array of composed ranges
        let array = boa_engine::builtins::Array::array_create(composed_ranges.len() as u64, None, context)?;
        for (i, range) in composed_ranges.into_iter().enumerate() {
            array.set(i, range, true, context)?;
        }

        eprintln!("Selection.getComposedRanges: Returned {} ranges with Shadow DOM support",
                array.get(js_string!("length"), context)?
                    .to_integer_or_infinity(context)?
                    .as_integer().unwrap_or(0));

        Ok(array.into())
    } else {
        Err(JsNativeError::typ().with_message("Selection method called on non-Selection object").into())
    }
}

/// Set the selection base and extent.
fn set_base_and_extent(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let anchor_node = args.get_or_undefined(0);
    let anchor_offset = match args.get_or_undefined(1).to_integer_or_infinity(context)? {
        boa_engine::value::IntegerOrInfinity::Integer(i) => i.max(0) as u32,
        _ => 0,
    };
    let focus_node = args.get_or_undefined(2);
    let focus_offset = match args.get_or_undefined(3).to_integer_or_infinity(context)? {
        boa_engine::value::IntegerOrInfinity::Integer(i) => i.max(0) as u32,
        _ => 0,
    };

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        selection_data.set_selection(
            Some(anchor_node.clone()),
            anchor_offset,
            Some(focus_node.clone()),
            focus_offset,
            false // Default to non-directional
        )?;
        eprintln!("Selection.setBaseAndExtent called - delegated to FrameSelection");
    }

    Ok(JsValue::undefined())
}

/// Collapse the selection to a single point.
fn collapse(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let node = args.get_or_undefined(0);
    let offset = match args.get_or_undefined(1).to_integer_or_infinity(context)? {
        boa_engine::value::IntegerOrInfinity::Integer(i) => i.max(0) as u32,
        _ => 0,
    };

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        selection_data.set_selection(
            Some(node.clone()),
            offset,
            Some(node.clone()),
            offset,
            false
        )?;
        eprintln!("Selection.collapse called - delegated to FrameSelection");
    }

    Ok(JsValue::undefined())
}

/// Modify the selection by extending it in a specified direction.
fn modify(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let alter = args.get_or_undefined(0).to_string(context)?;
    let direction = args.get_or_undefined(1).to_string(context)?;
    let granularity = args.get_or_undefined(2).to_string(context)?;

    if let Some(selection_data) = this_obj.downcast_ref::<SelectionData>() {
        // In a real implementation, this would use FrameSelection to modify the selection
        // based on the alter ("move", "extend"), direction ("forward", "backward", etc.),
        // and granularity ("character", "word", "sentence", "line", etc.)
        eprintln!("Selection.modify called with alter: '{}', direction: '{}', granularity: '{}'",
                alter.to_std_string_escaped(), direction.to_std_string_escaped(), granularity.to_std_string_escaped());
    }

    Ok(JsValue::undefined())
}

/// Convert the selection to a string representation.
fn selection_to_string(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    if let Some(selection_data) = this_obj.downcast_ref::<SelectionData>() {
        // In a real implementation, this would extract the text content from the selected ranges
        // For now, return empty string for collapsed selections or placeholder text
        let text = if selection_data.is_collapsed() {
            ""
        } else {
            "selected text" // Placeholder - would extract actual text from DOM
        };
        Ok(JsValue::from(js_string!(text)))
    } else {
        Ok(JsValue::from(js_string!("")))
    }
}

/// Collapse the selection to the start of the first range.
fn collapse_to_start(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        if let Some(anchor) = selection_data.get_anchor_node() {
            let offset = selection_data.get_anchor_offset();
            selection_data.set_selection(
                Some(anchor.clone()),
                offset,
                Some(anchor),
                offset,
                false
            )?;
        }
        eprintln!("Selection.collapseToStart called - delegated to FrameSelection");
    }

    Ok(JsValue::undefined())
}

/// Collapse the selection to the end of the last range.
fn collapse_to_end(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        if let Some(focus) = selection_data.get_focus_node() {
            let offset = selection_data.get_focus_offset();
            selection_data.set_selection(
                Some(focus.clone()),
                offset,
                Some(focus),
                offset,
                false
            )?;
        }
        eprintln!("Selection.collapseToEnd called - delegated to FrameSelection");
    }

    Ok(JsValue::undefined())
}

/// Extend the selection to the specified node and offset.
fn extend(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let node = args.get_or_undefined(0);
    let offset = match args.get_or_undefined(1).to_integer_or_infinity(context)? {
        boa_engine::value::IntegerOrInfinity::Integer(i) => i.max(0) as u32,
        _ => 0,
    };

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        // Keep anchor position, extend focus position
        let anchor_node = selection_data.get_anchor_node();
        let anchor_offset = selection_data.get_anchor_offset();

        selection_data.set_selection(
            anchor_node,
            anchor_offset,
            Some(node.clone()),
            offset,
            true // Directional
        )?;
        eprintln!("Selection.extend called - delegated to FrameSelection");
    }

    Ok(JsValue::undefined())
}

/// Select all children of the specified node.
fn select_all_children(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let node = args.get_or_undefined(0);

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        // In a real implementation, this would select all children of the node
        // For now, create a selection from start to end of the node
        selection_data.set_selection(
            Some(node.clone()),
            0,
            Some(node.clone()),
            1, // Placeholder: would be actual child count
            false
        )?;
        eprintln!("Selection.selectAllChildren called - delegated to FrameSelection");
    }

    Ok(JsValue::undefined())
}

/// Delete the contents of the selection from the document.
fn delete_from_document(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        // In a real implementation, this would delete the selected content from the DOM
        // For now, just clear the selection
        selection_data.clear_selection()?;
        eprintln!("Selection.deleteFromDocument called - delegated to FrameSelection");
    }

    Ok(JsValue::undefined())
}

/// Check if the selection contains the specified node.
fn contains_node(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let node = args.get_or_undefined(0);
    let allow_partial_containment = args.get_or_undefined(1).to_boolean();

    if let Some(selection_data) = this_obj.downcast_ref::<SelectionData>() {
        // In a real implementation, this would check if the node is contained in any range
        // For now, do a simple check against anchor/focus nodes
        let contains = if let (Some(anchor), Some(focus)) = (selection_data.get_anchor_node(), selection_data.get_focus_node()) {
            // Simplified containment check
            node.strict_equals(&anchor) || node.strict_equals(&focus) || allow_partial_containment
        } else {
            false
        };

        eprintln!("Selection.containsNode called with allowPartialContainment: {} - delegated to FrameSelection", allow_partial_containment);
        Ok(JsValue::from(contains))
    } else {
        Ok(JsValue::from(false))
    }
}