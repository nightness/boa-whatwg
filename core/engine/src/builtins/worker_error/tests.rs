//! Tests for Worker error handling and propagation

use crate::{Context, JsValue, js_string, Source, JsNativeErrorKind};
use super::*;

#[test]
fn worker_error_creation() {
    let error = WorkerError::new(
        WorkerErrorType::ScriptLoadError,
        "Failed to load script".to_string(),
    );

    assert_eq!(error.error_type, WorkerErrorType::ScriptLoadError);
    assert_eq!(error.message, "Failed to load script");
    assert!(error.filename.is_none());
    assert!(error.lineno.is_none());
    assert!(error.colno.is_none());
}

#[test]
fn worker_error_with_location() {
    let error = WorkerError::with_location(
        WorkerErrorType::ScriptParseError,
        "Syntax error".to_string(),
        "worker.js".to_string(),
        10,
        5,
    );

    assert_eq!(error.error_type, WorkerErrorType::ScriptParseError);
    assert_eq!(error.message, "Syntax error");
    assert_eq!(error.filename.as_ref().unwrap(), "worker.js");
    assert_eq!(error.lineno.unwrap(), 10);
    assert_eq!(error.colno.unwrap(), 5);
}

#[test]
fn worker_error_from_js_error() {
    let mut context = Context::default();

    // Create a JavaScript error
    let code = "throw new Error('Test error message');";
    let result = context.eval(Source::from_bytes(code));

    if let Err(js_error) = result {
        // Convert JsError to JsValue
        let error_val = js_error.to_opaque(&mut context);
        let worker_error = WorkerError::from_js_error(error_val, &mut context);

        assert_eq!(worker_error.error_type, WorkerErrorType::RuntimeError);
        assert!(worker_error.message.contains("Test error message"));
        assert!(worker_error.error_object.is_some());
    } else {
        panic!("Expected error from JavaScript code");
    }
}

#[test]
fn worker_error_to_js_error() {
    let mut context = Context::default();

    let worker_error = WorkerError::new(
        WorkerErrorType::CloneError,
        "DataCloneError: Failed to clone object".to_string(),
    );

    let js_error = worker_error.to_js_error(&mut context).unwrap();
    assert!(js_error.is_object());

    if let Some(error_obj) = js_error.as_object() {
        let message = error_obj.get(js_string!("message"), &mut context).unwrap();
        assert_eq!(
            message.to_string(&mut context).unwrap().to_std_string_escaped(),
            "DataCloneError: Failed to clone object"
        );
    }
}

#[test]
fn worker_error_js_error_types() {
    assert_eq!(WorkerErrorType::ScriptParseError.js_error_type(), "SyntaxError");
    assert_eq!(WorkerErrorType::SecurityError.js_error_type(), "SecurityError");
    assert_eq!(WorkerErrorType::NetworkError.js_error_type(), "NetworkError");
    assert_eq!(WorkerErrorType::CloneError.js_error_type(), "DataCloneError");
    assert_eq!(WorkerErrorType::InvalidStateError.js_error_type(), "InvalidStateError");
    assert_eq!(WorkerErrorType::RuntimeError.js_error_type(), "Error");
}

#[test]
fn worker_error_default_messages() {
    assert_eq!(
        WorkerErrorType::ScriptLoadError.default_message(),
        "Failed to load worker script"
    );
    assert_eq!(
        WorkerErrorType::ScriptParseError.default_message(),
        "Failed to parse worker script"
    );
    assert_eq!(
        WorkerErrorType::RuntimeError.default_message(),
        "Runtime error in worker"
    );
    assert_eq!(
        WorkerErrorType::CloneError.default_message(),
        "Failed to clone object for structured cloning"
    );
    assert_eq!(
        WorkerErrorType::MessageError.default_message(),
        "Failed to send message to worker"
    );
    assert_eq!(
        WorkerErrorType::SecurityError.default_message(),
        "Security error in worker operation"
    );
    assert_eq!(
        WorkerErrorType::NetworkError.default_message(),
        "Network error in worker"
    );
    assert_eq!(
        WorkerErrorType::TerminatedError.default_message(),
        "Worker has been terminated"
    );
    assert_eq!(
        WorkerErrorType::InvalidStateError.default_message(),
        "Invalid worker state for this operation"
    );
}

#[test]
fn worker_error_to_js_native_error() {
    let script_parse_error = WorkerError::new(
        WorkerErrorType::ScriptParseError,
        "Parse error".to_string(),
    );
    let js_native_error = script_parse_error.to_js_native_error();
    assert!(matches!(js_native_error.kind, JsNativeErrorKind::Syntax));

    let invalid_state_error = WorkerError::new(
        WorkerErrorType::InvalidStateError,
        "Invalid state".to_string(),
    );
    let js_native_error = invalid_state_error.to_js_native_error();
    assert!(matches!(js_native_error.kind, JsNativeErrorKind::Type));

    let runtime_error = WorkerError::new(
        WorkerErrorType::RuntimeError,
        "Runtime error".to_string(),
    );
    let js_native_error = runtime_error.to_js_native_error();
    assert!(matches!(js_native_error.kind, JsNativeErrorKind::Error));
}

