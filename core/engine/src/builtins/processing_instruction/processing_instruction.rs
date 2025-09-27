//! ProcessingInstruction interface implementation for DOM Level 4
//!
//! The ProcessingInstruction interface represents processing instructions in XML/HTML.
//! It inherits from CharacterData and represents PI nodes like <?xml-stylesheet?>.
//! https://dom.spec.whatwg.org/#interface-processinginstruction

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

/// The ProcessingInstruction data implementation
#[derive(Debug, Trace, Finalize, JsData)]
pub struct ProcessingInstructionData {
    /// The target of the processing instruction (the part immediately following the opening <?)
    target: GcRefCell<String>,
    /// The data content of the processing instruction (everything after the target)
    data: GcRefCell<String>,
}

impl ProcessingInstructionData {
    /// Create a new ProcessingInstruction with target and data
    pub fn new(target: String, data: String) -> Self {
        Self {
            target: GcRefCell::new(target),
            data: GcRefCell::new(data),
        }
    }

    /// Get the processing instruction target
    pub fn target(&self) -> String {
        self.target.borrow().clone()
    }

    /// Get the processing instruction data
    pub fn data(&self) -> String {
        self.data.borrow().clone()
    }

    /// Set the processing instruction data
    pub fn set_data(&self, data: String) {
        *self.data.borrow_mut() = data;
    }

    /// Get the length of the processing instruction data
    pub fn length(&self) -> u32 {
        self.data.borrow().chars().count() as u32
    }

    /// Extract a substring of the processing instruction data
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

    /// Append data to the processing instruction
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

/// The `ProcessingInstruction` object
#[derive(Debug, Trace, Finalize)]
pub struct ProcessingInstruction;

impl ProcessingInstruction {
    /// Create a new ProcessingInstruction
    pub fn create(context: &mut Context, target: String, data: String) -> JsResult<JsObject> {
        let pi_data = ProcessingInstructionData::new(target, data);

        let pi_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().processing_instruction().prototype(),
            pi_data,
        );

        Ok(pi_obj)
    }

