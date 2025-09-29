//! Structured cloning algorithm implementation for Web APIs
//!
//! Implements the structured cloning algorithm as defined in:
//! https://html.spec.whatwg.org/multipage/structured-data.html#structured-cloning

#[cfg(test)]
mod tests;

use crate::{
    Context, JsResult, JsValue, JsNativeError, JsObject, JsString, js_string,
    object::JsArray,
    builtins::{
        date::Date,
        regexp::RegExp,
        map::Map,
        set::Set,
        array_buffer::ArrayBuffer,
    },
    value::Type,
};
use boa_gc::{Finalize, Trace};
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

/// Structured clone result - can be serialized across threads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructuredCloneValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    BigInt(String), // Stored as string representation
    Array(Vec<StructuredCloneValue>),
    Object(HashMap<String, StructuredCloneValue>),
    Date(f64), // Stored as timestamp
    RegExp { pattern: String, flags: String },
    Map(Vec<(StructuredCloneValue, StructuredCloneValue)>),
    Set(Vec<StructuredCloneValue>),
    ArrayBuffer(Vec<u8>),
    // TODO: Add more types as needed (TypedArrays, etc.)
}

/// Transfer list for transferable objects
#[derive(Debug, Clone)]
pub struct TransferList {
    pub objects: Vec<JsObject>,
}

impl TransferList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add(&mut self, object: JsObject) {
        self.objects.push(object);
    }
}

/// Structured clone algorithm implementation
pub struct StructuredClone;

impl StructuredClone {
    /// Clone a JavaScript value using the structured cloning algorithm
    pub fn clone(
        value: &JsValue,
        context: &mut Context,
        transfer_list: Option<&TransferList>,
    ) -> JsResult<StructuredCloneValue> {
        let mut memory = HashSet::new();
        Self::internal_structured_clone(value, context, &mut memory, transfer_list)
    }

    /// Deserialize a structured clone value back to JavaScript
    pub fn deserialize(
        clone_value: &StructuredCloneValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let mut memory = HashMap::new();
        Self::internal_structured_deserialize(clone_value, context, &mut memory)
    }

    /// Internal recursive cloning implementation
    fn internal_structured_clone(
        value: &JsValue,
        context: &mut Context,
        memory: &mut HashSet<*const u8>,
        transfer_list: Option<&TransferList>,
    ) -> JsResult<StructuredCloneValue> {
        match value.get_type() {
            Type::Undefined => Ok(StructuredCloneValue::Undefined),
            Type::Null => Ok(StructuredCloneValue::Null),
            Type::Boolean => Ok(StructuredCloneValue::Boolean(value.as_boolean().unwrap())),
            Type::Number => Ok(StructuredCloneValue::Number(value.as_number().unwrap())),
            Type::String => {
                let js_string = value.as_string().unwrap();
                Ok(StructuredCloneValue::String(js_string.to_std_string_escaped()))
            }
            Type::BigInt => {
                let bigint_str = value.to_string(context)?.to_std_string_escaped();
                Ok(StructuredCloneValue::BigInt(bigint_str))
            }
            Type::Symbol => {
                Err(JsNativeError::typ()
                    .with_message("Symbols cannot be cloned")
                    .into())
            }
            Type::Object => {
                let obj = value.as_object().unwrap();

                // Use object address for circular reference detection (same as Hash impl)
                let obj_addr = obj.as_ref() as *const _ as *const u8;

                // Check for circular references
                if memory.contains(&obj_addr) {
                    return Err(JsNativeError::typ()
                        .with_message("Converting circular structure to structured clone")
                        .into());
                }
                memory.insert(obj_addr);

                let result = Self::clone_object(&obj, context, memory, transfer_list);
                memory.remove(&obj_addr);
                result
            }
        }
    }

    /// Clone an object based on its type
    fn clone_object(
        obj: &JsObject,
        context: &mut Context,
        memory: &mut HashSet<*const u8>,
        transfer_list: Option<&TransferList>,
    ) -> JsResult<StructuredCloneValue> {
        // Check if this is a transferable object
        if let Some(transfer_list) = transfer_list {
            for transferable in &transfer_list.objects {
                if std::ptr::eq(
                    obj.as_ref() as *const _,
                    transferable.as_ref() as *const _
                ) {
                    return Err(JsNativeError::typ()
                        .with_message("Object is not transferable")
                        .into());
                }
            }
        }

        // Handle specific object types
        if obj.is_array() {
            Self::clone_array(obj, context, memory, transfer_list)
        } else if let Some(date_data) = obj.downcast_ref::<Date>() {
            Self::clone_date(&*date_data, context)
        } else if let Some(regexp_data) = obj.downcast_ref::<RegExp>() {
            Self::clone_regexp(&*regexp_data, context)
        } else {
            // Handle plain objects
            Self::clone_plain_object(obj, context, memory, transfer_list)
        }
    }

