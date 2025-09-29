//! Implementation of the `CookieStore` Web API.
//!
//! The `CookieStore` interface provides access to HTTP cookies with an asynchronous API.
//! It's accessible via `cookieStore` and provides methods to manage cookies.
//!
//! More information:
//! - [WICG Cookie Store API](https://wicg.github.io/cookie-store/)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/CookieStore)

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

/// Cookie data structure for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    /// The name of the cookie
    pub name: String,
    /// The value of the cookie
    pub value: String,
    /// The domain of the cookie
    pub domain: Option<String>,
    /// The path of the cookie
    pub path: Option<String>,
    /// Whether the cookie is HTTP-only
    pub http_only: bool,
    /// Whether the cookie requires HTTPS
    pub secure: bool,
    /// The SameSite attribute
    pub same_site: Option<String>,
    /// Expiration timestamp (Unix time)
    pub expires: Option<u64>,
}

/// Serializable cookie storage for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CookieStorage {
    /// All cookies indexed by domain and name
    cookies: HashMap<String, HashMap<String, Cookie>>,
}

/// `CookieStore` implementation for the Cookie Store API.
#[derive(Debug, Clone, Finalize)]
pub struct CookieStore {
    /// Cookie storage
    storage: std::sync::Arc<std::sync::RwLock<CookieStorage>>,
    /// Storage file path for persistence
    storage_file: PathBuf,
}

unsafe impl Trace for CookieStore {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // No GC'd objects in CookieStore, nothing to trace
    }

    unsafe fn trace_non_roots(&self) {
        // No GC'd objects in CookieStore, nothing to trace
    }

    fn run_finalizer(&self) {
        // No cleanup needed for CookieStore
    }
}

impl JsData for CookieStore {}

impl CookieStore {
    /// Creates a new `CookieStore` instance.
    pub(crate) fn new() -> Self {
        let storage_file = Self::get_storage_file();
        let storage = Self::load_cookie_storage(&storage_file);

        Self {
            storage: std::sync::Arc::new(std::sync::RwLock::new(storage)),
            storage_file,
        }
    }

    /// Get the storage file path
    fn get_storage_file() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push("boa_cookie_store");
        if !path.exists() {
            fs::create_dir_all(&path).ok();
        }
        path.push("cookies.json");
        path
    }

    /// Load cookie storage from disk
    fn load_cookie_storage(storage_file: &PathBuf) -> CookieStorage {
        if storage_file.exists() {
            if let Ok(content) = fs::read_to_string(storage_file) {
                if let Ok(storage) = serde_json::from_str::<CookieStorage>(&content) {
                    return storage;
                }
            }
        }
        CookieStorage {
            cookies: HashMap::new(),
        }
    }

    /// Save cookie storage to disk
    fn save_cookie_storage(&self) {
        let storage = self.storage.read().unwrap();
        if let Ok(content) = serde_json::to_string_pretty(&*storage) {
            let _ = fs::write(&self.storage_file, content);
        }
    }

    /// Generate cookie key
    fn generate_cookie_key(domain: &str, name: &str) -> String {
        format!("{}:{}", domain, name)
    }

    /// Get current domain (simplified for headless browser)
    fn get_current_domain() -> String {
        "localhost".to_string()
    }

    /// Creates a CookieStore instance
    pub fn create_cookie_store() -> JsObject {
        let cookie_store = CookieStore::new();
        JsObject::from_proto_and_data(None, cookie_store)
    }
}

