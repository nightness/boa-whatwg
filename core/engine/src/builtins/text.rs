//! Text interface implementation for DOM Level 4
//!
//! The Text interface represents textual content in Element or Attr nodes.
//! It inherits from CharacterData and provides text-specific functionality.
//! https://dom.spec.whatwg.org/#interface-text

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
use super::{character_data::CharacterDataData, node::NodeType};

/// Text data structure for text DOM nodes
#[derive(Debug, Trace, Finalize, JsData)]
pub struct TextData {
    // Inherit from CharacterData
    character_data: CharacterDataData,
    // Text-specific properties
    whole_text: bool,
}

impl TextData {
    /// Create a new Text object
    pub fn new(data: String) -> Self {
        let character_data = CharacterDataData::new(NodeType::Text, data);
        Self {
            character_data,
            whole_text: false,
        }
    }

    /// Get reference to underlying character data
    pub fn character_data(&self) -> &CharacterDataData {
        &self.character_data
    }

    /// Get the whole text content (simplified implementation)
    pub fn get_whole_text(&self) -> String {
        // In a full implementation, this would include adjacent text nodes
        self.character_data.get_data()
    }

    /// Split the text node at the specified offset
    pub fn split_text_impl(&self, offset: u32) -> Result<TextData, String> {
        let data = self.character_data.get_data();
        let data_len = data.len() as u32;

        if offset > data_len {
            return Err("Index or size is negative or greater than the allowed amount".to_string());
        }

        let chars: Vec<char> = data.chars().collect();
        let split_pos = offset as usize;

        let remaining_text: String = chars.iter().skip(split_pos).collect();
        let new_text_node = TextData::new(remaining_text);

        // Update current node to contain only the first part
        let first_part: String = chars.iter().take(split_pos).collect();
        self.character_data.set_data(first_part);

        Ok(new_text_node)
    }

    /// Replace the entire contents of this Text node with the specified text
    pub fn replace_whole_text_impl(&self, content: String) -> Result<(), String> {
        // In a full implementation, this would replace all adjacent text nodes
        self.character_data.set_data(content);
        Ok(())
    }

    /// Check if this text node contains only whitespace
    pub fn is_element_content_whitespace(&self) -> bool {
        self.character_data.get_data().chars().all(|c| c.is_whitespace())
    }

    /// Get the assigned slot (for shadow DOM)
    pub fn get_assigned_slot(&self) -> Option<JsObject> {
        // TODO: Implement when shadow DOM slots are available
        None
    }
}

