//! This module implements the conversions from and into [`serde_json::Value`].

use super::JsValue;
use crate::{
    Context, JsResult, JsVariant,
    builtins::Array,
    error::JsNativeError,
    js_string,
    object::JsObject,
    property::{PropertyDescriptor, PropertyKey},
};
use serde_json::{Map, Value};
use std::collections::HashSet;

impl JsValue {
    /// Converts a [`serde_json::Value`] to a `JsValue`.
    ///
    /// # Example
    ///
    /// ```
    /// use boa_engine::{Context, JsValue};
    ///
    /// let data = r#"
    ///     {
    ///         "name": "John Doe",
    ///         "age": 43,
    ///         "phones": [
    ///             "+44 1234567",
    ///             "+44 2345678"
    ///         ]
    ///      }"#;
    ///
    /// let json: serde_json::Value = serde_json::from_str(data).unwrap();
    ///
    /// let mut context = Context::default();
    /// let value = JsValue::from_json(&json, &mut context).unwrap();
    /// #
    /// # assert_eq!(Some(json), value.to_json(&mut context).unwrap());
    /// ```
    pub fn from_json(json: &Value, context: &mut Context) -> JsResult<Self> {
        /// Biggest possible integer, as i64.
        const MAX_INT: i64 = i32::MAX as i64;

        /// Smallest possible integer, as i64.
        const MIN_INT: i64 = i32::MIN as i64;

        match json {
            Value::Null => Ok(Self::null()),
            Value::Bool(b) => Ok(Self::new(*b)),
            Value::Number(num) => num
                .as_i64()
                .filter(|n| (MIN_INT..=MAX_INT).contains(n))
                .map(|i| Self::new(i as i32))
                .or_else(|| num.as_f64().map(Self::new))
                .ok_or_else(|| {
                    JsNativeError::typ()
                        .with_message(format!("could not convert JSON number {num} to JsValue"))
                        .into()
                }),
            Value::String(string) => Ok(Self::from(js_string!(string.as_str()))),
            Value::Array(vec) => {
                let mut arr = Vec::with_capacity(vec.len());
                for val in vec {
                    arr.push(Self::from_json(val, context)?);
                }
                Ok(Array::create_array_from_list(arr, context).into())
            }
            Value::Object(obj) => {
                let js_obj = JsObject::with_object_proto(context.intrinsics());
                for (key, value) in obj {
                    let property = PropertyDescriptor::builder()
                        .value(Self::from_json(value, context)?)
                        .writable(true)
                        .enumerable(true)
                        .configurable(true);
                    js_obj
                        .borrow_mut()
                        .insert(js_string!(key.clone()), property);
                }

                Ok(js_obj.into())
            }
        }
    }

    /// Converts the `JsValue` to a [`serde_json::Value`].
    ///
    /// If the `JsValue` is `Undefined`, this method will return `None`.
    /// Otherwise it will return the corresponding `serde_json::Value`.
    ///
    /// # Example
    ///
    /// ```
    /// use boa_engine::{Context, JsValue};
    ///
    /// let data = r#"
    ///     {
    ///         "name": "John Doe",
    ///         "age": 43,
    ///         "phones": [
    ///             "+44 1234567",
    ///             "+44 2345678"
    ///         ]
    ///      }"#;
    ///
    /// let json: serde_json::Value = serde_json::from_str(data).unwrap();
    ///
    /// let mut context = Context::default();
    /// let value = JsValue::from_json(&json, &mut context).unwrap();
    ///
    /// let back_to_json = value.to_json(&mut context).unwrap();
    /// #
    /// # assert_eq!(Some(json), back_to_json);
    /// ```
    pub fn to_json(&self, context: &mut Context) -> JsResult<Option<Value>> {
        let mut seen_objects = HashSet::new();
        self.to_json_inner(context, &mut seen_objects)
    }

    fn to_json_inner(
        &self,
        context: &mut Context,
        seen_objects: &mut HashSet<JsObject>,
    ) -> JsResult<Option<Value>> {
        match self.variant() {
            JsVariant::Null => Ok(Some(Value::Null)),
            JsVariant::Undefined => Ok(None),
            JsVariant::Boolean(b) => Ok(Some(Value::from(b))),
            JsVariant::String(string) => Ok(Some(string.to_std_string_escaped().into())),
            JsVariant::Float64(rat) => Ok(Some(Value::from(rat))),
            JsVariant::Integer32(int) => Ok(Some(Value::from(int))),
            JsVariant::BigInt(_bigint) => Err(JsNativeError::typ()
                .with_message("cannot convert bigint to JSON")
                .into()),
            JsVariant::Object(obj) => {
                if seen_objects.contains(&obj) {
                    return Err(JsNativeError::typ()
                        .with_message("cyclic object value")
                        .into());
                }
                seen_objects.insert(obj.clone());
                let mut value_by_prop_key = |property_key, context: &mut Context| {
                    obj.borrow()
                        .properties()
                        .get(&property_key)
                        .and_then(|x| {
                            x.value()
                                .map(|val| val.to_json_inner(context, seen_objects))
                        })
                        .unwrap_or(Ok(Some(Value::Null)))
                };

                if obj.is_array() {
                    let len = obj.length_of_array_like(context)?;
                    let mut arr = Vec::with_capacity(len as usize);

                    for k in 0..len as u32 {
                        let val = value_by_prop_key(k.into(), context)?;
                        match val {
                            Some(val) => arr.push(val),

                            // Undefined in array. Substitute with null as Value doesn't support Undefined.
                            None => arr.push(Value::Null),
                        }
                    }
                    // Passing the object rather than its clone that was inserted to the set should be fine
                    // as they hash to the same value and therefore HashSet can still remove the clone
                    seen_objects.remove(&obj);
                    Ok(Some(Value::Array(arr)))
                } else {
                    let mut map = Map::new();

                    for index in obj.borrow().properties().index_property_keys() {
                        let key = index.to_string();
                        let value = value_by_prop_key(index.into(), context)?;
                        if let Some(value) = value {
                            map.insert(key, value);
                        }
                    }

                    for property_key in obj.borrow().properties().shape.keys() {
                        let key = match &property_key {
                            PropertyKey::String(string) => string.to_std_string_escaped(),
                            PropertyKey::Index(i) => i.get().to_string(),
                            PropertyKey::Symbol(_sym) => {
                                return Err(JsNativeError::typ()
                                    .with_message("cannot convert Symbol to JSON")
                                    .into());
                            }
                        };
                        let value = value_by_prop_key(property_key, context)?;
                        if let Some(value) = value {
                            map.insert(key, value);
                        }
                    }
                    seen_objects.remove(&obj);
                    Ok(Some(Value::Object(map)))
                }
            }
            JsVariant::Symbol(_sym) => Err(JsNativeError::typ()
                .with_message("cannot convert Symbol to JSON")
                .into()),
        }
    }
}

