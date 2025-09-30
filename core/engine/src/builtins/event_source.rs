//! WHATWG EventSource API implementation for Server-Sent Events.
//!
//! Implementation of the EventSource interface according to:
//! https://html.spec.whatwg.org/multipage/server-sent-events.html
//!
//! The EventSource interface represents a connection to a server-sent event source.
//! It allows for real-time communication via HTTP streaming.

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    js_string,
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    Context, JsArgs, JsData, JsResult, JsString, JsValue,
    native_function::NativeFunction,
};
use boa_gc::{Finalize, Trace};
use futures_util::{Stream, StreamExt};
use reqwest::Client;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tokio::{sync::mpsc, time::sleep};
use url::Url;
use bytes::Bytes;

/// EventSource connection states according to WHATWG specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ReadyState {
    /// Connection is being established
    Connecting = 0,
    /// Connection is open and receiving events
    Open = 1,
    /// Connection is closed
    Closed = 2,
}

impl From<ReadyState> for JsValue {
    fn from(state: ReadyState) -> Self {
        JsValue::from(state as u32)
    }
}

/// EventSource event types according to WHATWG specification
#[derive(Debug, Clone)]
pub struct ServerSentEvent {
    pub event_type: String,
    pub data: String,
    pub id: Option<String>,
    pub retry: Option<u32>,
}

/// Internal EventSource state management
#[derive(Debug, Clone, Trace, Finalize)]
pub struct EventSourceData {
    /// The URL of the event stream
    url: JsString,
    /// Whether to send credentials with CORS requests
    with_credentials: bool,
    /// Current connection state
    #[unsafe_ignore_trace]
    ready_state: Arc<AtomicU32>,
    /// Last event ID for reconnection
    #[unsafe_ignore_trace]
    last_event_id: Arc<Mutex<Option<String>>>,
    /// Reconnection timeout in milliseconds
    #[unsafe_ignore_trace]
    reconnect_time: Arc<AtomicU32>,
    /// Channel sender for controlling the connection
    #[unsafe_ignore_trace]
    control_sender: Arc<Mutex<Option<mpsc::UnboundedSender<EventSourceControl>>>>,
}

/// Control messages for the EventSource connection
#[derive(Debug)]
enum EventSourceControl {
    Close,
}

impl EventSourceData {
    /// Create new EventSource data
    pub fn new(url: JsString, with_credentials: bool) -> Self {
        Self {
            url,
            with_credentials,
            ready_state: Arc::new(AtomicU32::new(ReadyState::Connecting as u32)),
            last_event_id: Arc::new(Mutex::new(None)),
            reconnect_time: Arc::new(AtomicU32::new(3000)), // Default 3 seconds
            control_sender: Arc::new(Mutex::new(None)),
        }
    }

    /// Get the current ready state
    pub fn ready_state(&self) -> ReadyState {
        match self.ready_state.load(Ordering::Relaxed) {
            0 => ReadyState::Connecting,
            1 => ReadyState::Open,
            2 => ReadyState::Closed,
            _ => ReadyState::Closed,
        }
    }

    /// Set the ready state
    pub fn set_ready_state(&self, state: ReadyState) {
        self.ready_state.store(state as u32, Ordering::Relaxed);
    }

    /// Get the URL
    pub fn url(&self) -> &JsString {
        &self.url
    }

    /// Get with_credentials setting
    pub fn with_credentials(&self) -> bool {
        self.with_credentials
    }

    /// Close the connection
    pub fn close(&self) {
        self.set_ready_state(ReadyState::Closed);
        if let Ok(mut sender) = self.control_sender.lock() {
            if let Some(sender) = sender.take() {
                let _ = sender.send(EventSourceControl::Close);
            }
        }
    }
}

/// EventSource builtin object
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct EventSource {
    data: EventSourceData,
}

