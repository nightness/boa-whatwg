//! HTMLSlotElement interface implementation for Shadow DOM
//!
//! The HTMLSlotElement interface represents <slot> elements in shadow trees.
//! https://dom.spec.whatwg.org/#interface-htmlslotelement

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    builtins::element::ElementData,
    builtins::shadow_root::ShadowRootData,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    property::{Attribute, PropertyDescriptorBuilder},
    realm::Realm,
    string::{StaticJsStrings, JsString},
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult,
};
use boa_gc::{Finalize, Trace, GcRefCell};
use std::collections::{HashMap, VecDeque};

/// HTMLSlotElement data structure implementing the slot assignment algorithm
#[derive(Debug, Trace, Finalize, JsData)]
pub struct HTMLSlotElementData {
    // Inherit from Element
    element_data: ElementData,
    // Slot-specific properties
    name: GcRefCell<String>,
    assigned_nodes: GcRefCell<Vec<JsObject>>,
    manual_slot_assignment: GcRefCell<bool>,
}

impl HTMLSlotElementData {
    /// Create a new HTMLSlotElement
    pub fn new() -> Self {
        Self {
            element_data: ElementData::with_tag_name("slot".to_string()),
            name: GcRefCell::new(String::new()),
            assigned_nodes: GcRefCell::new(Vec::new()),
            manual_slot_assignment: GcRefCell::new(false),
        }
    }

    /// Get the slot name
    pub fn get_name(&self) -> String {
        self.name.borrow().clone()
    }

    /// Set the slot name
    pub fn set_name(&self, name: String) {
        *self.name.borrow_mut() = name;
    }

    /// Get assigned nodes (for slotAssignment: manual)
    pub fn get_assigned_nodes(&self) -> Vec<JsObject> {
        self.assigned_nodes.borrow().clone()
    }

    /// Assign nodes manually (for slotAssignment: manual)
    pub fn assign_nodes(&self, nodes: Vec<JsObject>) {
        *self.assigned_nodes.borrow_mut() = nodes;
        // TODO: Fire slotchange event
    }

    /// Assign a single node to this slot (manual slot assignment)
    pub fn assign_node(&self, node: JsObject) {
        if self.manual_slot_assignment.borrow().clone() {
            self.assigned_nodes.borrow_mut().push(node);
        }
    }

    /// Clear all assigned nodes
    pub fn clear_assigned_nodes(&self) {
        self.assigned_nodes.borrow_mut().clear();
    }

    /// Get assigned elements (filter assigned nodes to elements only)
    pub fn get_assigned_elements(&self) -> Vec<JsObject> {
        self.assigned_nodes.borrow()
            .iter()
            .filter(|node| {
                // In a full implementation, check if node is an Element
                // For now, assume all assigned nodes are elements
                true
            })
            .cloned()
            .collect()
    }

    /// Find slottables for this slot according to WHATWG algorithm
    pub fn find_slottables(&self, shadow_root: &ShadowRootData) -> Vec<JsObject> {
        let mut slottables = Vec::new();
        let slot_name = self.get_name();

        // Implementation of "find slottables" algorithm from WHATWG spec
        // https://dom.spec.whatwg.org/#find-slottables

        if let Some(host) = shadow_root.get_host() {
            // Get all children of the shadow host
            if let Some(element_data) = host.downcast_ref::<ElementData>() {
                let children = element_data.get_children();

                for child in children {
                    if self.is_slottable(&child) {
                        let child_slot_name = self.get_slottable_name(&child);

                        // Match slot name (empty string matches unnamed slots)
                        if (slot_name.is_empty() && child_slot_name.is_empty()) ||
                           (slot_name == child_slot_name) {
                            slottables.push(child);
                        }
                    }
                }
            }
        }

        slottables
    }

    /// Check if a node is slottable
    fn is_slottable(&self, node: &JsObject) -> bool {
        // Per WHATWG spec: Elements and Text nodes are slottable
        // For now, assume all nodes are slottable (simplified)
        true
    }

    /// Get the slot attribute value of a slottable
    fn get_slottable_name(&self, node: &JsObject) -> String {
        if let Some(element_data) = node.downcast_ref::<ElementData>() {
            element_data.get_attribute("slot").unwrap_or_default()
        } else {
            // Text nodes don't have slot attribute
            String::new()
        }
    }

