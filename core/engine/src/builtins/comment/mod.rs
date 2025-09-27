//! Comment interface implementation for DOM Level 4
//!
//! The Comment interface represents textual notations within markup.
//! It inherits from CharacterData and represents comments in XML/HTML.
//! https://dom.spec.whatwg.org/#interface-comment

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    property::Attribute,
    realm::Realm,
    string::{StaticJsStrings, JsString},
    Context, JsArgs, JsData, JsNativeError, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace, GcRefCell};

/// The Comment data implementation
#[derive(Debug, Trace, Finalize, JsData)]
pub struct CommentData {
    /// The text content of the comment
    data: GcRefCell<String>,
}

impl CommentData {
    /// Create a new Comment with data
    pub fn new(data: String) -> Self {
        Self {
            data: GcRefCell::new(data),
        }
    }

    /// Get the comment data
    pub fn data(&self) -> String {
        self.data.borrow().clone()
    }

    /// Set the comment data
    pub fn set_data(&self, data: String) {
        *self.data.borrow_mut() = data;
    }

    /// Get the length of the comment data
    pub fn length(&self) -> u32 {
        self.data.borrow().chars().count() as u32
    }

    /// Extract a substring of the comment data
    pub fn substring_data(&self, offset: u32, count: u32) -> Result<String, String> {
        let data = self.data.borrow();
        let chars: Vec<char> = data.chars().collect();
        let len = chars.len() as u32;

        if offset > len {
            return Err("INDEX_SIZE_ERR: offset exceeds data length".to_string());
        }

        let end_offset = std::cmp::min(offset + count, len);
        let substring: String = chars[offset as usize..end_offset as usize].iter().collect();
        Ok(substring)
    }

    /// Append data to the comment
    pub fn append_data(&self, data: String) {
        self.data.borrow_mut().push_str(&data);
    }

    /// Insert data at the specified offset
    pub fn insert_data(&self, offset: u32, data: String) -> Result<(), String> {
        let mut current_data = self.data.borrow_mut();
        let chars: Vec<char> = current_data.chars().collect();
        let len = chars.len() as u32;

        if offset > len {
            return Err("INDEX_SIZE_ERR: offset exceeds data length".to_string());
        }

        let (before, after) = chars.split_at(offset as usize);
        let mut new_data = String::new();
        new_data.extend(before.iter());
        new_data.push_str(&data);
        new_data.extend(after.iter());

        *current_data = new_data;
        Ok(())
    }

    /// Delete data from the specified offset
    pub fn delete_data(&self, offset: u32, count: u32) -> Result<(), String> {
        let mut current_data = self.data.borrow_mut();
        let chars: Vec<char> = current_data.chars().collect();
        let len = chars.len() as u32;

        if offset > len {
            return Err("INDEX_SIZE_ERR: offset exceeds data length".to_string());
        }

        let end_offset = std::cmp::min(offset + count, len);
        let mut new_data = String::new();
        new_data.extend(chars[0..offset as usize].iter());
        new_data.extend(chars[end_offset as usize..].iter());

        *current_data = new_data;
        Ok(())
    }

    /// Replace data in the specified range
    pub fn replace_data(&self, offset: u32, count: u32, data: String) -> Result<(), String> {
        self.delete_data(offset, count)?;
        self.insert_data(offset, data)?;
        Ok(())
    }
}

/// The `Comment` object
#[derive(Debug, Trace, Finalize)]
pub struct Comment;

impl Comment {
    /// Create a new Comment
    pub fn create(context: &mut Context, data: String) -> JsResult<JsObject> {
        let comment_data = CommentData::new(data);

        let comment_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().comment().prototype(),
            comment_data,
        );

