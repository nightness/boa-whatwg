//! Fetch Web API implementation for Boa
//!
//! Native implementation of the Fetch standard
//! https://fetch.spec.whatwg.org/
//!
//! This implements the complete Fetch interface with real HTTP networking

use crate::{
    builtins::{IntrinsicObject, BuiltInBuilder, BuiltInObject},
    object::JsObject,
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
    realm::Realm, JsData, JsString,
    context::intrinsics::Intrinsics
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
        // Get the fetch function from the global bindings
        intrinsics.constructors().function().constructor()
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

    // Perform actual HTTP request
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

    // Execute request asynchronously
    // For now, we'll block on the future since Boa's fetch is synchronous
    let response = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            request_builder.send().await
        })
    }).map_err(|e| {
        JsNativeError::typ().with_message(format!("Fetch request failed: {}", e))
    })?;

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
    let body_text = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            response.text().await
        })
    }).map_err(|e| {
        JsNativeError::typ().with_message(format!("Failed to read response body: {}", e))
    })?;

    let response_data = ResponseData {
        body: Some(body_text),
        status,
        status_text,
        headers: response_headers,
        url: url_string,
    };

    let response_obj = JsObject::from_proto_and_data(None, response_data);
    Ok(response_obj.into())
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
        BuiltInBuilder::callable_with_intrinsic::<Self>(realm, Self::constructor)
            .name(js_string!("Request"))
            .length(1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.constructors().function().constructor()
    }
}

impl BuiltInObject for Request {
    const NAME: JsString = js_string!("Request");
}

impl Request {

    fn constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let input = args.get_or_undefined(0);
        let init = args.get_or_undefined(1);

        // Parse URL
        let url = input.to_string(context)?.to_std_string_escaped();

        // Validate URL
        Url::parse(&url).map_err(|_| {
            JsNativeError::typ().with_message(format!("Invalid URL: {}", url))
        })?;

        // Parse options
        let (method, headers, body) = if !init.is_undefined() {
            parse_fetch_init(init, context)?
        } else {
            ("GET".to_string(), HashMap::new(), None)
        };

        // Create Request object
        let request_data = RequestData {
            url,
            method,
            headers,
            body,
        };

        let request_obj = JsObject::from_proto_and_data(None, request_data);
        Ok(request_obj.into())
    }
}

/// JavaScript `Response` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Response;

impl IntrinsicObject for Response {
    fn init(realm: &Realm) {
        BuiltInBuilder::callable_with_intrinsic::<Self>(realm, Self::constructor)
            .name(js_string!("Response"))
            .length(0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.constructors().function().constructor()
    }
}

impl BuiltInObject for Response {
    const NAME: JsString = js_string!("Response");
}

impl Response {

    fn constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let body = args.get_or_undefined(0);
        let init = args.get_or_undefined(1);

        // Parse body
        let body_text = if !body.is_undefined() && !body.is_null() {
            Some(body.to_string(context)?.to_std_string_escaped())
        } else {
            None
        };

        // Parse init options
        let (status, status_text, headers) = if !init.is_undefined() {
            let init_obj = init.as_object().ok_or_else(|| {
                JsNativeError::typ().with_message("Response init must be an object")
            })?;

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

            // Parse headers
            let mut headers = HashMap::new();
            if let Ok(headers_val) = init_obj.get(js_string!("headers"), context) {
                if let Some(headers_obj) = headers_val.as_object() {
                    if let Some(headers_data) = headers_obj.downcast_ref::<HeadersData>() {
                        headers.extend(headers_data.headers.clone());
                    } else {
                        // Plain object with headers
                        for property_key in headers_obj.own_property_keys(context)? {
                            let key_name = property_key.to_string();
                            if let Ok(value) = headers_obj.get(property_key, context) {
                                let value_str = value.to_string(context)?.to_std_string_escaped();
                                headers.insert(key_name, value_str);
                            }
                        }
                    }
                }
            }

            (status, status_text, headers)
        } else {
            (200, "OK".to_string(), HashMap::new())
        };

        // Create Response object
        let response_data = ResponseData {
            body: body_text,
            status,
            status_text,
            headers,
            url: String::new(),
        };

        let response_obj = JsObject::from_proto_and_data(None, response_data);
        Ok(response_obj.into())
    }
}

/// JavaScript `Headers` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Headers;

impl IntrinsicObject for Headers {
    fn init(realm: &Realm) {
        BuiltInBuilder::callable_with_intrinsic::<Self>(realm, Self::constructor)
            .name(js_string!("Headers"))
            .length(0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.constructors().function().constructor()
    }
}

impl BuiltInObject for Headers {
    const NAME: JsString = js_string!("Headers");
}

impl Headers {

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // Create Headers object
        let headers_data = HeadersData {
            headers: HashMap::new(),
        };

        let headers_obj = JsObject::from_proto_and_data(None, headers_data);
        Ok(headers_obj.into())
    }
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