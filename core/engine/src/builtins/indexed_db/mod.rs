//! Implementation of the IndexedDB API.
//!
//! The IndexedDB API provides a way for applications to store significant amounts
//! of structured data, including files/blobs, in the client. It uses indexes to
//! enable high-performance searches of this data.
//!
//! More information:
//! - [W3C IndexedDB 3.0 Specification](https://w3c.github.io/IndexedDB/)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API)

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
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
use crate::builtins::{BuiltInConstructor, BuiltInObject, IntrinsicObject};
use crate::context::intrinsics::StandardConstructor;

#[cfg(test)]
mod tests;

/// IndexedDB factory object that provides the main entry point
#[derive(Debug, Clone, Finalize)]
pub struct IdbFactory {
    databases: Arc<RwLock<HashMap<String, IdbDatabaseInfo>>>,
}

unsafe impl Trace for IdbFactory {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // No GC'd objects in IdbFactory, nothing to trace
    }

    unsafe fn trace_non_roots(&self) {
        // No GC'd objects in IdbFactory, nothing to trace
    }

    fn run_finalizer(&self) {
        // No cleanup needed for IdbFactory
    }
}

impl JsData for IdbFactory {}

/// Information about an IndexedDB database
#[derive(Debug, Clone)]
struct IdbDatabaseInfo {
    name: String,
    version: u32,
    object_stores: HashMap<String, IdbObjectStoreInfo>,
}

/// Information about an IndexedDB object store
#[derive(Debug, Clone)]
struct IdbObjectStoreInfo {
    name: String,
    key_path: Option<String>,
    auto_increment: bool,
    indexes: HashMap<String, IdbIndexInfo>,
}

/// Information about an IndexedDB index
#[derive(Debug, Clone)]
struct IdbIndexInfo {
    name: String,
    key_path: String,
    unique: bool,
}

/// IDBRequest object for asynchronous operations
#[derive(Debug, Clone, Finalize)]
pub struct IdbRequest {
    result: Option<JsValue>,
    error: Option<JsValue>,
    ready_state: String,
}

unsafe impl Trace for IdbRequest {
    unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) {
        if let Some(ref result) = self.result {
            unsafe { result.trace(tracer); }
        }
        if let Some(ref error) = self.error {
            unsafe { error.trace(tracer); }
        }
    }

    unsafe fn trace_non_roots(&self) {
        if let Some(ref result) = self.result {
            unsafe { result.trace_non_roots(); }
        }
        if let Some(ref error) = self.error {
            unsafe { error.trace_non_roots(); }
        }
    }

    fn run_finalizer(&self) {
        // No cleanup needed for IdbRequest
    }
}

impl JsData for IdbRequest {}

/// IDBDatabase object representing a connection to a database
#[derive(Debug, Clone, Finalize)]
pub struct IdbDatabase {
    name: String,
    version: u32,
    object_store_names: Vec<String>,
}

unsafe impl Trace for IdbDatabase {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // No GC'd objects in IdbDatabase, nothing to trace
    }

    unsafe fn trace_non_roots(&self) {
        // No GC'd objects in IdbDatabase, nothing to trace
    }

    fn run_finalizer(&self) {
        // No cleanup needed for IdbDatabase
    }
}

impl JsData for IdbDatabase {}

/// IDBTransaction object for database transactions
#[derive(Debug, Clone, Finalize)]
pub struct IdbTransaction {
    mode: String,
    object_store_names: Vec<String>,
}

unsafe impl Trace for IdbTransaction {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // No GC'd objects in IdbTransaction, nothing to trace
    }

    unsafe fn trace_non_roots(&self) {
        // No GC'd objects in IdbTransaction, nothing to trace
    }

    fn run_finalizer(&self) {
        // No cleanup needed for IdbTransaction
    }
}

impl JsData for IdbTransaction {}

/// IDBObjectStore object for object store operations
#[derive(Debug, Clone, Finalize)]
pub struct IdbObjectStore {
    name: String,
    key_path: Option<String>,
    auto_increment: bool,
}

