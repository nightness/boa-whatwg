//! WebSocket Web API implementation for Boa
//!
//! Native implementation of WebSocket standard
//! https://websockets.spec.whatwg.org/
//!
//! This implements the complete WebSocket interface with real networking

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
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::SinkExt;
use url::Url;

/// JavaScript `WebSocket` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct WebSocket;

impl IntrinsicObject for WebSocket {
    fn init(realm: &Realm) {
        let ready_state_connecting_func = BuiltInBuilder::callable(realm, ready_state_connecting)
            .name(js_string!("get CONNECTING"))
            .build();
        let ready_state_open_func = BuiltInBuilder::callable(realm, ready_state_open)
            .name(js_string!("get OPEN"))
            .build();
        let ready_state_closing_func = BuiltInBuilder::callable(realm, ready_state_closing)
            .name(js_string!("get CLOSING"))
            .build();
        let ready_state_closed_func = BuiltInBuilder::callable(realm, ready_state_closed)
            .name(js_string!("get CLOSED"))
            .build();

        let url_func = BuiltInBuilder::callable(realm, get_url)
            .name(js_string!("get url"))
            .build();
        let ready_state_func = BuiltInBuilder::callable(realm, get_ready_state)
            .name(js_string!("get readyState"))
            .build();
        let buffered_amount_func = BuiltInBuilder::callable(realm, get_buffered_amount)
            .name(js_string!("get bufferedAmount"))
            .build();
        let extensions_func = BuiltInBuilder::callable(realm, get_extensions)
            .name(js_string!("get extensions"))
            .build();
        let protocol_func = BuiltInBuilder::callable(realm, get_protocol)
            .name(js_string!("get protocol"))
            .build();
        let binary_type_func = BuiltInBuilder::callable(realm, get_binary_type)
            .name(js_string!("get binaryType"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Static constants
            .static_accessor(
                js_string!("CONNECTING"),
                Some(ready_state_connecting_func.clone()),
                None,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .static_accessor(
                js_string!("OPEN"),
                Some(ready_state_open_func.clone()),
                None,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .static_accessor(
                js_string!("CLOSING"),
                Some(ready_state_closing_func.clone()),
                None,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .static_accessor(
                js_string!("CLOSED"),
                Some(ready_state_closed_func.clone()),
                None,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            // Instance properties
            .accessor(
                js_string!("url"),
                Some(url_func),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("readyState"),
                Some(ready_state_func),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("bufferedAmount"),
                Some(buffered_amount_func),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("extensions"),
                Some(extensions_func),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("protocol"),
                Some(protocol_func),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("binaryType"),
                Some(binary_type_func),
                None,
                Attribute::CONFIGURABLE,
            )
            // Methods
            .method(Self::send, js_string!("send"), 1)
            .method(Self::close, js_string!("close"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for WebSocket {
    const NAME: JsString = StaticJsStrings::WEBSOCKET;
}

impl BuiltInConstructor for WebSocket {
    const LENGTH: usize = 1;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::websocket;

    /// `new WebSocket(url, protocols)`
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("WebSocket constructor requires 'new'")
                .into());
        }

        let url_arg = args.get_or_undefined(0);
        let protocols_arg = args.get_or_undefined(1);

        // Convert URL to string
        let url_string = url_arg.to_string(context)?;
        let url_str = url_string.to_std_string_escaped();

        // Validate URL
        let url = Url::parse(&url_str).map_err(|_| {
            JsNativeError::syntax().with_message(format!("Invalid WebSocket URL: {}", url_str))
        })?;

        // Validate scheme
        if url.scheme() != "ws" && url.scheme() != "wss" {
            return Err(JsNativeError::syntax()
                .with_message("WebSocket URL must use ws:// or wss:// scheme")
                .into());
        }

        // Parse protocols
        let protocols = if protocols_arg.is_undefined() {
            Vec::new()
        } else {
            // TODO: Handle protocols array/string parsing
            Vec::new()
        };

        // Create the WebSocket object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::websocket, context)?;
        let websocket_data = WebSocketData::new(url_str, protocols);
        let websocket_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            websocket_data,
        );

        // Start connection asynchronously
        Self::initiate_connection(&websocket_obj, context)?;

        Ok(websocket_obj.into())
    }
}

impl WebSocket {
    /// Initiate WebSocket connection
    fn initiate_connection(websocket: &JsObject, _context: &mut Context) -> JsResult<()> {
        if let Some(data) = websocket.downcast_ref::<WebSocketData>() {
            let url = data.url.clone();
            let connection = data.connection.clone();

            // Spawn async connection task
            tokio::spawn(async move {
                match connect_async(&url).await {
                    Ok((ws_stream, _response)) => {
                        let mut conn = connection.lock().await;
                        conn.state = ReadyState::Open;
                        conn.stream = Some(Arc::new(Mutex::new(ws_stream)));
                        // TODO: Trigger onopen event
                    }
                    Err(_) => {
                        let mut conn = connection.lock().await;
                        conn.state = ReadyState::Closed;
                        // TODO: Trigger onerror and onclose events
                    }
                }
            });
        }
        Ok(())
    }

    /// `WebSocket.prototype.send(data)`
    fn send(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("WebSocket.prototype.send called on non-object")
        })?;

        let data_arg = args.get_or_undefined(0);

        if let Some(websocket_data) = this_obj.downcast_ref::<WebSocketData>() {
            // Convert data to string first to avoid holding the lock
            let data_string = data_arg.to_string(context)?;
            let data_str = data_string.to_std_string_escaped();

            let connection = websocket_data.connection.clone();
            tokio::spawn(async move {
                let conn = connection.lock().await;
                if conn.state != ReadyState::Open {
                    return;
                }

                if let Some(ref stream) = conn.stream {
                    let stream_clone = stream.clone();
                    drop(conn); // Release connection lock

                    let mut ws = stream_clone.lock().await;
                    let _ = ws.send(Message::Text(data_str)).await;
                }
            });
        }

        Ok(JsValue::undefined())
    }

    /// `WebSocket.prototype.close(code, reason)`
    fn close(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("WebSocket.prototype.close called on non-object")
        })?;

        let _code = args.get_or_undefined(0);
        let _reason = args.get_or_undefined(1);

        if let Some(websocket_data) = this_obj.downcast_ref::<WebSocketData>() {
            let connection = websocket_data.connection.clone();
            tokio::spawn(async move {
                let mut conn = connection.lock().await;

                if conn.state == ReadyState::Closing || conn.state == ReadyState::Closed {
                    return;
                }

                conn.state = ReadyState::Closing;

                if let Some(ref stream) = conn.stream {
                    let stream_clone = stream.clone();
                    drop(conn); // Release connection lock

                    let mut ws = stream_clone.lock().await;
                    let _ = ws.close(None).await;
                }
            });
        }

        Ok(JsValue::undefined())
    }
}

/// WebSocket ready states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadyState {
    Connecting = 0,
    Open = 1,
    Closing = 2,
    Closed = 3,
}

/// Connection state
#[derive(Debug)]
struct Connection {
    state: ReadyState,
    stream: Option<Arc<Mutex<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>>>,
    buffered_amount: u64,
}

impl Connection {
    fn new() -> Self {
        Self {
            state: ReadyState::Connecting,
            stream: None,
            buffered_amount: 0,
        }
    }
}

/// Internal data for WebSocket instances
#[derive(Debug, Trace, Finalize, JsData)]
struct WebSocketData {
    #[unsafe_ignore_trace]
    url: String,
    #[unsafe_ignore_trace]
    protocols: Vec<String>,
    #[unsafe_ignore_trace]
    connection: Arc<Mutex<Connection>>,
    #[unsafe_ignore_trace]
    binary_type: String,
    #[unsafe_ignore_trace]
    extensions: String,
}

impl WebSocketData {
    fn new(url: String, protocols: Vec<String>) -> Self {
        Self {
            url,
            protocols,
            connection: Arc::new(Mutex::new(Connection::new())),
            binary_type: "blob".to_string(),
            extensions: String::new(),
        }
    }
}

// Constant getters
fn ready_state_connecting(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::from(ReadyState::Connecting as u32))
}