    /// Assign slottables to this slot (WHATWG assign slottables algorithm)
    pub fn assign_slottables(&self, shadow_root: &ShadowRootData) -> Vec<JsObject> {
        // Implementation of "assign slottables" algorithm
        // https://dom.spec.whatwg.org/#assign-slottables

        let slottables = if *self.manual_slot_assignment.borrow() {
            // For manual slot assignment, use explicitly assigned nodes
            self.get_assigned_nodes()
        } else {
            // For automatic slot assignment, find slottables
            self.find_slottables(shadow_root)
        };

        // Update each slottable's assigned slot
        for slottable in &slottables {
            // TODO: Set slottable's assigned slot to this slot
        }

        slottables
    }

    /// Get reference to underlying element data
    pub fn element_data(&self) -> &ElementData {
        &self.element_data
    }
}

/// JavaScript accessor implementations
impl HTMLSlotElementData {
    /// `HTMLSlotElement.prototype.name` getter
    fn get_name_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLSlotElement.name called on non-object")
        })?;

        if let Some(slot_data) = this_obj.downcast_ref::<HTMLSlotElementData>() {
            Ok(JsValue::from(js_string!(slot_data.get_name())))
        } else {
            Err(JsNativeError::typ()
                .with_message("HTMLSlotElement.name called on non-HTMLSlotElement object")
                .into())
        }
    }

    /// `HTMLSlotElement.prototype.name` setter
    fn set_name_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLSlotElement.name setter called on non-object")
        })?;

        let name_value = args.get_or_undefined(0);
        let name_string = name_value.to_string(context)?;

        if let Some(slot_data) = this_obj.downcast_ref::<HTMLSlotElementData>() {
            let new_name = name_string.to_std_string_escaped();
            slot_data.set_name(new_name.clone());

            // Update the name attribute on the element
            slot_data.element_data().set_attribute("name".to_string(), new_name);

            // TODO: Run assign slottables algorithm
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("HTMLSlotElement.name setter called on non-HTMLSlotElement object")
                .into())
        }
    }

    /// `HTMLSlotElement.prototype.assignedNodes(options)`
    fn assigned_nodes(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLSlotElement.assignedNodes called on non-object")
        })?;

        if let Some(slot_data) = this_obj.downcast_ref::<HTMLSlotElementData>() {
            let options = args.get_or_undefined(0);

            // Parse options.flatten (default: false)
            let flatten = if let Some(options_obj) = options.as_object() {
                if let Ok(flatten_value) = options_obj.get(js_string!("flatten"), context) {
                    flatten_value.to_boolean()
                } else {
                    false
                }
            } else {
                false
            };

            let nodes = if flatten {
                // TODO: Implement flattened assigned nodes (includes nested slots)
                slot_data.get_assigned_nodes()
            } else {
                slot_data.get_assigned_nodes()
            };

            // Convert to array
            let array = crate::builtins::Array::array_create(nodes.len() as u64, None, context)?;
            for (i, node) in nodes.iter().enumerate() {
                array.create_data_property_or_throw(i, node.clone(), context)?;
            }
            Ok(array.into())
        } else {
            Err(JsNativeError::typ()
                .with_message("HTMLSlotElement.assignedNodes called on non-HTMLSlotElement object")
                .into())
        }
    }

    /// `HTMLSlotElement.prototype.assignedElements(options)`
    fn assigned_elements(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLSlotElement.assignedElements called on non-object")
        })?;

        if let Some(slot_data) = this_obj.downcast_ref::<HTMLSlotElementData>() {
            let options = args.get_or_undefined(0);

            // Parse options.flatten (default: false)
            let flatten = if let Some(options_obj) = options.as_object() {
                if let Ok(flatten_value) = options_obj.get(js_string!("flatten"), context) {
                    flatten_value.to_boolean()
                } else {
                    false
                }
            } else {
                false
            };

            let elements = if flatten {
                // TODO: Implement flattened assigned elements
                slot_data.get_assigned_elements()
            } else {
                slot_data.get_assigned_elements()
            };

            // Convert to array
            let array = crate::builtins::Array::array_create(elements.len() as u64, None, context)?;
            for (i, element) in elements.iter().enumerate() {
                array.create_data_property_or_throw(i, element.clone(), context)?;
            }
            Ok(array.into())
        } else {
            Err(JsNativeError::typ()
                .with_message("HTMLSlotElement.assignedElements called on non-HTMLSlotElement object")
                .into())
        }
    }

    /// `HTMLSlotElement.prototype.assign(...nodes)` (for manual slot assignment)
    fn assign(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLSlotElement.assign called on non-object")
        })?;

        if let Some(slot_data) = this_obj.downcast_ref::<HTMLSlotElementData>() {
            let mut nodes = Vec::new();

            // Convert arguments to nodes
            for arg in args {
                if let Some(node_obj) = arg.as_object() {
                    nodes.push(node_obj.clone());
                }
                // TODO: Handle strings (convert to text nodes)
            }

            slot_data.assign_nodes(nodes);
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("HTMLSlotElement.assign called on non-HTMLSlotElement object")
                .into())
        }
    }
}