unsafe impl Trace for IdbObjectStore {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // No GC'd objects in IdbObjectStore, nothing to trace
    }

    unsafe fn trace_non_roots(&self) {
        // No GC'd objects in IdbObjectStore, nothing to trace
    }

    fn run_finalizer(&self) {
        // No cleanup needed for IdbObjectStore
    }
}

impl JsData for IdbObjectStore {}

impl IdbFactory {
    pub(crate) fn new() -> Self {
        Self {
            databases: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a mock IDBRequest that completes successfully
    fn create_success_request(result: JsValue, context: &mut Context) -> JsObject {
        let request = IdbRequest {
            result: Some(result),
            error: None,
            ready_state: "done".to_string(),
        };

        let request_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().object().prototype()),
            request
        );

        // Set properties
        request_obj.set(js_string!("readyState"), JsValue::from(JsString::from("done")), false, context).ok();
        request_obj.set(js_string!("onsuccess"), JsValue::null(), true, context).ok();
        request_obj.set(js_string!("onerror"), JsValue::null(), true, context).ok();

        request_obj
    }
}

impl IntrinsicObject for IdbFactory {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::open, js_string!("open"), 1)
            .method(Self::delete_database, js_string!("deleteDatabase"), 1)
            .method(Self::databases, js_string!("databases"), 0)
            .method(Self::cmp, js_string!("cmp"), 2)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for IdbFactory {
    const NAME: JsString = js_string!("IDBFactory");
}

impl BuiltInConstructor for IdbFactory {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &StandardConstructor =
        |constructors| constructors.idb_factory();

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // IDBFactory constructor is not meant to be called directly
        Err(JsNativeError::typ()
            .with_message("IDBFactory constructor cannot be called directly")
            .into())
    }
}

