//! WorkerNavigator Web API implementation for Boa
//!
//! Implements the WorkerNavigator interface for use in worker contexts
//! https://html.spec.whatwg.org/multipage/workers.html#workernavigator
//!
//! WorkerNavigator provides a subset of Navigator functionality available in workers

use crate::{
    builtins::BuiltInBuilder,
    object::JsObject,
    property::Attribute,
    string::StaticJsStrings,
    Context, JsData, JsResult, JsValue, js_string,
};
use boa_gc::{Finalize, Trace};

/// JavaScript `WorkerNavigator` implementation for worker contexts.
#[derive(Debug)]
pub struct WorkerNavigator;

/// Internal data for WorkerNavigator instances
#[derive(Debug, Trace, Finalize, JsData)]
struct WorkerNavigatorData {
    #[unsafe_ignore_trace]
    user_agent: String,
    #[unsafe_ignore_trace]
    platform: String,
    #[unsafe_ignore_trace]
    language: String,
    #[unsafe_ignore_trace]
    languages: Vec<String>,
    #[unsafe_ignore_trace]
    online: bool,
    #[unsafe_ignore_trace]
    hardware_concurrency: u32,
}

impl WorkerNavigatorData {
    fn new() -> Self {
        Self {
            user_agent: "Thalora/1.0 (Boa JavaScript Engine)".to_string(),
            platform: Self::detect_platform(),
            language: "en-US".to_string(),
            languages: vec!["en-US".to_string(), "en".to_string()],
            online: true, // In workers, assume online unless explicitly set
            hardware_concurrency: Self::detect_hardware_concurrency(),
        }
    }

    fn detect_platform() -> String {
        #[cfg(target_os = "windows")]
        return "Win32".to_string();
        #[cfg(target_os = "macos")]
        return "MacIntel".to_string();
        #[cfg(target_os = "linux")]
        return "Linux x86_64".to_string();
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return "Unknown".to_string();
    }

    fn detect_hardware_concurrency() -> u32 {
        std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1)
    }
}

impl WorkerNavigator {
    /// Create a WorkerNavigator object for use in worker contexts
    pub fn create(context: &mut Context) -> JsResult<JsObject> {
        let navigator_data = WorkerNavigatorData::new();

        // Create WorkerNavigator object with Object prototype
        let navigator_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            Some(JsObject::with_object_proto(context.intrinsics())),
            navigator_data,
        );

        // Add WorkerNavigator properties manually
        let user_agent_getter = BuiltInBuilder::callable(context.realm(), get_user_agent)
            .name(js_string!("get userAgent"))
            .build();

        let platform_getter = BuiltInBuilder::callable(context.realm(), get_platform)
            .name(js_string!("get platform"))
            .build();

        let language_getter = BuiltInBuilder::callable(context.realm(), get_language)
            .name(js_string!("get language"))
            .build();

        let languages_getter = BuiltInBuilder::callable(context.realm(), get_languages)
            .name(js_string!("get languages"))
            .build();

        let online_getter = BuiltInBuilder::callable(context.realm(), get_online)
            .name(js_string!("get onLine"))
            .build();

        let hardware_concurrency_getter = BuiltInBuilder::callable(context.realm(), get_hardware_concurrency)
            .name(js_string!("get hardwareConcurrency"))
            .build();

        // Add all getters as accessor properties
        navigator_obj.define_property_or_throw(
            js_string!("userAgent"),
            crate::property::PropertyDescriptorBuilder::new()
                .get(user_agent_getter)
                .configurable(true)
                .build(),
            context,
        )?;

        navigator_obj.define_property_or_throw(
            js_string!("platform"),
            crate::property::PropertyDescriptorBuilder::new()
                .get(platform_getter)
                .configurable(true)
                .build(),
            context,
        )?;

        navigator_obj.define_property_or_throw(
            js_string!("language"),
            crate::property::PropertyDescriptorBuilder::new()
                .get(language_getter)
                .configurable(true)
                .build(),
            context,
        )?;

        navigator_obj.define_property_or_throw(
            js_string!("languages"),
            crate::property::PropertyDescriptorBuilder::new()
                .get(languages_getter)
                .configurable(true)
                .build(),
            context,
        )?;

        navigator_obj.define_property_or_throw(
            js_string!("onLine"),
            crate::property::PropertyDescriptorBuilder::new()
                .get(online_getter)
                .configurable(true)
                .build(),
            context,
        )?;

        navigator_obj.define_property_or_throw(
            js_string!("hardwareConcurrency"),
            crate::property::PropertyDescriptorBuilder::new()
                .get(hardware_concurrency_getter)
                .configurable(true)
                .build(),
            context,
        )?;

        Ok(navigator_obj)
    }
}

// Property getters for WorkerNavigator

/// `WorkerNavigator.prototype.userAgent` getter
fn get_user_agent(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        crate::JsNativeError::typ().with_message("WorkerNavigator.userAgent getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<WorkerNavigatorData>() {
        Ok(JsValue::from(js_string!(data.user_agent.clone())))
    } else {
        Ok(JsValue::from(js_string!("Thalora/1.0 (Boa JavaScript Engine)")))
    }
}

/// `WorkerNavigator.prototype.platform` getter
fn get_platform(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        crate::JsNativeError::typ().with_message("WorkerNavigator.platform getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<WorkerNavigatorData>() {
        Ok(JsValue::from(js_string!(data.platform.clone())))
    } else {
        Ok(JsValue::from(js_string!(WorkerNavigatorData::detect_platform())))
    }
}

/// `WorkerNavigator.prototype.language` getter
fn get_language(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        crate::JsNativeError::typ().with_message("WorkerNavigator.language getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<WorkerNavigatorData>() {
        Ok(JsValue::from(js_string!(data.language.clone())))
    } else {
        Ok(JsValue::from(js_string!("en-US")))
    }
}

/// `WorkerNavigator.prototype.languages` getter
fn get_languages(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        crate::JsNativeError::typ().with_message("WorkerNavigator.languages getter called on non-object")
    })?;

    // Get languages data, cloning to avoid lifetime issues
    let languages = if let Some(data) = this_obj.downcast_ref::<WorkerNavigatorData>() {
        data.languages.clone()
    } else {
        vec!["en-US".to_string(), "en".to_string()]
    };

    // Create JavaScript array from languages
    let js_array = crate::builtins::Array::array_create(languages.len() as u64, None, context)?;
    for (i, lang) in languages.iter().enumerate() {
        js_array.set(i, js_string!(lang.clone()), true, context)?;
    }

    Ok(js_array.into())
}

/// `WorkerNavigator.prototype.onLine` getter
fn get_online(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        crate::JsNativeError::typ().with_message("WorkerNavigator.onLine getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<WorkerNavigatorData>() {
        Ok(JsValue::from(data.online))
    } else {
        Ok(JsValue::from(true)) // Default to online
    }
}

/// `WorkerNavigator.prototype.hardwareConcurrency` getter
fn get_hardware_concurrency(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        crate::JsNativeError::typ().with_message("WorkerNavigator.hardwareConcurrency getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<WorkerNavigatorData>() {
        Ok(JsValue::from(data.hardware_concurrency))
    } else {
        Ok(JsValue::from(WorkerNavigatorData::detect_hardware_concurrency()))
    }
}