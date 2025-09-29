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
use std::path::PathBuf;
use std::fs;
use boa_gc::{Finalize, Trace};
use serde::{Serialize, Deserialize};
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
    storage_path: PathBuf,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IdbDatabaseInfo {
    name: String,
    version: u32,
    object_stores: HashMap<String, IdbObjectStoreInfo>,
}

/// Information about an IndexedDB object store
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IdbObjectStoreInfo {
    name: String,
    key_path: Option<String>,
    auto_increment: bool,
    indexes: HashMap<String, IdbIndexInfo>,
    data: HashMap<String, serde_json::Value>, // Store actual data
    auto_increment_counter: u32,
}

impl IdbObjectStoreInfo {
    fn new(name: String, key_path: Option<String>, auto_increment: bool) -> Self {
        Self {
            name,
            key_path,
            auto_increment,
            indexes: HashMap::new(),
            data: HashMap::new(),
            auto_increment_counter: 1,
        }
    }
}

/// Information about an IndexedDB index
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    on_success: Option<JsValue>,
    on_error: Option<JsValue>,
    source: Option<JsValue>,
    transaction: Option<JsValue>,
}

unsafe impl Trace for IdbRequest {
    unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) {
        if let Some(ref result) = self.result {
            unsafe { result.trace(tracer); }
        }
        if let Some(ref error) = self.error {
            unsafe { error.trace(tracer); }
        }
        if let Some(ref on_success) = self.on_success {
            unsafe { on_success.trace(tracer); }
        }
        if let Some(ref on_error) = self.on_error {
            unsafe { on_error.trace(tracer); }
        }
        if let Some(ref source) = self.source {
            unsafe { source.trace(tracer); }
        }
        if let Some(ref transaction) = self.transaction {
            unsafe { transaction.trace(tracer); }
        }
    }

    unsafe fn trace_non_roots(&self) {
        if let Some(ref result) = self.result {
            unsafe { result.trace_non_roots(); }
        }
        if let Some(ref error) = self.error {
            unsafe { error.trace_non_roots(); }
        }
        if let Some(ref on_success) = self.on_success {
            unsafe { on_success.trace_non_roots(); }
        }
        if let Some(ref on_error) = self.on_error {
            unsafe { on_error.trace_non_roots(); }
        }
        if let Some(ref source) = self.source {
            unsafe { source.trace_non_roots(); }
        }
        if let Some(ref transaction) = self.transaction {
            unsafe { transaction.trace_non_roots(); }
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
        let storage_path = Self::get_storage_path();
        let databases = Self::load_databases(&storage_path);
        Self {
            databases: Arc::new(RwLock::new(databases)),
            storage_path,
        }
    }

    /// Get the storage path for IndexedDB data
    fn get_storage_path() -> PathBuf {
        // Use a standard location for IndexedDB storage
        let mut path = std::env::temp_dir();
        path.push("boa_indexeddb");
        if !path.exists() {
            fs::create_dir_all(&path).ok();
        }
        path
    }

    /// Load databases from persistent storage
    fn load_databases(storage_path: &PathBuf) -> HashMap<String, IdbDatabaseInfo> {
        let db_list_path = storage_path.join("databases.json");
        if db_list_path.exists() {
            if let Ok(content) = fs::read_to_string(&db_list_path) {
                if let Ok(databases) = serde_json::from_str(&content) {
                    return databases;
                }
            }
        }
        HashMap::new()
    }

    /// Save databases to persistent storage
    fn save_databases(&self) -> Result<(), Box<dyn std::error::Error>> {
        let databases = self.databases.read().map_err(|_| "Lock error")?;
        let db_list_path = self.storage_path.join("databases.json");
        let content = serde_json::to_string_pretty(&*databases)?;
        fs::write(&db_list_path, content)?;
        Ok(())
    }

    /// Save a specific database to storage
    fn save_database(&self, db_info: &IdbDatabaseInfo) -> Result<(), Box<dyn std::error::Error>> {
        let db_path = self.storage_path.join(format!("{}.json", db_info.name));
        let content = serde_json::to_string_pretty(db_info)?;
        fs::write(&db_path, content)?;
        self.save_databases()?;
        Ok(())
    }

    /// Load a specific database from storage
    fn load_database(&self, name: &str) -> Option<IdbDatabaseInfo> {
        let db_path = self.storage_path.join(format!("{}.json", name));
        if db_path.exists() {
            if let Ok(content) = fs::read_to_string(&db_path) {
                if let Ok(db_info) = serde_json::from_str(&content) {
                    return Some(db_info);
                }
            }
        }
        None
    }

    /// Create an IDBRequest that completes successfully
    fn create_success_request(result: JsValue, context: &mut Context) -> JsObject {
        let request = IdbRequest {
            result: Some(result.clone()),
            error: None,
            ready_state: "done".to_string(),
            on_success: None,
            on_error: None,
            source: None,
            transaction: None,
        };

        let request_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().object().prototype()),
            request
        );

        // Set properties
        request_obj.set(js_string!("readyState"), JsValue::from(JsString::from("done")), false, context).ok();
        request_obj.set(js_string!("result"), result, false, context).ok();
        request_obj.set(js_string!("error"), JsValue::null(), false, context).ok();
        request_obj.set(js_string!("onsuccess"), JsValue::null(), true, context).ok();
        request_obj.set(js_string!("onerror"), JsValue::null(), true, context).ok();
        request_obj.set(js_string!("source"), JsValue::null(), false, context).ok();
        request_obj.set(js_string!("transaction"), JsValue::null(), false, context).ok();

        // Simulate async completion by calling onsuccess if set
        Self::fire_success_event(&request_obj, context);

        request_obj
    }

    /// Create an IDBRequest that completes with an error
    fn create_error_request(error: JsValue, context: &mut Context) -> JsObject {
        let request = IdbRequest {
            result: None,
            error: Some(error.clone()),
            ready_state: "done".to_string(),
            on_success: None,
            on_error: None,
            source: None,
            transaction: None,
        };

        let request_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().object().prototype()),
            request
        );

        // Set properties
        request_obj.set(js_string!("readyState"), JsValue::from(JsString::from("done")), false, context).ok();
        request_obj.set(js_string!("result"), JsValue::undefined(), false, context).ok();
        request_obj.set(js_string!("error"), error, false, context).ok();
        request_obj.set(js_string!("onsuccess"), JsValue::null(), true, context).ok();
        request_obj.set(js_string!("onerror"), JsValue::null(), true, context).ok();
        request_obj.set(js_string!("source"), JsValue::null(), false, context).ok();
        request_obj.set(js_string!("transaction"), JsValue::null(), false, context).ok();

        // Simulate async completion by calling onerror if set
        Self::fire_error_event(&request_obj, context);

        request_obj
    }

    /// Fire success event on request
    fn fire_success_event(request_obj: &JsObject, context: &mut Context) {
        if let Ok(onsuccess) = request_obj.get(js_string!("onsuccess"), context) {
            if onsuccess.is_callable() {
                // Create event object
                let event = Self::create_event("success", request_obj, context);
                // Call the handler with the event
                let _ = onsuccess.as_callable().unwrap().call(&JsValue::undefined(), &[event], context);
            }
        }
    }

    /// Fire error event on request
    fn fire_error_event(request_obj: &JsObject, context: &mut Context) {
        if let Ok(onerror) = request_obj.get(js_string!("onerror"), context) {
            if onerror.is_callable() {
                // Create event object
                let event = Self::create_event("error", request_obj, context);
                // Call the handler with the event
                let _ = onerror.as_callable().unwrap().call(&JsValue::undefined(), &[event], context);
            }
        }
    }

    /// Create an event object for IndexedDB events
    fn create_event(event_type: &str, target: &JsObject, context: &mut Context) -> JsValue {
        let event_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().object().prototype()),
            ()
        );

        event_obj.set(js_string!("type"), JsValue::from(JsString::from(event_type)), false, context).ok();
        event_obj.set(js_string!("target"), JsValue::from(target.clone()), false, context).ok();
        event_obj.set(js_string!("currentTarget"), JsValue::from(target.clone()), false, context).ok();
        event_obj.set(js_string!("bubbles"), JsValue::from(false), false, context).ok();
        event_obj.set(js_string!("cancelable"), JsValue::from(false), false, context).ok();

        JsValue::from(event_obj)
    }

    /// Create an upgrade request for database version changes
    fn create_upgrade_request(result: JsValue, new_version: u32, context: &mut Context) -> JsObject {
        let request = IdbRequest {
            result: Some(result.clone()),
            error: None,
            ready_state: "done".to_string(),
            on_success: None,
            on_error: None,
            source: None,
            transaction: None,
        };

        let request_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().object().prototype()),
            request
        );

        // Set properties
        request_obj.set(js_string!("readyState"), JsValue::from(JsString::from("done")), false, context).ok();
        request_obj.set(js_string!("result"), result, false, context).ok();
        request_obj.set(js_string!("error"), JsValue::null(), false, context).ok();
        request_obj.set(js_string!("onsuccess"), JsValue::null(), true, context).ok();
        request_obj.set(js_string!("onerror"), JsValue::null(), true, context).ok();
        request_obj.set(js_string!("onupgradeneeded"), JsValue::null(), true, context).ok();
        request_obj.set(js_string!("source"), JsValue::null(), false, context).ok();
        request_obj.set(js_string!("transaction"), JsValue::null(), false, context).ok();

        // Fire upgrade needed event
        Self::fire_upgrade_needed_event(&request_obj, new_version, context);

        request_obj
    }

    /// Fire upgrade needed event
    fn fire_upgrade_needed_event(request_obj: &JsObject, new_version: u32, context: &mut Context) {
        if let Ok(onupgradeneeded) = request_obj.get(js_string!("onupgradeneeded"), context) {
            if onupgradeneeded.is_callable() {
                // Create upgrade event object
                let event_obj = JsObject::from_proto_and_data(
                    Some(context.intrinsics().constructors().object().prototype()),
                    ()
                );

                event_obj.set(js_string!("type"), JsValue::from(JsString::from("upgradeneeded")), false, context).ok();
                event_obj.set(js_string!("target"), JsValue::from(request_obj.clone()), false, context).ok();
                event_obj.set(js_string!("currentTarget"), JsValue::from(request_obj.clone()), false, context).ok();
                event_obj.set(js_string!("newVersion"), JsValue::from(new_version), false, context).ok();
                event_obj.set(js_string!("oldVersion"), JsValue::from(1), false, context).ok(); // Simplified

                let event = JsValue::from(event_obj);
                // Call the handler with the event
                let _ = onupgradeneeded.as_callable().unwrap().call(&JsValue::undefined(), &[event], context);
            }
        }
    }

    /// Add database methods to a database object
    fn add_database_methods(db_obj: &JsObject, context: &mut Context) -> JsResult<()> {
        // Add transaction method
        let transaction_fn = BuiltInBuilder::callable(context.realm(), Self::transaction)
            .name(js_string!("transaction"))
            .length(2)
            .build();

        db_obj.set(
            js_string!("transaction"),
            transaction_fn,
            true,
            context
        )?;

        // Add createObjectStore method
        let create_store_fn = BuiltInBuilder::callable(context.realm(), Self::create_object_store)
            .name(js_string!("createObjectStore"))
            .length(2)
            .build();

        db_obj.set(
            js_string!("createObjectStore"),
            create_store_fn,
            true,
            context
        )?;

        // Add close method
        let close_fn = BuiltInBuilder::callable(context.realm(), Self::close_database)
            .name(js_string!("close"))
            .length(0)
            .build();

        db_obj.set(
            js_string!("close"),
            close_fn,
            true,
            context
        )?;

        Ok(())
    }

    /// `db.close()`
    fn close_database(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // In a real implementation, this would close the database connection
        // For now, it's a no-op
        Ok(JsValue::undefined())
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

        // Validate that name argument is provided
        if args.is_empty() {
            return Err(JsNativeError::typ()
                .with_message("Failed to execute 'open' on 'IDBFactory': 1 argument required, but only 0 present.")
                .into());
        }

        let name_arg = args.get_or_undefined(0);
        if name_arg.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Failed to execute 'open' on 'IDBFactory': The database name provided is undefined.")
                .into());
        }

        let name = name_arg.to_string(context)?.to_std_string_escaped();
        let version = if args.len() > 1 {
            args.get_or_undefined(1).to_u32(context)?
        } else {
            1
        };

        // Check if database exists and needs upgrade
        let mut databases = _factory.databases.write().map_err(|_| {
            JsNativeError::error().with_message("Database lock error")
        })?;

        let needs_upgrade = if let Some(existing_db) = databases.get(&name) {
            version > existing_db.version
        } else {
            true // New database
        };

        // Create or update database info
        let db_info = if needs_upgrade {
            let mut new_db_info = IdbDatabaseInfo {
                name: name.clone(),
                version,
                object_stores: HashMap::new(),
            };

            // If upgrading, preserve existing object stores
            if let Some(existing_db) = databases.get(&name) {
                new_db_info.object_stores = existing_db.object_stores.clone();
            }

            databases.insert(name.clone(), new_db_info.clone());

            // Save to persistent storage
            drop(databases); // Release lock before save
            if let Err(e) = _factory.save_database(&new_db_info) {
                return Err(JsNativeError::error()
                    .with_message(format!("Failed to save database: {}", e))
                    .into());
            }
            new_db_info
        } else {
            databases.get(&name).unwrap().clone()
        };

        // Create database object
        let db = IdbDatabase {
            name: name.clone(),
            version: db_info.version,
            object_store_names: db_info.object_stores.keys().cloned().collect(),
        };

        let db_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().object().prototype()),
            db
        );

        // Set database properties
        db_obj.set(js_string!("name"), JsValue::from(JsString::from(name.clone())), false, context)?;
        db_obj.set(js_string!("version"), JsValue::from(db_info.version), false, context)?;
        let store_names: Vec<JsValue> = db_info.object_stores.keys()
            .map(|name| JsValue::from(JsString::from(name.clone())))
            .collect();
        let stores_array = Array::create_array_from_list(store_names, context);
        db_obj.set(js_string!("objectStoreNames"), JsValue::from(stores_array), false, context)?;

        // Add database methods
        Self::add_database_methods(&db_obj, context)?;

        // Create request
        let request_obj = if needs_upgrade {
            Self::create_upgrade_request(JsValue::from(db_obj), version, context)
        } else {
            Self::create_success_request(JsValue::from(db_obj), context)
        };

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

        // Validate that name argument is provided
        if args.is_empty() {
            return Err(JsNativeError::typ()
                .with_message("Failed to execute 'deleteDatabase' on 'IDBFactory': 1 argument required, but only 0 present.")
                .into());
        }

        let name_arg = args.get_or_undefined(0);
        if name_arg.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Failed to execute 'deleteDatabase' on 'IDBFactory': The database name provided is undefined.")
                .into());
        }

        let name = name_arg.to_string(context)?.to_std_string_escaped();

        // Remove database from storage
        let mut databases = _factory.databases.write().map_err(|_| {
            JsNativeError::error().with_message("Database lock error")
        })?;

        databases.remove(&name);
        drop(databases);

        // Delete database file
        let db_path = _factory.storage_path.join(format!("{}.json", name));
        if db_path.exists() {
            let _ = fs::remove_file(&db_path);
        }

        // Save updated database list
        let _ = _factory.save_databases();

        // Create success request
        let request_obj = Self::create_success_request(JsValue::undefined(), context);

        // Simulate async completion would require proper job queue integration
        // For now, just return the request immediately

        Ok(JsValue::from(request_obj))
    }

    /// `indexedDB.databases()`
    fn databases(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not an IDBFactory object")
            })?;

        let factory = obj.downcast_ref::<IdbFactory>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not an IDBFactory object")
            })?;

        // Get database list
        let databases = factory.databases.read().map_err(|_| {
            JsNativeError::error().with_message("Database lock error")
        })?;

        let db_list: Vec<JsValue> = databases.iter().map(|(name, info)| {
            let db_info = JsObject::from_proto_and_data(
                Some(context.intrinsics().constructors().object().prototype()),
                ()
            );
            db_info.set(js_string!("name"), JsValue::from(JsString::from(name.clone())), false, context).ok();
            db_info.set(js_string!("version"), JsValue::from(info.version), false, context).ok();
            JsValue::from(db_info)
        }).collect();

        drop(databases);

        let db_array = Array::create_array_from_list(db_list, context);
        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::from(db_array)], context).ok();
        Ok(JsValue::from(promise))
    }

    /// `indexedDB.cmp(first, second)`
    fn cmp(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Validate that both arguments are provided
        if args.len() < 2 {
            let error_msg = format!("Failed to execute 'cmp' on 'IDBFactory': 2 arguments required, but only {} present.", args.len());
            return Err(JsNativeError::typ()
                .with_message(error_msg)
                .into());
        }

        let first = args.get_or_undefined(0);
        let second = args.get_or_undefined(1);

        // Validate that arguments are not undefined
        if first.is_undefined() || second.is_undefined() {
            return Err(JsNativeError::error()
                .with_message("Failed to execute 'cmp' on 'IDBFactory': The parameter is not a valid key.")
                .into());
        }

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
    fn create_object_store(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let options = args.get_or_undefined(1);

        // Parse options
        let mut key_path: Option<String> = None;
        let mut auto_increment = false;

        if let Some(options_obj) = options.as_object() {
            if let Ok(key_path_val) = options_obj.get(js_string!("keyPath"), context) {
                if !key_path_val.is_undefined() && !key_path_val.is_null() {
                    key_path = Some(key_path_val.to_string(context)?.to_std_string_escaped());
                }
            }
            if let Ok(auto_inc_val) = options_obj.get(js_string!("autoIncrement"), context) {
                auto_increment = auto_inc_val.to_boolean();
            }
        }

        // Create object store
        let object_store = IdbObjectStore {
            name: name.clone(),
            key_path: key_path.clone(),
            auto_increment,
        };

        let store_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().object().prototype()),
            object_store
        );

        // Set properties
        store_obj.set(js_string!("name"), JsValue::from(JsString::from(name.clone())), false, context)?;
        store_obj.set(js_string!("keyPath"),
            if let Some(ref kp) = key_path {
                JsValue::from(JsString::from(kp.clone()))
            } else {
                JsValue::null()
            },
            false, context)?;
        store_obj.set(js_string!("autoIncrement"), JsValue::from(auto_increment), false, context)?;

        // Add object store methods
        Self::add_object_store_methods(&store_obj, context)?;

        // TODO: In a real implementation, we would add this store to the database schema
        // This requires access to the database object and proper transaction handling

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

    /// Add object store methods to an object store object
    fn add_object_store_methods(store_obj: &JsObject, context: &mut Context) -> JsResult<()> {
        // Add add method
        let add_fn = BuiltInBuilder::callable(context.realm(), Self::store_add)
            .name(js_string!("add"))
            .length(2)
            .build();

        store_obj.set(
            js_string!("add"),
            add_fn,
            true,
            context
        )?;

        // Add put method
        let put_fn = BuiltInBuilder::callable(context.realm(), Self::store_put)
            .name(js_string!("put"))
            .length(2)
            .build();

        store_obj.set(
            js_string!("put"),
            put_fn,
            true,
            context
        )?;

        // Add get method
        let get_fn = BuiltInBuilder::callable(context.realm(), Self::store_get)
            .name(js_string!("get"))
            .length(1)
            .build();

        store_obj.set(
            js_string!("get"),
            get_fn,
            true,
            context
        )?;

        // Add delete method
        let delete_fn = BuiltInBuilder::callable(context.realm(), Self::store_delete)
            .name(js_string!("delete"))
            .length(1)
            .build();

        store_obj.set(
            js_string!("delete"),
            delete_fn,
            true,
            context
        )?;

        Ok(())
    }

    /// Create an IDBFactory instance for window.indexedDB
    pub fn create_indexed_db() -> JsObject {
        let factory = IdbFactory::new();
        JsObject::from_proto_and_data(None, factory)
    }
}