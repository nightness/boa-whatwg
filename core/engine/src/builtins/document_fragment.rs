//! DocumentFragment interface implementation for DOM Level 4
//!
//! The DocumentFragment interface represents a minimal document object that has no parent.
//! It is used as a lightweight version of Document that stores a segment of a document structure
//! comprised of nodes just like a standard document.
//! https://dom.spec.whatwg.org/#interface-documentfragment

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    property::{Attribute, PropertyDescriptorBuilder},
    realm::Realm,
    string::{StaticJsStrings, JsString},
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult,
};
use boa_gc::{Finalize, Trace};
use boa_gc::GcRefCell;
use std::collections::HashMap;
use super::node::{NodeData, NodeType};

/// DocumentFragment data structure for lightweight document containers
#[derive(Debug, Trace, Finalize, JsData)]
pub struct DocumentFragmentData {
    // Inherit from Node
    node_data: NodeData,
    // DocumentFragment-specific properties
    children: GcRefCell<Vec<JsObject>>,
    element_children: GcRefCell<Vec<JsObject>>,
}

impl DocumentFragmentData {
    /// Create a new DocumentFragment object
    pub fn new() -> Self {
        let node_data = NodeData::new(NodeType::DocumentFragment);
        Self {
            node_data,
            children: GcRefCell::new(Vec::new()),
            element_children: GcRefCell::new(Vec::new()),
        }
    }

    /// Get the number of child elements
    pub fn get_child_element_count(&self) -> u32 {
        self.element_children.borrow().len() as u32
    }

    /// Get the first element child
    pub fn get_first_element_child(&self) -> Option<JsObject> {
        self.element_children.borrow().first().cloned()
    }

    /// Get the last element child
    pub fn get_last_element_child(&self) -> Option<JsObject> {
        self.element_children.borrow().last().cloned()
    }

    /// Get all element children as a collection
    pub fn get_children(&self) -> Vec<JsObject> {
        self.element_children.borrow().clone()
    }

    /// Append a node to the fragment
    pub fn append_impl(&self, node: JsObject) -> Result<(), String> {
        // Add to children list
        self.children.borrow_mut().push(node.clone());

        // If it's an element, add to element children too
        // Note: In a full implementation, we'd check the node type
        self.element_children.borrow_mut().push(node);

        Ok(())
    }

    /// Prepend a node to the fragment
    pub fn prepend_impl(&self, node: JsObject) -> Result<(), String> {
        // Add to beginning of children list
        self.children.borrow_mut().insert(0, node.clone());

        // If it's an element, add to element children too
        self.element_children.borrow_mut().insert(0, node);

        Ok(())
    }

    /// Replace all children with new nodes
    pub fn replace_children_impl(&self, nodes: Vec<JsObject>) -> Result<(), String> {
        // Clear existing children
        self.children.borrow_mut().clear();
        self.element_children.borrow_mut().clear();

        // Add new children
        for node in nodes {
            self.children.borrow_mut().push(node.clone());
            self.element_children.borrow_mut().push(node);
        }

        Ok(())
    }

    /// Query selector implementation (simplified)
    pub fn query_selector_impl(&self, _selectors: String) -> Result<Option<JsObject>, String> {
        // Simplified implementation - in reality would parse CSS selectors
        // For now, just return the first element child if any
        Ok(self.get_first_element_child())
    }

    /// Query selector all implementation (simplified)
    pub fn query_selector_all_impl(&self, _selectors: String) -> Result<Vec<JsObject>, String> {
        // Simplified implementation - return all element children
        Ok(self.get_children())
    }

    /// Get element by ID implementation (simplified)
    pub fn get_element_by_id_impl(&self, _id: String) -> Result<Option<JsObject>, String> {
        // Simplified implementation - would normally check id attributes
        Ok(self.get_first_element_child())
    }

    /// Move a node before another node (simplified)
    pub fn move_before_impl(&self, _node: JsObject, _reference_node: Option<JsObject>) -> Result<(), String> {
        // Simplified implementation
        Ok(())
    }

    /// Get reference to underlying node data
    pub fn node_data(&self) -> &NodeData {
        &self.node_data
    }
}

