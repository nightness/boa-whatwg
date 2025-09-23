//! CharacterData interface implementation for DOM Level 4
//!
//! The CharacterData interface represents Node objects that contain characters.
//! It's an abstract interface for Text, Comment, and ProcessingInstruction nodes.
//! https://dom.spec.whatwg.org/#interface-characterdata

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
use super::node::{NodeData, NodeType};

/// CharacterData data structure for character-containing DOM nodes
#[derive(Debug, Trace, Finalize, JsData)]
pub struct CharacterDataData {
    // Inherit from Node
    node_data: NodeData,
    // CharacterData-specific properties
    data: GcRefCell<String>,
}

impl CharacterDataData {
    /// Create a new CharacterData object
    pub fn new(node_type: NodeType, data: String) -> Self {
        let node_data = NodeData::new(node_type);
        Self {
            node_data,
            data: GcRefCell::new(data),
        }
    }

    /// Get the character data content
    pub fn get_data(&self) -> String {
        self.data.borrow().clone()
    }

    /// Set the character data content
    pub fn set_data(&self, new_data: String) {
        *self.data.borrow_mut() = new_data;
        // Update the node value as well
        self.node_data.set_node_value(Some(self.get_data()));
    }

    /// Get the length of the character data
    pub fn get_length(&self) -> u32 {
        self.data.borrow().chars().count() as u32
    }

    /// Extract a substring from the character data
    pub fn substring_data_impl(&self, offset: u32, count: u32) -> Result<String, String> {
        let data = self.data.borrow();
        let data_len = data.len() as u32;

        if offset > data_len {
            return Err("Index or size is negative or greater than the allowed amount".to_string());
        }

        let end_offset = std::cmp::min(offset + count, data_len);
        let start = offset as usize;
        let end = end_offset as usize;

        Ok(data.chars().skip(start).take(end - start).collect())
    }

    /// Append data to the existing character data
    pub fn append_data_impl(&self, data: String) {
        let mut current_data = self.data.borrow_mut();
        current_data.push_str(&data);
        // Update node value
        self.node_data.set_node_value(Some(current_data.clone()));
    }

    /// Insert data at the specified offset
    pub fn insert_data_impl(&self, offset: u32, data: String) -> Result<(), String> {
        let mut current_data = self.data.borrow_mut();
        let data_len = current_data.len() as u32;

        if offset > data_len {
            return Err("Index or size is negative or greater than the allowed amount".to_string());
        }

        let offset_pos = offset as usize;
        let mut chars: Vec<char> = current_data.chars().collect();
        let new_chars: Vec<char> = data.chars().collect();

        chars.splice(offset_pos..offset_pos, new_chars);
        *current_data = chars.into_iter().collect();

        // Update node value
        self.node_data.set_node_value(Some(current_data.clone()));
        Ok(())
    }

    /// Delete data from the specified offset
    pub fn delete_data_impl(&self, offset: u32, count: u32) -> Result<(), String> {
        let mut current_data = self.data.borrow_mut();
        let data_len = current_data.len() as u32;

        if offset > data_len {
            return Err("Index or size is negative or greater than the allowed amount".to_string());
        }

        let start = offset as usize;
        let end = std::cmp::min(offset + count, data_len) as usize;

        let mut chars: Vec<char> = current_data.chars().collect();
        chars.drain(start..end);
        *current_data = chars.into_iter().collect();

        // Update node value
        self.node_data.set_node_value(Some(current_data.clone()));
        Ok(())
    }

    /// Replace data at the specified offset
    pub fn replace_data_impl(&self, offset: u32, count: u32, data: String) -> Result<(), String> {
        let mut current_data = self.data.borrow_mut();
        let data_len = current_data.len() as u32;

        if offset > data_len {
            return Err("Index or size is negative or greater than the allowed amount".to_string());
        }

        let start = offset as usize;
        let end = std::cmp::min(offset + count, data_len) as usize;

        let mut chars: Vec<char> = current_data.chars().collect();
        let new_chars: Vec<char> = data.chars().collect();

        chars.splice(start..end, new_chars);
        *current_data = chars.into_iter().collect();

        // Update node value
        self.node_data.set_node_value(Some(current_data.clone()));
        Ok(())
    }

    /// Get reference to underlying node data
    pub fn node_data(&self) -> &NodeData {
        &self.node_data
    }
}

