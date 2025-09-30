//! NodeList interface implementation for DOM Level 4
//!
//! The NodeList interface represents a collection of nodes.
//! https://dom.spec.whatwg.org/#interface-nodelist

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    string::{StaticJsStrings, JsString},
    Context, JsArgs, JsData, JsNativeError, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace, GcRefCell};

/// The NodeList data implementation
#[derive(Debug, Trace, Finalize, JsData)]
pub struct NodeListData {
    /// The collection of nodes
    nodes: GcRefCell<Vec<JsObject>>,
    /// Whether this is a live NodeList (updates automatically)
    live: bool,
}

impl NodeListData {
    /// Create a new NodeList with the given nodes
    pub fn new(nodes: Vec<JsObject>, live: bool) -> Self {
        Self {
            nodes: GcRefCell::new(nodes),
            live,
        }
    }

    /// Create an empty NodeList
    pub fn empty() -> Self {
        Self::new(Vec::new(), false)
    }

    /// Get the length of the NodeList
    pub fn length(&self) -> usize {
        self.nodes.borrow().len()
    }

    /// Get the node at the specified index
    pub fn get_item(&self, index: usize) -> Option<JsObject> {
        self.nodes.borrow().get(index).cloned()
    }

    /// Get all nodes as a vector
    pub fn nodes(&self) -> Vec<JsObject> {
        self.nodes.borrow().clone()
    }

    /// Add a node to the list (for live NodeLists)
    pub fn add_node(&self, node: JsObject) {
        if self.live {
            self.nodes.borrow_mut().push(node);
        }
    }

    /// Remove a node from the list (for live NodeLists)
    pub fn remove_node(&self, node: &JsObject) {
        if self.live {
            self.nodes.borrow_mut().retain(|n| !std::ptr::eq(n.as_ref(), node.as_ref()));
        }
    }

    /// Clear all nodes (for live NodeLists)
    pub fn clear(&self) {
        if self.live {
            self.nodes.borrow_mut().clear();
        }
    }

    /// Replace all nodes (for static NodeLists)
    pub fn replace_nodes(&self, nodes: Vec<JsObject>) {
        *self.nodes.borrow_mut() = nodes;
    }

    /// Check if this is a live NodeList
    pub fn is_live(&self) -> bool {
        self.live
    }

    /// `NodeList.prototype.length` getter
    fn get_length_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.length called on non-object")
        })?;

        if let Some(nodelist_data) = this_obj.downcast_ref::<NodeListData>() {
            Ok(JsValue::new(nodelist_data.length() as i32))
        } else {
            Err(JsNativeError::typ()
                .with_message("NodeList.length called on non-NodeList object")
                .into())
        }
    }

    /// `NodeList.prototype.item(index)`
    fn item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.item called on non-object")
        })?;

        if let Some(nodelist_data) = this_obj.downcast_ref::<NodeListData>() {
            let index = args.get_or_undefined(0).to_length(context)? as usize;

            match nodelist_data.get_item(index) {
                Some(node) => Ok(node.into()),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("NodeList.item called on non-NodeList object")
                .into())
        }
    }

    /// `NodeList.prototype.forEach(callback, thisArg)`
    fn for_each(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.forEach called on non-object")
        })?;

        if let Some(nodelist_data) = this_obj.downcast_ref::<NodeListData>() {
            let callback = args.get_or_undefined(0);
            let this_arg = args.get_or_undefined(1);

            if !callback.is_callable() {
                return Err(JsNativeError::typ()
                    .with_message("NodeList.forEach callback is not callable")
                    .into());
            }

            let nodes = nodelist_data.nodes();
            for (index, node) in nodes.iter().enumerate() {
                let args = [node.clone().into(), JsValue::new(index), this.clone()];
                callback.call(this_arg, &args, context)?;
            }

            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("NodeList.forEach called on non-NodeList object")
                .into())
        }
    }

    /// `NodeList.prototype.keys()`
    fn keys(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.keys called on non-object")
        })?;

        if let Some(nodelist_data) = this_obj.downcast_ref::<NodeListData>() {
            // Create an array iterator over the indices
            let length = nodelist_data.length();
            let indices: Vec<JsValue> = (0..length).map(|i| JsValue::new(i)).collect();
            let array = crate::builtins::Array::array_create(length as u64, None, context)?;

            for (i, index) in indices.iter().enumerate() {
                array.create_data_property_or_throw(i, index.clone(), context)?;
            }

            // Return array iterator (simplified implementation)
            // In a full implementation, this would return a proper Iterator
            Ok(array.into())
        } else {
            Err(JsNativeError::typ()
                .with_message("NodeList.keys called on non-NodeList object")
                .into())
        }
    }

    /// `NodeList.prototype.values()`
    fn values(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.values called on non-object")
        })?;

        if let Some(nodelist_data) = this_obj.downcast_ref::<NodeListData>() {
            // Create an array with the node values
            let nodes = nodelist_data.nodes();
            let array = crate::builtins::Array::array_create(nodes.len() as u64, None, context)?;

            for (i, node) in nodes.iter().enumerate() {
                array.create_data_property_or_throw(i, node.clone(), context)?;
            }

            // Return array (simplified implementation)
            // In a full implementation, this would return a proper Iterator
            Ok(array.into())
        } else {
            Err(JsNativeError::typ()
                .with_message("NodeList.values called on non-NodeList object")
                .into())
        }
    }

    /// `NodeList.prototype.entries()`
    fn entries(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.entries called on non-object")
        })?;

        if let Some(nodelist_data) = this_obj.downcast_ref::<NodeListData>() {
            // Create an array of [index, node] pairs
            let nodes = nodelist_data.nodes();
            let array = crate::builtins::Array::array_create(nodes.len() as u64, None, context)?;

            for (i, node) in nodes.iter().enumerate() {
                let entry = crate::builtins::Array::array_create(2, None, context)?;
                entry.create_data_property_or_throw(0, JsValue::new(i), context)?;
                entry.create_data_property_or_throw(1, node.clone(), context)?;
                array.create_data_property_or_throw(i, entry, context)?;
            }

            // Return array (simplified implementation)
            // In a full implementation, this would return a proper Iterator
            Ok(array.into())
        } else {
            Err(JsNativeError::typ()
                .with_message("NodeList.entries called on non-NodeList object")
                .into())
        }
    }
}

