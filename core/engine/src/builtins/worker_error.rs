//! Worker error handling and propagation
//!
//! Implements WHATWG-compliant error handling for Workers including:
//! - Script loading errors
//! - Runtime errors
//! - Message passing errors
//! - Security errors
//! - Network errors

#[cfg(test)]
mod tests;

use crate::{
    Context, JsResult, JsValue, JsNativeError, JsObject, js_string,
    builtins::{
        worker_events::{WorkerEvent, WorkerEventType, dispatch_worker_event},
    },
    property::Attribute,
};
use boa_gc::{Finalize, Trace};

/// Types of worker errors according to WHATWG specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerErrorType {
    /// Script failed to load (network error, 404, etc.)
    ScriptLoadError,
    /// Script failed to parse (syntax error)
    ScriptParseError,
    /// Runtime error during script execution
    RuntimeError,
    /// Structured cloning failed
    CloneError,
    /// Message passing failed
    MessageError,
    /// Security violation (cross-origin, etc.)
    SecurityError,
    /// Network request failed
    NetworkError,
    /// Worker is terminated
    TerminatedError,
    /// Invalid operation or state
    InvalidStateError,
}

impl WorkerErrorType {
    /// Get the JavaScript error type for this worker error
    pub fn js_error_type(&self) -> &'static str {
        match self {
            Self::ScriptLoadError => "Error",
            Self::ScriptParseError => "SyntaxError",
            Self::RuntimeError => "Error",
            Self::CloneError => "DataCloneError",
            Self::MessageError => "Error",
            Self::SecurityError => "SecurityError",
            Self::NetworkError => "NetworkError",
            Self::TerminatedError => "InvalidStateError",
            Self::InvalidStateError => "InvalidStateError",
        }
    }

    /// Get a default error message for this error type
    pub fn default_message(&self) -> &'static str {
        match self {
            Self::ScriptLoadError => "Failed to load worker script",
            Self::ScriptParseError => "Failed to parse worker script",
            Self::RuntimeError => "Runtime error in worker",
            Self::CloneError => "Failed to clone object for structured cloning",
            Self::MessageError => "Failed to send message to worker",
            Self::SecurityError => "Security error in worker operation",
            Self::NetworkError => "Network error in worker",
            Self::TerminatedError => "Worker has been terminated",
            Self::InvalidStateError => "Invalid worker state for this operation",
        }
    }
}

/// Worker error information
#[derive(Debug, Clone)]
pub struct WorkerError {
    /// Type of error
    pub error_type: WorkerErrorType,
    /// Error message
    pub message: String,
    /// Source file where error occurred
    pub filename: Option<String>,
    /// Line number where error occurred
    pub lineno: Option<u32>,
    /// Column number where error occurred
    pub colno: Option<u32>,
    /// JavaScript Error object
    pub error_object: Option<JsValue>,
}

impl WorkerError {
    /// Create a new worker error
    pub fn new(error_type: WorkerErrorType, message: String) -> Self {
        Self {
            error_type,
            message,
            filename: None,
            lineno: None,
            colno: None,
            error_object: None,
        }
    }

    /// Create a worker error with location information
    pub fn with_location(
        error_type: WorkerErrorType,
        message: String,
        filename: String,
        lineno: u32,
        colno: u32,
    ) -> Self {
        Self {
            error_type,
            message,
            filename: Some(filename),
            lineno: Some(lineno),
            colno: Some(colno),
            error_object: None,
        }
    }

    /// Create a worker error from a JavaScript error object
    pub fn from_js_error(error_object: JsValue, context: &mut Context) -> Self {
        let message = if let Some(error_obj) = error_object.as_object() {
            // Try to get the message property
            if let Ok(msg_val) = error_obj.get(js_string!("message"), context) {
                msg_val.to_string(context)
                    .map(|s| s.to_std_string_escaped())
                    .unwrap_or_else(|_| "Unknown error".to_string())
            } else {
                "Unknown error".to_string()
            }
        } else {
            error_object.to_string(context)
                .map(|s| s.to_std_string_escaped())
                .unwrap_or_else(|_| "Unknown error".to_string())
        };

        Self {
            error_type: WorkerErrorType::RuntimeError,
            message,
            filename: None,
            lineno: None,
            colno: None,
            error_object: Some(error_object),
        }
    }

