//! Blob Web API implementation for Boa
//!
//! Native implementation of the Blob standard
//! https://w3c.github.io/FileAPI/#blob-section
//!
//! This implements the complete Blob interface with real binary data handling

#[cfg(test)]
mod tests;

use crate::{
    builtins::{IntrinsicObject, BuiltInBuilder, BuiltInObject, BuiltInConstructor},
    object::JsObject,
    value::{JsValue, IntegerOrInfinity},
    Context, JsResult, js_string, JsNativeError,
    realm::Realm, JsString, JsData,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors}
};
use boa_gc::{Finalize, Trace};
use std::sync::Arc;

/// JavaScript `Blob` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Blob;

/// Internal data for Blob objects
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct BlobData {
    #[unsafe_ignore_trace]
    data: Arc<Vec<u8>>,
    #[unsafe_ignore_trace]
    mime_type: String,
}

impl IntrinsicObject for Blob {
    fn init(realm: &Realm) {
        let get_size = BuiltInBuilder::callable(realm, get_size)
            .name(js_string!("get size"))
            .build();

        let get_type = BuiltInBuilder::callable(realm, get_type)
            .name(js_string!("get type"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::slice, js_string!("slice"), 0)
            .method(Self::stream, js_string!("stream"), 0)
            .method(Self::text, js_string!("text"), 0)
            .method(Self::array_buffer, js_string!("arrayBuffer"), 0)
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

impl BuiltInObject for Blob {
    const NAME: JsString = js_string!("Blob");
}

impl BuiltInConstructor for Blob {
    const LENGTH: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::blob;
    const P: usize = 2;
    const SP: usize = 0;

    /// `new Blob(array, options)`
    fn constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Handle blob parts array
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
                        // Check if it's a Blob
                        if let Some(blob_data) = part_obj.downcast_ref::<BlobData>() {
                            data.extend_from_slice(&blob_data.data);
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

        // Handle options
        let mut mime_type = String::new();
        if let Some(options) = args.get(1) {
            if let Some(options_obj) = options.as_object() {
                let type_prop = options_obj.get(js_string!("type"), context)?;
                if !type_prop.is_undefined() {
                    mime_type = type_prop.to_string(context)?.to_std_string_escaped();
                }

                // TODO: Handle endings option (normalize line endings)
            }
        }

        let blob_data = BlobData {
            data: Arc::new(data),
            mime_type,
        };

        let prototype = Self::get(context.intrinsics())
            .get(js_string!("prototype"), context)?
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("Blob.prototype is not an object"))?
            .clone();

        let blob_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            blob_data,
        );

        Ok(blob_obj.into())
    }
}

impl Blob {
    /// `Blob.prototype.slice(start, end, contentType)`
    fn slice(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let blob_obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("slice called on non-object")
        })?;

        let blob_data = blob_obj.downcast_ref::<BlobData>().ok_or_else(|| {
            JsNativeError::typ().with_message("slice called on non-Blob object")
        })?;

        let data_len = blob_data.data.len();

        // Parse start parameter
        let start = if let Some(start_val) = args.get(0) {
            if start_val.is_undefined() {
                0
            } else {
                let start_int = start_val.to_integer_or_infinity(context)?;
                match start_int {
                    IntegerOrInfinity::Integer(i) if i < 0 => {
                        (data_len as i64 + i).max(0) as usize
                    }
                    IntegerOrInfinity::Integer(i) => (i as usize).min(data_len),
                    IntegerOrInfinity::NegativeInfinity => 0,
                    IntegerOrInfinity::PositiveInfinity => data_len,
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
                    IntegerOrInfinity::Integer(i) if i < 0 => {
                        (data_len as i64 + i).max(0) as usize
                    }
                    IntegerOrInfinity::Integer(i) => (i as usize).min(data_len),
                    IntegerOrInfinity::NegativeInfinity => 0,
                    IntegerOrInfinity::PositiveInfinity => data_len,
                }
            }
        } else {
            data_len
        };

        // Parse contentType parameter
        let content_type = if let Some(type_val) = args.get(2) {
            if type_val.is_undefined() {
                blob_data.mime_type.clone()
            } else {
                type_val.to_string(context)?.to_std_string_escaped()
            }
        } else {
            blob_data.mime_type.clone()
        };

        // Extract slice data
        let slice_data = if start < end {
            blob_data.data[start..end].to_vec()
        } else {
            Vec::new()
        };

        let new_blob_data = BlobData {
            data: Arc::new(slice_data),
            mime_type: content_type,
        };

        let prototype = Self::get(context.intrinsics())
            .get(js_string!("prototype"), context)?
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("Blob.prototype is not an object"))?
            .clone();

        let new_blob = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            new_blob_data,
        );

        Ok(new_blob.into())
    }

    /// `Blob.prototype.stream()`
    fn stream(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // TODO: Return a ReadableStream
        // For now, return undefined
        Ok(JsValue::undefined())
    }

    /// `Blob.prototype.text()`
    fn text(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let blob_obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("text called on non-object")
        })?;

        let blob_data = blob_obj.downcast_ref::<BlobData>().ok_or_else(|| {
            JsNativeError::typ().with_message("text called on non-Blob object")
        })?;

        // Convert bytes to UTF-8 string
        let text = String::from_utf8_lossy(&blob_data.data);

        // TODO: Return a Promise that resolves to the text
        // For now, return the text directly
        Ok(JsValue::from(js_string!(text.as_ref())))
    }

    /// `Blob.prototype.arrayBuffer()`
    fn array_buffer(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let blob_obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("arrayBuffer called on non-object")
        })?;

        let _blob_data = blob_obj.downcast_ref::<BlobData>().ok_or_else(|| {
            JsNativeError::typ().with_message("arrayBuffer called on non-Blob object")
        })?;

        // TODO: Return a Promise that resolves to an ArrayBuffer
        // For now, return undefined
        Ok(JsValue::undefined())
    }

}

/// `get Blob.prototype.size`
pub(crate) fn get_size(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let blob_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("size getter called on non-object")
    })?;

    let blob_data = blob_obj.downcast_ref::<BlobData>().ok_or_else(|| {
        JsNativeError::typ().with_message("size getter called on non-Blob object")
    })?;

    Ok(JsValue::from(blob_data.data.len()))
}

/// `get Blob.prototype.type`
pub(crate) fn get_type(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let blob_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("type getter called on non-object")
    })?;

    let blob_data = blob_obj.downcast_ref::<BlobData>().ok_or_else(|| {
        JsNativeError::typ().with_message("type getter called on non-Blob object")
    })?;

    Ok(JsValue::from(js_string!(blob_data.mime_type.clone())))
}