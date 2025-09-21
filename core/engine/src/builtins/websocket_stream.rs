//! WebSocketStream Web API implementation for Boa
//!
//! Native implementation of WebSocketStream standard
//! https://websockets.spec.whatwg.org/#websocketstream
//!
//! This implements the WebSocketStream interface for Chrome 124+

use crate::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::Attribute
};
use boa_gc::{Finalize, Trace};
use std::sync::Arc;

/// JavaScript `WebSocketStream` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct WebSocketStream;

impl IntrinsicObject for WebSocketStream {
    fn init(realm: &Realm) {
        let url_func = BuiltInBuilder::callable(realm, get_url)
            .name(js_string!("get url"))
            .build();

        let ready_state_func = BuiltInBuilder::callable(realm, get_ready_state)
            .name(js_string!("get readyState"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("url"),
                Some(url_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("readyState"),
                Some(ready_state_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(close, js_string!("close"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for WebSocketStream {
    const NAME: JsString = StaticJsStrings::WEBSOCKET_STREAM;
}

impl BuiltInConstructor for WebSocketStream {
    const LENGTH: usize = 1;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::websocket_stream;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("WebSocketStream constructor requires 'new'")
                .into());
        }

        let url = args.get_or_undefined(0);

        // Validate URL
        let url_string = url.to_string(context)?;
        if url_string.is_empty() {
            return Err(JsNativeError::typ()
                .with_message("WebSocketStream URL must be a non-empty string")
                .into());
        }

        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::websocket_stream,
            context,
        )?;

        let options = args.get(1).cloned().unwrap_or(JsValue::undefined());
        let websocket_stream_data = WebSocketStreamData::new(url_string.to_std_string_escaped(), options);

        let websocket_stream = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            websocket_stream_data,
        );

        Ok(websocket_stream.into())
    }
}

/// Internal data for WebSocketStream objects
#[derive(Debug, Trace, Finalize, JsData)]
pub struct WebSocketStreamData {
    url: String,
    #[unsafe_ignore_trace]
    ready_state: Arc<std::sync::Mutex<u8>>,
    #[unsafe_ignore_trace]
    options: JsValue,
}

impl WebSocketStreamData {
    fn new(url: String, options: JsValue) -> Self {
        Self {
            url,
            ready_state: Arc::new(std::sync::Mutex::new(0)), // CONNECTING
            options,
        }
    }

    fn get_ready_state(&self) -> u8 {
        *self.ready_state.lock().unwrap()
    }

    fn set_ready_state(&self, state: u8) {
        *self.ready_state.lock().unwrap() = state;
    }

    fn close(&self) {
        self.set_ready_state(3); // CLOSED
    }
}

/// `WebSocketStream.prototype.url` getter
fn get_url(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("WebSocketStream.prototype.url called on non-object")
    })?;

    if let Some(websocket_stream) = this_obj.downcast_ref::<WebSocketStreamData>() {
        Ok(JsString::from(websocket_stream.url.as_str()).into())
    } else {
        Err(JsNativeError::typ()
            .with_message("WebSocketStream.prototype.url called on non-WebSocketStream object")
            .into())
    }
}

/// `WebSocketStream.prototype.readyState` getter
fn get_ready_state(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("WebSocketStream.prototype.readyState called on non-object")
    })?;

    if let Some(websocket_stream) = this_obj.downcast_ref::<WebSocketStreamData>() {
        Ok(JsValue::from(websocket_stream.get_ready_state()))
    } else {
        Err(JsNativeError::typ()
            .with_message("WebSocketStream.prototype.readyState called on non-WebSocketStream object")
            .into())
    }
}

/// `WebSocketStream.prototype.close(code, reason)`
fn close(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("WebSocketStream.prototype.close called on non-object")
    })?;

    if let Some(websocket_stream) = this_obj.downcast_ref::<WebSocketStreamData>() {
        websocket_stream.close();
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("WebSocketStream.prototype.close called on non-WebSocketStream object")
            .into())
    }
}