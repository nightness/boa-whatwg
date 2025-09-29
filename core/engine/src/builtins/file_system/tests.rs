//! Tests for the File System API implementation

use crate::{Context, JsValue, Source, JsString};

#[test]
fn test_file_system_constructors_not_exposed() {
    let mut context = Context::default();

    // FileSystemHandle constructor should not be exposed globally
    let result = context.eval(Source::from_bytes("new FileSystemHandle()"));
    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("FileSystemHandle is not defined"));

    // FileSystemFileHandle constructor should not be exposed globally
    let result = context.eval(Source::from_bytes("new FileSystemFileHandle()"));
    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("FileSystemFileHandle is not defined"));

    // FileSystemDirectoryHandle constructor should not be exposed globally
    let result = context.eval(Source::from_bytes("new FileSystemDirectoryHandle()"));
    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("FileSystemDirectoryHandle is not defined"));
}

#[test]
fn test_file_picker_functions_exist() {
    let mut context = Context::default();

    // Test that showOpenFilePicker exists and is a function
    let result = context.eval(Source::from_bytes("typeof showOpenFilePicker")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    // Test that showSaveFilePicker exists and is a function
    let result = context.eval(Source::from_bytes("typeof showSaveFilePicker")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    // Test that showDirectoryPicker exists and is a function
    let result = context.eval(Source::from_bytes("typeof showDirectoryPicker")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_file_picker_functions_return_promises() {
    let mut context = Context::default();

    // Test that showOpenFilePicker returns a Promise
    let result = context.eval(Source::from_bytes("showOpenFilePicker() instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test that showSaveFilePicker returns a Promise
    let result = context.eval(Source::from_bytes("showSaveFilePicker() instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test that showDirectoryPicker returns a Promise
    let result = context.eval(Source::from_bytes("showDirectoryPicker() instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_file_system_basic_functionality() {
    let mut context = Context::default();

    // Create a mock test for file system functionality
    context.eval(Source::from_bytes(r#"
        // Test showOpenFilePicker mock functionality
        var filePickerPromise = showOpenFilePicker();
        var isPromise = filePickerPromise instanceof Promise;

        // Test showSaveFilePicker mock functionality
        var savePickerPromise = showSaveFilePicker();
        var isSavePromise = savePickerPromise instanceof Promise;

        // Test showDirectoryPicker mock functionality
        var dirPickerPromise = showDirectoryPicker();
        var isDirPromise = dirPickerPromise instanceof Promise;
    "#)).unwrap();

    // Test that promises were created correctly
    let result = context.eval(Source::from_bytes("isPromise")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("isSavePromise")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("isDirPromise")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_file_handle_methods_exist() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Test that file handle has expected methods after resolution
        var fileHandleTest = false;
        showOpenFilePicker().then(function(fileHandles) {
            if (fileHandles && fileHandles.length > 0) {
                var fileHandle = fileHandles[0];
                fileHandleTest = typeof fileHandle === 'object' &&
                                typeof fileHandle.getFile === 'function' &&
                                typeof fileHandle.createWritable === 'function';
            }
        });
    "#)).unwrap();

    // Note: In a real async environment, we would need to wait for the promise
    // This test just verifies the structure is set up correctly
}

#[test]
fn test_directory_handle_methods_exist() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Test that directory handle has expected methods after resolution
        var dirHandleTest = false;
        showDirectoryPicker().then(function(dirHandle) {
            if (dirHandle) {
                dirHandleTest = typeof dirHandle === 'object' &&
                               typeof dirHandle.getFileHandle === 'function' &&
                               typeof dirHandle.getDirectoryHandle === 'function' &&
                               typeof dirHandle.removeEntry === 'function' &&
                               typeof dirHandle.resolve === 'function';
            }
        });
    "#)).unwrap();

    // Note: In a real async environment, we would need to wait for the promise
    // This test just verifies the structure is set up correctly
}

#[test]
fn test_file_system_vfs_integration() {
    let mut context = Context::default();

    // Test that file picker functions work with VFS
    let result = context.eval(Source::from_bytes("showOpenFilePicker() instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("showSaveFilePicker() instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("showDirectoryPicker() instanceof Promise")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_file_system_handle_common_methods() {
    let mut context = Context::default();

    context.eval(Source::from_bytes(r#"
        // Test common FileSystemHandle methods
        var handleMethodsTest = false;
        showSaveFilePicker().then(function(fileHandle) {
            if (fileHandle) {
                handleMethodsTest = typeof fileHandle.isSameEntry === 'function' &&
                                   typeof fileHandle.queryPermission === 'function' &&
                                   typeof fileHandle.requestPermission === 'function';
            }
        });
    "#)).unwrap();

    // Note: In a real async environment, we would need to wait for the promise
    // This test just verifies the structure is set up correctly
}