impl DocumentFragmentData {
    /// `DocumentFragment.prototype.childElementCount` getter
    fn get_child_element_count_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("DocumentFragment.childElementCount called on non-object")
        })?;

        if let Some(fragment_data) = this_obj.downcast_ref::<DocumentFragmentData>() {
            Ok(JsValue::from(fragment_data.get_child_element_count()))
        } else {
            Err(JsNativeError::typ()
                .with_message("DocumentFragment.childElementCount called on non-DocumentFragment object")
                .into())
        }
    }

    /// `DocumentFragment.prototype.firstElementChild` getter
    fn get_first_element_child_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("DocumentFragment.firstElementChild called on non-object")
        })?;

        if let Some(fragment_data) = this_obj.downcast_ref::<DocumentFragmentData>() {
            match fragment_data.get_first_element_child() {
                Some(element) => Ok(element.into()),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("DocumentFragment.firstElementChild called on non-DocumentFragment object")
                .into())
        }
    }

    /// `DocumentFragment.prototype.lastElementChild` getter
    fn get_last_element_child_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("DocumentFragment.lastElementChild called on non-object")
        })?;

        if let Some(fragment_data) = this_obj.downcast_ref::<DocumentFragmentData>() {
            match fragment_data.get_last_element_child() {
                Some(element) => Ok(element.into()),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("DocumentFragment.lastElementChild called on non-DocumentFragment object")
                .into())
        }
    }

    /// `DocumentFragment.prototype.children` getter
    fn get_children_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("DocumentFragment.children called on non-object")
        })?;

        if let Some(fragment_data) = this_obj.downcast_ref::<DocumentFragmentData>() {
            let children = fragment_data.get_children();
            // In a full implementation, this would return an HTMLCollection
            // For now, return an array-like object
            let array = crate::builtins::Array::array_create(children.len() as u64, None, _context)?;
            for (i, child) in children.iter().enumerate() {
                array.create_data_property_or_throw(i, child.clone(), _context)?;
            }
            Ok(array.into())
        } else {
            Err(JsNativeError::typ()
                .with_message("DocumentFragment.children called on non-DocumentFragment object")
                .into())
        }
    }

    /// `DocumentFragment.prototype.append(...nodes)`
    fn append(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("DocumentFragment.append called on non-object")
        })?;

        if let Some(fragment_data) = this_obj.downcast_ref::<DocumentFragmentData>() {
            for arg in args {
                if let Some(node_obj) = arg.as_object() {
                    match fragment_data.append_impl(node_obj.clone()) {
                        Ok(_) => {},
                        Err(err) => return Err(JsNativeError::error()
                            .with_message(err)
                            .into()),
                    }
                }
                // Note: In full implementation, would also handle string arguments
            }
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("DocumentFragment.append called on non-DocumentFragment object")
                .into())
        }
    }

    /// `DocumentFragment.prototype.prepend(...nodes)`
    fn prepend(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("DocumentFragment.prepend called on non-object")
        })?;

        if let Some(fragment_data) = this_obj.downcast_ref::<DocumentFragmentData>() {
            // Process arguments in normal order, inserting each at position 0
            // This results in final order: [last_arg, second_last_arg, first_arg, existing_children]
            for arg in args.iter() {
                if let Some(node_obj) = arg.as_object() {
                    fragment_data.children.borrow_mut().insert(0, node_obj.clone());
                    fragment_data.element_children.borrow_mut().insert(0, node_obj.clone());
                }
            }

            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("DocumentFragment.prepend called on non-DocumentFragment object")
                .into())
        }
    }

    /// `DocumentFragment.prototype.replaceChildren(...nodes)`
    fn replace_children(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("DocumentFragment.replaceChildren called on non-object")
        })?;

        if let Some(fragment_data) = this_obj.downcast_ref::<DocumentFragmentData>() {
            let mut nodes = Vec::new();
            for arg in args {
                if let Some(node_obj) = arg.as_object() {
                    nodes.push(node_obj.clone());
                }
            }

            match fragment_data.replace_children_impl(nodes) {
                Ok(_) => Ok(JsValue::undefined()),
                Err(err) => Err(JsNativeError::error()
                    .with_message(err)
                    .into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("DocumentFragment.replaceChildren called on non-DocumentFragment object")
                .into())
        }
    }

    /// `DocumentFragment.prototype.querySelector(selectors)`
    fn query_selector(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("DocumentFragment.querySelector called on non-object")
        })?;

        let selectors_arg = args.get_or_undefined(0);
        let selectors = selectors_arg.to_string(context)?;

        if let Some(fragment_data) = this_obj.downcast_ref::<DocumentFragmentData>() {
            match fragment_data.query_selector_impl(selectors.to_std_string_escaped()) {
                Ok(Some(element)) => Ok(element.into()),
                Ok(None) => Ok(JsValue::null()),
                Err(err) => Err(JsNativeError::error()
                    .with_message(err)
                    .into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("DocumentFragment.querySelector called on non-DocumentFragment object")
                .into())
        }
    }

    /// `DocumentFragment.prototype.querySelectorAll(selectors)`
    fn query_selector_all(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("DocumentFragment.querySelectorAll called on non-object")
        })?;

        let selectors_arg = args.get_or_undefined(0);
        let selectors = selectors_arg.to_string(context)?;

        if let Some(fragment_data) = this_obj.downcast_ref::<DocumentFragmentData>() {
            match fragment_data.query_selector_all_impl(selectors.to_std_string_escaped()) {
                Ok(elements) => {
                    // Return as NodeList-like array
                    let array = crate::builtins::Array::array_create(elements.len() as u64, None, context)?;
                    for (i, element) in elements.iter().enumerate() {
                        array.create_data_property_or_throw(i, element.clone(), context)?;
                    }
                    Ok(array.into())
                },
                Err(err) => Err(JsNativeError::error()
                    .with_message(err)
                    .into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("DocumentFragment.querySelectorAll called on non-DocumentFragment object")
                .into())
        }
    }

    /// `DocumentFragment.prototype.getElementById(id)`
    fn get_element_by_id(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("DocumentFragment.getElementById called on non-object")
        })?;

        let id_arg = args.get_or_undefined(0);
        let id = id_arg.to_string(context)?;

        if let Some(fragment_data) = this_obj.downcast_ref::<DocumentFragmentData>() {
            match fragment_data.get_element_by_id_impl(id.to_std_string_escaped()) {
                Ok(Some(element)) => Ok(element.into()),
                Ok(None) => Ok(JsValue::null()),
                Err(err) => Err(JsNativeError::error()
                    .with_message(err)
                    .into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("DocumentFragment.getElementById called on non-DocumentFragment object")
                .into())
        }
    }
}

