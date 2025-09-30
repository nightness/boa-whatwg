//! ServiceWorkerContainer Web API implementation for Boa
//!
//! Implements the ServiceWorkerContainer interface as defined in:
//! https://w3c.github.io/ServiceWorker/#serviceworkercontainer-interface
//!
//! The ServiceWorkerContainer provides the main entry point for service worker
//! registration and management via navigator.serviceWorker


use crate::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInBuilder, event_target::EventTarget},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{JsObject, ObjectInitializer},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsData, JsNativeError, JsResult, js_string, JsArgs,
    JsString, realm::Realm, property::{Attribute, PropertyDescriptor}
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::Mutex as AsyncMutex;
use url::Url;

/// JavaScript `ServiceWorkerContainer` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct ServiceWorkerContainer;

/// Service worker registration information
#[derive(Debug, Clone)]
pub struct ServiceWorkerRegistration {
    /// Scope URL for this registration
    pub scope: String,
    /// Script URL of the service worker
    pub script_url: String,
    /// Current installing worker
    pub installing: Option<JsObject>,
    /// Current waiting worker
    pub waiting: Option<JsObject>,
    /// Current active worker
    pub active: Option<JsObject>,
    /// Registration update via cache setting
    pub update_via_cache: UpdateViaCache,
}

/// Update via cache options for service worker registration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateViaCache {
    Imports,
    All,
    None,
}

impl UpdateViaCache {
    fn as_str(&self) -> &'static str {
        match self {
            UpdateViaCache::Imports => "imports",
            UpdateViaCache::All => "all",
            UpdateViaCache::None => "none",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "imports" => UpdateViaCache::Imports,
            "all" => UpdateViaCache::All,
            "none" => UpdateViaCache::None,
            _ => UpdateViaCache::Imports, // Default
        }
    }
}

/// ServiceWorkerContainer internal data
#[derive(Debug, Trace, Finalize, JsData)]
pub struct ServiceWorkerContainerData {
    /// Map of scope URLs to registrations
    #[unsafe_ignore_trace]
    registrations: Arc<Mutex<HashMap<String, ServiceWorkerRegistration>>>,
    /// Ready promise - resolves when a service worker is active
    #[unsafe_ignore_trace]
    ready_promise: Option<JsObject>,
    /// Controller service worker (if any)
    controller: Option<JsObject>,
}

impl ServiceWorkerContainerData {
    fn new() -> Self {
        Self {
            registrations: Arc::new(Mutex::new(HashMap::new())),
            ready_promise: None,
            controller: None,
        }
    }
}

impl IntrinsicObject for ServiceWorkerContainer {
    fn init(_realm: &Realm) {
        // ServiceWorkerContainer is not initialized as a global constructor
        // It's created on-demand via navigator.serviceWorker
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for ServiceWorkerContainer {
    const NAME: JsString = StaticJsStrings::SERVICE_WORKER_CONTAINER;
}

impl ServiceWorkerContainer {
    /// `ServiceWorkerContainer.prototype.register(scriptURL, options)`
    fn register(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ServiceWorkerContainer.register called on non-object")
        })?;

        let script_url = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // Convert scriptURL to string
        let script_url_str = script_url.to_string(context)?.to_std_string_escaped();
        if script_url_str.is_empty() {
            return Err(JsNativeError::typ()
                .with_message("Service worker script URL cannot be empty")
                .into());
        }

        // Parse options
        let mut scope = None;
        let mut update_via_cache = UpdateViaCache::Imports;

        if !options.is_undefined() {
            if let Some(options_obj) = options.as_object() {
                // Parse scope
                if let Ok(scope_val) = options_obj.get(js_string!("scope"), context) {
                    if !scope_val.is_undefined() {
                        scope = Some(scope_val.to_string(context)?.to_std_string_escaped());
                    }
                }

                // Parse updateViaCache
                if let Ok(update_val) = options_obj.get(js_string!("updateViaCache"), context) {
                    if !update_val.is_undefined() {
                        let update_str = update_val.to_string(context)?.to_std_string_escaped();
                        update_via_cache = UpdateViaCache::from_str(&update_str);
                    }
                }
            }
        }

        // Validate and normalize URLs
        let script_url = Self::resolve_url(&script_url_str, context)?;
        let scope_url = if let Some(scope_str) = scope {
            Self::resolve_url(&scope_str, context)?
        } else {
            // Default scope is the directory of the script URL
            Self::get_default_scope(&script_url)?
        };

        eprintln!("ServiceWorker: Registering script '{}' with scope '{}'", script_url, scope_url);

        // Create registration
        let registration = ServiceWorkerRegistration {
            scope: scope_url.clone(),
            script_url: script_url.clone(),
            installing: None,
            waiting: None,
            active: None,
            update_via_cache,
        };

        // Store registration
        if let Some(data) = this_obj.downcast_ref::<ServiceWorkerContainerData>() {
            if let Ok(mut registrations) = data.registrations.try_lock() {
                registrations.insert(scope_url.clone(), registration);
            }
        }

        // Create ServiceWorkerRegistration object
        let registration_obj = Self::create_registration_object(&scope_url, &script_url, context)?;

        // Return a Promise that resolves to the registration
        let promise_constructor = context.intrinsics().constructors().promise().constructor();
        let promise = crate::builtins::promise::Promise::promise_resolve(
            &promise_constructor,
            registration_obj.into(),
            context,
        )?;

        Ok(promise.into())
    }