        Ok(comment_obj)
    }

    /// Get the data property
    fn data(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Comment.prototype.data called on non-object")
        })?;

        if let Some(comment_data) = this_obj.downcast_ref::<CommentData>() {
            Ok(JsString::from(comment_data.data()).into())
        } else {
            Err(JsNativeError::typ()
                .with_message("Comment.prototype.data called on non-Comment object")
                .into())
        }
    }

    /// Set the data property
    fn set_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Comment.prototype.data setter called on non-object")
        })?;

        if let Some(comment_data) = this_obj.downcast_ref::<CommentData>() {
            let new_data = args.get_or_undefined(0).to_string(context)?;
            comment_data.set_data(new_data.to_std_string().unwrap_or_default());
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("Comment.prototype.data setter called on non-Comment object")
                .into())
        }
    }

    /// Get the length property
    fn length(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Comment.prototype.length called on non-object")
        })?;

        if let Some(comment_data) = this_obj.downcast_ref::<CommentData>() {
            Ok(JsValue::new(comment_data.length()))
        } else {
            Err(JsNativeError::typ()
                .with_message("Comment.prototype.length called on non-Comment object")
                .into())
        }
    }

    /// substringData method
    fn substring_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Comment.prototype.substringData called on non-object")
        })?;

        if let Some(comment_data) = this_obj.downcast_ref::<CommentData>() {
            let offset = args.get_or_undefined(0).to_u32(context)?;
            let count = args.get_or_undefined(1).to_u32(context)?;

            match comment_data.substring_data(offset, count) {
                Ok(substring) => Ok(JsString::from(substring).into()),
                Err(err) => Err(JsNativeError::range().with_message(err).into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Comment.prototype.substringData called on non-Comment object")
                .into())
        }
    }

    /// appendData method
    fn append_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Comment.prototype.appendData called on non-object")
        })?;

        if let Some(comment_data) = this_obj.downcast_ref::<CommentData>() {
            let data = args.get_or_undefined(0).to_string(context)?;
            comment_data.append_data(data.to_std_string().unwrap_or_default());
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("Comment.prototype.appendData called on non-Comment object")
                .into())
        }
    }

    /// insertData method
    fn insert_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Comment.prototype.insertData called on non-object")
        })?;

        if let Some(comment_data) = this_obj.downcast_ref::<CommentData>() {
            let offset = args.get_or_undefined(0).to_u32(context)?;
            let data = args.get_or_undefined(1).to_string(context)?;

            match comment_data.insert_data(offset, data.to_std_string().unwrap_or_default()) {
                Ok(()) => Ok(JsValue::undefined()),
                Err(err) => Err(JsNativeError::range().with_message(err).into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Comment.prototype.insertData called on non-Comment object")
                .into())
        }
    }

    /// deleteData method
    fn delete_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Comment.prototype.deleteData called on non-object")
        })?;

        if let Some(comment_data) = this_obj.downcast_ref::<CommentData>() {
            let offset = args.get_or_undefined(0).to_u32(context)?;
            let count = args.get_or_undefined(1).to_u32(context)?;

            match comment_data.delete_data(offset, count) {
                Ok(()) => Ok(JsValue::undefined()),
                Err(err) => Err(JsNativeError::range().with_message(err).into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Comment.prototype.deleteData called on non-Comment object")
                .into())
        }
    }

    /// replaceData method
    fn replace_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Comment.prototype.replaceData called on non-object")
        })?;

        if let Some(comment_data) = this_obj.downcast_ref::<CommentData>() {
            let offset = args.get_or_undefined(0).to_u32(context)?;
            let count = args.get_or_undefined(1).to_u32(context)?;
            let data = args.get_or_undefined(2).to_string(context)?;

            match comment_data.replace_data(offset, count, data.to_std_string().unwrap_or_default()) {
                Ok(()) => Ok(JsValue::undefined()),
                Err(err) => Err(JsNativeError::range().with_message(err).into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Comment.prototype.replaceData called on non-Comment object")
                .into())
        }
    }
}

impl IntrinsicObject for Comment {
    fn init(realm: &Realm) {
        let data_getter = BuiltInBuilder::callable(realm, Self::data)
            .name(js_string!("get data"))
            .build();

        let data_setter = BuiltInBuilder::callable(realm, Self::set_data)
            .name(js_string!("set data"))
            .build();

        let length_getter = BuiltInBuilder::callable(realm, Self::length)
            .name(js_string!("get length"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("data"),
                Some(data_getter),
                Some(data_setter),
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("length"),
                Some(length_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .method(Self::substring_data, js_string!("substringData"), 2)
            .method(Self::append_data, js_string!("appendData"), 1)
            .method(Self::insert_data, js_string!("insertData"), 2)
            .method(Self::delete_data, js_string!("deleteData"), 2)
            .method(Self::replace_data, js_string!("replaceData"), 3)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Comment {
    const NAME: JsString = js_string!("Comment");
}

impl BuiltInConstructor for Comment {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::comment;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::comment,
            context,
        )?;

        // Get optional data parameter (default empty string)
        let data_arg = args.get_or_undefined(0);
        let data = if data_arg.is_undefined() {
            String::new()
        } else {
            data_arg.to_string(context)?.to_std_string().unwrap_or_default()
        };

        let comment_data = CommentData::new(data);

        let comment_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            comment_data,
        );

        Ok(comment_obj.into())
    }
}

#[cfg(test)]
mod tests;