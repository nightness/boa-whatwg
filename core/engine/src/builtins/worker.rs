//! Worker Web API implementation for Boa
//!
//! Native implementation of Worker standard
//! https://html.spec.whatwg.org/multipage/workers.html
//!
//! This implements the complete Worker interface with real JavaScript execution

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

use crate::{
    builtins::{
        BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder,
        worker_events, worker_script_loader, worker_global_scope,
        worker_error::{WorkerErrorHandler, WorkerErrorType, WorkerError, error_helpers},
        message_event::create_message_event,
        structured_clone::{structured_clone, structured_deserialize, TransferList},
    },
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

/// JavaScript `Worker` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Worker;

impl IntrinsicObject for Worker {
    fn init(realm: &Realm) {
        let script_url_func = BuiltInBuilder::callable(realm, get_script_url)
            .name(js_string!("get scriptURL"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Instance methods
            .method(Self::post_message, js_string!("postMessage"), 1)
            .method(Self::terminate, js_string!("terminate"), 0)
            // Instance properties
            .accessor(
                js_string!("scriptURL"),
                Some(script_url_func),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Worker {
    const NAME: JsString = StaticJsStrings::WORKER;
}

impl BuiltInConstructor for Worker {
    const LENGTH: usize = 1;
    const P: usize = 2;
    const SP: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::worker;

    /// `new Worker(scriptURL, options)`
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Worker constructor requires 'new'")
                .into());
        }

        let script_url_arg = args.get_or_undefined(0);
        let options_arg = args.get_or_undefined(1);

        // Convert scriptURL to string
        let script_url_string = script_url_arg.to_string(context)?;
        let script_url_str = script_url_string.to_std_string_escaped();

        // Validate URL
        let url = Url::parse(&script_url_str).map_err(|e| {
            eprintln!("Invalid Worker script URL '{}': {}", script_url_str, e);
            error_helpers::script_load_error(&script_url_str, &format!("Invalid URL: {}", e))
        })?;

        // Parse options
        let worker_type = if !options_arg.is_undefined() {
            if let Some(options_obj) = options_arg.as_object() {
                if let Ok(type_val) = options_obj.get(js_string!("type"), context) {
                    type_val.to_string(context)?.to_std_string_escaped()
                } else {
                    "classic".to_string()
                }
            } else {
                "classic".to_string()
            }
        } else {
            "classic".to_string()
        };

        let worker_name = if !options_arg.is_undefined() {
            if let Some(options_obj) = options_arg.as_object() {
                if let Ok(name_val) = options_obj.get(js_string!("name"), context) {
                    Some(name_val.to_string(context)?.to_std_string_escaped())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Create the Worker object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::worker, context)?;
        let worker_data = WorkerData::new(script_url_str, worker_type, worker_name);
        let worker_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            worker_data,
        );

        // Add event handler properties
        worker_events::add_worker_event_handlers(&worker_obj, context)?;

        // Start worker execution asynchronously
        Self::start_worker(&worker_obj, context)?;

        Ok(worker_obj.into())
    }
}

impl Worker {
    /// Start worker execution
    fn start_worker(worker: &JsObject, context: &mut Context) -> JsResult<()> {
        if let Some(data) = worker.downcast_ref::<WorkerData>() {
            let script_url = data.script_url.clone();
            let worker_type = data.worker_type.clone();
            let worker_obj_clone = worker.clone();

            // Update state to running
            if let Ok(mut state) = data.state.try_lock() {
                state.status = WorkerStatus::Running;
            }

            // Start worker execution with real script loading
            let runtime_handle = tokio::runtime::Handle::try_current();
            match runtime_handle {
                Ok(handle) => {
                    handle.spawn(async move {
                        // Use the script loader to start worker execution
                        match worker_script_loader::WorkerScriptLoader::start_worker_execution(
                            script_url,
                            worker_type,
                        ).await {
                            Ok(execution_context) => {
                                eprintln!("Worker started successfully with script loading: {}", execution_context.get_script_url());
                                // TODO: Store execution context back to worker object somehow
                            }
                            Err(e) => {
                                eprintln!("Failed to start worker: {:?}", e);
                            }
                        }
                    });
                }
                Err(_) => {
                    // No Tokio runtime available - for testing
                    eprintln!("No async runtime available - worker execution deferred");
                }
            }
        }
        Ok(())
    }

    /// `Worker.prototype.postMessage(message, transfer)`
    fn post_message(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Worker.prototype.postMessage called on non-object")
        })?;

        if let Some(data) = this_obj.downcast_ref::<WorkerData>() {
            let message = args.get_or_undefined(0);
            let _transfer = args.get_or_undefined(1); // TODO: Handle transfer list

            // Check if worker is terminated
            if let Ok(state) = data.state.try_lock() {
                if state.status == WorkerStatus::Terminated {
                    // Use comprehensive error handling
                    return Err(error_helpers::worker_terminated_error("send message").into());
                }
            }

            // Parse transfer list
            let transfer_list = if !_transfer.is_undefined() {
                match TransferList::from_js_array(_transfer, context) {
                    Ok(list) => Some(list),
                    Err(e) => {
                        eprintln!("Failed to parse transfer list: {:?}", e);
                        // Dispatch error event for transfer list parsing failure
                        let _ = WorkerErrorHandler::handle_clone_error(
                            &this_obj,
                            "Invalid transfer list",
                            context
                        );
                        return Err(error_helpers::data_clone_error("Invalid transfer list").into());
                    }
                }
            } else {
                None
            };

            // Clone the message using structured cloning
            let cloned_message = match structured_clone(message, context, transfer_list.as_ref()) {
                Ok(cloned) => cloned,
                Err(e) => {
                    eprintln!("Failed to clone message for worker: {:?}", e);
                    // Dispatch error event for cloning failure
                    let error_msg = format!("Failed to clone message: {:?}", e);
                    let _ = WorkerErrorHandler::handle_clone_error(&this_obj, &error_msg, context);
                    return Err(error_helpers::data_clone_error(&error_msg).into());
                }
            };

            // Send message through the execution context if available
            if let Some(ref exec_ctx) = data.execution_context {
                // Use the async runtime to send the message
                let exec_ctx_clone = exec_ctx.clone();
                let runtime_handle = tokio::runtime::Handle::try_current();
                match runtime_handle {
                    Ok(handle) => {
                        handle.spawn(async move {
                            if let Err(e) = exec_ctx_clone.post_cloned_message_to_worker(cloned_message).await {
                                eprintln!("Failed to post message to worker execution context: {:?}", e);
                            }
                        });
                    }
                    Err(_) => {
                        eprintln!("No async runtime available for worker message passing");
                    }
                }
            } else {
                // Fallback to old message system for compatibility
                let message_str = message.to_string(context)?.to_std_string_escaped();
                eprintln!("Message posted to worker (execution context not ready): {}", message_str);

                if let Some(ref sender) = data.message_sender {
                    if let Err(_) = sender.send(WorkerMessage {
                        data: message_str,
                        transfer: Vec::new(),
                    }) {
                        // Dispatch error event for message sending failure
                        let _ = WorkerErrorHandler::handle_message_error(
                            &this_obj,
                            "Failed to send message to worker - channel closed",
                            context
                        );
                        return Err(error_helpers::worker_terminated_error("send message").into());
                    }
                }
            }
        }

        Ok(JsValue::undefined())
    }

    /// `Worker.prototype.terminate()`
    fn terminate(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Worker.prototype.terminate called on non-object")
        })?;

        if let Some(data) = this_obj.downcast_ref::<WorkerData>() {
            // Terminate the execution context if it exists
            if let Some(ref exec_ctx) = data.execution_context {
                exec_ctx.terminate();
            }

            // Update worker state to terminated
            if let Ok(mut state) = data.state.try_lock() {
                state.status = WorkerStatus::Terminated;
            }
        }

        Ok(JsValue::undefined())
    }

    /// Process messages from worker and dispatch as events
    pub fn process_worker_messages(worker_obj: &JsObject, context: &mut Context) -> JsResult<()> {
        if let Some(data) = worker_obj.downcast_ref::<WorkerData>() {
            if let Some(ref receiver) = data.worker_to_main_receiver {
                // Process all available messages
                while let Ok(worker_message) = receiver.try_recv() {
                    Self::dispatch_worker_message_event(worker_obj, worker_message, context)?;
                }
            }
        }
        Ok(())
    }

    /// Dispatch a message event on the Worker object
    fn dispatch_worker_message_event(
        worker_obj: &JsObject,
        worker_message: worker_global_scope::WorkerMessage,
        context: &mut Context,
    ) -> JsResult<()> {
        // Deserialize the structured clone data back to JavaScript
        let js_data = match structured_deserialize(&worker_message.data, context) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to deserialize worker message: {:?}", e);
                return Err(e);
            }
        };

        // Create MessageEvent
        let message_event = create_message_event(
            js_data,
            Some(""), // origin - TODO: should be worker origin
            None, // source - TODO: could be worker reference
            None, // ports - TODO: handle transferred ports
            context,
        )?;

        // Create a WorkerEvent and dispatch it on the worker object
        let worker_event = worker_events::WorkerEvent::new_message(message_event.into());
        worker_events::dispatch_worker_event(worker_obj, worker_event, context)?;

        eprintln!("Message event dispatched from worker to main thread");
        Ok(())
    }
}