    /// Clone an array
    fn clone_array(
        obj: &JsObject,
        context: &mut Context,
        memory: &mut HashSet<*const u8>,
        transfer_list: Option<&TransferList>,
    ) -> JsResult<StructuredCloneValue> {
        let array = JsArray::from_object(obj.clone())?;
        let length = array.length(context)?;
        let mut cloned_array = Vec::new();

        for i in 0..length {
            let element = array.get(i, context)?;
            let cloned_element = Self::internal_structured_clone(&element, context, memory, transfer_list)?;
            cloned_array.push(cloned_element);
        }

        Ok(StructuredCloneValue::Array(cloned_array))
    }

    /// Clone a Date object
    fn clone_date(date_data: &Date, _context: &mut Context) -> JsResult<StructuredCloneValue> {
        let timestamp = date_data.get_time_value();
        Ok(StructuredCloneValue::Date(timestamp))
    }

    /// Clone a RegExp object
    fn clone_regexp(regexp_data: &RegExp, _context: &mut Context) -> JsResult<StructuredCloneValue> {
        let pattern = regexp_data.get_original_source().to_std_string_escaped();
        let flags = regexp_data.get_original_flags().to_std_string_escaped();
        Ok(StructuredCloneValue::RegExp { pattern, flags })
    }

    // TODO: Implement cloning for Map, Set, ArrayBuffer objects

    /// Clone a plain object
    fn clone_plain_object(
        obj: &JsObject,
        context: &mut Context,
        memory: &mut HashSet<*const u8>,
        transfer_list: Option<&TransferList>,
    ) -> JsResult<StructuredCloneValue> {
        let mut cloned_object = HashMap::new();

        // Get all enumerable own properties
        let keys = obj.own_property_keys(context)?;
        for key in keys {
            let property_key = key.to_string();
            if let Ok(value) = obj.get(key, context) {
                let cloned_value = Self::internal_structured_clone(&value, context, memory, transfer_list)?;
                cloned_object.insert(property_key, cloned_value);
            }
        }

        Ok(StructuredCloneValue::Object(cloned_object))
    }

    /// Internal recursive deserialization implementation
    fn internal_structured_deserialize(
        clone_value: &StructuredCloneValue,
        context: &mut Context,
        memory: &mut HashMap<usize, JsValue>,
    ) -> JsResult<JsValue> {
        match clone_value {
            StructuredCloneValue::Undefined => Ok(JsValue::undefined()),
            StructuredCloneValue::Null => Ok(JsValue::null()),
            StructuredCloneValue::Boolean(b) => Ok(JsValue::from(*b)),
            StructuredCloneValue::Number(n) => Ok(JsValue::from(*n)),
            StructuredCloneValue::String(s) => Ok(js_string!(s.clone()).into()),
            StructuredCloneValue::BigInt(s) => {
                // Parse BigInt from string representation
                // For now, convert to regular number (limited precision)
                if let Ok(num) = s.parse::<f64>() {
                    Ok(JsValue::from(num))
                } else {
                    Ok(JsValue::from(0.0))
                }
            }
            StructuredCloneValue::Array(arr) => {
                Self::deserialize_array(arr, context, memory)
            }
            StructuredCloneValue::Object(obj) => {
                Self::deserialize_object(obj, context, memory)
            }
            StructuredCloneValue::Date(timestamp) => {
                Self::deserialize_date(*timestamp, context)
            }
            StructuredCloneValue::RegExp { pattern, flags } => {
                Self::deserialize_regexp(pattern, flags, context)
            }
            StructuredCloneValue::Map(_entries) => {
                // TODO: Deserialize Map objects
                eprintln!("Warning: Map deserialization not implemented");
                Ok(JsValue::undefined())
            }
            StructuredCloneValue::Set(_values) => {
                // TODO: Deserialize Set objects
                eprintln!("Warning: Set deserialization not implemented");
                Ok(JsValue::undefined())
            }
            StructuredCloneValue::ArrayBuffer(_data) => {
                // TODO: Deserialize ArrayBuffer objects
                eprintln!("Warning: ArrayBuffer deserialization not implemented");
                Ok(JsValue::undefined())
            }
        }
    }

