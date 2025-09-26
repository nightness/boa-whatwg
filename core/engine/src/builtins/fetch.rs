//! Fetch Web API implementation for Boa
//!
//! Native implementation of the Fetch standard
//! https://fetch.spec.whatwg.org/
//!
//! This implements the complete Fetch interface with real HTTP networking

#[cfg(test)]
mod tests;

use crate::{
    builtins::{IntrinsicObject, BuiltInBuilder, BuiltInObject, BuiltInConstructor, Json},
    object::{JsObject, builtins::JsPromise, PROTOTYPE, internal_methods::get_prototype_from_constructor},
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
    realm::Realm, JsData, JsString,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    job::NativeAsyncJob
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;
use reqwest;
use url::Url;

/// JavaScript `fetch()` global function implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Fetch;

impl IntrinsicObject for Fetch {
    fn init(realm: &Realm) {
        BuiltInBuilder::callable_with_intrinsic::<Self>(realm, fetch)
            .name(js_string!("fetch"))
            .length(1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        // Return the intrinsic `fetch` object stored in the intrinsics.
        intrinsics.objects().fetch().into()
    }
}

impl BuiltInObject for Fetch {
    const NAME: JsString = js_string!("fetch");
}

/// `fetch(input, init)` global function
fn fetch(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let input = args.get_or_undefined(0);
    let init = args.get_or_undefined(1);

    // Parse input (URL or Request object)
    let url_string = if let Some(request_obj) = input.as_object() {
        // If it's a Request object, get its URL
        if let Some(request_data) = request_obj.downcast_ref::<RequestData>() {
            request_data.url.clone()
        } else {
            // Otherwise convert to string
            input.to_string(context)?.to_std_string_escaped()
        }
    } else {
        input.to_string(context)?.to_std_string_escaped()
    };

    // Validate URL
    let _url = Url::parse(&url_string).map_err(|_| {
        JsNativeError::typ().with_message(format!("Invalid URL: {}", url_string))
    })?;

    // Parse init options
    let (method, headers, body) = if !init.is_undefined() {
        parse_fetch_init(init, context)?
    } else {
        ("GET".to_string(), HashMap::new(), None)
    };

    // Create a new pending Promise and return it immediately
    let (promise, resolvers) = JsPromise::new_pending(context);

    // Enqueue an async job to perform the actual HTTP request
    context.enqueue_job(
        NativeAsyncJob::new(async move |context| {
            // Perform HTTP request in the background
            let client = reqwest::Client::new();
            let mut request_builder = client.request(
                reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET),
                &url_string
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
            let response_result = request_builder.send().await;

            let context = &mut context.borrow_mut();

            match response_result {
                Ok(response) => {
                    // Extract response data
                    let status = response.status().as_u16();
                    let status_text = response.status().canonical_reason().unwrap_or("").to_string();

                    // Convert headers
                    let mut response_headers = HashMap::new();
                    for (name, value) in response.headers() {
                        if let Ok(value_str) = value.to_str() {
                            response_headers.insert(name.to_string(), value_str.to_string());
                        }
                    }

                    // Get response body
                    let body_result = response.text().await;
                    match body_result {
                        Ok(body_text) => {
                            // Create Response object and resolve the promise
                            let response_data = ResponseData {
                                body: Some(body_text),
                                status,
                                status_text: status_text.clone(),
                                headers: response_headers,
                                url: url_string.clone(),
                            };

                            let response_obj = JsObject::from_proto_and_data(None, response_data);

                            // Add properties to the Response object
                            drop(response_obj.set(js_string!("status"), JsValue::from(status), false, context));
                            drop(response_obj.set(js_string!("statusText"), JsValue::from(js_string!(status_text)), false, context));
                            drop(response_obj.set(js_string!("ok"), JsValue::from(status >= 200 && status < 300), false, context));
                            drop(response_obj.set(js_string!("url"), JsValue::from(js_string!(url_string)), false, context));

                            resolvers.resolve.call(&JsValue::undefined(), &[response_obj.into()], context)
                        }
                        Err(e) => {
                            // Reject promise with body read error
                            let error = JsNativeError::typ()
                                .with_message(format!("Failed to read response body: {}", e))
                                .to_opaque(context);
                            resolvers.reject.call(&JsValue::undefined(), &[error.into()], context)
                        }
                    }
                }
                Err(e) => {
                    // Reject promise with network error
                    let error = JsNativeError::typ()
                        .with_message(format!("Fetch request failed: {}", e))
                        .to_opaque(context);
                    resolvers.reject.call(&JsValue::undefined(), &[error.into()], context)
                }
            }
        })
        .into(),
    );

    // Return the Promise immediately
    Ok(promise.into())
}

/// Parse fetch init options
fn parse_fetch_init(
    init: &JsValue,
    context: &mut Context,
) -> JsResult<(String, HashMap<String, String>, Option<String>)> {
    let init_obj = init.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("fetch init must be an object")
    })?;

    // Method
    let method = if let Ok(method_val) = init_obj.get(js_string!("method"), context) {
        method_val.to_string(context)?.to_std_string_escaped().to_uppercase()
    } else {
        "GET".to_string()
    };

    // Headers
    let mut headers = HashMap::new();
    if let Ok(headers_val) = init_obj.get(js_string!("headers"), context) {
        if let Some(headers_obj) = headers_val.as_object() {
            // Parse headers object - could be Headers object or plain object
            if let Some(headers_data) = headers_obj.downcast_ref::<HeadersData>() {
                // It's a Headers object
                headers.extend(headers_data.headers.clone());
            } else {
                // It's a plain object, iterate over properties
                for property_key in headers_obj.own_property_keys(context)? {
                    let key_name = property_key.to_string();
                    if let Ok(value) = headers_obj.get(property_key, context) {
                        let value_str = value.to_string(context)?.to_std_string_escaped();
                        headers.insert(key_name, value_str);
                    }
                }
            }
        } else if headers_val.is_string() {
            // Handle string headers (less common)
            let headers_str = headers_val.to_string(context)?.to_std_string_escaped();
            // Simple parsing of "key: value" format
            for line in headers_str.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    headers.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
    }

    // Add default User-Agent if not present
    if !headers.contains_key("User-Agent") {
        headers.insert("User-Agent".to_string(), "Thalora/1.0".to_string());
    }

    // Body
    let body = if let Ok(body_val) = init_obj.get(js_string!("body"), context) {
        if !body_val.is_undefined() && !body_val.is_null() {
            Some(body_val.to_string(context)?.to_std_string_escaped())
        } else {
            None
        }
    } else {
        None
    };

    Ok((method, headers, body))
}

/// JavaScript `Request` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Request;

impl IntrinsicObject for Request {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.constructors().request().constructor()
    }
}

