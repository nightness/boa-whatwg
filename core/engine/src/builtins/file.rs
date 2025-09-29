//! File Web API implementation for Boa
//!
//! Native implementation of the File interface from the File API
//! https://w3c.github.io/FileAPI/#file-section
//!
//! This implements the File interface which inherits from Blob

#[cfg(test)]
mod tests;

use crate::{
    builtins::{IntrinsicObject, BuiltInBuilder, BuiltInObject, BuiltInConstructor},
    object::JsObject,
    value::JsValue,
    Context, JsResult, js_string, JsNativeError,
    realm::Realm, JsString, JsData,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors}
};
use crate::builtins::blob::BlobData;
use boa_gc::{Finalize, Trace};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// JavaScript `File` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct File;

/// Internal data for File objects (extends BlobData)
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct FileData {
    // Inherits from Blob
    blob_data: BlobData,

    // File-specific properties
    name: String,
    last_modified: u64,
    webkit_relative_path: String,
}

impl FileData {
    /// Create new FileData from blob data and file properties
    pub fn new(blob_data: BlobData, name: String, last_modified: Option<u64>) -> Self {
        let last_modified = last_modified.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        });

        Self {
            blob_data,
            name,
            last_modified,
            webkit_relative_path: String::new(),
        }
    }

    /// Get the underlying blob data
    pub fn blob(&self) -> &BlobData {
        &self.blob_data
    }

    /// Get file name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get last modified timestamp
    pub fn last_modified(&self) -> u64 {
        self.last_modified
    }

    /// Get webkit relative path
    pub fn webkit_relative_path(&self) -> &str {
        &self.webkit_relative_path
    }
}