    /// Deserialize an array
    fn deserialize_array(
        arr: &[StructuredCloneValue],
        context: &mut Context,
        memory: &mut HashMap<usize, JsValue>,
    ) -> JsResult<JsValue> {
        let js_array = JsArray::new(context);

        for (index, element) in arr.iter().enumerate() {
            let deserialized_element = Self::internal_structured_deserialize(element, context, memory)?;
            js_array.set(index, deserialized_element, true, context)?;
        }

        Ok(js_array.into())
    }

    /// Deserialize a plain object
    fn deserialize_object(
        obj: &HashMap<String, StructuredCloneValue>,
        context: &mut Context,
        memory: &mut HashMap<usize, JsValue>,
    ) -> JsResult<JsValue> {
        let js_object = JsObject::with_object_proto(context.intrinsics());

        for (key, value) in obj {
            let deserialized_value = Self::internal_structured_deserialize(value, context, memory)?;
            js_object.set(js_string!(key.clone()), deserialized_value, true, context)?;
        }

        Ok(js_object.into())
    }

    /// Deserialize a Date object
    fn deserialize_date(timestamp: f64, context: &mut Context) -> JsResult<JsValue> {
        let date_constructor = context.intrinsics().constructors().date().constructor();
        let args = [JsValue::from(timestamp)];
        let new_target = Some(&date_constructor);
        Ok(date_constructor.construct(&args, new_target, context)?.into())
    }

    /// Deserialize a RegExp object
    fn deserialize_regexp(pattern: &str, flags: &str, context: &mut Context) -> JsResult<JsValue> {
        let regexp_constructor = context.intrinsics().constructors().regexp().constructor();
        let args = [
            js_string!(pattern).into(),
            js_string!(flags).into(),
        ];
        let new_target = Some(&regexp_constructor);
        Ok(regexp_constructor.construct(&args, new_target, context)?.into())
    }

    /// Deserialize a Map object
    fn deserialize_map(
        entries: &[(StructuredCloneValue, StructuredCloneValue)],
        context: &mut Context,
        memory: &mut HashMap<usize, JsValue>,
    ) -> JsResult<JsValue> {
        let map_constructor = context.intrinsics().constructors().map().constructor();
        let new_target = Some(&map_constructor);
        let map_obj = map_constructor.construct(&[], new_target, context)?;

        // TODO: Set the entries in the map
        eprintln!("Warning: Map deserialization not fully implemented");

        Ok(map_obj.into())
    }

    /// Deserialize a Set object
    fn deserialize_set(
        values: &[StructuredCloneValue],
        context: &mut Context,
        memory: &mut HashMap<usize, JsValue>,
    ) -> JsResult<JsValue> {
        let set_constructor = context.intrinsics().constructors().set().constructor();
        let new_target = Some(&set_constructor);
        let set_obj = set_constructor.construct(&[], new_target, context)?;

        // TODO: Add the values to the set
        eprintln!("Warning: Set deserialization not fully implemented");

        Ok(set_obj.into())
    }

    /// Deserialize an ArrayBuffer object
    fn deserialize_array_buffer(data: &[u8], context: &mut Context) -> JsResult<JsValue> {
        let array_buffer_constructor = context.intrinsics().constructors().array_buffer().constructor();
        let length = JsValue::from(data.len());
        let new_target = Some(&array_buffer_constructor);
        let array_buffer = array_buffer_constructor.construct(&[length], new_target, context)?;

        // TODO: Copy the data into the ArrayBuffer
        eprintln!("Warning: ArrayBuffer data copying not fully implemented");

        Ok(array_buffer.into())
    }

    /// Serialize a structured clone value to bytes for cross-thread transfer
    pub fn serialize_to_bytes(value: &StructuredCloneValue) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let serialized = serde_json::to_vec(value)?;
        Ok(serialized)
    }

    /// Deserialize a structured clone value from bytes
    pub fn deserialize_from_bytes(bytes: &[u8]) -> Result<StructuredCloneValue, Box<dyn std::error::Error + Send + Sync>> {
        let value = serde_json::from_slice(bytes)?;
        Ok(value)
    }
}

/// Convenience function for cloning values
pub fn structured_clone(
    value: &JsValue,
    context: &mut Context,
    transfer_list: Option<&TransferList>,
) -> JsResult<StructuredCloneValue> {
    StructuredClone::clone(value, context, transfer_list)
}

/// Convenience function for deserializing values
pub fn structured_deserialize(
    clone_value: &StructuredCloneValue,
    context: &mut Context,
) -> JsResult<JsValue> {
    StructuredClone::deserialize(clone_value, context)
}