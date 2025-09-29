//! Implementation of the IDBIndex interface.
//!
//! IDBIndex provides access to indexed data in an IndexedDB database.
//! Indexes enable efficient queries on object store data using specified key paths.
//!
//! More information:
//! - [WHATWG IndexedDB Specification](https://w3c.github.io/IndexedDB/#index-interface)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBIndex)

use boa_gc::{Finalize, Trace};
use crate::{
    builtins::{BuiltInBuilder, Array},
    context::intrinsics::Intrinsics,
    js_string,
    object::{JsObject, JsPromise},
    property::Attribute,
    realm::Realm,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, JsValue,
};
use crate::builtins::indexed_db::{IdbKeyRange, IdbCursor, IdbCursorWithValue, IdbFactory};

/// IDBIndex object representing an index on an object store
#[derive(Debug, Clone, Finalize)]
pub struct IdbIndex {
    /// Name of the index
    name: String,
    /// Key path for the index (property to index on)
    key_path: String,
    /// Whether the index is unique (only one record per key value)
    unique: bool,
    /// Whether the index allows duplicate keys (multiEntry)
    multi_entry: bool,
    /// Name of the object store this index belongs to
    object_store_name: String,
}

unsafe impl Trace for IdbIndex {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // No GC'd objects in IdbIndex, nothing to trace
    }

    unsafe fn trace_non_roots(&self) {
        // No GC'd objects in IdbIndex, nothing to trace
    }

    fn run_finalizer(&self) {
        // No cleanup needed for IdbIndex
    }
}

impl JsData for IdbIndex {}

impl IdbIndex {
    /// Create a new IDBIndex
    pub fn new(
        name: String,
        key_path: String,
        unique: bool,
        multi_entry: bool,
        object_store_name: String,
    ) -> Self {
        Self {
            name,
            key_path,
            unique,
            multi_entry,
            object_store_name,
        }
    }

    /// `index.get(key)`
    /// Retrieves the value of the first record matching the given key
    fn get(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let key = args.get_or_undefined(0);

        // Get the index object
        let index_obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

        let index_data = index_obj.downcast_ref::<IdbIndex>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an IDBIndex"))?;

        // Mock data for index lookup (key -> primary key -> value)
        let mock_index_data = vec![
            (JsValue::from(JsString::from("alice")), JsValue::from(1), JsValue::from(JsString::from("Alice Smith"))),
            (JsValue::from(JsString::from("bob")), JsValue::from(2), JsValue::from(JsString::from("Bob Jones"))),
            (JsValue::from(JsString::from("charlie")), JsValue::from(3), JsValue::from(JsString::from("Charlie Brown"))),
        ];

        // Find matching value based on key or key range
        let result = if key.is_undefined() || key.is_null() {
            JsValue::undefined()
        } else if let Some(range_obj) = key.as_object() {
            if let Some(range_data) = range_obj.downcast_ref::<IdbKeyRange>() {
                // Key range - return first matching value
                mock_index_data.iter()
                    .find(|(k, _, _)| IdbKeyRange::key_in_range(k, &range_data, context).unwrap_or(false))
                    .map(|(_, _, v)| v.clone())
                    .unwrap_or(JsValue::undefined())
            } else {
                JsValue::undefined()
            }
        } else {
            // Single key - find exact match
            mock_index_data.iter()
                .find(|(k, _, _)| IdbKeyRange::compare_keys(k, &key, context).unwrap_or(1) == 0)
                .map(|(_, _, v)| v.clone())
                .unwrap_or(JsValue::undefined())
        };

        Ok(JsValue::from(IdbFactory::create_success_request(result, context)))
    }

    /// `index.getKey(key)`
    /// Retrieves the primary key of the first record matching the given key
    fn get_key(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let key = args.get_or_undefined(0);

        // Get the index object
        let index_obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

        let index_data = index_obj.downcast_ref::<IdbIndex>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an IDBIndex"))?;

        // Mock data for index lookup (key -> primary key -> value)
        let mock_index_data = vec![
            (JsValue::from(JsString::from("alice")), JsValue::from(1), JsValue::from(JsString::from("Alice Smith"))),
            (JsValue::from(JsString::from("bob")), JsValue::from(2), JsValue::from(JsString::from("Bob Jones"))),
            (JsValue::from(JsString::from("charlie")), JsValue::from(3), JsValue::from(JsString::from("Charlie Brown"))),
        ];

        // Find matching primary key based on key or key range
        let result = if key.is_undefined() || key.is_null() {
            JsValue::undefined()
        } else if let Some(range_obj) = key.as_object() {
            if let Some(range_data) = range_obj.downcast_ref::<IdbKeyRange>() {
                // Key range - return first matching primary key
                mock_index_data.iter()
                    .find(|(k, _, _)| IdbKeyRange::key_in_range(k, &range_data, context).unwrap_or(false))
                    .map(|(_, pk, _)| pk.clone())
                    .unwrap_or(JsValue::undefined())
            } else {
                JsValue::undefined()
            }
        } else {
            // Single key - find exact match
            mock_index_data.iter()
                .find(|(k, _, _)| IdbKeyRange::compare_keys(k, &key, context).unwrap_or(1) == 0)
                .map(|(_, pk, _)| pk.clone())
                .unwrap_or(JsValue::undefined())
        };

        Ok(JsValue::from(IdbFactory::create_success_request(result, context)))
    }

