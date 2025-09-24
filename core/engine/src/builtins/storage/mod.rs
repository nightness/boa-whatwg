//! Implementation of the `Storage` Web API.
//!
//! The `Storage` interface provides access to the local storage and session storage for a particular domain.
//! It allows you to add, modify or delete stored data items.
//!
//! More information:
//! - [WHATWG Specification](https://html.spec.whatwg.org/multipage/webstorage.html#the-storage-interface)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/Storage)

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use boa_gc::{Finalize, Trace};
use crate::{
    builtins::BuiltInBuilder,
    context::intrinsics::Intrinsics,
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, JsValue,
};
use crate::builtins::{BuiltInConstructor, BuiltInObject, IntrinsicObject};
use crate::context::intrinsics::StandardConstructor;

/// `Storage` implementation for the Web Storage API.
#[derive(Debug, Clone, Finalize)]
pub struct Storage {
    /// The storage data, shared between instances for the same storage type
    data: Arc<RwLock<HashMap<String, String>>>,
    /// Storage type identifier for debugging
    storage_type: &'static str,
}

// SAFETY: Storage is safe to trace because HashMap<String, String> doesn't contain any GC'd objects
unsafe impl Trace for Storage {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // No GC'd objects in Storage, nothing to trace
    }

    unsafe fn trace_non_roots(&self) {
        // No GC'd objects in Storage, nothing to trace
    }

    fn run_finalizer(&self) {
        // No cleanup needed for Storage
    }
}

impl JsData for Storage {}

impl Storage {
    /// Creates a new `Storage` instance.
    pub(crate) fn new(storage_type: &'static str) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            storage_type,
        }
    }

    /// Creates a `Storage` instance with pre-populated data.
    pub(crate) fn with_data(data: HashMap<String, String>, storage_type: &'static str) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
            storage_type,
        }
    }

    /// Gets the number of items in storage.
    fn length_internal(&self) -> usize {
        self.data.read().unwrap().len()
    }

    /// Gets the key at the specified index.
    fn key_internal(&self, index: usize) -> Option<String> {
        let data = self.data.read().unwrap();
        let keys: Vec<_> = data.keys().collect();
        keys.get(index).map(|s| (*s).clone())
    }

    /// Gets an item from storage by key.
    fn get_item_internal(&self, key: &str) -> Option<String> {
        self.data.read().unwrap().get(key).cloned()
    }

    /// Sets an item in storage.
    fn set_item_internal(&self, key: String, value: String) -> JsResult<()> {
        let mut data = self.data.write().unwrap();
        data.insert(key, value);
        Ok(())
    }

    /// Removes an item from storage by key.
    fn remove_item_internal(&self, key: &str) {
        let mut data = self.data.write().unwrap();
        data.remove(key);
    }

    /// Clears all items from storage.
    fn clear_internal(&self) {
        let mut data = self.data.write().unwrap();
        data.clear();
    }

    /// Creates a localStorage instance
    pub fn create_local_storage() -> JsObject {
        let storage = Storage::new("localStorage");
        JsObject::from_proto_and_data(None, storage)
    }

    /// Creates a sessionStorage instance
    pub fn create_session_storage() -> JsObject {
        let storage = Storage::new("sessionStorage");
        JsObject::from_proto_and_data(None, storage)
    }
}

impl IntrinsicObject for Storage {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("length"),
                Some(BuiltInBuilder::callable(realm, Self::get_length)
                    .name(js_string!("get length"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(Self::key, js_string!("key"), 1)
            .method(Self::get_item, js_string!("getItem"), 1)
            .method(Self::set_item, js_string!("setItem"), 2)
            .method(Self::remove_item, js_string!("removeItem"), 1)
            .method(Self::clear, js_string!("clear"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Storage {
    const NAME: JsString = js_string!("Storage");
}

impl BuiltInConstructor for Storage {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &StandardConstructor =
        |constructors| constructors.storage();

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // Storage constructor is not meant to be called directly
        Err(JsNativeError::typ()
            .with_message("Storage constructor cannot be called directly")
            .into())
    }
}

// Storage prototype methods
impl Storage {
    /// `Storage.prototype.length` getter
    fn get_length(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Storage object")
            })?;

        let storage = obj.downcast_ref::<Storage>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Storage object")
            })?;

        Ok(JsValue::from(storage.length_internal()))
    }

    /// `Storage.prototype.key(index)`
    fn key(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Storage object")
            })?;

        let storage = obj.downcast_ref::<Storage>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Storage object")
            })?;

        let index = args.get_or_undefined(0).to_length(context)?;

        match storage.key_internal(index as usize) {
            Some(key) => Ok(JsValue::from(JsString::from(key))),
            None => Ok(JsValue::null()),
        }
    }

    /// `Storage.prototype.getItem(key)`
    fn get_item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Storage object")
            })?;

        let storage = obj.downcast_ref::<Storage>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Storage object")
            })?;

        let key = args.get_or_undefined(0).to_string(context)?;

        match storage.get_item_internal(&key.to_std_string_escaped()) {
            Some(value) => Ok(JsValue::from(JsString::from(value))),
            None => Ok(JsValue::null()),
        }
    }

    /// `Storage.prototype.setItem(key, value)`
    fn set_item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Storage object")
            })?;

        let storage = obj.downcast_ref::<Storage>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Storage object")
            })?;

        let key = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let value = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();

        storage.set_item_internal(key, value)?;
        Ok(JsValue::undefined())
    }

    /// `Storage.prototype.removeItem(key)`
    fn remove_item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Storage object")
            })?;

        let storage = obj.downcast_ref::<Storage>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Storage object")
            })?;

        let key = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        storage.remove_item_internal(&key);
        Ok(JsValue::undefined())
    }

    /// `Storage.prototype.clear()`
    fn clear(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Storage object")
            })?;

        let storage = obj.downcast_ref::<Storage>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a Storage object")
            })?;

        storage.clear_internal();
        Ok(JsValue::undefined())
    }
}

#[cfg(test)]
mod tests;