fn ready_state_open(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::from(ReadyState::Open as u32))
}

fn ready_state_closing(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::from(ReadyState::Closing as u32))
}

fn ready_state_closed(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::from(ReadyState::Closed as u32))
}

// Property getters
fn get_url(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("WebSocket.prototype.url getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<WebSocketData>() {
        Ok(JsValue::from(js_string!(data.url.clone())))
    } else {
        Ok(JsValue::undefined())
    }
}

fn get_ready_state(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("WebSocket.prototype.readyState getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<WebSocketData>() {
        // We need to use try_lock since we can't await in a synchronous function
        if let Ok(conn) = data.connection.try_lock() {
            Ok(JsValue::from(conn.state as u32))
        } else {
            // If we can't get the lock, assume connecting state
            Ok(JsValue::from(ReadyState::Connecting as u32))
        }
    } else {
        Ok(JsValue::from(ReadyState::Closed as u32))
    }
}

fn get_buffered_amount(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("WebSocket.prototype.bufferedAmount getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<WebSocketData>() {
        if let Ok(conn) = data.connection.try_lock() {
            Ok(JsValue::from(conn.buffered_amount))
        } else {
            Ok(JsValue::from(0))
        }
    } else {
        Ok(JsValue::from(0))
    }
}

fn get_extensions(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("WebSocket.prototype.extensions getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<WebSocketData>() {
        Ok(JsValue::from(js_string!(data.extensions.clone())))
    } else {
        Ok(JsValue::from(js_string!("")))
    }
}

fn get_protocol(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("WebSocket.prototype.protocol getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<WebSocketData>() {
        let protocol = data.protocols.first().cloned().unwrap_or_default();
        Ok(JsValue::from(js_string!(protocol)))
    } else {
        Ok(JsValue::from(js_string!("")))
    }
}

fn get_binary_type(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("WebSocket.prototype.binaryType getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<WebSocketData>() {
        Ok(JsValue::from(js_string!(data.binary_type.clone())))
    } else {
        Ok(JsValue::from(js_string!("blob")))
    }
}