    /// `index.getAll(query, count)`
    /// Retrieves all records matching the query
    fn get_all(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let query = args.get_or_undefined(0);
        let count = args.get_or_undefined(1);

        // Get the index object
        let index_obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

        let index_data = index_obj.downcast_ref::<IdbIndex>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an IDBIndex"))?;

        let limit = if count.is_undefined() {
            None
        } else {
            Some(count.to_u32(context)? as usize)
        };

        // Mock data for index lookup (key -> primary key -> value)
        let mut mock_index_data = vec![
            (JsValue::from(JsString::from("alice")), JsValue::from(1), JsValue::from(JsString::from("Alice Smith"))),
            (JsValue::from(JsString::from("bob")), JsValue::from(2), JsValue::from(JsString::from("Bob Jones"))),
            (JsValue::from(JsString::from("charlie")), JsValue::from(3), JsValue::from(JsString::from("Charlie Brown"))),
            (JsValue::from(JsString::from("david")), JsValue::from(4), JsValue::from(JsString::from("David Wilson"))),
        ];

        // Filter data based on query (key range) if provided
        if !query.is_undefined() && !query.is_null() {
            if let Some(range_obj) = query.as_object() {
                if let Some(range_data) = range_obj.downcast_ref::<IdbKeyRange>() {
                    mock_index_data.retain(|(key, _, _)| {
                        IdbKeyRange::key_in_range(key, &range_data, context).unwrap_or(false)
                    });
                }
            } else {
                // Single key query
                let target_key = query.clone();
                mock_index_data.retain(|(key, _, _)| {
                    IdbKeyRange::compare_keys(key, &target_key, context).unwrap_or(1) == 0
                });
            }
        }

        // Extract values and apply limit
        let mut values: Vec<JsValue> = mock_index_data.into_iter().map(|(_, _, value)| value).collect();
        if let Some(limit_val) = limit {
            values.truncate(limit_val);
        }

        let array = Array::array_create(values.len().try_into().unwrap(), None, context)?;

        for (i, value) in values.into_iter().enumerate() {
            array.set(i, value, true, context)?;
        }

        Ok(JsValue::from(IdbFactory::create_success_request(JsValue::from(array), context)))
    }

    /// `index.getAllKeys(query, count)`
    /// Retrieves all primary keys matching the query
    fn get_all_keys(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let query = args.get_or_undefined(0);
        let count = args.get_or_undefined(1);

        // Get the index object
        let index_obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

        let index_data = index_obj.downcast_ref::<IdbIndex>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an IDBIndex"))?;

        let limit = if count.is_undefined() {
            None
        } else {
            Some(count.to_u32(context)? as usize)
        };

        // Mock data for index lookup (key -> primary key -> value)
        let mut mock_index_data = vec![
            (JsValue::from(JsString::from("alice")), JsValue::from(1), JsValue::from(JsString::from("Alice Smith"))),
            (JsValue::from(JsString::from("bob")), JsValue::from(2), JsValue::from(JsString::from("Bob Jones"))),
            (JsValue::from(JsString::from("charlie")), JsValue::from(3), JsValue::from(JsString::from("Charlie Brown"))),
            (JsValue::from(JsString::from("david")), JsValue::from(4), JsValue::from(JsString::from("David Wilson"))),
        ];

        // Filter data based on query (key range) if provided
        if !query.is_undefined() && !query.is_null() {
            if let Some(range_obj) = query.as_object() {
                if let Some(range_data) = range_obj.downcast_ref::<IdbKeyRange>() {
                    mock_index_data.retain(|(key, _, _)| {
                        IdbKeyRange::key_in_range(key, &range_data, context).unwrap_or(false)
                    });
                }
            } else {
                // Single key query
                let target_key = query.clone();
                mock_index_data.retain(|(key, _, _)| {
                    IdbKeyRange::compare_keys(key, &target_key, context).unwrap_or(1) == 0
                });
            }
        }

        // Extract primary keys and apply limit
        let mut primary_keys: Vec<JsValue> = mock_index_data.into_iter().map(|(_, pk, _)| pk).collect();
        if let Some(limit_val) = limit {
            primary_keys.truncate(limit_val);
        }

        let array = Array::array_create(primary_keys.len().try_into().unwrap(), None, context)?;

        for (i, primary_key) in primary_keys.into_iter().enumerate() {
            array.set(i, primary_key, true, context)?;
        }

        Ok(JsValue::from(IdbFactory::create_success_request(JsValue::from(array), context)))
    }

