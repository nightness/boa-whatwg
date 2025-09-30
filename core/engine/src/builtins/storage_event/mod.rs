//! Implementation of the `StorageEvent` Web API.
//!
//! The `StorageEvent` is fired at a Window when a storage area changes.
//! This happens when storage is changed from a different window/context.
//!
//! More information:
//! - [WHATWG Specification](https://html.spec.whatwg.org/multipage/webstorage.html#the-storageevent-interface)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/StorageEvent)

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

/// `StorageEvent` implementation for the Web Storage API events.
#[derive(Debug, Clone, Finalize)]
pub struct StorageEvent {
    /// The key being changed. null if clear() was called.
    key: Option<String>,
    /// The old value. null if the key is new.
    old_value: Option<String>,
    /// The new value. null if the key was deleted.
    new_value: Option<String>,
    /// The URL of the document whose key changed.
    url: String,
    /// The Storage object that was affected.
    storage_area: Option<JsObject>,
}

unsafe impl Trace for StorageEvent {
    unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) {
        if let Some(ref storage_area) = self.storage_area {
            unsafe { storage_area.trace(tracer); }
        }
    }

    unsafe fn trace_non_roots(&self) {
        if let Some(ref storage_area) = self.storage_area {
            unsafe { storage_area.trace_non_roots(); }
        }
    }

    fn run_finalizer(&self) {
        // No cleanup needed for StorageEvent
    }
}

impl JsData for StorageEvent {}

impl StorageEvent {
    /// Creates a new `StorageEvent` instance.
    pub(crate) fn new(
        key: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
        url: String,
        storage_area: Option<JsObject>,
    ) -> Self {
        Self {
            key,
            old_value,
            new_value,
            url,
            storage_area,
        }
    }

    /// Creates a StorageEvent object from parameters
    pub fn create_storage_event(
        key: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
        url: String,
        storage_area: Option<JsObject>,
        context: &mut Context,
    ) -> JsObject {
        let event = StorageEvent::new(key.clone(), old_value.clone(), new_value.clone(), url.clone(), storage_area.clone());

        let event_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().storage_event().prototype()),
            event
        );

        // Set event properties
        event_obj.set(js_string!("type"), JsValue::from(JsString::from("storage")), false, context).ok();
        event_obj.set(js_string!("bubbles"), JsValue::from(false), false, context).ok();
        event_obj.set(js_string!("cancelable"), JsValue::from(false), false, context).ok();

        // Set StorageEvent-specific properties
        event_obj.set(
            js_string!("key"),
            key.map(|k| JsValue::from(JsString::from(k))).unwrap_or(JsValue::null()),
            false,
            context
        ).ok();

        event_obj.set(
            js_string!("oldValue"),
            old_value.map(|v| JsValue::from(JsString::from(v))).unwrap_or(JsValue::null()),
            false,
            context
        ).ok();

        event_obj.set(
            js_string!("newValue"),
            new_value.map(|v| JsValue::from(JsString::from(v))).unwrap_or(JsValue::null()),
            false,
            context
        ).ok();

        event_obj.set(js_string!("url"), JsValue::from(JsString::from(url)), false, context).ok();

        event_obj.set(
            js_string!("storageArea"),
            storage_area.map(|s| JsValue::from(s)).unwrap_or(JsValue::null()),
            false,
            context
        ).ok();

        event_obj
    }
}