impl CharacterDataData {
    /// `CharacterData.prototype.data` getter
    fn get_data_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CharacterData.data called on non-object")
        })?;

        if let Some(char_data) = this_obj.downcast_ref::<CharacterDataData>() {
            Ok(JsValue::from(js_string!(char_data.get_data())))
        } else {
            Err(JsNativeError::typ()
                .with_message("CharacterData.data called on non-CharacterData object")
                .into())
        }
    }

    /// `CharacterData.prototype.data` setter
    fn set_data_accessor(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CharacterData.data called on non-object")
        })?;

        let data_arg = args.get_or_undefined(0);
        let data_string = data_arg.to_string(_context)?;

        if let Some(char_data) = this_obj.downcast_ref::<CharacterDataData>() {
            char_data.set_data(data_string.to_std_string_escaped());
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("CharacterData.data called on non-CharacterData object")
                .into())
        }
    }

    /// `CharacterData.prototype.length` getter
    fn get_length_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CharacterData.length called on non-object")
        })?;

        if let Some(char_data) = this_obj.downcast_ref::<CharacterDataData>() {
            Ok(JsValue::from(char_data.get_length()))
        } else {
            Err(JsNativeError::typ()
                .with_message("CharacterData.length called on non-CharacterData object")
                .into())
        }
    }

    /// `CharacterData.prototype.substringData(offset, count)`
    fn substring_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CharacterData.substringData called on non-object")
        })?;

        let offset_arg = args.get_or_undefined(0);
        let count_arg = args.get_or_undefined(1);

        let offset = offset_arg.to_u32(context)?;
        let count = count_arg.to_u32(context)?;

        if let Some(char_data) = this_obj.downcast_ref::<CharacterDataData>() {
            match char_data.substring_data_impl(offset, count) {
                Ok(result) => Ok(JsValue::from(js_string!(result))),
                Err(err) => Err(JsNativeError::range()
                    .with_message(err)
                    .into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("CharacterData.substringData called on non-CharacterData object")
                .into())
        }
    }

    /// `CharacterData.prototype.appendData(data)`
    fn append_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CharacterData.appendData called on non-object")
        })?;

        let data_arg = args.get_or_undefined(0);
        let data_string = data_arg.to_string(context)?;

        if let Some(char_data) = this_obj.downcast_ref::<CharacterDataData>() {
            char_data.append_data_impl(data_string.to_std_string_escaped());
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("CharacterData.appendData called on non-CharacterData object")
                .into())
        }
    }

    /// `CharacterData.prototype.insertData(offset, data)`
    fn insert_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CharacterData.insertData called on non-object")
        })?;

        let offset_arg = args.get_or_undefined(0);
        let data_arg = args.get_or_undefined(1);

        let offset = offset_arg.to_u32(context)?;
        let data_string = data_arg.to_string(context)?;

        if let Some(char_data) = this_obj.downcast_ref::<CharacterDataData>() {
            match char_data.insert_data_impl(offset, data_string.to_std_string_escaped()) {
                Ok(_) => Ok(JsValue::undefined()),
                Err(err) => Err(JsNativeError::range()
                    .with_message(err)
                    .into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("CharacterData.insertData called on non-CharacterData object")
                .into())
        }
    }

    /// `CharacterData.prototype.deleteData(offset, count)`
    fn delete_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CharacterData.deleteData called on non-object")
        })?;

        let offset_arg = args.get_or_undefined(0);
        let count_arg = args.get_or_undefined(1);

        let offset = offset_arg.to_u32(context)?;
        let count = count_arg.to_u32(context)?;

        if let Some(char_data) = this_obj.downcast_ref::<CharacterDataData>() {
            match char_data.delete_data_impl(offset, count) {
                Ok(_) => Ok(JsValue::undefined()),
                Err(err) => Err(JsNativeError::range()
                    .with_message(err)
                    .into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("CharacterData.deleteData called on non-CharacterData object")
                .into())
        }
    }

    /// `CharacterData.prototype.replaceData(offset, count, data)`
    fn replace_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CharacterData.replaceData called on non-object")
        })?;

        let offset_arg = args.get_or_undefined(0);
        let count_arg = args.get_or_undefined(1);
        let data_arg = args.get_or_undefined(2);

        let offset = offset_arg.to_u32(context)?;
        let count = count_arg.to_u32(context)?;
        let data_string = data_arg.to_string(context)?;

        if let Some(char_data) = this_obj.downcast_ref::<CharacterDataData>() {
            match char_data.replace_data_impl(offset, count, data_string.to_std_string_escaped()) {
                Ok(_) => Ok(JsValue::undefined()),
                Err(err) => Err(JsNativeError::range()
                    .with_message(err)
                    .into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("CharacterData.replaceData called on non-CharacterData object")
                .into())
        }
    }
}

/// The `CharacterData` object
#[derive(Debug, Trace, Finalize)]
pub struct CharacterData;

impl CharacterData {
    // Static method implementations for BuiltInBuilder
    fn get_data_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        CharacterDataData::get_data_accessor(this, args, context)
    }

    fn set_data_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        CharacterDataData::set_data_accessor(this, args, context)
    }

    fn get_length_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        CharacterDataData::get_length_accessor(this, args, context)
    }

    fn substring_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        CharacterDataData::substring_data(this, args, context)
    }

    fn append_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        CharacterDataData::append_data(this, args, context)
    }

    fn insert_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        CharacterDataData::insert_data(this, args, context)
    }

    fn delete_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        CharacterDataData::delete_data(this, args, context)
    }

    fn replace_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        CharacterDataData::replace_data(this, args, context)
    }
}

impl IntrinsicObject for CharacterData {
    fn init(realm: &Realm) {
        let data_get_func = BuiltInBuilder::callable(realm, Self::get_data_accessor)
            .name(js_string!("get data"))
            .build();
        let data_set_func = BuiltInBuilder::callable(realm, Self::set_data_accessor)
            .name(js_string!("set data"))
            .build();
        let length_get_func = BuiltInBuilder::callable(realm, Self::get_length_accessor)
            .name(js_string!("get length"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::substring_data, js_string!("substringData"), 2)
            .method(Self::append_data, js_string!("appendData"), 1)
            .method(Self::insert_data, js_string!("insertData"), 2)
            .method(Self::delete_data, js_string!("deleteData"), 2)
            .method(Self::replace_data, js_string!("replaceData"), 3)
            .accessor(
                js_string!("data"),
                Some(data_get_func),
                Some(data_set_func),
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
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

impl BuiltInObject for CharacterData {
    const NAME: JsString = StaticJsStrings::CHARACTER_DATA;
}

impl BuiltInConstructor for CharacterData {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::character_data;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // CharacterData constructor should not be called directly
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Constructor CharacterData requires 'new'")
                .into());
        }

        // CharacterData is abstract - should not be constructed directly
        Err(JsNativeError::typ()
            .with_message("Illegal constructor")
            .into())
    }
}

#[cfg(test)]
mod tests;