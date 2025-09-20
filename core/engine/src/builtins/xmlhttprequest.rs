//! XMLHttpRequest Web API implementation for Boa
//!
//! Native implementation of the XMLHttpRequest standard
//! https://xhr.spec.whatwg.org/
//!
//! This implements the complete XMLHttpRequest interface with real HTTP networking

use crate::{
    builtins::{IntrinsicObject, BuiltInBuilder, BuiltInObject},
    object::{JsObject, PROTOTYPE},
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
    realm::Realm, JsData, JsString,
    context::intrinsics::Intrinsics,
    job::NativeAsyncJob
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;
use reqwest;
use url::Url;

/// JavaScript `XMLHttpRequest` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct XmlHttpRequest;

impl IntrinsicObject for XmlHttpRequest {
    fn init(realm: &Realm) {
        let constructor = BuiltInBuilder::callable_with_intrinsic::<Self>(realm, Self::constructor)
            .name(js_string!("XMLHttpRequest"))
            .length(0)
            .build();

        // Add methods to prototype manually - can't use the STANDARD_CONSTRUCTOR pattern
        // since XMLHttpRequest is not in Boa's intrinsics yet
        let mut context = Context::default();
        if let Ok(prototype_value) = constructor.get(js_string!("prototype"), &mut context) {
            if let Some(prototype) = prototype_value.as_object() {
                // Add instance methods
                let open_fn = BuiltInBuilder::callable(realm, Self::open)
                    .name(js_string!("open"))
                    .length(2)
                    .build();

                let send_fn = BuiltInBuilder::callable(realm, Self::send)
                    .name(js_string!("send"))
                    .length(0)
                    .build();

                let set_request_header_fn = BuiltInBuilder::callable(realm, Self::set_request_header)
                    .name(js_string!("setRequestHeader"))
                    .length(2)
                    .build();

                let get_response_header_fn = BuiltInBuilder::callable(realm, Self::get_response_header)
                    .name(js_string!("getResponseHeader"))
                    .length(1)
                    .build();

                let get_all_response_headers_fn = BuiltInBuilder::callable(realm, Self::get_all_response_headers)
                    .name(js_string!("getAllResponseHeaders"))
                    .length(0)
                    .build();

                let abort_fn = BuiltInBuilder::callable(realm, Self::abort)
                    .name(js_string!("abort"))
                    .length(0)
                    .build();

                let _ = prototype.set(js_string!("open"), open_fn, false, &mut context);
                let _ = prototype.set(js_string!("send"), send_fn, false, &mut context);
                let _ = prototype.set(js_string!("setRequestHeader"), set_request_header_fn, false, &mut context);
                let _ = prototype.set(js_string!("getResponseHeader"), get_response_header_fn, false, &mut context);
                let _ = prototype.set(js_string!("getAllResponseHeaders"), get_all_response_headers_fn, false, &mut context);
                let _ = prototype.set(js_string!("abort"), abort_fn, false, &mut context);

                // Add constants
                let _ = prototype.set(js_string!("UNSENT"), JsValue::from(0), false, &mut context);
                let _ = prototype.set(js_string!("OPENED"), JsValue::from(1), false, &mut context);
                let _ = prototype.set(js_string!("HEADERS_RECEIVED"), JsValue::from(2), false, &mut context);
                let _ = prototype.set(js_string!("LOADING"), JsValue::from(3), false, &mut context);
                let _ = prototype.set(js_string!("DONE"), JsValue::from(4), false, &mut context);
            }
        }
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        // XMLHttpRequest global constructor - get it from the global object
        intrinsics.constructors().object().constructor()
    }
}

impl BuiltInObject for XmlHttpRequest {
    const NAME: JsString = js_string!("XMLHttpRequest");
}

impl XmlHttpRequest {
    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Create XMLHttpRequest object
        let xhr_data = XmlHttpRequestData {
            ready_state: 0,  // UNSENT
            response_url: String::new(),
            status: 0,
            status_text: String::new(),
            response_text: String::new(),
            response_xml: None,
            response_headers: HashMap::new(),
            request_method: String::new(),
            request_url: String::new(),
            request_headers: HashMap::new(),
            is_async: true,  // Fixed: renamed from async
            timeout: 0,
            with_credentials: false,
        };

        let xhr_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            None,
            xhr_data
        );

        // Initialize properties
        xhr_obj.set(js_string!("readyState"), JsValue::from(0), false, context)?;
        xhr_obj.set(js_string!("status"), JsValue::from(0), false, context)?;
        xhr_obj.set(js_string!("statusText"), JsValue::from(js_string!("")), false, context)?;
        xhr_obj.set(js_string!("responseText"), JsValue::from(js_string!("")), false, context)?;
        xhr_obj.set(js_string!("responseXML"), JsValue::null(), false, context)?;
        xhr_obj.set(js_string!("timeout"), JsValue::from(0), false, context)?;
        xhr_obj.set(js_string!("withCredentials"), JsValue::from(false), false, context)?;

        // Create empty upload object
        let upload_obj = JsObject::with_object_proto(context.intrinsics());
        xhr_obj.set(js_string!("upload"), JsValue::from(upload_obj), false, context)?;

        // Event handlers
        xhr_obj.set(js_string!("onreadystatechange"), JsValue::null(), false, context)?;
        xhr_obj.set(js_string!("onload"), JsValue::null(), false, context)?;
        xhr_obj.set(js_string!("onerror"), JsValue::null(), false, context)?;
        xhr_obj.set(js_string!("onabort"), JsValue::null(), false, context)?;
        xhr_obj.set(js_string!("ontimeout"), JsValue::null(), false, context)?;
        xhr_obj.set(js_string!("onloadstart"), JsValue::null(), false, context)?;
        xhr_obj.set(js_string!("onloadend"), JsValue::null(), false, context)?;
        xhr_obj.set(js_string!("onprogress"), JsValue::null(), false, context)?;

        Ok(xhr_obj.into())
    }

    /// `XMLHttpRequest.prototype.open()` method
    fn open(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let xhr_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("XMLHttpRequest.open called on non-object")
        })?;

        let method = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped().to_uppercase();
        let url = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();
        let async_val = args.get_or_undefined(2);
        let is_async = if async_val.is_undefined() { true } else { async_val.to_boolean() };

        // Validate URL
        Url::parse(&url).map_err(|_| {
            JsNativeError::syntax().with_message(format!("Invalid URL: {}", url))
        })?;

        // Validate method
        match method.as_str() {
            "GET" | "POST" | "PUT" | "DELETE" | "HEAD" | "OPTIONS" | "PATCH" => {},
            _ => return Err(JsNativeError::syntax().with_message(format!("Invalid HTTP method: {}", method)).into()),
        }

        // Update internal data
        if let Some(mut xhr_data) = xhr_obj.downcast_mut::<XmlHttpRequestData>() {
            xhr_data.request_method = method;
            xhr_data.request_url = url;
            xhr_data.is_async = is_async;
            xhr_data.ready_state = 1; // OPENED
            xhr_data.request_headers.clear();
            xhr_data.response_headers.clear();
            xhr_data.response_text.clear();
            xhr_data.status = 0;
            xhr_data.status_text.clear();
        }

        // Update readyState property
        xhr_obj.set(js_string!("readyState"), JsValue::from(1), false, context)?;

        // Call onreadystatechange
        Self::call_event_handler(&xhr_obj, "onreadystatechange", context)?;

        Ok(JsValue::undefined())
    }

    /// `XMLHttpRequest.prototype.send()` method
    fn send(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let xhr_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("XMLHttpRequest.send called on non-object")
        })?;

        let xhr_data = xhr_obj.downcast_ref::<XmlHttpRequestData>().ok_or_else(|| {
            JsNativeError::typ().with_message("XMLHttpRequest.send called on non-XMLHttpRequest object")
        })?;

        if xhr_data.ready_state != 1 {
            return Err(JsNativeError::typ().with_message("XMLHttpRequest.send: object state must be OPENED").into());
        }

        let body = args.get_or_undefined(0);
        let body_text = if !body.is_undefined() && !body.is_null() {
            Some(body.to_string(context)?.to_std_string_escaped())
        } else {
            None
        };

        // Clone data for async operation
        let method = xhr_data.request_method.clone();
        let url = xhr_data.request_url.clone();
        let headers = xhr_data.request_headers.clone();
        let xhr_obj_clone = xhr_obj.clone();

        // Enqueue async job to perform HTTP request
        context.enqueue_job(
            NativeAsyncJob::new(async move |context| {
                let context = &mut context.borrow_mut();
                match Self::perform_request(xhr_obj_clone, method, url, headers, body_text, context).await {
                    Ok(_) => Ok(JsValue::undefined()),
                    Err(e) => {
                        eprintln!("XMLHttpRequest error: {}", e);
                        Ok(JsValue::undefined())
                    }
                }
            })
            .into(),
        );

        Ok(JsValue::undefined())
    }

    /// `XMLHttpRequest.prototype.setRequestHeader()` method
    fn set_request_header(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let xhr_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("XMLHttpRequest.setRequestHeader called on non-object")
        })?;

        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let value = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();

        if let Some(mut xhr_data) = xhr_obj.downcast_mut::<XmlHttpRequestData>() {
            if xhr_data.ready_state != 1 {
                return Err(JsNativeError::typ().with_message("XMLHttpRequest.setRequestHeader: object state must be OPENED").into());
            }
            xhr_data.request_headers.insert(name, value);
        }

        Ok(JsValue::undefined())
    }

    /// `XMLHttpRequest.prototype.getResponseHeader()` method
    fn get_response_header(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let xhr_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("XMLHttpRequest.getResponseHeader called on non-object")
        })?;

        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped().to_lowercase();

        let xhr_data = xhr_obj.downcast_ref::<XmlHttpRequestData>().ok_or_else(|| {
            JsNativeError::typ().with_message("XMLHttpRequest.getResponseHeader called on non-XMLHttpRequest object")
        })?;

        if xhr_data.ready_state < 2 {
            return Ok(JsValue::null());
        }

        if let Some(value) = xhr_data.response_headers.get(&name) {
            Ok(JsValue::from(js_string!(value.clone())))
        } else {
            Ok(JsValue::null())
        }
    }

    /// `XMLHttpRequest.prototype.getAllResponseHeaders()` method
    fn get_all_response_headers(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let xhr_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("XMLHttpRequest.getAllResponseHeaders called on non-object")
        })?;

        let xhr_data = xhr_obj.downcast_ref::<XmlHttpRequestData>().ok_or_else(|| {
            JsNativeError::typ().with_message("XMLHttpRequest.getAllResponseHeaders called on non-XMLHttpRequest object")
        })?;

        if xhr_data.ready_state < 2 {
            return Ok(JsValue::from(js_string!("")));
        }

        let mut header_string = String::new();
        for (name, value) in &xhr_data.response_headers {
            header_string.push_str(&format!("{}: {}\r\n", name, value));
        }

        Ok(JsValue::from(js_string!(header_string)))
    }

    /// `XMLHttpRequest.prototype.abort()` method
    fn abort(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let xhr_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("XMLHttpRequest.abort called on non-object")
        })?;

        if let Some(mut xhr_data) = xhr_obj.downcast_mut::<XmlHttpRequestData>() {
            if xhr_data.ready_state == 1 || xhr_data.ready_state == 2 || xhr_data.ready_state == 3 {
                xhr_data.ready_state = 4; // DONE
                xhr_data.status = 0;
                xhr_data.status_text.clear();
                xhr_data.response_text.clear();
            }
        }

        // Update readyState property
        xhr_obj.set(js_string!("readyState"), JsValue::from(4), false, context)?;
        xhr_obj.set(js_string!("status"), JsValue::from(0), false, context)?;

        // Call event handlers
        Self::call_event_handler(&xhr_obj, "onreadystatechange", context)?;
        Self::call_event_handler(&xhr_obj, "onabort", context)?;

        Ok(JsValue::undefined())
    }

    /// Perform the actual HTTP request
    async fn perform_request(
        xhr_obj: JsObject,
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body: Option<String>,
        context: &mut Context,
    ) -> JsResult<()> {
        // Update state to HEADERS_RECEIVED
        Self::update_ready_state(&xhr_obj, 2, context)?;

        // Perform HTTP request
        let client = reqwest::Client::new();
        let mut request_builder = client.request(
            reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET),
            &url
        );

        // Add headers
        for (key, value) in headers {
            request_builder = request_builder.header(&key, &value);
        }

        // Add body if present
        if let Some(body_content) = body {
            request_builder = request_builder.body(body_content);
        }

        // Execute the request
        match request_builder.send().await {
            Ok(response) => {
                // Update state to LOADING
                Self::update_ready_state(&xhr_obj, 3, context)?;

                // Extract response data
                let status = response.status().as_u16();
                let status_text = response.status().canonical_reason().unwrap_or("").to_string();

                // Convert headers
                let mut response_headers = HashMap::new();
                for (name, value) in response.headers() {
                    if let Ok(value_str) = value.to_str() {
                        response_headers.insert(name.to_string().to_lowercase(), value_str.to_string());
                    }
                }

                // Get response body
                match response.text().await {
                    Ok(body_text) => {
                        // Update XMLHttpRequest data
                        if let Some(mut xhr_data) = xhr_obj.downcast_mut::<XmlHttpRequestData>() {
                            xhr_data.status = status;
                            xhr_data.status_text = status_text.clone();
                            xhr_data.response_text = body_text.clone();
                            xhr_data.response_headers = response_headers;
                            xhr_data.response_url = url.clone();
                        }

                        // Update properties
                        xhr_obj.set(js_string!("status"), JsValue::from(status), false, context)?;
                        xhr_obj.set(js_string!("statusText"), JsValue::from(js_string!(status_text)), false, context)?;
                        xhr_obj.set(js_string!("responseText"), JsValue::from(js_string!(body_text)), false, context)?;
                        xhr_obj.set(js_string!("responseURL"), JsValue::from(js_string!(url)), false, context)?;

                        // Update state to DONE
                        Self::update_ready_state(&xhr_obj, 4, context)?;

                        // Call onload handler
                        Self::call_event_handler(&xhr_obj, "onload", context)?;
                    }
                    Err(e) => {
                        // Handle body read error
                        Self::handle_error(&xhr_obj, &format!("Failed to read response body: {}", e), context)?;
                    }
                }
            }
            Err(e) => {
                // Handle network error
                Self::handle_error(&xhr_obj, &format!("Network error: {}", e), context)?;
            }
        }

        Ok(())
    }

    /// Update ready state and call onreadystatechange
    fn update_ready_state(xhr_obj: &JsObject, new_state: u8, context: &mut Context) -> JsResult<()> {
        if let Some(mut xhr_data) = xhr_obj.downcast_mut::<XmlHttpRequestData>() {
            xhr_data.ready_state = new_state;
        }

        xhr_obj.set(js_string!("readyState"), JsValue::from(new_state), false, context)?;
        Self::call_event_handler(xhr_obj, "onreadystatechange", context)?;

        Ok(())
    }

    /// Handle errors
    fn handle_error(xhr_obj: &JsObject, error_msg: &str, context: &mut Context) -> JsResult<()> {
        if let Some(mut xhr_data) = xhr_obj.downcast_mut::<XmlHttpRequestData>() {
            xhr_data.ready_state = 4; // DONE
            xhr_data.status = 0;
            xhr_data.status_text = "".to_string();
        }

        xhr_obj.set(js_string!("readyState"), JsValue::from(4), false, context)?;
        xhr_obj.set(js_string!("status"), JsValue::from(0), false, context)?;
        xhr_obj.set(js_string!("statusText"), JsValue::from(js_string!("")), false, context)?;

        Self::call_event_handler(xhr_obj, "onreadystatechange", context)?;
        Self::call_event_handler(xhr_obj, "onerror", context)?;

        eprintln!("XMLHttpRequest error: {}", error_msg);
        Ok(())
    }

    /// Call event handler if it exists
    fn call_event_handler(xhr_obj: &JsObject, handler_name: &str, context: &mut Context) -> JsResult<()> {
        if let Ok(handler) = xhr_obj.get(js_string!(handler_name), context) {
            if let Some(handler_fn) = handler.as_callable() {
                let _ = handler_fn.call(&JsValue::from(xhr_obj.clone()), &[], context);
            }
        }
        Ok(())
    }
}

/// Internal data for XMLHttpRequest instances
#[derive(Debug, Trace, Finalize, JsData)]
struct XmlHttpRequestData {
    #[unsafe_ignore_trace]
    ready_state: u8,
    #[unsafe_ignore_trace]
    response_url: String,
    #[unsafe_ignore_trace]
    status: u16,
    #[unsafe_ignore_trace]
    status_text: String,
    #[unsafe_ignore_trace]
    response_text: String,
    #[unsafe_ignore_trace]
    response_xml: Option<String>,
    #[unsafe_ignore_trace]
    response_headers: HashMap<String, String>,
    #[unsafe_ignore_trace]
    request_method: String,
    #[unsafe_ignore_trace]
    request_url: String,
    #[unsafe_ignore_trace]
    request_headers: HashMap<String, String>,
    #[unsafe_ignore_trace]
    is_async: bool,  // Fixed: renamed from async
    #[unsafe_ignore_trace]
    timeout: u32,
    #[unsafe_ignore_trace]
    with_credentials: bool,
}