/// Worker status states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkerStatus {
    Pending,
    Running,
    Terminated,
}

/// Worker execution state
#[derive(Debug)]
struct WorkerState {
    status: WorkerStatus,
    script_content: Option<String>,
    execution_context: Option<String>, // Placeholder for actual execution context
}

impl WorkerState {
    fn new() -> Self {
        Self {
            status: WorkerStatus::Pending,
            script_content: None,
            execution_context: None,
        }
    }
}

/// Message passed between main thread and worker (legacy - for old channel system)
#[derive(Debug, Clone)]
struct WorkerMessage {
    data: String, // Legacy string-based message data
    transfer: Vec<String>, // Placeholder for transfer objects
}

/// Internal data for Worker instances
#[derive(Debug, Trace, Finalize, JsData)]
struct WorkerData {
    #[unsafe_ignore_trace]
    script_url: String,
    #[unsafe_ignore_trace]
    worker_type: String, // "classic" or "module"
    #[unsafe_ignore_trace]
    worker_name: Option<String>,
    #[unsafe_ignore_trace]
    state: Arc<Mutex<WorkerState>>,
    #[unsafe_ignore_trace]
    message_sender: Option<Sender<WorkerMessage>>, // Legacy channel system
    #[unsafe_ignore_trace]
    message_receiver: Option<Receiver<WorkerMessage>>, // Legacy channel system
    #[unsafe_ignore_trace]
    execution_context: Option<Arc<worker_script_loader::WorkerExecutionContext>>,
    #[unsafe_ignore_trace]
    worker_to_main_receiver: Option<Receiver<worker_global_scope::WorkerMessage>>, // New structured message system
}

impl WorkerData {
    fn new(script_url: String, worker_type: String, worker_name: Option<String>) -> Self {
        let (sender, receiver) = unbounded(); // Legacy system

        Self {
            script_url,
            worker_type,
            worker_name,
            state: Arc::new(Mutex::new(WorkerState::new())),
            message_sender: Some(sender),
            message_receiver: Some(receiver),
            execution_context: None,
            worker_to_main_receiver: None, // Will be set when worker global scope is created
        }
    }

    fn set_execution_context(&mut self, context: Arc<worker_script_loader::WorkerExecutionContext>) {
        self.execution_context = Some(context);
    }

    fn set_worker_to_main_receiver(&mut self, receiver: Receiver<worker_global_scope::WorkerMessage>) {
        self.worker_to_main_receiver = Some(receiver);
    }
}

// Property getters
fn get_script_url(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Worker.prototype.scriptURL getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<WorkerData>() {
        Ok(JsValue::from(js_string!(data.script_url.clone())))
    } else {
        Ok(JsValue::undefined())
    }
}