impl IntrinsicObject for File {
    fn init(realm: &Realm) {
        let get_name = BuiltInBuilder::callable(realm, get_name)
            .name(js_string!("get name"))
            .build();

        let get_last_modified = BuiltInBuilder::callable(realm, get_last_modified)
            .name(js_string!("get lastModified"))
            .build();

        let get_webkit_relative_path = BuiltInBuilder::callable(realm, get_webkit_relative_path)
            .name(js_string!("get webkitRelativePath"))
            .build();

        let get_size = BuiltInBuilder::callable(realm, get_size)
            .name(js_string!("get size"))
            .build();

        let get_type = BuiltInBuilder::callable(realm, get_type)
            .name(js_string!("get type"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Inherit Blob methods
            .method(Self::slice, js_string!("slice"), 0)
            .method(Self::stream, js_string!("stream"), 0)
            .method(Self::text, js_string!("text"), 0)
            .method(Self::array_buffer, js_string!("arrayBuffer"), 0)

            // File-specific properties
            .accessor(
                js_string!("name"),
                Some(get_name),
                None,
                crate::property::Attribute::ENUMERABLE | crate::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("lastModified"),
                Some(get_last_modified),
                None,
                crate::property::Attribute::ENUMERABLE | crate::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("webkitRelativePath"),
                Some(get_webkit_relative_path),
                None,
                crate::property::Attribute::ENUMERABLE | crate::property::Attribute::CONFIGURABLE,
            )

            // Inherited Blob properties
            .accessor(
                js_string!("size"),
                Some(get_size),
                None,
                crate::property::Attribute::ENUMERABLE | crate::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("type"),
                Some(get_type),
                None,
                crate::property::Attribute::ENUMERABLE | crate::property::Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for File {
    const NAME: JsString = js_string!("File");
}

impl BuiltInConstructor for File {
    const LENGTH: usize = 2;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::file;
    const P: usize = 2;
    const SP: usize = 0;

    /// `new File(fileBits, fileName, options)`
    fn constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // File constructor requires at least fileName parameter
        if args.is_empty() {
            return Err(JsNativeError::typ()
                .with_message("File constructor requires at least 1 argument")
                .into());
        }

        // Handle file bits array (same as Blob constructor)
        let mut data = Vec::new();
        if let Some(parts) = args.get(0) {
            if let Some(array) = parts.as_object() {
                // Handle array-like object
                let length_prop = array.get(js_string!("length"), context)?;
                let length = length_prop.to_length(context)?;

                for i in 0..length {
                    let part = array.get(i, context)?;

                    if let Some(part_str) = part.as_string() {
                        // String part
                        data.extend_from_slice(part_str.to_std_string_escaped().as_bytes());
                    } else if let Some(part_obj) = part.as_object() {
                        // Check if it's a Blob or File
                        if let Some(blob_data) = part_obj.downcast_ref::<BlobData>() {
                            data.extend_from_slice(&blob_data.data);
                        } else if let Some(file_data) = part_obj.downcast_ref::<FileData>() {
                            data.extend_from_slice(&file_data.blob_data.data);
                        } else {
                            // Convert to string
                            let part_str = part.to_string(context)?;
                            data.extend_from_slice(part_str.to_std_string_escaped().as_bytes());
                        }
                    } else {
                        // Convert to string
                        let part_str = part.to_string(context)?;
                        data.extend_from_slice(part_str.to_std_string_escaped().as_bytes());
                    }
                }
            } else if !parts.is_undefined() && !parts.is_null() {
                // Single item, convert to string
                let part_str = parts.to_string(context)?;
                data.extend_from_slice(part_str.to_std_string_escaped().as_bytes());
            }
        }

        // Extract fileName (required parameter)
        let file_name = if let Some(name_arg) = args.get(1) {
            name_arg.to_string(context)?.to_std_string_escaped()
        } else {
            return Err(JsNativeError::typ()
                .with_message("File constructor requires fileName parameter")
                .into());
        };

        // Handle options
        let mut mime_type = String::new();
        let mut last_modified = None;
        if let Some(options) = args.get(2) {
            if let Some(options_obj) = options.as_object() {
                // Extract type
                let type_prop = options_obj.get(js_string!("type"), context)?;
                if !type_prop.is_undefined() {
                    mime_type = type_prop.to_string(context)?.to_std_string_escaped();
                }

                // Extract lastModified
                let last_modified_prop = options_obj.get(js_string!("lastModified"), context)?;
                if !last_modified_prop.is_undefined() {
                    last_modified = Some(last_modified_prop.to_number(context)? as u64);
                }

                // TODO: Handle endings option (normalize line endings)
            }
        }

        // Create blob data
        let blob_data = BlobData {
            data: Arc::new(data),
            mime_type,
        };

        // Create file data
        let file_data = FileData::new(blob_data, file_name, last_modified);

        let prototype = Self::get(context.intrinsics())
            .get(js_string!("prototype"), context)?
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("File.prototype is not an object"))?
            .clone();

        let file_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            file_data,
        );

        Ok(file_obj.into())
    }
}

impl File {
    /// `File.prototype.slice(start, end, contentType)` - inherits from Blob
    fn slice(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Delegate to blob slice implementation but return a new File
        let file_obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("slice called on non-object")
        })?;

        let file_data = file_obj.downcast_ref::<FileData>().ok_or_else(|| {
            JsNativeError::typ().with_message("slice called on non-File object")
        })?;

        // Use blob slice logic from the blob module
        // For now, implement simplified version
        let data_len = file_data.blob_data.data.len();

        // Parse start parameter
        let start = if let Some(start_val) = args.get(0) {
            if start_val.is_undefined() {
                0
            } else {
                let start_int = start_val.to_integer_or_infinity(context)?;
                match start_int {
                    crate::value::IntegerOrInfinity::Integer(i) if i < 0 => {
                        (data_len as i64 + i).max(0) as usize
                    }
                    crate::value::IntegerOrInfinity::Integer(i) => (i as usize).min(data_len),
                    crate::value::IntegerOrInfinity::NegativeInfinity => 0,
                    crate::value::IntegerOrInfinity::PositiveInfinity => data_len,
                }
            }
        } else {
            0
        };

        // Parse end parameter
        let end = if let Some(end_val) = args.get(1) {
            if end_val.is_undefined() {
                data_len
            } else {
                let end_int = end_val.to_integer_or_infinity(context)?;
                match end_int {
                    crate::value::IntegerOrInfinity::Integer(i) if i < 0 => {
                        (data_len as i64 + i).max(0) as usize
                    }
                    crate::value::IntegerOrInfinity::Integer(i) => (i as usize).min(data_len),
                    crate::value::IntegerOrInfinity::NegativeInfinity => 0,
                    crate::value::IntegerOrInfinity::PositiveInfinity => data_len,
                }
            }
        } else {
            data_len
        };

        // Parse contentType parameter
        let content_type = if let Some(type_val) = args.get(2) {
            if type_val.is_undefined() {
                file_data.blob_data.mime_type.clone()
            } else {
                type_val.to_string(context)?.to_std_string_escaped()
            }
        } else {
            file_data.blob_data.mime_type.clone()
        };

        // Extract slice data
        let slice_data = if start < end {
            file_data.blob_data.data[start..end].to_vec()
        } else {
            Vec::new()
        };

        let new_blob_data = BlobData {
            data: Arc::new(slice_data),
            mime_type: content_type,
        };

        // Return a new File object with the same name and lastModified
        let new_file_data = FileData::new(
            new_blob_data,
            file_data.name.clone(),
            Some(file_data.last_modified),
        );

        let prototype = Self::get(context.intrinsics())
            .get(js_string!("prototype"), context)?
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("File.prototype is not an object"))?
            .clone();