/// The `DocumentFragment` object
#[derive(Debug, Trace, Finalize)]
pub struct DocumentFragment;

impl DocumentFragment {
    // Static method implementations for BuiltInBuilder
    fn get_child_element_count_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        DocumentFragmentData::get_child_element_count_accessor(this, args, context)
    }

    fn get_first_element_child_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        DocumentFragmentData::get_first_element_child_accessor(this, args, context)
    }

    fn get_last_element_child_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        DocumentFragmentData::get_last_element_child_accessor(this, args, context)
    }

    fn get_children_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        DocumentFragmentData::get_children_accessor(this, args, context)
    }

    fn append(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        DocumentFragmentData::append(this, args, context)
    }

    fn prepend(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        DocumentFragmentData::prepend(this, args, context)
    }

    fn replace_children(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        DocumentFragmentData::replace_children(this, args, context)
    }

    fn query_selector(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        DocumentFragmentData::query_selector(this, args, context)
    }

    fn query_selector_all(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        DocumentFragmentData::query_selector_all(this, args, context)
    }

    fn get_element_by_id(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        DocumentFragmentData::get_element_by_id(this, args, context)
    }
}

impl IntrinsicObject for DocumentFragment {
    fn init(realm: &Realm) {
        let child_element_count_get_func = BuiltInBuilder::callable(realm, Self::get_child_element_count_accessor)
            .name(js_string!("get childElementCount"))
            .build();
        let first_element_child_get_func = BuiltInBuilder::callable(realm, Self::get_first_element_child_accessor)
            .name(js_string!("get firstElementChild"))
            .build();
        let last_element_child_get_func = BuiltInBuilder::callable(realm, Self::get_last_element_child_accessor)
            .name(js_string!("get lastElementChild"))
            .build();
        let children_get_func = BuiltInBuilder::callable(realm, Self::get_children_accessor)
            .name(js_string!("get children"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Methods
            .method(Self::append, js_string!("append"), 0)
            .method(Self::prepend, js_string!("prepend"), 0)
            .method(Self::replace_children, js_string!("replaceChildren"), 0)
            .method(Self::query_selector, js_string!("querySelector"), 1)
            .method(Self::query_selector_all, js_string!("querySelectorAll"), 1)
            .method(Self::get_element_by_id, js_string!("getElementById"), 1)
            // Properties using accessor pattern like Text
            .accessor(
                js_string!("childElementCount"),
                Some(child_element_count_get_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("firstElementChild"),
                Some(first_element_child_get_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("lastElementChild"),
                Some(last_element_child_get_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("children"),
                Some(children_get_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for DocumentFragment {
    const NAME: JsString = StaticJsStrings::DOCUMENT_FRAGMENT;
}

impl BuiltInConstructor for DocumentFragment {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::document_fragment;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // DocumentFragment constructor should be called with 'new'
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Constructor DocumentFragment requires 'new'")
                .into());
        }

        // Create a new DocumentFragment object
        let fragment_data = DocumentFragmentData::new();

        let fragment_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().document_fragment().prototype(),
            fragment_data,
        );

        Ok(fragment_obj.into())
    }
}

#[cfg(test)]
mod tests;