impl TextData {
    /// `Text.prototype.wholeText` getter
    fn get_whole_text_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Text.wholeText called on non-object")
        })?;

        if let Some(text_data) = this_obj.downcast_ref::<TextData>() {
            Ok(JsValue::from(js_string!(text_data.get_whole_text())))
        } else {
            Err(JsNativeError::typ()
                .with_message("Text.wholeText called on non-Text object")
                .into())
        }
    }

    /// `Text.prototype.assignedSlot` getter
    fn get_assigned_slot_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Text.assignedSlot called on non-object")
        })?;

        if let Some(text_data) = this_obj.downcast_ref::<TextData>() {
            match text_data.get_assigned_slot() {
                Some(slot) => Ok(slot.into()),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Text.assignedSlot called on non-Text object")
                .into())
        }
    }

    /// `Text.prototype.splitText(offset)`
    fn split_text(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Text.splitText called on non-object")
        })?;

        let offset_arg = args.get_or_undefined(0);
        let offset = offset_arg.to_u32(context)?;

        if let Some(text_data) = this_obj.downcast_ref::<TextData>() {
            match text_data.split_text_impl(offset) {
                Ok(new_text_node) => {
                    // Create a new Text object
                    let text_obj = JsObject::from_proto_and_data_with_shared_shape(
                        context.root_shape(),
                        context.intrinsics().constructors().text().prototype(),
                        new_text_node,
                    );
                    Ok(text_obj.into())
                },
                Err(err) => Err(JsNativeError::range()
                    .with_message(err)
                    .into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Text.splitText called on non-Text object")
                .into())
        }
    }

    /// `Text.prototype.replaceWholeText(content)`
    fn replace_whole_text(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Text.replaceWholeText called on non-object")
        })?;

        let content_arg = args.get_or_undefined(0);
        let content_string = content_arg.to_string(context)?;

        if let Some(text_data) = this_obj.downcast_ref::<TextData>() {
            match text_data.replace_whole_text_impl(content_string.to_std_string_escaped()) {
                Ok(_) => Ok(this_obj.clone().into()),
                Err(err) => Err(JsNativeError::error()
                    .with_message(err)
                    .into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Text.replaceWholeText called on non-Text object")
                .into())
        }
    }

    /// Delegate to CharacterData methods
    fn get_data_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Text.data called on non-object")
        })?;

        if let Some(text_data) = this_obj.downcast_ref::<TextData>() {
            Ok(JsValue::from(js_string!(text_data.character_data.get_data())))
        } else {
            Err(JsNativeError::typ()
                .with_message("Text.data called on non-Text object")
                .into())
        }
    }

    fn set_data_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Text.data called on non-object")
        })?;

        let data_arg = args.get_or_undefined(0);
        let data_string = data_arg.to_string(context)?;

        if let Some(text_data) = this_obj.downcast_ref::<TextData>() {
            text_data.character_data.set_data(data_string.to_std_string_escaped());
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("Text.data called on non-Text object")
                .into())
        }
    }

    fn get_length_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Text.length called on non-object")
        })?;

        if let Some(text_data) = this_obj.downcast_ref::<TextData>() {
            Ok(JsValue::from(text_data.character_data.get_length()))
        } else {
            Err(JsNativeError::typ()
                .with_message("Text.length called on non-Text object")
                .into())
        }
    }

    fn substring_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Text.substringData called on non-object")
        })?;

        let offset_arg = args.get_or_undefined(0);
        let count_arg = args.get_or_undefined(1);

        let offset = offset_arg.to_u32(context)?;
        let count = count_arg.to_u32(context)?;

        if let Some(text_data) = this_obj.downcast_ref::<TextData>() {
            match text_data.character_data.substring_data_impl(offset, count) {
                Ok(result) => Ok(JsValue::from(js_string!(result))),
                Err(err) => Err(JsNativeError::range()
                    .with_message(err)
                    .into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Text.substringData called on non-Text object")
                .into())
        }
    }

    fn append_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Text.appendData called on non-object")
        })?;

        let data_arg = args.get_or_undefined(0);
        let data_string = data_arg.to_string(context)?;

        if let Some(text_data) = this_obj.downcast_ref::<TextData>() {
            text_data.character_data.append_data_impl(data_string.to_std_string_escaped());
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("Text.appendData called on non-Text object")
                .into())
        }
    }

    fn insert_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Text.insertData called on non-object")
        })?;

        let offset_arg = args.get_or_undefined(0);
        let data_arg = args.get_or_undefined(1);

        let offset = offset_arg.to_u32(context)?;
        let data_string = data_arg.to_string(context)?;

        if let Some(text_data) = this_obj.downcast_ref::<TextData>() {
            match text_data.character_data.insert_data_impl(offset, data_string.to_std_string_escaped()) {
                Ok(_) => Ok(JsValue::undefined()),
                Err(err) => Err(JsNativeError::range()
                    .with_message(err)
                    .into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Text.insertData called on non-Text object")
                .into())
        }
    }

    fn delete_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Text.deleteData called on non-object")
        })?;

        let offset_arg = args.get_or_undefined(0);
        let count_arg = args.get_or_undefined(1);

        let offset = offset_arg.to_u32(context)?;
        let count = count_arg.to_u32(context)?;

        if let Some(text_data) = this_obj.downcast_ref::<TextData>() {
            match text_data.character_data.delete_data_impl(offset, count) {
                Ok(_) => Ok(JsValue::undefined()),
                Err(err) => Err(JsNativeError::range()
                    .with_message(err)
                    .into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Text.deleteData called on non-Text object")
                .into())
        }
    }

    fn replace_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Text.replaceData called on non-object")
        })?;

        let offset_arg = args.get_or_undefined(0);
        let count_arg = args.get_or_undefined(1);
        let data_arg = args.get_or_undefined(2);

        let offset = offset_arg.to_u32(context)?;
        let count = count_arg.to_u32(context)?;
        let data_string = data_arg.to_string(context)?;

        if let Some(text_data) = this_obj.downcast_ref::<TextData>() {
            match text_data.character_data.replace_data_impl(offset, count, data_string.to_std_string_escaped()) {
                Ok(_) => Ok(JsValue::undefined()),
                Err(err) => Err(JsNativeError::range()
                    .with_message(err)
                    .into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Text.replaceData called on non-Text object")
                .into())
        }
    }
}