// IDBFactory prototype methods
impl IdbFactory {
    /// `indexedDB.open(name, version)`
    pub fn open(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not an IDBFactory object")
            })?;

        let _factory = obj.downcast_ref::<IdbFactory>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not an IDBFactory object")
            })?;

        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let version = if args.len() > 1 {
            args.get_or_undefined(1).to_u32(context)?
        } else {
            1
        };

        // Create mock database
        let mock_db = IdbDatabase {
            name: name.clone(),
            version,
            object_store_names: vec![],
        };

        let db_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().object().prototype()),
            mock_db
        );

        // Set database properties
        db_obj.set(js_string!("name"), JsValue::from(JsString::from(name)), false, context)?;
        db_obj.set(js_string!("version"), JsValue::from(version), false, context)?;
        let empty_array = Array::create_array_from_list([], context);
        db_obj.set(js_string!("objectStoreNames"), JsValue::from(empty_array), false, context)?;

        // Add database methods - would need proper realm access for BuiltInBuilder
        // For now, keeping as is since this is mock implementation

        // Create request and simulate async completion
        let request_obj = Self::create_success_request(JsValue::from(db_obj), context);

        // Simulate async completion would require proper job queue integration
        // For now, just return the request immediately

        Ok(JsValue::from(request_obj))
    }

    /// `indexedDB.deleteDatabase(name)`
    fn delete_database(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not an IDBFactory object")
            })?;

        let _factory = obj.downcast_ref::<IdbFactory>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not an IDBFactory object")
            })?;

        let _name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

        // Create success request
        let request_obj = Self::create_success_request(JsValue::undefined(), context);

        // Simulate async completion would require proper job queue integration
        // For now, just return the request immediately

        Ok(JsValue::from(request_obj))
    }

    /// `indexedDB.databases()`
    fn databases(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Return empty array as Promise
        let empty_array = Array::create_array_from_list([], context);
        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::from(empty_array)], context).ok();
        Ok(JsValue::from(promise))
    }

    /// `indexedDB.cmp(first, second)`
    fn cmp(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let first = args.get_or_undefined(0);
        let second = args.get_or_undefined(1);

        // Basic comparison implementation
        if first.is_number() && second.is_number() {
            let first_num = first.to_number(context)?;
            let second_num = second.to_number(context)?;

            let result = if first_num < second_num {
                -1
            } else if first_num > second_num {
                1
            } else {
                0
            };

            Ok(JsValue::from(result))
        } else {
            // String comparison fallback
            let first_str = first.to_string(context)?.to_std_string_escaped();
            let second_str = second.to_string(context)?.to_std_string_escaped();

            let result = match first_str.cmp(&second_str) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Greater => 1,
                std::cmp::Ordering::Equal => 0,
            };

            Ok(JsValue::from(result))
        }
    }

    /// `db.transaction(storeNames, mode)`
    fn transaction(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let store_names = args.get_or_undefined(0);
        let mode = args.get_or_undefined(1).to_string(context).unwrap_or_else(|_| js_string!("readonly"));

        let store_names_vec = if store_names.is_string() {
            vec![store_names.to_string(context)?.to_std_string_escaped()]
        } else if store_names.is_object() {
            let mut names = Vec::new();
            let array_obj = store_names.as_object().unwrap();
            // Try to get length property
            if let Ok(length_val) = array_obj.get(js_string!("length"), context) {
                if let Ok(length) = length_val.to_u32(context) {
                    for i in 0..length {
                        if let Ok(name) = array_obj.get(i, context) {
                            names.push(name.to_string(context)?.to_std_string_escaped());
                        }
                    }
                }
            }
            names
        } else {
            vec![]
        };

        let transaction = IdbTransaction {
            mode: mode.to_std_string_escaped(),
            object_store_names: store_names_vec,
        };

        let transaction_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().object().prototype()),
            transaction
        );

        // Add transaction methods - would need proper realm access

        Ok(JsValue::from(transaction_obj))
    }

    /// `db.createObjectStore(name, options)`
    fn create_object_store(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let _options = args.get_or_undefined(1);

        let object_store = IdbObjectStore {
            name: name.clone(),
            key_path: None,
            auto_increment: false,
        };

        let store_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().object().prototype()),
            object_store
        );

        store_obj.set(js_string!("name"), JsValue::from(JsString::from(name)), false, context)?;

        Ok(JsValue::from(store_obj))
    }

    /// `transaction.objectStore(name)`
    fn object_store(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

        let object_store = IdbObjectStore {
            name: name.clone(),
            key_path: None,
            auto_increment: false,
        };

        let store_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().object().prototype()),
            object_store
        );

        // Set properties
        store_obj.set(js_string!("name"), JsValue::from(JsString::from(name)), false, context)?;

        // Add object store methods - would need proper realm access

        Ok(JsValue::from(store_obj))
    }

    /// `objectStore.add(value, key)`
    fn store_add(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let _value = args.get_or_undefined(0);
        let key = args.get_or_undefined(1);

        let result_key = if key.is_undefined() {
            JsValue::from(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i32)
        } else {
            key.clone()
        };

        Ok(JsValue::from(Self::create_success_request(result_key, context)))
    }

    /// `objectStore.put(value, key)`
    fn store_put(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let _value = args.get_or_undefined(0);
        let key = args.get_or_undefined(1);

        let result_key = if key.is_undefined() {
            JsValue::from(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i32)
        } else {
            key.clone()
        };

        Ok(JsValue::from(Self::create_success_request(result_key, context)))
    }

    /// `objectStore.get(key)`
    fn store_get(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Return null as no data exists in mock implementation
        Ok(JsValue::from(Self::create_success_request(JsValue::null(), context)))
    }

    /// `objectStore.delete(key)`
    fn store_delete(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(Self::create_success_request(JsValue::undefined(), context)))
    }

    /// Create an IDBFactory instance for window.indexedDB
    pub fn create_indexed_db() -> JsObject {
        let factory = IdbFactory::new();
        JsObject::from_proto_and_data(None, factory)
    }
}