#[test]
fn error_helpers_script_load_error() {
    let error = error_helpers::script_load_error("worker.js", "Network timeout");
    let error_string = error.to_string();
    assert!(error_string.contains("Failed to load worker script 'worker.js': Network timeout"));
}

#[test]
fn error_helpers_script_parse_error() {
    let error = error_helpers::script_parse_error("worker.js", "Unexpected token");
    let error_string = error.to_string();
    assert!(error_string.contains("Failed to parse worker script 'worker.js': Unexpected token"));
}

#[test]
fn error_helpers_data_clone_error() {
    let error = error_helpers::data_clone_error("Function objects cannot be cloned");
    let error_string = error.to_string();
    assert!(error_string.contains("DataCloneError: Function objects cannot be cloned"));
}

#[test]
fn error_helpers_worker_terminated_error() {
    let error = error_helpers::worker_terminated_error("send message");
    let error_string = error.to_string();
    assert!(error_string.contains("Cannot send message - worker has been terminated"));
}

#[test]
fn error_helpers_security_error() {
    let error = error_helpers::security_error("Cross-origin request blocked");
    let error_string = error.to_string();
    assert!(error_string.contains("SecurityError: Cross-origin request blocked"));
}

#[test]
fn error_helpers_network_error() {
    let error = error_helpers::network_error("https://example.com/worker.js", "Connection failed");
    let error_string = error.to_string();
    assert!(error_string.contains("NetworkError loading 'https://example.com/worker.js': Connection failed"));
}

#[test]
fn worker_error_handler_create_error_event() {
    let mut context = Context::default();

    let error = WorkerError::with_location(
        WorkerErrorType::RuntimeError,
        "Uncaught TypeError: Cannot read property 'x' of undefined".to_string(),
        "worker.js".to_string(),
        42,
        15,
    );

    let error_event = WorkerErrorHandler::create_error_event(&error, &mut context).unwrap();
    assert!(error_event.is_object());

    if let Some(event_obj) = error_event.as_object() {
        // Check ErrorEvent properties
        let event_type = event_obj.get(js_string!("type"), &mut context).unwrap();
        assert_eq!(
            event_type.to_string(&mut context).unwrap().to_std_string_escaped(),
            "error"
        );

        let message = event_obj.get(js_string!("message"), &mut context).unwrap();
        assert_eq!(
            message.to_string(&mut context).unwrap().to_std_string_escaped(),
            "Uncaught TypeError: Cannot read property 'x' of undefined"
        );

        let filename = event_obj.get(js_string!("filename"), &mut context).unwrap();
        assert_eq!(
            filename.to_string(&mut context).unwrap().to_std_string_escaped(),
            "worker.js"
        );

        let lineno = event_obj.get(js_string!("lineno"), &mut context).unwrap();
        assert_eq!(lineno.as_number().unwrap(), 42.0);

        let colno = event_obj.get(js_string!("colno"), &mut context).unwrap();
        assert_eq!(colno.as_number().unwrap(), 15.0);

        let bubbles = event_obj.get(js_string!("bubbles"), &mut context).unwrap();
        assert_eq!(bubbles.as_boolean().unwrap(), false);

        let cancelable = event_obj.get(js_string!("cancelable"), &mut context).unwrap();
        assert_eq!(cancelable.as_boolean().unwrap(), true);

        // Should have an error property
        let error_prop = event_obj.get(js_string!("error"), &mut context).unwrap();
        assert!(error_prop.is_object());
    }
}

#[test]
fn worker_error_comprehensive_error_handling() {
    // Test that error types are comprehensive
    let all_error_types = [
        WorkerErrorType::ScriptLoadError,
        WorkerErrorType::ScriptParseError,
        WorkerErrorType::RuntimeError,
        WorkerErrorType::CloneError,
        WorkerErrorType::MessageError,
        WorkerErrorType::SecurityError,
        WorkerErrorType::NetworkError,
        WorkerErrorType::TerminatedError,
        WorkerErrorType::InvalidStateError,
    ];

    // Ensure all error types have valid JS error types and messages
    for error_type in &all_error_types {
        assert!(!error_type.js_error_type().is_empty());
        assert!(!error_type.default_message().is_empty());

        // Test that each error type can be converted to a WorkerError
        let worker_error = WorkerError::new(error_type.clone(), "Test message".to_string());
        assert_eq!(worker_error.error_type, *error_type);

        // Test that each error type can be converted to a JsNativeError
        let js_native_error = worker_error.to_js_native_error();
        assert!(!js_native_error.to_string().is_empty());
    }
}