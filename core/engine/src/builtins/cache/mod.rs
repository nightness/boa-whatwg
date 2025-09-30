//! Implementation of the `Cache` Web API.
//!
//! The `Cache` interface provides a storage mechanism for Request/Response object pairs
//! that are cached in memory or persisted on disk. It's primarily used by Service Workers.
//!
//! More information:
//! - [WHATWG Service Worker Specification](https://w3c.github.io/ServiceWorker/#cache-interface)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/Cache)

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

/// Serializable cache entry for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// The request URL
    url: String,
    /// The request method
    method: String,
    /// Request headers
    request_headers: HashMap<String, String>,
    /// Response status
    status: u16,
    /// Response status text
    status_text: String,
    /// Response headers
    response_headers: HashMap<String, String>,
    /// Response body (as base64 for binary data)
    body: String,
    /// Timestamp when cached
    timestamp: u64,
}

/// `Cache` implementation for the Service Worker Cache API.
#[derive(Debug, Clone, Finalize)]
pub struct Cache {
    /// Cache name
    name: String,
    /// In-memory cache entries
    entries: std::sync::Arc<std::sync::RwLock<HashMap<String, CacheEntry>>>,
    /// Cache file path for persistence
    cache_file: PathBuf,
}

unsafe impl Trace for Cache {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // No GC'd objects in Cache, nothing to trace
    }

    unsafe fn trace_non_roots(&self) {
        // No GC'd objects in Cache, nothing to trace
    }

    fn run_finalizer(&self) {
        // No cleanup needed for Cache
    }
}

impl JsData for Cache {}

impl Cache {
    /// Creates a new `Cache` instance.
    pub(crate) fn new(name: String) -> Self {
        let cache_file = Self::get_cache_file(&name);
        let entries = Self::load_cache_entries(&cache_file);

        Self {
            name,
            entries: std::sync::Arc::new(std::sync::RwLock::new(entries)),
            cache_file,
        }
    }

    /// Get the cache file path for a cache name
    fn get_cache_file(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push("boa_cache_api");
        if !path.exists() {
            fs::create_dir_all(&path).ok();
        }
        path.push(format!("{}.json", name));
        path
    }

    /// Load cache entries from disk
    fn load_cache_entries(cache_file: &PathBuf) -> HashMap<String, CacheEntry> {
        if cache_file.exists() {
            if let Ok(content) = fs::read_to_string(cache_file) {
                if let Ok(entries) = serde_json::from_str::<HashMap<String, CacheEntry>>(&content) {
                    return entries;
                }
            }
        }
        HashMap::new()
    }

    /// Save cache entries to disk
    fn save_cache_entries(&self) {
        let entries = self.entries.read().unwrap();
        if let Ok(content) = serde_json::to_string_pretty(&*entries) {
            let _ = fs::write(&self.cache_file, content);
        }
    }

    /// Generate cache key from request
    fn generate_cache_key(url: &str, method: &str) -> String {
        format!("{}:{}", method.to_uppercase(), url)
    }

    /// Creates a Cache instance
    pub fn create_cache(name: String) -> JsObject {
        let cache = Cache::new(name);
        JsObject::from_proto_and_data(None, cache)
    }
}