impl IntrinsicObject for StorageEvent {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("key"),
                Some(BuiltInBuilder::callable(realm, Self::get_key)
                    .name(js_string!("get key"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("oldValue"),
                Some(BuiltInBuilder::callable(realm, Self::get_old_value)
                    .name(js_string!("get oldValue"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("newValue"),
                Some(BuiltInBuilder::callable(realm, Self::get_new_value)
                    .name(js_string!("get newValue"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("url"),
                Some(BuiltInBuilder::callable(realm, Self::get_url)
                    .name(js_string!("get url"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("storageArea"),
                Some(BuiltInBuilder::callable(realm, Self::get_storage_area)
                    .name(js_string!("get storageArea"))
                    .build()),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(Self::init_storage_event, js_string!("initStorageEvent"), 5)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for StorageEvent {
    const NAME: JsString = js_string!("StorageEvent");
}

impl BuiltInConstructor for StorageEvent {
    const LENGTH: usize = 1;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &StandardConstructor =
        |constructors| constructors.storage_event();

    fn constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let event_type = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let event_init = args.get_or_undefined(1);

        let mut key: Option<String> = None;
        let mut old_value: Option<String> = None;
        let mut new_value: Option<String> = None;
        let mut url = String::from("about:blank");
        let mut storage_area: Option<JsObject> = None;

        if let Some(init_obj) = event_init.as_object() {
            if let Ok(key_val) = init_obj.get(js_string!("key"), context) {
                if !key_val.is_null() && !key_val.is_undefined() {
                    key = Some(key_val.to_string(context)?.to_std_string_escaped());
                }
            }

            if let Ok(old_val) = init_obj.get(js_string!("oldValue"), context) {
                if !old_val.is_null() && !old_val.is_undefined() {
                    old_value = Some(old_val.to_string(context)?.to_std_string_escaped());
                }
            }

            if let Ok(new_val) = init_obj.get(js_string!("newValue"), context) {
                if !new_val.is_null() && !new_val.is_undefined() {
                    new_value = Some(new_val.to_string(context)?.to_std_string_escaped());
                }
            }

            if let Ok(url_val) = init_obj.get(js_string!("url"), context) {
                if !url_val.is_null() && !url_val.is_undefined() {
                    url = url_val.to_string(context)?.to_std_string_escaped();
                }
            }

            if let Ok(storage_val) = init_obj.get(js_string!("storageArea"), context) {
                if let Some(storage_obj) = storage_val.as_object() {
                    storage_area = Some(storage_obj.clone());
                }
            }
        }

        let storage_event = StorageEvent::new(key, old_value, new_value, url, storage_area);
        let event_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().storage_event().prototype()),
            storage_event
        );

        // Set basic event properties
        event_obj.set(js_string!("type"), JsValue::from(JsString::from(event_type)), false, context)?;
        event_obj.set(js_string!("bubbles"), JsValue::from(false), false, context)?;
        event_obj.set(js_string!("cancelable"), JsValue::from(false), false, context)?;

        Ok(event_obj.into())
    }
}

// StorageEvent prototype methods
impl StorageEvent {
    /// `StorageEvent.prototype.key` getter
    fn get_key(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        let storage_event = obj.downcast_ref::<StorageEvent>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        Ok(storage_event.key.as_ref()
            .map(|k| JsValue::from(JsString::from(k.clone())))
            .unwrap_or(JsValue::null()))
    }

    /// `StorageEvent.prototype.oldValue` getter
    fn get_old_value(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        let storage_event = obj.downcast_ref::<StorageEvent>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        Ok(storage_event.old_value.as_ref()
            .map(|v| JsValue::from(JsString::from(v.clone())))
            .unwrap_or(JsValue::null()))
    }

    /// `StorageEvent.prototype.newValue` getter
    fn get_new_value(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        let storage_event = obj.downcast_ref::<StorageEvent>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        Ok(storage_event.new_value.as_ref()
            .map(|v| JsValue::from(JsString::from(v.clone())))
            .unwrap_or(JsValue::null()))
    }

    /// `StorageEvent.prototype.url` getter
    fn get_url(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        let storage_event = obj.downcast_ref::<StorageEvent>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        Ok(JsValue::from(JsString::from(storage_event.url.clone())))
    }

    /// `StorageEvent.prototype.storageArea` getter
    fn get_storage_area(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        let storage_event = obj.downcast_ref::<StorageEvent>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        Ok(storage_event.storage_area.as_ref()
            .map(|s| JsValue::from(s.clone()))
            .unwrap_or(JsValue::null()))
    }

    /// `StorageEvent.prototype.initStorageEvent(type, bubbles, cancelable, key, oldValue, newValue, url, storageArea)`
    fn init_storage_event(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        let _storage_event = obj.downcast_ref::<StorageEvent>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a StorageEvent object")
            })?;

        let event_type = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let _bubbles = args.get_or_undefined(1).to_boolean();
        let _cancelable = args.get_or_undefined(2).to_boolean();
        let key = args.get_or_undefined(3);
        let old_value = args.get_or_undefined(4);
        let new_value = args.get_or_undefined(5);
        let url = args.get_or_undefined(6).to_string(context)?.to_std_string_escaped();
        let storage_area = args.get_or_undefined(7);

        // Set properties on the event object
        obj.set(js_string!("type"), JsValue::from(JsString::from(event_type)), false, context)?;

        obj.set(
            js_string!("key"),
            if key.is_null() { JsValue::null() } else { key.clone() },
            false,
            context
        )?;

        obj.set(
            js_string!("oldValue"),
            if old_value.is_null() { JsValue::null() } else { old_value.clone() },
            false,
            context
        )?;

        obj.set(
            js_string!("newValue"),
            if new_value.is_null() { JsValue::null() } else { new_value.clone() },
            false,
            context
        )?;

        obj.set(js_string!("url"), JsValue::from(JsString::from(url)), false, context)?;

        obj.set(
            js_string!("storageArea"),
            if storage_area.is_null() { JsValue::null() } else { storage_area.clone() },
            false,
            context
        )?;

        Ok(JsValue::undefined())
    }
}

#[cfg(test)]
mod tests;