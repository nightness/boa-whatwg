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

/// The Selection object represents the text selection made by the user.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct SelectionData {
    anchor_node: Option<JsValue>,
    anchor_offset: u32,
    focus_node: Option<JsValue>,
    focus_offset: u32,
    is_collapsed: bool,
    range_count: u32,
    selection_type: SelectionType,
    ranges: Vec<JsValue>, // Array of Range objects
}

#[derive(Debug, Clone, Trace, Finalize)]
enum SelectionType {
    None,
    Caret,
    Range,
}

impl SelectionData {
    fn new() -> Self {
        Self {
            anchor_node: None,
            anchor_offset: 0,
            focus_node: None,
            focus_offset: 0,
            is_collapsed: true,
            range_count: 0,
            selection_type: SelectionType::None,
            ranges: Vec::new(),
        }
    }

    fn update_state(&mut self) {
        if self.anchor_node.is_none() && self.focus_node.is_none() {
            self.selection_type = SelectionType::None;
            self.is_collapsed = true;
            self.range_count = 0;
        } else if self.anchor_node == self.focus_node && self.anchor_offset == self.focus_offset {
            self.selection_type = SelectionType::Caret;
            self.is_collapsed = true;
            self.range_count = if self.anchor_node.is_some() { 1 } else { 0 };
        } else {
            self.selection_type = SelectionType::Range;
            self.is_collapsed = false;
            self.range_count = 1;
        }
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

        let add_range_func = BuiltInBuilder::callable(realm, add_range)
            .name(js_string!("addRange"))
            .length(1)
            .build();

        let remove_all_ranges_func = BuiltInBuilder::callable(realm, remove_all_ranges)
            .name(js_string!("removeAllRanges"))
            .length(0)
            .build();

        let get_range_at_func = BuiltInBuilder::callable(realm, get_range_at)
            .name(js_string!("getRangeAt"))
            .length(1)
            .build();

        let get_composed_ranges_func = BuiltInBuilder::callable(realm, get_composed_ranges)
            .name(js_string!("getComposedRanges"))
            .length(1)
            .build();

        let set_base_and_extent_func = BuiltInBuilder::callable(realm, set_base_and_extent)
            .name(js_string!("setBaseAndExtent"))
            .length(4)
            .build();

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
        args: &[JsValue],
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
        Ok(selection_data.anchor_node.clone().unwrap_or(JsValue::null()))
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
        Ok(JsValue::from(selection_data.anchor_offset))
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
        Ok(selection_data.focus_node.clone().unwrap_or(JsValue::null()))
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
        Ok(JsValue::from(selection_data.focus_offset))
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
        Ok(JsValue::from(selection_data.is_collapsed))
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
        Ok(JsValue::from(selection_data.range_count))
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
        let type_str = match selection_data.selection_type {
            SelectionType::None => "None",
            SelectionType::Caret => "Caret",
            SelectionType::Range => "Range",
        };
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
        let direction = if selection_data.is_collapsed {
            "none"
        } else {
            // For real implementation, we'd calculate based on anchor/focus positions
            "forward"
        };
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
        // Clear existing ranges (Selection API typically supports only one range)
        selection_data.ranges.clear();

        // Add the new range
        selection_data.ranges.push(range.clone());

        // Update selection state based on range
        // In a real implementation, we'd extract nodes and offsets from the Range object
        selection_data.range_count = 1;
        selection_data.is_collapsed = false;
        selection_data.selection_type = SelectionType::Range;

        println!("Selection.addRange called - range added");
    }

    Ok(JsValue::undefined())
}

/// Remove all ranges from the selection.
fn remove_all_ranges(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        selection_data.ranges.clear();
        selection_data.anchor_node = None;
        selection_data.focus_node = None;
        selection_data.anchor_offset = 0;
        selection_data.focus_offset = 0;
        selection_data.update_state();

        println!("Selection.removeAllRanges called - all ranges removed");
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

        if index_f64 < 0.0 || index_f64 >= selection_data.range_count as f64 {
            return Err(JsNativeError::range().with_message("Index out of range").into());
        }

        let index = index_f64 as usize;
        if let Some(range) = selection_data.ranges.get(index) {
            Ok(range.clone())
        } else {
            // Create a mock range object for compatibility
            let range_obj = boa_engine::object::ObjectInitializer::new(context)
                .property(js_string!("startContainer"), selection_data.anchor_node.clone().unwrap_or(JsValue::null()), Attribute::READONLY)
                .property(js_string!("startOffset"), selection_data.anchor_offset, Attribute::READONLY)
                .property(js_string!("endContainer"), selection_data.focus_node.clone().unwrap_or(JsValue::null()), Attribute::READONLY)
                .property(js_string!("endOffset"), selection_data.focus_offset, Attribute::READONLY)
                .property(js_string!("collapsed"), selection_data.is_collapsed, Attribute::READONLY)
                .build();
            Ok(range_obj.into())
        }
    } else {
        Err(JsNativeError::typ().with_message("Selection method called on non-Selection object").into())
    }
}

/// Get composed ranges (Chrome 137 feature).
fn get_composed_ranges(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _shadow_roots = args.get_or_undefined(0);

    // For now, return empty array - in real implementation would handle shadow DOM
    let array = boa_engine::builtins::Array::array_create(0, None, context)?;
    println!("Selection.getComposedRanges called - returning empty array");
    Ok(array.into())
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
        selection_data.anchor_node = Some(anchor_node.clone());
        selection_data.anchor_offset = anchor_offset;
        selection_data.focus_node = Some(focus_node.clone());
        selection_data.focus_offset = focus_offset;
        selection_data.update_state();

        println!("Selection.setBaseAndExtent called - selection updated");
    }

    Ok(JsValue::undefined())
}