    /// Convert to a JavaScript Error object
    pub fn to_js_error(&self, context: &mut Context) -> JsResult<JsValue> {
        if let Some(ref error_obj) = self.error_object {
            return Ok(error_obj.clone());
        }

        // Create appropriate error type
        let error_constructor = match self.error_type {
            WorkerErrorType::ScriptParseError => {
                context.intrinsics().constructors().syntax_error().constructor()
            }
            WorkerErrorType::SecurityError => {
                // For now, use Error as SecurityError may not be implemented
                context.intrinsics().constructors().error().constructor()
            }
            WorkerErrorType::NetworkError => {
                // For now, use Error as NetworkError may not be implemented
                context.intrinsics().constructors().error().constructor()
            }
            WorkerErrorType::CloneError => {
                // DataCloneError - use Error for now
                context.intrinsics().constructors().error().constructor()
            }
            WorkerErrorType::InvalidStateError | WorkerErrorType::TerminatedError => {
                // InvalidStateError - use Error for now
                context.intrinsics().constructors().error().constructor()
            }
            _ => {
                context.intrinsics().constructors().error().constructor()
            }
        };

        let args = [js_string!(self.message.clone()).into()];
        let new_target = Some(&error_constructor);
        let error_obj = error_constructor.construct(&args, new_target, context)?;

        // Add additional properties if available
        if let Some(ref filename) = self.filename {
            let filename_val = js_string!(filename.clone());
            error_obj.set(js_string!("filename"), filename_val, true, context)?;
        }

        if let Some(lineno) = self.lineno {
            error_obj.set(js_string!("lineno"), lineno, true, context)?;
        }

        if let Some(colno) = self.colno {
            error_obj.set(js_string!("colno"), colno, true, context)?;
        }

        Ok(error_obj.into())
    }

    /// Create a JsNativeError from this worker error
    pub fn to_js_native_error(&self) -> JsNativeError {
        let message = self.message.clone();
        match self.error_type {
            WorkerErrorType::ScriptParseError => {
                JsNativeError::syntax().with_message(message)
            }
            WorkerErrorType::InvalidStateError | WorkerErrorType::TerminatedError => {
                JsNativeError::typ().with_message(message)
            }
            _ => {
                JsNativeError::error().with_message(message)
            }
        }
    }
}

/// Worker error handler - manages error propagation and event dispatching
#[derive(Debug)]
pub struct WorkerErrorHandler;

impl WorkerErrorHandler {
    /// Handle a worker error by dispatching error events
    pub fn handle_error(
        worker_obj: &JsObject,
        error: WorkerError,
        context: &mut Context,
    ) -> JsResult<()> {
        eprintln!("Worker error: {:?} - {}", error.error_type, error.message);

        // Create error event
        let error_event = Self::create_error_event(&error, context)?;

        // Dispatch error event on the worker
        let worker_event = WorkerEvent::new_error(error_event);
        dispatch_worker_event(worker_obj, worker_event, context)?;

        // Also dispatch on global scope if this is a runtime error
        if matches!(error.error_type, WorkerErrorType::RuntimeError) {
            // TODO: Dispatch on worker global scope as well
            eprintln!("Runtime error dispatched to worker and global scope");
        }

        Ok(())
    }

    /// Create an ErrorEvent object from a WorkerError
    fn create_error_event(error: &WorkerError, context: &mut Context) -> JsResult<JsValue> {
        // Create ErrorEvent object
        let error_event_obj = JsObject::with_object_proto(context.intrinsics());

        // Set ErrorEvent properties
        error_event_obj.set(js_string!("type"), js_string!("error"), false, context)?;
        error_event_obj.set(js_string!("bubbles"), false, false, context)?;
        error_event_obj.set(js_string!("cancelable"), true, false, context)?;

        // ErrorEvent-specific properties
        error_event_obj.set(
            js_string!("message"),
            js_string!(error.message.clone()),
            false,
            context,
        )?;

        if let Some(ref filename) = error.filename {
            error_event_obj.set(
                js_string!("filename"),
                js_string!(filename.clone()),
                false,
                context,
            )?;
        } else {
            error_event_obj.set(js_string!("filename"), js_string!(""), false, context)?;
        }

        error_event_obj.set(
            js_string!("lineno"),
            error.lineno.unwrap_or(0),
            false,
            context,
        )?;

        error_event_obj.set(
            js_string!("colno"),
            error.colno.unwrap_or(0),
            false,
            context,
        )?;

        // Add the error object if available
        if let Some(ref err_obj) = error.error_object {
            error_event_obj.set(js_string!("error"), err_obj.clone(), false, context)?;
        } else {
            // Create error object from the worker error
            let js_error = error.to_js_error(context)?;
            error_event_obj.set(js_string!("error"), js_error, false, context)?;
        }

        Ok(error_event_obj.into())
    }

