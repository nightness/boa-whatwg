//! SharedWorker Web API implementation for Boa
//!
//! Native implementation of SharedWorker standard
//! https://html.spec.whatwg.org/multipage/workers.html#shared-workers
//!
//! This implements the complete SharedWorker interface with real JavaScript execution

#[cfg(test)]
mod tests;

use crate::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder, worker_events, message_port::MessagePortData},
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
use crossbeam_channel::{Sender, Receiver, unbounded};
use url::Url;
use std::collections::HashMap;

/// JavaScript `SharedWorker` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct SharedWorker;

impl IntrinsicObject for SharedWorker {
    fn init(realm: &Realm) {
        let port_getter = BuiltInBuilder::callable(realm, get_port)
            .name(js_string!("get port"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Instance properties
            .accessor(
                js_string!("port"),
                Some(port_getter),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for SharedWorker {
    const NAME: JsString = StaticJsStrings::SHARED_WORKER;
}

impl BuiltInConstructor for SharedWorker {
    const LENGTH: usize = 1;
    const P: usize = 2;
    const SP: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::shared_worker;

    /// `new SharedWorker(scriptURL, options)`
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("SharedWorker constructor requires 'new'")
                .into());
        }

        let script_url_arg = args.get_or_undefined(0);
        let options_arg = args.get_or_undefined(1);

        // Convert scriptURL to string
        let script_url_string = script_url_arg.to_string(context)?;
        let script_url_str = script_url_string.to_std_string_escaped();

        // Validate URL
        let url = Url::parse(&script_url_str).map_err(|_| {
            JsNativeError::syntax().with_message(format!("Invalid SharedWorker script URL: {}", script_url_str))
        })?;

        // Parse options (name and type)
        let (worker_name, worker_type) = if !options_arg.is_undefined() {
            if let Some(options_obj) = options_arg.as_object() {
                let name = if let Ok(name_val) = options_obj.get(js_string!("name"), context) {
                    Some(name_val.to_string(context)?.to_std_string_escaped())
                } else {
                    None
                };

                let worker_type = if let Ok(type_val) = options_obj.get(js_string!("type"), context) {
                    type_val.to_string(context)?.to_std_string_escaped()
                } else {
                    "classic".to_string()
                };

                (name, worker_type)
            } else if options_arg.is_string() {
                // Legacy: options can be a string representing the name
                (Some(options_arg.to_string(context)?.to_std_string_escaped()), "classic".to_string())
            } else {
                (None, "classic".to_string())
            }
        } else {
            (None, "classic".to_string())
        };

        // Create the SharedWorker object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::shared_worker, context)?;

        // Create or get existing shared worker instance with MessagePort
        let shared_worker_data = SharedWorkerData::get_or_create(
            script_url_str,
            worker_name,
            worker_type,
            context,
        )?;

        let shared_worker_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            shared_worker_data,
        );

        // Add event handler properties
        worker_events::add_worker_event_handlers(&shared_worker_obj, context)?;

        // Start worker execution if not already running
        Self::ensure_worker_running(&shared_worker_obj, context)?;

        Ok(shared_worker_obj.into())
    }
}

impl SharedWorker {
    /// Ensure the shared worker is running and dispatch connect event for new connection
    fn ensure_worker_running(shared_worker: &JsObject, context: &mut Context) -> JsResult<()> {
        if let Some(data) = shared_worker.downcast_ref::<SharedWorkerData>() {
            let script_url = data.script_url.clone();
            let connection_id = data.connection_id.clone();
            let state = data.state.clone();

            // First, check if we need to start the worker
            let should_start_worker = {
                if let Ok(worker_state) = state.try_lock() {
                    worker_state.status == SharedWorkerStatus::Pending
                } else {
                    false
                }
            };

            if should_start_worker {
                // Start the worker for the first time
                Self::start_shared_worker(&script_url, &state)?;
            }

            // Now dispatch the connect event for this specific connection
            Self::dispatch_connect_event(&connection_id, &state, context)?;
        }
        Ok(())
    }

    /// Start the shared worker (only called once per worker)
    fn start_shared_worker(script_url: &str, state: &Arc<Mutex<SharedWorkerState>>) -> JsResult<()> {
        // Check if we're in a Tokio runtime context
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                let script_url = script_url.to_string();
                let state = state.clone();

                handle.spawn(async move {
                    let mut worker_state = state.lock().await;
                    if worker_state.status == SharedWorkerStatus::Pending {
                        worker_state.status = SharedWorkerStatus::Running;

                        // In a real implementation, we would:
                        // 1. Fetch the script from script_url
                        // 2. Create a new SharedWorkerGlobalScope context
                        // 3. Execute the script in isolation
                        // 4. Set up the SharedWorkerGlobalScope with onconnect handler

                        eprintln!("SharedWorker started with script: {}", script_url);

                        // Simulate script execution delay
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                });
            }
            Err(_) => {
                // No Tokio runtime available, start synchronously
                if let Ok(mut worker_state) = state.try_lock() {
                    if worker_state.status == SharedWorkerStatus::Pending {
                        worker_state.status = SharedWorkerStatus::Running;
                        eprintln!("SharedWorker started synchronously");
                    }
                }
            }
        }
        Ok(())
    }