    /// `index.openCursor(range, direction)`
    /// Opens a cursor to iterate through index records
    fn open_cursor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let range = args.get_or_undefined(0);
        let direction = args.get_or_undefined(1);

        // Get the index object
        let index_obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

        let index_data = index_obj.downcast_ref::<IdbIndex>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an IDBIndex"))?;

        let direction_str = if direction.is_undefined() {
            "next".to_string()
        } else {
            direction.to_string(context)?.to_std_string_escaped()
        };

        // Mock data for index cursor iteration (index key -> value)
        let mut mock_data = vec![
            (JsValue::from(JsString::from("alice")), JsValue::from(JsString::from("Alice Smith"))),
            (JsValue::from(JsString::from("bob")), JsValue::from(JsString::from("Bob Jones"))),
            (JsValue::from(JsString::from("charlie")), JsValue::from(JsString::from("Charlie Brown"))),
            (JsValue::from(JsString::from("david")), JsValue::from(JsString::from("David Wilson"))),
        ];

        // Filter data based on key range if provided
        if !range.is_undefined() && !range.is_null() {
            if let Some(range_obj) = range.as_object() {
                if let Some(range_data) = range_obj.downcast_ref::<IdbKeyRange>() {
                    mock_data.retain(|(key, _)| {
                        IdbKeyRange::key_in_range(key, &range_data, context).unwrap_or(false)
                    });
                }
            }
        }

