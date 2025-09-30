//! CDATASection interface implementation for DOM Level 4
//!
//! The CDATASection interface represents CDATA sections in XML documents.
//! It inherits from Text and represents unparsed text content like <![CDATA[...]]>.
//! https://dom.spec.whatwg.org/#interface-cdatasection

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

/// The CDATASection data implementation
#[derive(Debug, Trace, Finalize, JsData)]
pub struct CDATASectionData {
    /// The text content of the CDATA section
    data: GcRefCell<String>,
}

impl CDATASectionData {
    /// Create a new CDATASection with data
    pub fn new(data: String) -> Self {
        Self {
            data: GcRefCell::new(data),
        }
    }

    /// Get the CDATA section data
    pub fn data(&self) -> String {
        self.data.borrow().clone()
    }

    /// Set the CDATA section data
    pub fn set_data(&self, data: String) {
        *self.data.borrow_mut() = data;
    }

    /// Get the length of the CDATA section data
    pub fn length(&self) -> u32 {
        self.data.borrow().chars().count() as u32
    }

    /// Extract a substring of the CDATA section data
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

    /// Append data to the CDATA section
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

/// The `CDATASection` object
#[derive(Debug, Trace, Finalize)]
pub struct CDATASection;

impl CDATASection {
    /// Create a new CDATASection
    pub fn create(context: &mut Context, data: String) -> JsResult<JsObject> {
        let cdata_data = CDATASectionData::new(data);

        let cdata_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().cdata_section().prototype(),
            cdata_data,
        );

        Ok(cdata_obj)
    }

    /// Get the data property
    fn data(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CDATASection.prototype.data called on non-object")
        })?;

        if let Some(cdata_data) = this_obj.downcast_ref::<CDATASectionData>() {
            Ok(JsString::from(cdata_data.data()).into())
        } else {
            Err(JsNativeError::typ()
                .with_message("CDATASection.prototype.data called on non-CDATASection object")
                .into())
        }
    }

    /// Set the data property
    fn set_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CDATASection.prototype.data setter called on non-object")
        })?;

        if let Some(cdata_data) = this_obj.downcast_ref::<CDATASectionData>() {
            let new_data = args.get_or_undefined(0).to_string(context)?;
            cdata_data.set_data(new_data.to_std_string().unwrap_or_default());
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("CDATASection.prototype.data setter called on non-CDATASection object")
                .into())
        }
    }

    /// Get the length property
    fn length(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CDATASection.prototype.length called on non-object")
        })?;

        if let Some(cdata_data) = this_obj.downcast_ref::<CDATASectionData>() {
            Ok(JsValue::new(cdata_data.length()))
        } else {
            Err(JsNativeError::typ()
                .with_message("CDATASection.prototype.length called on non-CDATASection object")
                .into())
        }
    }

    /// substringData method
    fn substring_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CDATASection.prototype.substringData called on non-object")
        })?;

        if let Some(cdata_data) = this_obj.downcast_ref::<CDATASectionData>() {
            let offset = args.get_or_undefined(0).to_u32(context)?;
            let count = args.get_or_undefined(1).to_u32(context)?;

            match cdata_data.substring_data(offset, count) {
                Ok(substring) => Ok(JsString::from(substring).into()),
                Err(err) => Err(JsNativeError::range().with_message(err).into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("CDATASection.prototype.substringData called on non-CDATASection object")
                .into())
        }
    }

    /// appendData method
    fn append_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CDATASection.prototype.appendData called on non-object")
        })?;

        if let Some(cdata_data) = this_obj.downcast_ref::<CDATASectionData>() {
            let data = args.get_or_undefined(0).to_string(context)?;
            cdata_data.append_data(data.to_std_string().unwrap_or_default());
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("CDATASection.prototype.appendData called on non-CDATASection object")
                .into())
        }
    }

    /// insertData method
    fn insert_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CDATASection.prototype.insertData called on non-object")
        })?;

        if let Some(cdata_data) = this_obj.downcast_ref::<CDATASectionData>() {
            let offset = args.get_or_undefined(0).to_u32(context)?;
            let data = args.get_or_undefined(1).to_string(context)?;

            match cdata_data.insert_data(offset, data.to_std_string().unwrap_or_default()) {
                Ok(()) => Ok(JsValue::undefined()),
                Err(err) => Err(JsNativeError::range().with_message(err).into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("CDATASection.prototype.insertData called on non-CDATASection object")
                .into())
        }
    }

    /// deleteData method
    fn delete_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CDATASection.prototype.deleteData called on non-object")
        })?;

        if let Some(cdata_data) = this_obj.downcast_ref::<CDATASectionData>() {
            let offset = args.get_or_undefined(0).to_u32(context)?;
            let count = args.get_or_undefined(1).to_u32(context)?;

            match cdata_data.delete_data(offset, count) {
                Ok(()) => Ok(JsValue::undefined()),
                Err(err) => Err(JsNativeError::range().with_message(err).into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("CDATASection.prototype.deleteData called on non-CDATASection object")
                .into())
        }
    }

    /// replaceData method
    fn replace_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CDATASection.prototype.replaceData called on non-object")
        })?;

        if let Some(cdata_data) = this_obj.downcast_ref::<CDATASectionData>() {
            let offset = args.get_or_undefined(0).to_u32(context)?;
            let count = args.get_or_undefined(1).to_u32(context)?;
            let data = args.get_or_undefined(2).to_string(context)?;

            match cdata_data.replace_data(offset, count, data.to_std_string().unwrap_or_default()) {
                Ok(()) => Ok(JsValue::undefined()),
                Err(err) => Err(JsNativeError::range().with_message(err).into()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("CDATASection.prototype.replaceData called on non-CDATASection object")
                .into())
        }
    }
}

impl IntrinsicObject for CDATASection {
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

impl BuiltInObject for CDATASection {
    const NAME: JsString = js_string!("CDATASection");
}

impl BuiltInConstructor for CDATASection {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::cdata_section;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::cdata_section,
            context,
        )?;

        // Get optional data parameter (default empty string)
        let data_arg = args.get_or_undefined(0);
        let data = if data_arg.is_undefined() {
            String::new()
        } else {
            data_arg.to_string(context)?.to_std_string().unwrap_or_default()
        };

        let cdata_data = CDATASectionData::new(data);

        let cdata_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            cdata_data,
        );

        Ok(cdata_obj.into())
    }
}