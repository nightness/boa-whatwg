//! Implementation of the `CacheStorage` Web API.
//!
//! The `CacheStorage` interface provides storage for Cache objects.
//! It's accessible via the global `caches` object and provides methods
//! to manage named caches.
//!
//! More information:
//! - [WHATWG Service Worker Specification](https://w3c.github.io/ServiceWorker/#cachestorage-interface)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/CacheStorage)

use boa_gc::{Finalize, Trace};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use serde::{Serialize, Deserialize};
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
use crate::builtins::cache::Cache;

/// Serializable cache metadata for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheMetadata {
    /// Cache names
    cache_names: Vec<String>,
}

/// `CacheStorage` implementation for the Service Worker Cache API.
#[derive(Debug, Clone, Finalize)]
pub struct CacheStorage {
    /// Active cache instances
    caches: std::sync::Arc<std::sync::RwLock<HashMap<String, JsObject>>>,
    /// Metadata file path for persistence
    metadata_file: PathBuf,
}

unsafe impl Trace for CacheStorage {
    unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) {
        let caches = self.caches.read().unwrap();
        for cache in caches.values() {
            unsafe { cache.trace(tracer); }
        }
    }

    unsafe fn trace_non_roots(&self) {
        let caches = self.caches.read().unwrap();
        for cache in caches.values() {
            unsafe { cache.trace_non_roots(); }
        }
    }

    fn run_finalizer(&self) {
        // No cleanup needed for CacheStorage
    }
}

impl JsData for CacheStorage {}

impl CacheStorage {
    /// Creates a new `CacheStorage` instance.
    pub(crate) fn new() -> Self {
        let metadata_file = Self::get_metadata_file();
        let cache_names = Self::load_cache_metadata(&metadata_file);
        let mut caches_map = HashMap::new();

        // Load existing caches
        for cache_name in &cache_names.cache_names {
            let cache = Cache::create_cache(cache_name.clone());
            caches_map.insert(cache_name.clone(), cache);
        }

        Self {
            caches: std::sync::Arc::new(std::sync::RwLock::new(caches_map)),
            metadata_file,
        }
    }

    /// Get the metadata file path
    fn get_metadata_file() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push("boa_cache_api");
        if !path.exists() {
            fs::create_dir_all(&path).ok();
        }
        path.push("cache_metadata.json");
        path
    }

    /// Load cache metadata from disk
    fn load_cache_metadata(metadata_file: &PathBuf) -> CacheMetadata {
        if metadata_file.exists() {
            if let Ok(content) = fs::read_to_string(metadata_file) {
                if let Ok(metadata) = serde_json::from_str::<CacheMetadata>(&content) {
                    return metadata;
                }
            }
        }
        CacheMetadata {
            cache_names: Vec::new(),
        }
    }

    /// Save cache metadata to disk
    fn save_cache_metadata(&self) {
        let caches = self.caches.read().unwrap();
        let cache_names: Vec<String> = caches.keys().cloned().collect();
        let metadata = CacheMetadata { cache_names };

        if let Ok(content) = serde_json::to_string_pretty(&metadata) {
            let _ = fs::write(&self.metadata_file, content);
        }
    }

    /// Creates a CacheStorage instance
    pub fn create_cache_storage() -> JsObject {
        let cache_storage = CacheStorage::new();
        JsObject::from_proto_and_data(None, cache_storage)
    }
}