impl IntrinsicObject for EventSource {
    fn init(realm: &Realm) {
        // Create getter functions
        let url_func = BuiltInBuilder::callable(realm, get_url)
            .name(js_string!("get url"))
            .build();
        let ready_state_func = BuiltInBuilder::callable(realm, get_ready_state)
            .name(js_string!("get readyState"))
            .build();
        let with_credentials_func = BuiltInBuilder::callable(realm, get_with_credentials)
            .name(js_string!("get withCredentials"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_property(
                js_string!("CONNECTING"),
                ReadyState::Connecting,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .static_property(
                js_string!("OPEN"),
                ReadyState::Open,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .static_property(
                js_string!("CLOSED"),
                ReadyState::Closed,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            // Instance properties as accessors
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
                js_string!("withCredentials"),
                Some(with_credentials_func),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .method(Self::close, js_string!("close"), 0)
            .property(
                js_string!("onopen"),
                JsValue::null(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .property(
                js_string!("onmessage"),
                JsValue::null(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .property(
                js_string!("onerror"),
                JsValue::null(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for EventSource {
    const NAME: JsString = StaticJsStrings::EVENT_SOURCE;
}

impl BuiltInConstructor for EventSource {
    const LENGTH: usize = 1;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::event_source;

    /// Constructor for EventSource
    ///
    /// `new EventSource(url [, eventSourceInitDict])`
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let url be the result of parsing url.
        let url_arg = args.get_or_undefined(0);
        let url_string = url_arg.to_string(context)?;
        let url_str = url_string.to_std_string_escaped();

        // Validate URL
        let _parsed_url = Url::parse(&url_str).map_err(|_| {
            JsNativeError::syntax().with_message("Invalid URL provided to EventSource constructor")
        })?;

        // 2. Parse eventSourceInitDict
        let with_credentials = if let Some(init_dict) = args.get(1) {
            if init_dict.is_object() {
                let init_obj = init_dict.as_object().unwrap();
                match init_obj.get(js_string!("withCredentials"), context) {
                    Ok(val) => val.to_boolean(),
                    Err(_) => false,
                }
            } else {
                false
            }
        } else {
            false
        };

        // 3. Create the EventSource object
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::event_source,
            context,
        )?;

        let data = EventSourceData::new(url_string.clone(), with_credentials);
        let event_source = EventSource { data };

        let object = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            event_source,
        );

        Ok(object.into())
    }
}

impl EventSource {
    /// Start the EventSource connection
    async fn start_connection(
        data: EventSourceData,
        _object: JsObject,
        url: Url,
    ) {
        let client = Client::new();
        let mut retry_count = 0;
        const MAX_RETRIES: u32 = 10;

        let (control_tx, mut control_rx) = mpsc::unbounded_channel();

        // Store the control sender
        if let Ok(mut sender) = data.control_sender.lock() {
            *sender = Some(control_tx);
        }

        loop {
            // Check if we should close
            if data.ready_state() == ReadyState::Closed {
                break;
            }

            // Set connecting state
            data.set_ready_state(ReadyState::Connecting);

            // Build request with appropriate headers
            let mut request_builder = client
                .get(url.clone())
                .header("Accept", "text/event-stream")
                .header("Cache-Control", "no-cache");

            // Add Last-Event-ID header if available
            if let Ok(last_id) = data.last_event_id.lock() {
                if let Some(ref id) = *last_id {
                    request_builder = request_builder.header("Last-Event-ID", id);
                }
            }

            // Set credentials if needed
            if data.with_credentials() {
                request_builder = request_builder.header("Credentials", "include");
            }

            // Make the request
            match request_builder.send().await {
                Ok(response) => {
                    // Check if response is successful
                    if response.status().is_success() {
                        // Verify content type
                        if let Some(content_type) = response.headers().get("content-type") {
                            if let Ok(content_type_str) = content_type.to_str() {
                                if content_type_str.starts_with("text/event-stream") {
                                    // Connection successful, set state to open
                                    data.set_ready_state(ReadyState::Open);
                                    retry_count = 0;

                                    // Process the stream
                                    let stream = response.bytes_stream();
                                    if let Err(_) = Self::process_stream(stream, &data, &mut control_rx).await {
                                        // Stream processing failed
                                        break;
                                    }
                                } else {
                                    // Wrong content type, fail permanently
                                    data.set_ready_state(ReadyState::Closed);
                                    break;
                                }
                            } else {
                                // Invalid content type header
                                data.set_ready_state(ReadyState::Closed);
                                break;
                            }
                        } else {
                            // No content type header
                            data.set_ready_state(ReadyState::Closed);
                            break;
                        }
                    } else {
                        // HTTP error, attempt to reconnect
                        retry_count += 1;
                        if retry_count > MAX_RETRIES {
                            data.set_ready_state(ReadyState::Closed);
                            break;
                        }

                        // Wait before reconnecting
                        let retry_delay = data.reconnect_time.load(Ordering::Relaxed);
                        sleep(Duration::from_millis(retry_delay as u64)).await;
                    }
                }
                Err(_) => {
                    // Network error, attempt to reconnect
                    retry_count += 1;
                    if retry_count > MAX_RETRIES {
                        data.set_ready_state(ReadyState::Closed);
                        break;
                    }

                    // Wait before reconnecting
                    let retry_delay = data.reconnect_time.load(Ordering::Relaxed);
                    sleep(Duration::from_millis(retry_delay as u64)).await;
                }
            }

            // Check for close command
            match control_rx.try_recv() {
                Ok(EventSourceControl::Close) => {
                    data.set_ready_state(ReadyState::Closed);
                    break;
                }
                Err(_) => {}
            }
        }
    }

    /// Process the Server-Sent Events stream
    async fn process_stream(
        mut stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
        data: &EventSourceData,
        control_rx: &mut mpsc::UnboundedReceiver<EventSourceControl>,
    ) -> Result<(), ()> {
        let mut buffer = String::new();
        let mut current_event = ServerSentEvent {
            event_type: "message".to_string(),
            data: String::new(),
            id: None,
            retry: None,
        };

        while let Some(chunk_result) = stream.next().await {
            // Check for close command
            match control_rx.try_recv() {
                Ok(EventSourceControl::Close) => {
                    return Err(());
                }
                Err(_) => {}
            }

            match chunk_result {
                Ok(chunk) => {
                    // Convert bytes to string (EventSource requires UTF-8)
                    match String::from_utf8(chunk.to_vec()) {
                        Ok(text) => {
                            buffer.push_str(&text);

                            // Process complete lines
                            while let Some(line_end) = buffer.find('\n') {
                                let line_string = buffer[..line_end].trim_end_matches('\r').to_string();
                                buffer.drain(..=line_end);

                                // Process the line according to EventSource spec
                                if line_string.is_empty() {
                                    // Empty line, dispatch the event
                                    Self::dispatch_event(&current_event, data);

                                    // Reset for next event
                                    current_event = ServerSentEvent {
                                        event_type: "message".to_string(),
                                        data: String::new(),
                                        id: None,
                                        retry: None,
                                    };
                                } else if let Some(colon_pos) = line_string.find(':') {
                                    // Line with field and value
                                    let field = &line_string[..colon_pos];
                                    let value = if colon_pos + 1 < line_string.len() && line_string.chars().nth(colon_pos + 1) == Some(' ') {
                                        &line_string[colon_pos + 2..]
                                    } else {
                                        &line_string[colon_pos + 1..]
                                    };

                                    Self::process_field(field, value, &mut current_event, data);
                                } else if !line_string.starts_with(':') {
                                    // Line with field but no value
                                    Self::process_field(&line_string, "", &mut current_event, data);
                                }
                                // Lines starting with ':' are comments and are ignored
                            }
                        }
                        Err(_) => {
                            // Invalid UTF-8, close connection
                            return Err(());
                        }
                    }
                }
                Err(_) => {
                    // Stream error
                    return Err(());
                }
            }
        }

        Ok(())
    }

    /// Process a field according to EventSource specification
    fn process_field(field: &str, value: &str, event: &mut ServerSentEvent, data: &EventSourceData) {
        match field {
            "event" => {
                event.event_type = value.to_string();
            }
            "data" => {
                if !event.data.is_empty() {
                    event.data.push('\n');
                }
                event.data.push_str(value);
            }
            "id" => {
                if !value.contains('\0') {
                    event.id = Some(value.to_string());
                }
            }
            "retry" => {
                if let Ok(retry_ms) = value.parse::<u32>() {
                    event.retry = Some(retry_ms);
                    data.reconnect_time.store(retry_ms, Ordering::Relaxed);
                }
            }
            _ => {
                // Unknown field, ignore
            }
        }
    }

    /// Dispatch an event to JavaScript
    fn dispatch_event(event: &ServerSentEvent, data: &EventSourceData) {
        // Update last event ID if provided
        if let Some(ref id) = event.id {
            if let Ok(mut last_id) = data.last_event_id.lock() {
                *last_id = Some(id.clone());
            }
        }

        // TODO: Dispatch the actual event to JavaScript
        // This requires event system integration which would need more context setup
        // For now, we'll print to demonstrate the events are being received
        eprintln!("EventSource event: type={}, data={:?}, id={:?}",
                 event.event_type, event.data, event.id);
    }

    /// Close the EventSource connection
    fn close(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(event_source) = object.downcast_ref::<EventSource>() {
                event_source.data.close();
            }
        }
        Ok(JsValue::undefined())
    }
}

/// Get the URL property of EventSource
fn get_url(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(object) = this.as_object() {
        if let Some(event_source) = object.downcast_ref::<EventSource>() {
            return Ok(JsValue::from(event_source.data.url().clone()));
        }
    }
    Ok(JsValue::undefined())
}

/// Get the readyState property of EventSource
fn get_ready_state(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(object) = this.as_object() {
        if let Some(event_source) = object.downcast_ref::<EventSource>() {
            return Ok(event_source.data.ready_state().into());
        }
    }
    Ok(JsValue::undefined())
}

/// Get the withCredentials property of EventSource
fn get_with_credentials(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(object) = this.as_object() {
        if let Some(event_source) = object.downcast_ref::<EventSource>() {
            return Ok(JsValue::from(event_source.data.with_credentials()));
        }
    }
    Ok(JsValue::undefined())
}

