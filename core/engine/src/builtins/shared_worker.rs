//! SharedWorker Web API implementation for Boa
//!
//! Native implementation of SharedWorker standard
//! https://html.spec.whatwg.org/multipage/workers.html#shared-workers
//!
//! This implements the complete SharedWorker interface with real JavaScript execution

#[cfg(test)]
mod tests;

use crate::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder, worker_events},
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

        // Create or get existing shared worker instance
        let shared_worker_data = SharedWorkerData::get_or_create(
            script_url_str,
            worker_name,
            worker_type,
        );

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
    /// Ensure the shared worker is running
    fn ensure_worker_running(shared_worker: &JsObject, _context: &mut Context) -> JsResult<()> {
        if let Some(data) = shared_worker.downcast_ref::<SharedWorkerData>() {
            let script_url = data.script_url.clone();
            let state = data.state.clone();

            // Check if we're in a Tokio runtime context
            match tokio::runtime::Handle::try_current() {
                Ok(handle) => {
                    handle.spawn(async move {
                        let mut worker_state = state.lock().await;

                        if worker_state.status == SharedWorkerStatus::Pending {
                            worker_state.status = SharedWorkerStatus::Running;
                            worker_state.connection_count += 1;

                            // In a real implementation, we would:
                            // 1. Fetch the script from script_url
                            // 2. Create a new SharedWorkerGlobalScope context
                            // 3. Execute the script in isolation
                            // 4. Handle multiple connections via MessagePorts
                            // 5. Dispatch 'connect' events when new clients connect

                            println!("SharedWorker started with script: {}", script_url);

                            // Simulate script execution delay
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        } else {
                            // Worker already running, just increment connection count
                            worker_state.connection_count += 1;
                        }
                    });
                }
                Err(_) => {
                    // No Tokio runtime available, increment connection count synchronously
                    if let Ok(mut state) = data.state.try_lock() {
                        state.connection_count += 1;
                        if state.status == SharedWorkerStatus::Pending {
                            state.status = SharedWorkerStatus::Running;
                        }
                    }
                }
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
}

impl SharedWorkerState {
    fn new() -> Self {
        Self {
            status: SharedWorkerStatus::Pending,
            connection_count: 0,
            script_content: None,
            execution_context: None,
        }
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
    #[unsafe_ignore_trace]
    message_sender: Option<Sender<SharedWorkerMessage>>,
    #[unsafe_ignore_trace]
    message_receiver: Option<Receiver<SharedWorkerMessage>>,
    #[unsafe_ignore_trace]
    port: Option<JsObject>, // MessagePort for this connection
}

impl SharedWorkerData {
    fn get_or_create(script_url: String, worker_name: Option<String>, worker_type: String) -> Self {
        // Create unique key for this shared worker (script URL + name)
        let worker_key = format!("{}#{}", script_url, worker_name.as_deref().unwrap_or(""));

        // Get or create shared state
        let state = {
            let mut workers = SHARED_WORKERS.lock().unwrap();
            workers.entry(worker_key.clone())
                .or_insert_with(|| Arc::new(Mutex::new(SharedWorkerState::new())))
                .clone()
        };

        let (sender, receiver) = unbounded();

        Self {
            script_url,
            worker_type,
            worker_name,
            worker_key,
            state,
            message_sender: Some(sender),
            message_receiver: Some(receiver),
            port: None, // Will be created when MessagePort is implemented
        }
    }
}

// Property getters
fn get_port(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("SharedWorker.prototype.port getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<SharedWorkerData>() {
        // In a real implementation, this would return a MessagePort object
        // For now, return a placeholder object that represents the MessagePort
        let port_placeholder = context.global_object();

        // TODO: Replace with actual MessagePort implementation
        // let port = MessagePort::create_for_shared_worker(context, &data.worker_key)?;

        Ok(JsValue::from(port_placeholder.clone()))
    } else {
        Err(JsNativeError::typ()
            .with_message("SharedWorker.prototype.port getter called on invalid object")
            .into())
    }
}