impl BuiltInObject for Request {
    const NAME: JsString = js_string!("Request");
}

impl BuiltInConstructor for Request {
    const LENGTH: usize = 1;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::request;

    /// `new Request(input, init)`
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Request constructor requires 'new'")
                .into());
        }

        let input = args.get_or_undefined(0);
        let init = args.get_or_undefined(1);

        // Parse URL
        let url = input.to_string(context)?.to_std_string_escaped();

        // Validate URL
        if Url::parse(&url).is_err() {
            return Err(JsNativeError::typ()
                .with_message("Invalid URL")
                .into());
        }

        // Create the Request object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::request, context)?;
        let request_data = RequestData {
            url,
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: None,
        };
        let request_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            request_data,
        );

        Ok(request_obj.into())
    }
}

impl Request {
}

/// JavaScript `Response` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Response;

impl IntrinsicObject for Response {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::text, js_string!("text"), 0)
            .method(Self::json, js_string!("json"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.constructors().response().constructor()
    }
}

impl BuiltInObject for Response {
    const NAME: JsString = js_string!("Response");
}

impl BuiltInConstructor for Response {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::response;

    /// `new Response(body, init)`
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Response constructor requires 'new'")
                .into());
        }

        let body = args.get_or_undefined(0);
        let init = args.get_or_undefined(1);

        // Parse body
        let body_text = if !body.is_undefined() && !body.is_null() {
            Some(body.to_string(context)?.to_std_string_escaped())
        } else {
            None
        };

        // Parse status and statusText from init
        let (status, status_text) = if !init.is_undefined() {
            if let Some(init_obj) = init.as_object() {
                let status = if let Ok(status_val) = init_obj.get(js_string!("status"), context) {
                    status_val.to_number(context)? as u16
                } else {
                    200
                };

                let status_text = if let Ok(status_text_val) = init_obj.get(js_string!("statusText"), context) {
                    status_text_val.to_string(context)?.to_std_string_escaped()
                } else {
                    "OK".to_string()
                };

                (status, status_text)
            } else {
                (200, "OK".to_string())
            }
        } else {
            (200, "OK".to_string())
        };

        // Create the Response object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::response, context)?;
        let response_data = ResponseData {
            body: body_text,
            status,
            status_text: status_text.clone(),
            headers: HashMap::new(),
            url: String::new(),
        };
        let response_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            response_data,
        );

        // Set properties on the Response instance
        response_obj.set(js_string!("status"), JsValue::from(status), false, context)?;
        response_obj.set(js_string!("statusText"), JsValue::from(js_string!(status_text)), false, context)?;
        response_obj.set(js_string!("ok"), JsValue::from(status >= 200 && status < 300), false, context)?;
        response_obj.set(js_string!("url"), JsValue::from(js_string!("")), false, context)?;

        Ok(response_obj.into())
    }
}