/// The `Text` object
#[derive(Debug, Trace, Finalize)]
pub struct Text;

impl Text {
    // Static method implementations for BuiltInBuilder
    fn get_data_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        TextData::get_data_accessor(this, args, context)
    }

    fn set_data_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        TextData::set_data_accessor(this, args, context)
    }

    fn get_length_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        TextData::get_length_accessor(this, args, context)
    }

    fn get_whole_text_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        TextData::get_whole_text_accessor(this, args, context)
    }

    fn get_assigned_slot_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        TextData::get_assigned_slot_accessor(this, args, context)
    }

    fn substring_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        TextData::substring_data(this, args, context)
    }

    fn append_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        TextData::append_data(this, args, context)
    }

    fn insert_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        TextData::insert_data(this, args, context)
    }

    fn delete_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        TextData::delete_data(this, args, context)
    }

    fn replace_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        TextData::replace_data(this, args, context)
    }

    fn split_text(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        TextData::split_text(this, args, context)
    }

    fn replace_whole_text(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        TextData::replace_whole_text(this, args, context)
    }
}

impl IntrinsicObject for Text {
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
        let whole_text_get_func = BuiltInBuilder::callable(realm, Self::get_whole_text_accessor)
            .name(js_string!("get wholeText"))
            .build();
        let assigned_slot_get_func = BuiltInBuilder::callable(realm, Self::get_assigned_slot_accessor)
            .name(js_string!("get assignedSlot"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // CharacterData methods
            .method(Self::substring_data, js_string!("substringData"), 2)
            .method(Self::append_data, js_string!("appendData"), 1)
            .method(Self::insert_data, js_string!("insertData"), 2)
            .method(Self::delete_data, js_string!("deleteData"), 2)
            .method(Self::replace_data, js_string!("replaceData"), 3)
            // Text-specific methods
            .method(Self::split_text, js_string!("splitText"), 1)
            .method(Self::replace_whole_text, js_string!("replaceWholeText"), 1)
            // CharacterData properties
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
            // Text-specific properties
            .accessor(
                js_string!("wholeText"),
                Some(whole_text_get_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("assignedSlot"),
                Some(assigned_slot_get_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Text {
    const NAME: JsString = StaticJsStrings::TEXT;
}

impl BuiltInConstructor for Text {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::text;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Text constructor should be called with 'new'
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Constructor Text requires 'new'")
                .into());
        }

        // Get optional data argument, default to empty string
        let data_arg = args.get_or_undefined(0);
        let data_string = if data_arg.is_undefined() {
            String::new()
        } else {
            data_arg.to_string(context)?.to_std_string_escaped()
        };

        // Create a new Text object
        let text_data = TextData::new(data_string);

        let text_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().text().prototype(),
            text_data,
        );

        Ok(text_obj.into())
    }
}

#[cfg(test)]
mod tests;