/// The `HTMLSlotElement` object
#[derive(Debug, Trace, Finalize)]
pub struct HTMLSlotElement;

impl HTMLSlotElement {
    // Static method implementations for BuiltInBuilder
    fn get_name_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        HTMLSlotElementData::get_name_accessor(this, args, context)
    }

    fn set_name_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        HTMLSlotElementData::set_name_accessor(this, args, context)
    }

    fn assigned_nodes(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        HTMLSlotElementData::assigned_nodes(this, args, context)
    }

    fn assigned_elements(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        HTMLSlotElementData::assigned_elements(this, args, context)
    }

    fn assign(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        HTMLSlotElementData::assign(this, args, context)
    }

    /// Create a new HTMLSlotElement instance
    pub fn create_slot_element(context: &mut Context) -> JsResult<JsObject> {
        let slot_data = HTMLSlotElementData::new();

        let slot_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().html_slot_element().prototype(),
            slot_data,
        );

        Ok(slot_obj)
    }
}

impl IntrinsicObject for HTMLSlotElement {
    fn init(realm: &Realm) {
        let name_get_func = BuiltInBuilder::callable(realm, Self::get_name_accessor)
            .name(js_string!("get name"))
            .build();
        let name_set_func = BuiltInBuilder::callable(realm, Self::set_name_accessor)
            .name(js_string!("set name"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Properties
            .accessor(
                js_string!("name"),
                Some(name_get_func),
                Some(name_set_func),
                Attribute::CONFIGURABLE,
            )
            // Methods
            .method(Self::assigned_nodes, js_string!("assignedNodes"), 0)
            .method(Self::assigned_elements, js_string!("assignedElements"), 0)
            .method(Self::assign, js_string!("assign"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLSlotElement {
    const NAME: JsString = StaticJsStrings::HTML_SLOT_ELEMENT;
}

impl BuiltInConstructor for HTMLSlotElement {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_slot_element;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // HTMLSlotElement constructor should be called with 'new'
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Constructor HTMLSlotElement requires 'new'")
                .into());
        }

        let slot_obj = Self::create_slot_element(context)?;
        Ok(slot_obj.into())
    }
}

/// Global slot assignment algorithms per WHATWG spec
pub struct SlotAssignmentAlgorithms;

impl SlotAssignmentAlgorithms {
    /// Assign slottables for a shadow root (main entry point)
    pub fn assign_slottables_for_tree(shadow_root: &ShadowRootData) {
        // Implementation of "assign slottables for a tree" algorithm
        // https://dom.spec.whatwg.org/#assign-slottables-for-a-tree

        let slots = shadow_root.find_slots();

        // Signal slot change for each slot that needs it
        for slot in slots {
            if let Some(slot_data) = slot.downcast_ref::<HTMLSlotElementData>() {
                let old_assigned = slot_data.get_assigned_nodes();
                let new_assigned = slot_data.assign_slottables(shadow_root);

                // If assignment changed, signal slot change
                if old_assigned != new_assigned {
                    Self::signal_slot_change(&slot);
                }
            }
        }
    }

    /// Signal slot change event
    fn signal_slot_change(slot: &JsObject) {
        // TODO: Implement slotchange event firing
        // This should fire a slotchange event on the slot element
    }

    /// Find all flattened assigned nodes for a slot
    pub fn find_flattened_assigned_nodes(slot: &JsObject) -> Vec<JsObject> {
        // Implementation of flattened assigned nodes algorithm
        // This recursively flattens nested slot assignments
        let mut result = Vec::new();

        if let Some(slot_data) = slot.downcast_ref::<HTMLSlotElementData>() {
            let assigned = slot_data.get_assigned_nodes();

            for node in assigned {
                if Self::is_slot_element(&node) {
                    // Recursively get flattened nodes from nested slot
                    let nested_nodes = Self::find_flattened_assigned_nodes(&node);
                    result.extend(nested_nodes);
                } else {
                    result.push(node);
                }
            }
        }

        result
    }

    /// Check if a node is a slot element
    fn is_slot_element(node: &JsObject) -> bool {
        node.downcast_ref::<HTMLSlotElementData>().is_some()
    }
}

#[cfg(test)]
mod tests;