        let new_file = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            new_file_data,
        );

        Ok(new_file.into())
    }

    /// `File.prototype.stream()` - inherits from Blob
    fn stream(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // TODO: Return a ReadableStream
        // For now, return undefined
        Ok(JsValue::undefined())
    }

    /// `File.prototype.text()` - inherits from Blob
    fn text(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let file_obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("text called on non-object")
        })?;

        let file_data = file_obj.downcast_ref::<FileData>().ok_or_else(|| {
            JsNativeError::typ().with_message("text called on non-File object")
        })?;

        // Convert bytes to UTF-8 string
        let text = String::from_utf8_lossy(&file_data.blob_data.data);

        // TODO: Return a Promise that resolves to the text
        // For now, return the text directly
        Ok(JsValue::from(js_string!(text.as_ref())))
    }

    /// `File.prototype.arrayBuffer()` - inherits from Blob
    fn array_buffer(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let _file_obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("arrayBuffer called on non-object")
        })?;

        let _file_data = _file_obj.downcast_ref::<FileData>().ok_or_else(|| {
            JsNativeError::typ().with_message("arrayBuffer called on non-File object")
        })?;

        // TODO: Return a Promise that resolves to an ArrayBuffer
        // For now, return undefined
        Ok(JsValue::undefined())
    }
}

/// `get File.prototype.name`
pub(crate) fn get_name(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let file_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("name getter called on non-object")
    })?;

    let file_data = file_obj.downcast_ref::<FileData>().ok_or_else(|| {
        JsNativeError::typ().with_message("name getter called on non-File object")
    })?;

    Ok(JsValue::from(js_string!(file_data.name.clone())))
}

/// `get File.prototype.lastModified`
pub(crate) fn get_last_modified(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let file_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("lastModified getter called on non-object")
    })?;

    let file_data = file_obj.downcast_ref::<FileData>().ok_or_else(|| {
        JsNativeError::typ().with_message("lastModified getter called on non-File object")
    })?;

    Ok(JsValue::from(file_data.last_modified as f64))
}

/// `get File.prototype.webkitRelativePath`
pub(crate) fn get_webkit_relative_path(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let file_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("webkitRelativePath getter called on non-object")
    })?;

    let file_data = file_obj.downcast_ref::<FileData>().ok_or_else(|| {
        JsNativeError::typ().with_message("webkitRelativePath getter called on non-File object")
    })?;

    Ok(JsValue::from(js_string!(file_data.webkit_relative_path.clone())))
}

/// `get File.prototype.size` - inherits from Blob
pub(crate) fn get_size(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let file_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("size getter called on non-object")
    })?;

    let file_data = file_obj.downcast_ref::<FileData>().ok_or_else(|| {
        JsNativeError::typ().with_message("size getter called on non-File object")
    })?;

    Ok(JsValue::from(file_data.blob_data.data.len()))
}

/// `get File.prototype.type` - inherits from Blob
pub(crate) fn get_type(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let file_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("type getter called on non-object")
    })?;

    let file_data = file_obj.downcast_ref::<FileData>().ok_or_else(|| {
        JsNativeError::typ().with_message("type getter called on non-File object")
    })?;

    Ok(JsValue::from(js_string!(file_data.blob_data.mime_type.clone())))
}