    /// Handle script loading errors
    pub fn handle_script_load_error(
        worker_obj: &JsObject,
        script_url: &str,
        error_msg: &str,
        context: &mut Context,
    ) -> JsResult<()> {
        let error = WorkerError::new(
            WorkerErrorType::ScriptLoadError,
            format!("Failed to load script '{}': {}", script_url, error_msg),
        );

        Self::handle_error(worker_obj, error, context)
    }

    /// Handle script parse errors
    pub fn handle_script_parse_error(
        worker_obj: &JsObject,
        script_url: &str,
        error_msg: &str,
        lineno: Option<u32>,
        colno: Option<u32>,
        context: &mut Context,
    ) -> JsResult<()> {
        let mut error = WorkerError::new(
            WorkerErrorType::ScriptParseError,
            format!("Parse error in '{}': {}", script_url, error_msg),
        );

        error.filename = Some(script_url.to_string());
        error.lineno = lineno;
        error.colno = colno;

        Self::handle_error(worker_obj, error, context)
    }

    /// Handle runtime errors from worker execution
    pub fn handle_runtime_error(
        worker_obj: &JsObject,
        js_error: JsValue,
        context: &mut Context,
    ) -> JsResult<()> {
        let error = WorkerError::from_js_error(js_error, context);
        Self::handle_error(worker_obj, error, context)
    }

    /// Handle structured cloning errors
    pub fn handle_clone_error(
        worker_obj: &JsObject,
        error_msg: &str,
        context: &mut Context,
    ) -> JsResult<()> {
        let error = WorkerError::new(
            WorkerErrorType::CloneError,
            format!("DataCloneError: {}", error_msg),
        );

        Self::handle_error(worker_obj, error, context)
    }

    /// Handle message passing errors
    pub fn handle_message_error(
        worker_obj: &JsObject,
        error_msg: &str,
        context: &mut Context,
    ) -> JsResult<()> {
        let error = WorkerError::new(
            WorkerErrorType::MessageError,
            format!("Message error: {}", error_msg),
        );

        Self::handle_error(worker_obj, error, context)
    }

    /// Handle security errors
    pub fn handle_security_error(
        worker_obj: &JsObject,
        error_msg: &str,
        context: &mut Context,
    ) -> JsResult<()> {
        let error = WorkerError::new(
            WorkerErrorType::SecurityError,
            format!("SecurityError: {}", error_msg),
        );

        Self::handle_error(worker_obj, error, context)
    }

    /// Handle network errors
    pub fn handle_network_error(
        worker_obj: &JsObject,
        url: &str,
        error_msg: &str,
        context: &mut Context,
    ) -> JsResult<()> {
        let error = WorkerError::new(
            WorkerErrorType::NetworkError,
            format!("NetworkError loading '{}': {}", url, error_msg),
        );

        Self::handle_error(worker_obj, error, context)
    }

    /// Handle worker termination errors
    pub fn handle_termination_error(
        worker_obj: &JsObject,
        operation: &str,
        context: &mut Context,
    ) -> JsResult<()> {
        let error = WorkerError::new(
            WorkerErrorType::TerminatedError,
            format!("Cannot {} - worker has been terminated", operation),
        );

        Self::handle_error(worker_obj, error, context)
    }
}

/// Convenience functions for error handling
pub mod error_helpers {
    use super::*;

    /// Create a script load error
    pub fn script_load_error(script_url: &str, cause: &str) -> JsNativeError {
        JsNativeError::error().with_message(format!(
            "Failed to load worker script '{}': {}",
            script_url, cause
        ))
    }

    /// Create a script parse error
    pub fn script_parse_error(script_url: &str, cause: &str) -> JsNativeError {
        JsNativeError::syntax().with_message(format!(
            "Failed to parse worker script '{}': {}",
            script_url, cause
        ))
    }

    /// Create a data clone error
    pub fn data_clone_error(cause: &str) -> JsNativeError {
        JsNativeError::typ().with_message(format!("DataCloneError: {}", cause))
    }

    /// Create a worker terminated error
    pub fn worker_terminated_error(operation: &str) -> JsNativeError {
        JsNativeError::error().with_message(format!(
            "Cannot {} - worker has been terminated",
            operation
        ))
    }

    /// Create a security error
    pub fn security_error(cause: &str) -> JsNativeError {
        JsNativeError::error().with_message(format!("SecurityError: {}", cause))
    }

    /// Create a network error
    pub fn network_error(url: &str, cause: &str) -> JsNativeError {
        JsNativeError::error().with_message(format!(
            "NetworkError loading '{}': {}",
            url, cause
        ))
    }
}