/// The `NodeList` object
#[derive(Debug, Trace, Finalize)]
pub struct NodeList;

impl NodeList {
    /// Create a new NodeList from a vector of nodes
    pub fn create_from_nodes(nodes: Vec<JsObject>, live: bool, context: &mut Context) -> JsResult<JsObject> {
        let nodelist_data = NodeListData::new(nodes, live);

        let nodelist_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().nodelist().prototype(),
            nodelist_data,
        );

        Ok(nodelist_obj)
    }

    /// Static method implementations for BuiltInBuilder
    fn get_length_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeListData::get_length_accessor(this, args, context)
    }

    fn item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.item called on non-object")
        })?;

        if let Some(nodelist_data) = this_obj.downcast_ref::<NodeListData>() {
            let index = args.get_or_undefined(0).to_length(context)? as usize;
            match nodelist_data.get_item(index) {
                Some(node) => Ok(node.into()),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("NodeList.item called on non-NodeList object")
                .into())
        }
    }

    fn for_each(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.forEach called on non-object")
        })?;

        if let Some(nodelist_data) = this_obj.downcast_ref::<NodeListData>() {
            let callback = args.get_or_undefined(0);
            let this_arg = args.get_or_undefined(1);

            if !callback.is_callable() {
                return Err(JsNativeError::typ()
                    .with_message("NodeList.forEach callback is not callable")
                    .into());
            }

            let nodes = nodelist_data.nodes();
            for (index, node) in nodes.iter().enumerate() {
                let args = [node.clone().into(), JsValue::new(index), this.clone()];
                callback.call(this_arg, &args, context)?;
            }

            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("NodeList.forEach called on non-NodeList object")
                .into())
        }
    }

    fn keys(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.keys called on non-object")
        })?;

        if let Some(nodelist_data) = this_obj.downcast_ref::<NodeListData>() {
            let length = nodelist_data.length();
            let indices: Vec<JsValue> = (0..length).map(|i| JsValue::new(i)).collect();
            let array = crate::builtins::Array::array_create(length as u64, None, context)?;

            for (i, index) in indices.iter().enumerate() {
                array.create_data_property_or_throw(i, index.clone(), context)?;
            }

            Ok(array.into())
        } else {
            Err(JsNativeError::typ()
                .with_message("NodeList.keys called on non-NodeList object")
                .into())
        }
    }

    fn values(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.values called on non-object")
        })?;

        if let Some(nodelist_data) = this_obj.downcast_ref::<NodeListData>() {
            let nodes = nodelist_data.nodes();
            let array = crate::builtins::Array::array_create(nodes.len() as u64, None, context)?;

            for (i, node) in nodes.iter().enumerate() {
                array.create_data_property_or_throw(i, node.clone(), context)?;
            }

            Ok(array.into())
        } else {
            Err(JsNativeError::typ()
                .with_message("NodeList.values called on non-NodeList object")
                .into())
        }
    }

    fn entries(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.entries called on non-object")
        })?;

        if let Some(nodelist_data) = this_obj.downcast_ref::<NodeListData>() {
            let nodes = nodelist_data.nodes();
            let array = crate::builtins::Array::array_create(nodes.len() as u64, None, context)?;

            for (i, node) in nodes.iter().enumerate() {
                let entry = crate::builtins::Array::array_create(2, None, context)?;
                entry.create_data_property_or_throw(0, JsValue::new(i), context)?;
                entry.create_data_property_or_throw(1, node.clone(), context)?;
                array.create_data_property_or_throw(i, entry, context)?;
            }

            Ok(array.into())
        } else {
            Err(JsNativeError::typ()
                .with_message("NodeList.entries called on non-NodeList object")
                .into())
        }
    }
}

impl IntrinsicObject for NodeList {
    fn init(realm: &Realm) {
        let length_get_func = BuiltInBuilder::callable(realm, Self::get_length_accessor)
            .name(js_string!("get length"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Methods
            .method(Self::item, js_string!("item"), 1)
            .method(Self::for_each, js_string!("forEach"), 1)
            .method(Self::keys, js_string!("keys"), 0)
            .method(Self::values, js_string!("values"), 0)
            .method(Self::entries, js_string!("entries"), 0)
            // Properties
            .accessor(
                js_string!("length"),
                Some(length_get_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for NodeList {
    const NAME: JsString = StaticJsStrings::NODELIST;
}

impl BuiltInConstructor for NodeList {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::nodelist;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // NodeList constructor should be called with 'new'
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Constructor NodeList requires 'new'")
                .into());
        }

        // Create a new empty NodeList object
        let nodelist_data = NodeListData::empty();

        let nodelist_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().nodelist().prototype(),
            nodelist_data,
        );

        Ok(nodelist_obj.into())
    }
}

#[cfg(test)]
mod tests;