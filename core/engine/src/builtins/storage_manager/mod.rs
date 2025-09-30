//! Implementation of the `StorageManager` Web API.
//!
//! The `StorageManager` interface provides access to storage quota and usage information.
//! It's accessible via `navigator.storage`.
//!
//! More information:
//! - [WHATWG Specification](https://storage.spec.whatwg.org/)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/StorageManager)

use boa_gc::{Finalize, Trace};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use crate::{
    builtins::BuiltInBuilder,
    context::intrinsics::Intrinsics,
    js_string,
    object::{JsObject, JsPromise},
    property::Attribute,
    realm::Realm,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, JsValue,
};
use crate::builtins::{BuiltInConstructor, BuiltInObject, IntrinsicObject};
use crate::context::intrinsics::StandardConstructor;

/// `StorageManager` implementation for the Storage Standard.
#[derive(Debug, Clone, Finalize)]
pub struct StorageManager {
    /// Storage quota in bytes (default 10GB)
    quota: u64,
    /// Current usage tracking
    usage_cache: std::sync::Arc<std::sync::RwLock<HashMap<String, u64>>>,
}

unsafe impl Trace for StorageManager {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // No GC'd objects in StorageManager, nothing to trace
    }

    unsafe fn trace_non_roots(&self) {
        // No GC'd objects in StorageManager, nothing to trace
    }

    fn run_finalizer(&self) {
        // No cleanup needed for StorageManager
    }
}

impl JsData for StorageManager {}

impl StorageManager {
    /// Creates a new `StorageManager` instance.
    pub(crate) fn new() -> Self {
        Self {
            quota: 10 * 1024 * 1024 * 1024, // 10GB default quota
            usage_cache: std::sync::Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Calculate current storage usage across all storage APIs
    fn calculate_storage_usage(&self) -> u64 {
        let mut total_usage = 0u64;

        // Calculate Web Storage usage (localStorage + sessionStorage)
        total_usage += self.calculate_web_storage_usage();

        // Calculate IndexedDB usage
        total_usage += self.calculate_indexeddb_usage();

        // Cache API usage would go here when implemented
        // File System API usage would go here when implemented

        total_usage
    }

    /// Calculate Web Storage usage
    fn calculate_web_storage_usage(&self) -> u64 {
        let mut usage = 0u64;
        let storage_dir = self.get_web_storage_dir();

        if storage_dir.exists() {
            for entry in fs::read_dir(storage_dir).unwrap_or_else(|_| fs::read_dir(".").unwrap()) {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                        if let Ok(metadata) = fs::metadata(&path) {
                            usage += metadata.len();
                        }
                    }
                }
            }
        }

        usage
    }

    /// Calculate IndexedDB usage
    fn calculate_indexeddb_usage(&self) -> u64 {
        let mut usage = 0u64;
        let idb_dir = self.get_indexeddb_dir();

        if idb_dir.exists() {
            for entry in fs::read_dir(idb_dir).unwrap_or_else(|_| fs::read_dir(".").unwrap()) {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(metadata) = fs::metadata(&path) {
                            usage += metadata.len();
                        }
                    }
                }
            }
        }

        usage
    }

    /// Get Web Storage directory
    fn get_web_storage_dir(&self) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push("boa_web_storage");
        path
    }

    /// Get IndexedDB directory
    fn get_indexeddb_dir(&self) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push("boa_indexeddb");
        path
    }

    /// Creates a StorageManager instance for navigator.storage
    pub fn create_storage_manager() -> JsObject {
        let manager = StorageManager::new();
        JsObject::from_proto_and_data(None, manager)
    }
}

impl IntrinsicObject for StorageManager {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::estimate, js_string!("estimate"), 0)
            .method(Self::persist, js_string!("persist"), 0)
            .method(Self::persisted, js_string!("persisted"), 0)
            .method(Self::get_directory, js_string!("getDirectory"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for StorageManager {
    const NAME: JsString = js_string!("StorageManager");
}

impl BuiltInConstructor for StorageManager {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &StandardConstructor =
        |constructors| constructors.storage_manager();

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // StorageManager constructor is not meant to be called directly
        Err(JsNativeError::typ()
            .with_message("StorageManager constructor cannot be called directly")
            .into())
    }
}

// StorageManager prototype methods
impl StorageManager {
    /// `navigator.storage.estimate()`
    /// Returns a Promise that resolves to storage quota and usage information
    fn estimate(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageManager object")
            })?;

        let manager = obj.downcast_ref::<StorageManager>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageManager object")
            })?;

        let usage = manager.calculate_storage_usage();
        let quota = manager.quota;

        // Create estimate result object
        let estimate_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().object().prototype()),
            ()
        );

        estimate_obj.set(js_string!("quota"), JsValue::from(quota), false, context)?;
        estimate_obj.set(js_string!("usage"), JsValue::from(usage), false, context)?;
        estimate_obj.set(js_string!("usageDetails"), JsValue::undefined(), false, context)?;

        // Return a resolved Promise
        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::from(estimate_obj)], context)?;

        Ok(JsValue::from(promise))
    }

    /// `navigator.storage.persist()`
    /// Requests persistent storage permission
    fn persist(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // In a headless browser, we can always grant persistent storage
        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::from(true)], context)?;

        Ok(JsValue::from(promise))
    }

    /// `navigator.storage.persisted()`
    /// Returns whether storage is persistent
    fn persisted(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // In a headless browser, storage is always persistent
        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::from(true)], context)?;

        Ok(JsValue::from(promise))
    }

    /// `navigator.storage.getDirectory()`
    /// Returns the origin private file system directory
    fn get_directory(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // This would return a FileSystemDirectoryHandle when File System API is implemented
        // For now, return a rejected promise
        let (promise, resolvers) = JsPromise::new_pending(context);
        let error_value = JsNativeError::error()
            .with_message("File System API not yet implemented")
            .to_opaque(context);
        resolvers.reject.call(&JsValue::undefined(), &[error_value.into()], context)?;

        Ok(JsValue::from(promise))
    }
}

#[cfg(test)]
mod tests;