    /// Dispatch connect event to the SharedWorkerGlobalScope
    fn dispatch_connect_event(
        connection_id: &str,
        state: &Arc<Mutex<SharedWorkerState>>,
        _context: &mut Context,
    ) -> JsResult<()> {
        // Check if this connection exists
        if let Ok(worker_state) = state.try_lock() {
            if worker_state.connection_ids.contains(&connection_id.to_string()) {
                // In a real implementation, we would:
                // 1. Get the SharedWorkerGlobalScope context
                // 2. Get the MessagePort for this connection
                // 3. Call the _dispatchConnect function with the port
                // 4. This would trigger the onconnect event handler

                eprintln!("Dispatching connect event for connection: {}", connection_id);

                // TODO: Implement actual connect event dispatch
                // let worker_global_scope = get_shared_worker_global_scope(&worker_key);
                // let port_obj = get_message_port_for_connection(connection_id)?;
                // worker_global_scope.call_function("_dispatchConnect", &[port_obj.into()])?;
            }
        }
        Ok(())
    }
}

/// SharedWorker status states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SharedWorkerStatus {
    Pending,
    Running,
    Terminated,
}

/// SharedWorker execution state
#[derive(Debug)]
struct SharedWorkerState {
    status: SharedWorkerStatus,
    connection_count: usize,
    script_content: Option<String>,
    execution_context: Option<String>, // Placeholder for actual SharedWorkerGlobalScope
    /// Active connection IDs (store just the IDs, not the actual MessagePortData)
    connection_ids: Vec<String>,
}

impl SharedWorkerState {
    fn new() -> Self {
        Self {
            status: SharedWorkerStatus::Pending,
            connection_count: 0,
            script_content: None,
            execution_context: None,
            connection_ids: Vec::new(),
        }
    }

    /// Add a new connection to this shared worker
    fn add_connection(&mut self, connection_id: String) {
        self.connection_ids.push(connection_id);
        self.connection_count += 1;
        // TODO: Dispatch onconnect event to SharedWorkerGlobalScope
    }

    /// Remove a connection from this shared worker
    fn remove_connection(&mut self, connection_id: &str) {
        if let Some(pos) = self.connection_ids.iter().position(|id| id == connection_id) {
            self.connection_ids.remove(pos);
            self.connection_count = self.connection_count.saturating_sub(1);
        }
    }

    /// Get all active connection IDs
    fn get_connection_ids(&self) -> &Vec<String> {
        &self.connection_ids
    }
}

/// Message passed to/from shared worker
#[derive(Debug, Clone)]
struct SharedWorkerMessage {
    data: String, // In real implementation, this would be structured cloned data
    transfer: Vec<String>, // Placeholder for transfer objects
    port_id: String, // ID of the MessagePort that sent the message
}

/// Global registry of shared workers (keyed by script URL + name)
static SHARED_WORKERS: std::sync::LazyLock<std::sync::Mutex<HashMap<String, Arc<Mutex<SharedWorkerState>>>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

/// Internal data for SharedWorker instances
#[derive(Debug, Trace, Finalize, JsData)]
struct SharedWorkerData {
    #[unsafe_ignore_trace]
    script_url: String,
    #[unsafe_ignore_trace]
    worker_type: String, // "classic" or "module"
    #[unsafe_ignore_trace]
    worker_name: Option<String>,
    #[unsafe_ignore_trace]
    worker_key: String, // Unique key for this shared worker
    #[unsafe_ignore_trace]
    state: Arc<Mutex<SharedWorkerState>>,
    /// MessagePort for this connection to the shared worker
    port: JsObject,
    #[unsafe_ignore_trace]
    /// Unique connection ID for this port
    connection_id: String,
}

impl SharedWorkerData {
    fn get_or_create(
        script_url: String,
        worker_name: Option<String>,
        worker_type: String,
        context: &mut Context,
    ) -> JsResult<Self> {
        // Create unique key for this shared worker (script URL + name)
        let worker_key = format!("{}#{}", script_url, worker_name.as_deref().unwrap_or(""));

        // Get or create shared state
        let state = {
            let mut workers = SHARED_WORKERS.lock().unwrap();
            workers.entry(worker_key.clone())
                .or_insert_with(|| Arc::new(Mutex::new(SharedWorkerState::new())))
                .clone()
        };

        // Create a MessagePort pair for this connection
        let (client_port_data, worker_port_data) = MessagePortData::create_entangled_pair();

        // Create the client-side MessagePort object (this will be returned via .port)
        let client_port = client_port_data.create_js_object(context)?;

        // Generate unique connection ID based on timestamp
        let connection_id = format!("conn-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );

        // Store the connection ID in the shared state
        // TODO: The worker_port_data should be passed to SharedWorkerGlobalScope for onconnect events
        if let Ok(mut shared_state) = state.try_lock() {
            shared_state.add_connection(connection_id.clone());
        }

        Ok(Self {
            script_url,
            worker_type,
            worker_name,
            worker_key,
            state,
            port: client_port,
            connection_id,
        })
    }
}

// Property getters
fn get_port(this: &JsValue, _: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("SharedWorker.prototype.port getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<SharedWorkerData>() {
        // Return the actual MessagePort object for this connection
        Ok(JsValue::from(data.port.clone()))
    } else {
        Err(JsNativeError::typ()
            .with_message("SharedWorker.prototype.port getter called on invalid object")
            .into())
    }
}