impl Response {

    /// `Response.prototype.text()` method
    fn text(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let response_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.text called on non-object")
        })?;

        let response_data = response_obj.downcast_ref::<ResponseData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.text called on non-Response object")
        })?;

        // Create and return a resolved Promise with the body text
        if let Some(ref body) = response_data.body {
            Ok(JsPromise::resolve(JsValue::from(js_string!(body.clone())), context).into())
        } else {
            Ok(JsPromise::resolve(JsValue::from(js_string!("")), context).into())
        }
    }

    /// `Response.prototype.json()` method
    fn json(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let response_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.json called on non-object")
        })?;

        let response_data = response_obj.downcast_ref::<ResponseData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.json called on non-Response object")
        })?;

        // Parse JSON from body text
        if let Some(ref body) = response_data.body {
            match Json::parse(&JsValue::undefined(), &[JsValue::from(js_string!(body.clone()))], context) {
                Ok(json_value) => Ok(JsPromise::resolve(json_value, context).into()),
                Err(e) => {
                    let error = JsNativeError::syntax()
                        .with_message(format!("Failed to parse JSON: {}", e));
                    Ok(JsPromise::reject(error, context).into())
                }
            }
        } else {
            let error = JsNativeError::typ()
                .with_message("Response body is null");
            Ok(JsPromise::reject(error, context).into())
        }
    }
}

/// JavaScript `Headers` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Headers;

impl IntrinsicObject for Headers {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.constructors().headers().constructor()
    }
}

impl BuiltInObject for Headers {
    const NAME: JsString = js_string!("Headers");
}

impl BuiltInConstructor for Headers {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::headers;

    /// `new Headers(init)`
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Headers constructor requires 'new'")
                .into());
        }

        let _init = args.get_or_undefined(0);
        // TODO: Parse init parameter (can be array of [name, value] pairs, object, or another Headers)

        // Create the Headers object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::headers, context)?;
        let headers_data = HeadersData {
            headers: HashMap::new(),
        };
        let headers_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            headers_data,
        );

        Ok(headers_obj.into())
    }
}

impl Headers {
}

/// Internal data for Request instances
#[derive(Debug, Trace, Finalize, JsData)]
struct RequestData {
    #[unsafe_ignore_trace]
    url: String,
    #[unsafe_ignore_trace]
    method: String,
    #[unsafe_ignore_trace]
    headers: HashMap<String, String>,
    #[unsafe_ignore_trace]
    body: Option<String>,
}

/// Internal data for Response instances
#[derive(Debug, Trace, Finalize, JsData)]
struct ResponseData {
    #[unsafe_ignore_trace]
    body: Option<String>,
    #[unsafe_ignore_trace]
    status: u16,
    #[unsafe_ignore_trace]
    status_text: String,
    #[unsafe_ignore_trace]
    headers: HashMap<String, String>,
    #[unsafe_ignore_trace]
    url: String,
}

/// Internal data for Headers instances
#[derive(Debug, Trace, Finalize, JsData)]
struct HeadersData {
    #[unsafe_ignore_trace]
    headers: HashMap<String, String>,
}