    /// `ServiceWorkerContainer.prototype.getRegistration(clientURL)`
    fn get_registration(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ServiceWorkerContainer.getRegistration called on non-object")
        })?;

        let client_url = args.get_or_undefined(0);
        let client_url_str = if client_url.is_undefined() {
            // Use current page URL as default
            "."
        } else {
            &client_url.to_string(context)?.to_std_string_escaped()
        };

        let resolved_url = Self::resolve_url(client_url_str, context)?;

        // Find matching registration
        if let Some(data) = this_obj.downcast_ref::<ServiceWorkerContainerData>() {
            if let Ok(registrations) = data.registrations.try_lock() {
                for (scope, registration) in registrations.iter() {
                    if resolved_url.starts_with(scope) {
                        let registration_obj = Self::create_registration_object(
                            &registration.scope,
                            &registration.script_url,
                            context
                        )?;

                        // Return a Promise that resolves to the registration
                        let promise_constructor = context.intrinsics().constructors().promise().constructor();
                        let promise = crate::builtins::promise::Promise::promise_resolve(
                            &promise_constructor,
                            registration_obj.into(),
                            context,
                        )?;
                        return Ok(promise.into());
                    }
                }
            }
        }

        // No registration found - return promise resolving to undefined
        let promise_constructor = context.intrinsics().constructors().promise().constructor();
        let promise = crate::builtins::promise::Promise::promise_resolve(
            &promise_constructor,
            JsValue::undefined(),
            context,
        )?;
        Ok(promise.into())
    }

    /// `ServiceWorkerContainer.prototype.getRegistrations()`
    fn get_registrations(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ServiceWorkerContainer.getRegistrations called on non-object")
        })?;

        let mut registration_objects: Vec<JsValue> = Vec::new();

        if let Some(data) = this_obj.downcast_ref::<ServiceWorkerContainerData>() {
            if let Ok(registrations) = data.registrations.try_lock() {
                for registration in registrations.values() {
                    let registration_obj = Self::create_registration_object(
                        &registration.scope,
                        &registration.script_url,
                        context
                    )?;
                    registration_objects.push(registration_obj.into());
                }
            }
        }

        // Create array of registrations
        let array = crate::builtins::Array::array_create(registration_objects.len() as u64, None, context)?;
        for (i, reg_obj) in registration_objects.into_iter().enumerate() {
            array.set(i, reg_obj, true, context)?;
        }

        // Return promise resolving to array
        let promise_constructor = context.intrinsics().constructors().promise().constructor();
        let promise = crate::builtins::promise::Promise::promise_resolve(
            &promise_constructor,
            array.into(),
            context,
        )?;
        Ok(promise.into())
    }

    /// `ServiceWorkerContainer.prototype.startMessages()`
    fn start_messages(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // Start receiving messages from service workers
        eprintln!("ServiceWorker: Starting message channel");
        Ok(JsValue::undefined())
    }

    /// Get the ready promise
    fn get_ready(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ServiceWorkerContainer.ready getter called on non-object")
        })?;

        if let Some(data) = this_obj.downcast_ref::<ServiceWorkerContainerData>() {
            if let Some(ref ready_promise) = data.ready_promise {
                return Ok(ready_promise.clone().into());
            }
        }

        // Create ready promise if it doesn't exist - this should resolve when a service worker becomes active
        let promise_constructor = context.intrinsics().constructors().promise().constructor();
        let promise = crate::builtins::promise::Promise::promise_resolve(
            &promise_constructor,
            JsValue::undefined(), // For now, resolve with undefined since no service worker is active
            context,
        )?;

        // Store the promise for future access
        if let Some(mut data) = this_obj.downcast_mut::<ServiceWorkerContainerData>() {
            data.ready_promise = Some(promise.clone());
        }

        Ok(promise.into())
    }

    /// Event handler getters and setters
    fn get_oncontrollerchange(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Invalid 'this' value")
        })?;
        this_obj.get(js_string!("__oncontrollerchange"), context)
    }

    fn set_oncontrollerchange(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Invalid 'this' value")
        })?;
        let handler = args.get_or_undefined(0);
        this_obj.set(js_string!("__oncontrollerchange"), handler.clone(), false, context)?;
        Ok(JsValue::undefined())
    }

    fn get_onmessage(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Invalid 'this' value")
        })?;
        this_obj.get(js_string!("__onmessage"), context)
    }

    fn set_onmessage(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Invalid 'this' value")
        })?;
        let handler = args.get_or_undefined(0);
        this_obj.set(js_string!("__onmessage"), handler.clone(), false, context)?;
        Ok(JsValue::undefined())
    }

    fn get_onmessageerror(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Invalid 'this' value")
        })?;
        this_obj.get(js_string!("__onmessageerror"), context)
    }

    fn set_onmessageerror(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Invalid 'this' value")
        })?;
        let handler = args.get_or_undefined(0);
        this_obj.set(js_string!("__onmessageerror"), handler.clone(), false, context)?;
        Ok(JsValue::undefined())
    }

    /// Helper methods

    /// Resolve a URL relative to the current context
    fn resolve_url(url: &str, _context: &mut Context) -> JsResult<String> {
        // For now, do basic URL validation
        if url.is_empty() {
            return Err(JsNativeError::typ()
                .with_message("URL cannot be empty")
                .into());
        }

        // Simple URL resolution - in a real implementation this would use the base URL
        let resolved = if url.starts_with('/') {
            format!("https://localhost{}", url)
        } else if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else {
            format!("https://localhost/{}", url)
        };

        Ok(resolved)
    }

    /// Get default scope for a script URL (directory of the script)
    fn get_default_scope(script_url: &str) -> JsResult<String> {
        if let Some(last_slash) = script_url.rfind('/') {
            Ok(script_url[..=last_slash].to_string())
        } else {
            Ok("./".to_string())
        }
    }

    /// Create a ServiceWorkerRegistration object
    fn create_registration_object(scope: &str, script_url: &str, context: &mut Context) -> JsResult<JsObject> {
        let registration_obj = JsObject::with_object_proto(context.intrinsics());

        // Set registration properties
        registration_obj.set(js_string!("scope"), js_string!(scope), false, context)?;
        registration_obj.set(js_string!("installing"), JsValue::null(), false, context)?;
        registration_obj.set(js_string!("waiting"), JsValue::null(), false, context)?;
        registration_obj.set(js_string!("active"), JsValue::null(), false, context)?;
        registration_obj.set(js_string!("updateViaCache"), js_string!("imports"), false, context)?;

        // Add stub methods (would need real implementation)
        let update_func = crate::builtins::BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
            eprintln!("ServiceWorkerRegistration.update() called - not implemented");
            Ok(JsValue::undefined())
        })
        .name(js_string!("update"))
        .build();
        registration_obj.set(js_string!("update"), update_func, false, context)?;

        let unregister_func = crate::builtins::BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
            eprintln!("ServiceWorkerRegistration.unregister() called - not implemented");
            Ok(JsValue::undefined())
        })
        .name(js_string!("unregister"))
        .build();
        registration_obj.set(js_string!("unregister"), unregister_func, false, context)?;

        Ok(registration_obj)
    }

    /// Create a ServiceWorkerContainer instance
    pub fn create(context: &mut Context) -> JsResult<JsObject> {
        let data = ServiceWorkerContainerData::new();
        let container = JsObject::from_proto_and_data(
            Some(JsObject::with_object_proto(context.intrinsics())),
            data,
        );

        // Add properties
        container.set(js_string!("controller"), JsValue::null(), false, context)?;

        // Add methods
        let register_func = crate::builtins::BuiltInBuilder::callable(context.realm(), Self::register)
            .name(js_string!("register"))
            .length(1)
            .build();
        container.set(js_string!("register"), register_func, false, context)?;

        let get_registration_func = crate::builtins::BuiltInBuilder::callable(context.realm(), Self::get_registration)
            .name(js_string!("getRegistration"))
            .length(0)
            .build();
        container.set(js_string!("getRegistration"), get_registration_func, false, context)?;

        let get_registrations_func = crate::builtins::BuiltInBuilder::callable(context.realm(), Self::get_registrations)
            .name(js_string!("getRegistrations"))
            .length(0)
            .build();
        container.set(js_string!("getRegistrations"), get_registrations_func, false, context)?;

        let start_messages_func = crate::builtins::BuiltInBuilder::callable(context.realm(), Self::start_messages)
            .name(js_string!("startMessages"))
            .length(0)
            .build();
        container.set(js_string!("startMessages"), start_messages_func, false, context)?;

        // Add ready getter
        let ready_getter = crate::builtins::BuiltInBuilder::callable(context.realm(), Self::get_ready)
            .name(js_string!("get ready"))
            .build();
        container.define_property_or_throw(
            js_string!("ready"),
            crate::property::PropertyDescriptor::builder()
                .get(ready_getter)
                .configurable(true)
                .enumerable(false)
                .build(),
            context,
        )?;

        // Initialize event handler properties
        container.set(js_string!("__oncontrollerchange"), JsValue::null(), false, context)?;
        container.set(js_string!("__onmessage"), JsValue::null(), false, context)?;
        container.set(js_string!("__onmessageerror"), JsValue::null(), false, context)?;

        // Add event handler accessors
        let oncontrollerchange_getter = crate::builtins::BuiltInBuilder::callable(context.realm(), Self::get_oncontrollerchange)
            .name(js_string!("get oncontrollerchange"))
            .build();
        let oncontrollerchange_setter = crate::builtins::BuiltInBuilder::callable(context.realm(), Self::set_oncontrollerchange)
            .name(js_string!("set oncontrollerchange"))
            .build();
        container.define_property_or_throw(
            js_string!("oncontrollerchange"),
            crate::property::PropertyDescriptor::builder()
                .get(oncontrollerchange_getter)
                .set(oncontrollerchange_setter)
                .configurable(true)
                .enumerable(false)
                .build(),
            context,
        )?;

        let onmessage_getter = crate::builtins::BuiltInBuilder::callable(context.realm(), Self::get_onmessage)
            .name(js_string!("get onmessage"))
            .build();
        let onmessage_setter = crate::builtins::BuiltInBuilder::callable(context.realm(), Self::set_onmessage)
            .name(js_string!("set onmessage"))
            .build();
        container.define_property_or_throw(
            js_string!("onmessage"),
            crate::property::PropertyDescriptor::builder()
                .get(onmessage_getter)
                .set(onmessage_setter)
                .configurable(true)
                .enumerable(false)
                .build(),
            context,
        )?;

        let onmessageerror_getter = crate::builtins::BuiltInBuilder::callable(context.realm(), Self::get_onmessageerror)
            .name(js_string!("get onmessageerror"))
            .build();
        let onmessageerror_setter = crate::builtins::BuiltInBuilder::callable(context.realm(), Self::set_onmessageerror)
            .name(js_string!("set onmessageerror"))
            .build();
        container.define_property_or_throw(
            js_string!("onmessageerror"),
            crate::property::PropertyDescriptor::builder()
                .get(onmessageerror_getter)
                .set(onmessageerror_setter)
                .configurable(true)
                .enumerable(false)
                .build(),
            context,
        )?;

        Ok(container)
    }
}

impl ServiceWorkerContainer {
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::service_worker_container;
}