impl IntrinsicObject for CookieStore {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::get, js_string!("get"), 0)
            .method(Self::get_all, js_string!("getAll"), 0)
            .method(Self::set, js_string!("set"), 1)
            .method(Self::delete, js_string!("delete"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for CookieStore {
    const NAME: JsString = js_string!("CookieStore");
}

impl BuiltInConstructor for CookieStore {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &StandardConstructor =
        |constructors| constructors.cookie_store();

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // CookieStore constructor is not meant to be called directly
        Err(JsNativeError::typ()
            .with_message("CookieStore constructor cannot be called directly")
            .into())
    }
}

// CookieStore prototype methods
impl CookieStore {
    /// `cookieStore.get(name)` or `cookieStore.get(options)`
    /// Returns a Promise that resolves to a cookie object or null
    fn get(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a CookieStore object")
            })?;

        let cookie_store = obj.downcast_ref::<CookieStore>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a CookieStore object")
            })?;

        let name_or_options = args.get_or_undefined(0);
        let cookie_name = if let Some(options_obj) = name_or_options.as_object() {
            // Extract name from options object
            if let Ok(name_val) = options_obj.get(js_string!("name"), context) {
                name_val.to_string(context)?.to_std_string_escaped()
            } else {
                return Err(JsNativeError::typ()
                    .with_message("Cookie name is required")
                    .into());
            }
        } else {
            // Treat as cookie name string
            name_or_options.to_string(context)?.to_std_string_escaped()
        };

        let domain = Self::get_current_domain();

        let (promise, resolvers) = JsPromise::new_pending(context);

        let storage = cookie_store.storage.read().unwrap();
        if let Some(domain_cookies) = storage.cookies.get(&domain) {
            if let Some(cookie) = domain_cookies.get(&cookie_name) {
                // Create cookie object
                let cookie_obj = JsObject::from_proto_and_data(
                    Some(context.intrinsics().constructors().object().prototype()),
                    ()
                );

                cookie_obj.set(js_string!("name"), JsValue::from(JsString::from(cookie.name.clone())), false, context)?;
                cookie_obj.set(js_string!("value"), JsValue::from(JsString::from(cookie.value.clone())), false, context)?;

                if let Some(domain_val) = &cookie.domain {
                    cookie_obj.set(js_string!("domain"), JsValue::from(JsString::from(domain_val.clone())), false, context)?;
                }

                if let Some(path_val) = &cookie.path {
                    cookie_obj.set(js_string!("path"), JsValue::from(JsString::from(path_val.clone())), false, context)?;
                }

                cookie_obj.set(js_string!("secure"), JsValue::from(cookie.secure), false, context)?;
                cookie_obj.set(js_string!("httpOnly"), JsValue::from(cookie.http_only), false, context)?;

                if let Some(same_site) = &cookie.same_site {
                    cookie_obj.set(js_string!("sameSite"), JsValue::from(JsString::from(same_site.clone())), false, context)?;
                }

                resolvers.resolve.call(&JsValue::undefined(), &[JsValue::from(cookie_obj)], context)?;
            } else {
                resolvers.resolve.call(&JsValue::undefined(), &[JsValue::null()], context)?;
            }
        } else {
            resolvers.resolve.call(&JsValue::undefined(), &[JsValue::null()], context)?;
        }

        Ok(JsValue::from(promise))
    }

    /// `cookieStore.getAll(name)` or `cookieStore.getAll(options)`
    /// Returns a Promise that resolves to an array of cookie objects
    fn get_all(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a CookieStore object")
            })?;

        let cookie_store = obj.downcast_ref::<CookieStore>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a CookieStore object")
            })?;

        let domain = Self::get_current_domain();

        let storage = cookie_store.storage.read().unwrap();
        let cookies: Vec<JsValue> = if let Some(domain_cookies) = storage.cookies.get(&domain) {
            domain_cookies.values()
                .map(|cookie| {
                    let cookie_obj = JsObject::from_proto_and_data(
                        Some(context.intrinsics().constructors().object().prototype()),
                        ()
                    );

                    cookie_obj.set(js_string!("name"), JsValue::from(JsString::from(cookie.name.clone())), false, context).ok();
                    cookie_obj.set(js_string!("value"), JsValue::from(JsString::from(cookie.value.clone())), false, context).ok();

                    if let Some(domain_val) = &cookie.domain {
                        cookie_obj.set(js_string!("domain"), JsValue::from(JsString::from(domain_val.clone())), false, context).ok();
                    }

                    if let Some(path_val) = &cookie.path {
                        cookie_obj.set(js_string!("path"), JsValue::from(JsString::from(path_val.clone())), false, context).ok();
                    }

                    cookie_obj.set(js_string!("secure"), JsValue::from(cookie.secure), false, context).ok();
                    cookie_obj.set(js_string!("httpOnly"), JsValue::from(cookie.http_only), false, context).ok();

                    if let Some(same_site) = &cookie.same_site {
                        cookie_obj.set(js_string!("sameSite"), JsValue::from(JsString::from(same_site.clone())), false, context).ok();
                    }

                    JsValue::from(cookie_obj)
                })
                .collect()
        } else {
            Vec::new()
        };

        use crate::builtins::array::Array;
        let cookies_array = Array::create_array_from_list(cookies, context);

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::from(cookies_array)], context)?;

        Ok(JsValue::from(promise))
    }

    /// `cookieStore.set(name, value)` or `cookieStore.set(options)`
    /// Returns a Promise that resolves when the cookie is set
    fn set(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a CookieStore object")
            })?;

        let cookie_store = obj.downcast_ref::<CookieStore>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a CookieStore object")
            })?;

        let first_arg = args.get_or_undefined(0);
        let second_arg = args.get_or_undefined(1);

        let (name, value, domain, path, secure, http_only, same_site) = if let Some(options_obj) = first_arg.as_object() {
            // Options object format
            let name = options_obj.get(js_string!("name"), context)?.to_string(context)?.to_std_string_escaped();
            let value = options_obj.get(js_string!("value"), context)?.to_string(context)?.to_std_string_escaped();
            let domain = if let Ok(domain_val) = options_obj.get(js_string!("domain"), context) {
                if domain_val.is_null() || domain_val.is_undefined() {
                    None
                } else {
                    Some(domain_val.to_string(context)?.to_std_string_escaped())
                }
            } else {
                None
            };
            let path = if let Ok(path_val) = options_obj.get(js_string!("path"), context) {
                if path_val.is_null() || path_val.is_undefined() {
                    None
                } else {
                    Some(path_val.to_string(context)?.to_std_string_escaped())
                }
            } else {
                None
            };
            let secure = options_obj.get(js_string!("secure"), context).unwrap_or(JsValue::from(false)).to_boolean();
            let http_only = options_obj.get(js_string!("httpOnly"), context).unwrap_or(JsValue::from(false)).to_boolean();
            let same_site = if let Ok(same_site_val) = options_obj.get(js_string!("sameSite"), context) {
                if same_site_val.is_null() || same_site_val.is_undefined() {
                    None
                } else {
                    Some(same_site_val.to_string(context)?.to_std_string_escaped())
                }
            } else {
                None
            };

            (name, value, domain, path, secure, http_only, same_site)
        } else {
            // Name-value format
            let name = first_arg.to_string(context)?.to_std_string_escaped();
            let value = second_arg.to_string(context)?.to_std_string_escaped();
            (name, value, None, None, false, false, None)
        };

        let domain = domain.unwrap_or_else(|| Self::get_current_domain());

        let cookie = Cookie {
            name: name.clone(),
            value,
            domain: Some(domain.clone()),
            path,
            http_only,
            secure,
            same_site,
            expires: None, // No expiration by default
        };

        {
            let mut storage = cookie_store.storage.write().unwrap();
            storage.cookies.entry(domain.clone()).or_insert_with(HashMap::new).insert(name, cookie);
        }

        cookie_store.save_cookie_storage();

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::undefined()], context)?;

        Ok(JsValue::from(promise))
    }

    /// `cookieStore.delete(name)` or `cookieStore.delete(options)`
    /// Returns a Promise that resolves when the cookie is deleted
    fn delete(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a CookieStore object")
            })?;

        let cookie_store = obj.downcast_ref::<CookieStore>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a CookieStore object")
            })?;

        let name_or_options = args.get_or_undefined(0);
        let cookie_name = if let Some(options_obj) = name_or_options.as_object() {
            // Extract name from options object
            if let Ok(name_val) = options_obj.get(js_string!("name"), context) {
                name_val.to_string(context)?.to_std_string_escaped()
            } else {
                return Err(JsNativeError::typ()
                    .with_message("Cookie name is required")
                    .into());
            }
        } else {
            // Treat as cookie name string
            name_or_options.to_string(context)?.to_std_string_escaped()
        };

        let domain = Self::get_current_domain();

        let deleted = {
            let mut storage = cookie_store.storage.write().unwrap();
            if let Some(domain_cookies) = storage.cookies.get_mut(&domain) {
                domain_cookies.remove(&cookie_name).is_some()
            } else {
                false
            }
        };

        if deleted {
            cookie_store.save_cookie_storage();
        }

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::undefined()], context)?;

        Ok(JsValue::from(promise))
    }
}

#[cfg(test)]
mod tests;