impl IntrinsicObject for CacheStorage {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::open, js_string!("open"), 1)
            .method(Self::has, js_string!("has"), 1)
            .method(Self::delete, js_string!("delete"), 1)
            .method(Self::keys, js_string!("keys"), 0)
            .method(Self::match_request, js_string!("match"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for CacheStorage {
    const NAME: JsString = js_string!("CacheStorage");
}

impl BuiltInConstructor for CacheStorage {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &StandardConstructor =
        |constructors| constructors.cache_storage();

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // CacheStorage constructor is not meant to be called directly
        Err(JsNativeError::typ()
            .with_message("CacheStorage constructor cannot be called directly")
            .into())
    }
}

// CacheStorage prototype methods
impl CacheStorage {
    /// `caches.open(cacheName)`
    /// Returns a Promise that resolves to the Cache object matching the cacheName
    fn open(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a CacheStorage object")
            })?;

        let cache_storage = obj.downcast_ref::<CacheStorage>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a CacheStorage object")
            })?;

        let cache_name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

        let (promise, resolvers) = JsPromise::new_pending(context);

        let cache = {
            let mut caches = cache_storage.caches.write().unwrap();
            caches.entry(cache_name.clone()).or_insert_with(|| {
                Cache::create_cache(cache_name.clone())
            }).clone()
        };

        // Set the prototype for the cache
        let cache_prototype = context.intrinsics().constructors().cache().prototype();
        cache.set_prototype(Some(cache_prototype));

        cache_storage.save_cache_metadata();

        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::from(cache)], context)?;

        Ok(JsValue::from(promise))
    }

    /// `caches.has(cacheName)`
    /// Returns a Promise that resolves to true if a Cache object matches the cacheName
    fn has(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a CacheStorage object")
            })?;

        let cache_storage = obj.downcast_ref::<CacheStorage>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a CacheStorage object")
            })?;

        let cache_name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

        let has_cache = {
            let caches = cache_storage.caches.read().unwrap();
            caches.contains_key(&cache_name)
        };

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::from(has_cache)], context)?;

        Ok(JsValue::from(promise))
    }

    /// `caches.delete(cacheName)`
    /// Returns a Promise that resolves to true if the Cache was deleted
    fn delete(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a CacheStorage object")
            })?;

        let cache_storage = obj.downcast_ref::<CacheStorage>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a CacheStorage object")
            })?;

        let cache_name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

        let deleted = {
            let mut caches = cache_storage.caches.write().unwrap();
            caches.remove(&cache_name).is_some()
        };

        if deleted {
            cache_storage.save_cache_metadata();

            // Also try to delete the cache file
            let mut cache_file = std::env::temp_dir();
            cache_file.push("boa_cache_api");
            cache_file.push(format!("{}.json", cache_name));
            if cache_file.exists() {
                let _ = fs::remove_file(cache_file);
            }
        }

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::from(deleted)], context)?;

        Ok(JsValue::from(promise))
    }

    /// `caches.keys()`
    /// Returns a Promise that resolves to an array of cache names
    fn keys(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a CacheStorage object")
            })?;

        let cache_storage = obj.downcast_ref::<CacheStorage>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a CacheStorage object")
            })?;

        let cache_names = {
            let caches = cache_storage.caches.read().unwrap();
            caches.keys().map(|name| JsValue::from(JsString::from(name.clone()))).collect::<Vec<_>>()
        };

        use crate::builtins::array::Array;
        let names_array = Array::create_array_from_list(cache_names, context);

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::from(names_array)], context)?;

        Ok(JsValue::from(promise))
    }

    /// `caches.match(request, options)`
    /// Returns a Promise that resolves to the Response associated with the first matching request
    fn match_request(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a CacheStorage object")
            })?;

        let cache_storage = obj.downcast_ref::<CacheStorage>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a CacheStorage object")
            })?;

        let request = args.get_or_undefined(0);

        let (promise, resolvers) = JsPromise::new_pending(context);

        // Search through all caches for a match
        let caches = cache_storage.caches.read().unwrap();
        for cache in caches.values() {
            // Try to call match on this cache
            if let Ok(match_method) = cache.get(js_string!("match"), context) {
                if let Some(match_func) = match_method.as_callable() {
                    if let Ok(result) = match_func.call(&JsValue::from(cache.clone()), &[request.clone()], context) {
                        if !result.is_undefined() {
                            resolvers.resolve.call(&JsValue::undefined(), &[result], context)?;
                            return Ok(JsValue::from(promise));
                        }
                    }
                }
            }
        }

        // No match found in any cache
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::undefined()], context)?;

        Ok(JsValue::from(promise))
    }
}

#[cfg(test)]
mod tests;