    /// Get the target property
    fn target(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ProcessingInstruction.prototype.target called on non-object")
        })?;

        if let Some(pi_data) = this_obj.downcast_ref::<ProcessingInstructionData>() {
            Ok(JsString::from(pi_data.target()).into())
        } else {
            Err(JsNativeError::typ()
                .with_message("ProcessingInstruction.prototype.target called on non-ProcessingInstruction object")
                .into())
        }
    }

    /// Get the data property
    fn data(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ProcessingInstruction.prototype.data called on non-object")
        })?;

        if let Some(pi_data) = this_obj.downcast_ref::<ProcessingInstructionData>() {
            Ok(JsString::from(pi_data.data()).into())
        } else {
            Err(JsNativeError::typ()
                .with_message("ProcessingInstruction.prototype.data called on non-ProcessingInstruction object")
                .into())
        }
    }

    /// Set the data property
    fn set_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ProcessingInstruction.prototype.data setter called on non-object")
        })?;

        if let Some(pi_data) = this_obj.downcast_ref::<ProcessingInstructionData>() {
            let new_data = args.get_or_undefined(0).to_string(context)?;
            pi_data.set_data(new_data.to_std_string().unwrap_or_default());
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("ProcessingInstruction.prototype.data setter called on non-ProcessingInstruction object")
                .into())
        }
    }

    /// Get the length property
    fn length(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ProcessingInstruction.prototype.length called on non-object")
        })?;

        if let Some(pi_data) = this_obj.downcast_ref::<ProcessingInstructionData>() {
            Ok(JsValue::new(pi_data.length()))
        } else {
            Err(JsNativeError::typ()
                .with_message("ProcessingInstruction.prototype.length called on non-ProcessingInstruction object")
                .into())
        }
    }

    /// substringData method
    fn substring_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ProcessingInstruction.prototype.substringData called on non-object")
        })?;

        if let Some(pi_data) = this_obj.downcast_ref::<ProcessingInstructionData>() {
            let offset = args.get_or_undefined(0).to_u32(context)?;
            let count = args.get_or_undefined(1).to_u32(context)?;

            match pi_data.substring_data(offset, count) {
                Ok(substring) => Ok(JsString::from(substring).into()),
                Err(err) => Err(JsNativeError::range().with_message(err).into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("ProcessingInstruction.prototype.substringData called on non-ProcessingInstruction object")
                .into())
        }
    }

    /// appendData method
    fn append_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ProcessingInstruction.prototype.appendData called on non-object")
        })?;

        if let Some(pi_data) = this_obj.downcast_ref::<ProcessingInstructionData>() {
            let data = args.get_or_undefined(0).to_string(context)?;
            pi_data.append_data(data.to_std_string().unwrap_or_default());
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("ProcessingInstruction.prototype.appendData called on non-ProcessingInstruction object")
                .into())
        }
    }

    /// insertData method
    fn insert_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ProcessingInstruction.prototype.insertData called on non-object")
        })?;

        if let Some(pi_data) = this_obj.downcast_ref::<ProcessingInstructionData>() {
            let offset = args.get_or_undefined(0).to_u32(context)?;
            let data = args.get_or_undefined(1).to_string(context)?;

            match pi_data.insert_data(offset, data.to_std_string().unwrap_or_default()) {
                Ok(()) => Ok(JsValue::undefined()),
                Err(err) => Err(JsNativeError::range().with_message(err).into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("ProcessingInstruction.prototype.insertData called on non-ProcessingInstruction object")
                .into())
        }
    }

    /// deleteData method
    fn delete_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ProcessingInstruction.prototype.deleteData called on non-object")
        })?;

        if let Some(pi_data) = this_obj.downcast_ref::<ProcessingInstructionData>() {
            let offset = args.get_or_undefined(0).to_u32(context)?;
            let count = args.get_or_undefined(1).to_u32(context)?;

            match pi_data.delete_data(offset, count) {
                Ok(()) => Ok(JsValue::undefined()),
                Err(err) => Err(JsNativeError::range().with_message(err).into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("ProcessingInstruction.prototype.deleteData called on non-ProcessingInstruction object")
                .into())
        }
    }

    /// replaceData method
    fn replace_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ProcessingInstruction.prototype.replaceData called on non-object")
        })?;

        if let Some(pi_data) = this_obj.downcast_ref::<ProcessingInstructionData>() {
            let offset = args.get_or_undefined(0).to_u32(context)?;
            let count = args.get_or_undefined(1).to_u32(context)?;
            let data = args.get_or_undefined(2).to_string(context)?;

            match pi_data.replace_data(offset, count, data.to_std_string().unwrap_or_default()) {
                Ok(()) => Ok(JsValue::undefined()),
                Err(err) => Err(JsNativeError::range().with_message(err).into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("ProcessingInstruction.prototype.replaceData called on non-ProcessingInstruction object")
                .into())
        }
    }
}

impl IntrinsicObject for ProcessingInstruction {
    fn init(realm: &Realm) {
        let target_getter = BuiltInBuilder::callable(realm, Self::target)
            .name(js_string!("get target"))
            .build();

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
                js_string!("target"),
                Some(target_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
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

impl BuiltInObject for ProcessingInstruction {
    const NAME: JsString = js_string!("ProcessingInstruction");
}

impl BuiltInConstructor for ProcessingInstruction {
    const LENGTH: usize = 2;
    const P: usize = 0;
    const SP: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::processing_instruction;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::processing_instruction,
            context,
        )?;

        // ProcessingInstruction requires target parameter
        let target_arg = args.get_or_undefined(0);
        let target = if target_arg.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("ProcessingInstruction constructor requires target parameter")
                .into());
        } else {
            target_arg.to_string(context)?.to_std_string().unwrap_or_default()
        };

        // Get optional data parameter (default empty string)
        let data_arg = args.get_or_undefined(1);
        let data = if data_arg.is_undefined() {
            String::new()
        } else {
            data_arg.to_string(context)?.to_std_string().unwrap_or_default()
        };

        let pi_data = ProcessingInstructionData::new(target, data);

        let pi_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            pi_data,
        );

        Ok(pi_obj.into())
    }
}