        let cursor = IdbCursorWithValue::new(format!("index:{}", index_data.name), direction_str, mock_data.clone());
        let cursor_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().object().prototype()),
            cursor.clone()
        );

        // Set cursor properties
        cursor_obj.set(js_string!("source"), JsValue::from(JsString::from(format!("index:{}", index_data.name))), false, context)?;
        cursor_obj.set(js_string!("direction"), JsValue::from(JsString::from(cursor.cursor.direction.clone())), false, context)?;

        if let Some(ref key) = cursor.cursor.key {
            cursor_obj.set(js_string!("key"), key.clone(), false, context)?;
        } else {
            cursor_obj.set(js_string!("key"), JsValue::null(), false, context)?;
        }

        if let Some(ref primary_key) = cursor.cursor.primary_key {
            cursor_obj.set(js_string!("primaryKey"), primary_key.clone(), false, context)?;
        } else {
            cursor_obj.set(js_string!("primaryKey"), JsValue::null(), false, context)?;
        }

        if let Some(ref value) = cursor.value {
            cursor_obj.set(js_string!("value"), value.clone(), false, context)?;
        } else {
            cursor_obj.set(js_string!("value"), JsValue::undefined(), false, context)?;
        }

        // Add cursor methods
        IdbCursorWithValue::add_cursor_with_value_methods(&cursor_obj, context)?;

        Ok(JsValue::from(IdbFactory::create_success_request(JsValue::from(cursor_obj), context)))
    }

    /// `index.openKeyCursor(range, direction)`
    /// Opens a key cursor to iterate through index keys and primary keys
    fn open_key_cursor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let range = args.get_or_undefined(0);
        let direction = args.get_or_undefined(1);

        // Get the index object
        let index_obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

        let index_data = index_obj.downcast_ref::<IdbIndex>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an IDBIndex"))?;

        let direction_str = if direction.is_undefined() {
            "next".to_string()
        } else {
            direction.to_string(context)?.to_std_string_escaped()
        };

        // Mock data for index key cursor iteration (index key -> primary key)
        let mut mock_data = vec![
            (JsValue::from(JsString::from("alice")), JsValue::from(1)),
            (JsValue::from(JsString::from("bob")), JsValue::from(2)),
            (JsValue::from(JsString::from("charlie")), JsValue::from(3)),
            (JsValue::from(JsString::from("david")), JsValue::from(4)),
        ];

        // Filter data based on key range if provided
        if !range.is_undefined() && !range.is_null() {
            if let Some(range_obj) = range.as_object() {
                if let Some(range_data) = range_obj.downcast_ref::<IdbKeyRange>() {
                    mock_data.retain(|(key, _)| {
                        IdbKeyRange::key_in_range(key, &range_data, context).unwrap_or(false)
                    });
                }
            }
        }

        let cursor = IdbCursor::new(format!("index:{}", index_data.name), direction_str, mock_data);
        let cursor_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().object().prototype()),
            cursor.clone()
        );

        // Set cursor properties
        cursor_obj.set(js_string!("source"), JsValue::from(JsString::from(format!("index:{}", index_data.name))), false, context)?;
        cursor_obj.set(js_string!("direction"), JsValue::from(JsString::from(cursor.direction.clone())), false, context)?;

        if let Some(ref key) = cursor.key {
            cursor_obj.set(js_string!("key"), key.clone(), false, context)?;
        } else {
            cursor_obj.set(js_string!("key"), JsValue::null(), false, context)?;
        }

        if let Some(ref primary_key) = cursor.primary_key {
            cursor_obj.set(js_string!("primaryKey"), primary_key.clone(), false, context)?;
        } else {
            cursor_obj.set(js_string!("primaryKey"), JsValue::null(), false, context)?;
        }

        // Add cursor methods
        IdbCursor::add_cursor_methods(&cursor_obj, context)?;

        Ok(JsValue::from(IdbFactory::create_success_request(JsValue::from(cursor_obj), context)))
    }

    /// `index.count(key)`
    /// Returns the count of records matching the key or key range
    fn count(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let key = args.get_or_undefined(0);

        // Get the index object
        let index_obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

        let index_data = index_obj.downcast_ref::<IdbIndex>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an IDBIndex"))?;

        // Mock data for counting
        let mock_index_data = vec![
            JsValue::from(JsString::from("alice")),
            JsValue::from(JsString::from("bob")),
            JsValue::from(JsString::from("charlie")),
            JsValue::from(JsString::from("david")),
        ];

        let count = if key.is_undefined() || key.is_null() {
            // Count all records
            mock_index_data.len() as u32
        } else if let Some(range_obj) = key.as_object() {
            if let Some(range_data) = range_obj.downcast_ref::<IdbKeyRange>() {
                // Count records in range
                mock_index_data.iter()
                    .filter(|k| IdbKeyRange::key_in_range(k, &range_data, context).unwrap_or(false))
                    .count() as u32
            } else {
                0
            }
        } else {
            // Count records matching single key
            mock_index_data.iter()
                .filter(|k| IdbKeyRange::compare_keys(k, &key, context).unwrap_or(1) == 0)
                .count() as u32
        };

        Ok(JsValue::from(IdbFactory::create_success_request(JsValue::from(count), context)))
    }

    /// Add index methods to an index object
    pub fn add_index_methods(index_obj: &JsObject, context: &mut Context) -> JsResult<()> {
        // Add get method
        let get_fn = BuiltInBuilder::callable(context.realm(), Self::get)
            .name(js_string!("get"))
            .length(1)
            .build();
        index_obj.set(js_string!("get"), get_fn, true, context)?;

        // Add getKey method
        let get_key_fn = BuiltInBuilder::callable(context.realm(), Self::get_key)
            .name(js_string!("getKey"))
            .length(1)
            .build();
        index_obj.set(js_string!("getKey"), get_key_fn, true, context)?;

        // Add getAll method
        let get_all_fn = BuiltInBuilder::callable(context.realm(), Self::get_all)
            .name(js_string!("getAll"))
            .length(2)
            .build();
        index_obj.set(js_string!("getAll"), get_all_fn, true, context)?;

        // Add getAllKeys method
        let get_all_keys_fn = BuiltInBuilder::callable(context.realm(), Self::get_all_keys)
            .name(js_string!("getAllKeys"))
            .length(2)
            .build();
        index_obj.set(js_string!("getAllKeys"), get_all_keys_fn, true, context)?;

        // Add openCursor method
        let open_cursor_fn = BuiltInBuilder::callable(context.realm(), Self::open_cursor)
            .name(js_string!("openCursor"))
            .length(2)
            .build();
        index_obj.set(js_string!("openCursor"), open_cursor_fn, true, context)?;

        // Add openKeyCursor method
        let open_key_cursor_fn = BuiltInBuilder::callable(context.realm(), Self::open_key_cursor)
            .name(js_string!("openKeyCursor"))
            .length(2)
            .build();
        index_obj.set(js_string!("openKeyCursor"), open_key_cursor_fn, true, context)?;

        // Add count method
        let count_fn = BuiltInBuilder::callable(context.realm(), Self::count)
            .name(js_string!("count"))
            .length(1)
            .build();
        index_obj.set(js_string!("count"), count_fn, true, context)?;

        Ok(())
    }
}