impl IntrinsicObject for Cache {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::match_request, js_string!("match"), 1)
            .method(Self::match_all, js_string!("matchAll"), 0)
            .method(Self::add, js_string!("add"), 1)
            .method(Self::add_all, js_string!("addAll"), 1)
            .method(Self::put, js_string!("put"), 2)
            .method(Self::delete, js_string!("delete"), 1)
            .method(Self::keys, js_string!("keys"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Cache {
    const NAME: JsString = js_string!("Cache");
}

impl BuiltInConstructor for Cache {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &StandardConstructor =
        |constructors| constructors.cache();

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // Cache constructor is not meant to be called directly
        Err(JsNativeError::typ()
            .with_message("Cache constructor cannot be called directly")
            .into())
    }
}

// Cache prototype methods
impl Cache {
    /// `cache.match(request, options)`
    /// Returns a Promise that resolves to the Response associated with the first matching request
    fn match_request(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Cache object")
            })?;

        let cache = obj.downcast_ref::<Cache>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Cache object")
            })?;

        let request = args.get_or_undefined(0);
        let url = if let Some(request_obj) = request.as_object() {
            // Try to get URL from Request object
            if let Ok(url_val) = request_obj.get(js_string!("url"), context) {
                url_val.to_string(context)?.to_std_string_escaped()
            } else {
                request.to_string(context)?.to_std_string_escaped()
            }
        } else {
            request.to_string(context)?.to_std_string_escaped()
        };

        let method = "GET"; // Default method for cache lookup
        let cache_key = Self::generate_cache_key(&url, method);

        let (promise, resolvers) = JsPromise::new_pending(context);

        let entries = cache.entries.read().unwrap();
        if let Some(entry) = entries.get(&cache_key) {
            // Create a mock Response object
            let response_obj = JsObject::from_proto_and_data(
                Some(context.intrinsics().constructors().object().prototype()),
                ()
            );

            response_obj.set(js_string!("status"), JsValue::from(entry.status), false, context)?;
            response_obj.set(js_string!("statusText"), JsValue::from(JsString::from(entry.status_text.clone())), false, context)?;
            response_obj.set(js_string!("url"), JsValue::from(JsString::from(entry.url.clone())), false, context)?;

            // Add body as a property
            response_obj.set(js_string!("body"), JsValue::from(JsString::from(entry.body.clone())), false, context)?;

            resolvers.resolve.call(&JsValue::undefined(), &[JsValue::from(response_obj)], context)?;
        } else {
            resolvers.resolve.call(&JsValue::undefined(), &[JsValue::undefined()], context)?;
        }

        Ok(JsValue::from(promise))
    }

    /// `cache.matchAll(request, options)`
    /// Returns a Promise that resolves to an array of all matching responses
    fn match_all(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Cache object")
            })?;

        let _cache = obj.downcast_ref::<Cache>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Cache object")
            })?;

        let (promise, resolvers) = JsPromise::new_pending(context);

        // For now, return empty array - can be enhanced later
        use crate::builtins::array::Array;
        let empty_array = Array::create_array_from_list([], context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::from(empty_array)], context)?;

        Ok(JsValue::from(promise))
    }

    /// `cache.add(request)`
    /// Fetches the request and stores the response in the cache
    fn add(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Cache object")
            })?;

        let _cache = obj.downcast_ref::<Cache>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Cache object")
            })?;

        let (promise, resolvers) = JsPromise::new_pending(context);

        // For now, resolve immediately - real implementation would fetch the URL
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::undefined()], context)?;

        Ok(JsValue::from(promise))
    }

    /// `cache.addAll(requests)`
    /// Fetches multiple requests and stores all responses in the cache
    fn add_all(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Cache object")
            })?;

        let _cache = obj.downcast_ref::<Cache>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Cache object")
            })?;

        let (promise, resolvers) = JsPromise::new_pending(context);

        // For now, resolve immediately - real implementation would fetch all URLs
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::undefined()], context)?;

        Ok(JsValue::from(promise))
    }

    /// `cache.put(request, response)`
    /// Stores a request/response pair in the cache
    fn put(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Cache object")
            })?;

        let cache = obj.downcast_ref::<Cache>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Cache object")
            })?;

        let request = args.get_or_undefined(0);
        let response = args.get_or_undefined(1);

        let url = request.to_string(context)?.to_std_string_escaped();
        let method = "GET"; // Default method

        // Extract response data
        let status = if let Some(resp_obj) = response.as_object() {
            resp_obj.get(js_string!("status"), context)?.to_u32(context).unwrap_or(200)
        } else {
            200
        } as u16;

        let status_text = if let Some(resp_obj) = response.as_object() {
            resp_obj.get(js_string!("statusText"), context)?.to_string(context)?.to_std_string_escaped()
        } else {
            "OK".to_string()
        };

        let body = ""; // For now, empty body - could be enhanced

        let cache_entry = CacheEntry {
            url: url.clone(),
            method: method.to_string(),
            request_headers: HashMap::new(),
            status,
            status_text,
            response_headers: HashMap::new(),
            body: body.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        let cache_key = Self::generate_cache_key(&url, method);

        {
            let mut entries = cache.entries.write().unwrap();
            entries.insert(cache_key, cache_entry);
        }

        cache.save_cache_entries();

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::undefined()], context)?;

        Ok(JsValue::from(promise))
    }

    /// `cache.delete(request, options)`
    /// Removes entries from the cache
    fn delete(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Cache object")
            })?;

        let cache = obj.downcast_ref::<Cache>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Cache object")
            })?;

        let request = args.get_or_undefined(0);
        let url = request.to_string(context)?.to_std_string_escaped();
        let method = "GET"; // Default method
        let cache_key = Self::generate_cache_key(&url, &method);

        let deleted = {
            let mut entries = cache.entries.write().unwrap();
            entries.remove(&cache_key).is_some()
        };

        if deleted {
            cache.save_cache_entries();
        }

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::from(deleted)], context)?;

        Ok(JsValue::from(promise))
    }

    /// `cache.keys(request, options)`
    /// Returns a Promise that resolves to an array of cache keys (Request objects)
    fn keys(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Cache object")
            })?;

        let cache = obj.downcast_ref::<Cache>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Cache object")
            })?;

        let entries = cache.entries.read().unwrap();
        let keys: Vec<JsValue> = entries.values()
            .map(|entry| {
                // Create mock Request object
                let request_obj = JsObject::from_proto_and_data(
                    Some(context.intrinsics().constructors().object().prototype()),
                    ()
                );
                request_obj.set(js_string!("url"), JsValue::from(JsString::from(entry.url.clone())), false, context).ok();
                request_obj.set(js_string!("method"), JsValue::from(JsString::from(entry.method.clone())), false, context).ok();
                JsValue::from(request_obj)
            })
            .collect();

        use crate::builtins::array::Array;
        let keys_array = Array::create_array_from_list(keys, context);

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::from(keys_array)], context)?;

        Ok(JsValue::from(promise))
    }
}

#[cfg(test)]
mod tests;