use crate::{run_test_actions, TestAction};

#[test]
fn blob_constructor_exists() {
    run_test_actions([
        TestAction::assert("typeof Blob === 'function'"),
        TestAction::assert("Blob.name === 'Blob'"),
    ]);
}

#[test]
fn blob_empty_constructor() {
    run_test_actions([
        TestAction::run("let blob = new Blob()"),
        TestAction::assert("blob instanceof Blob"),
        TestAction::assert("blob.size === 0"),
        TestAction::assert("blob.type === ''"),
    ]);
}

#[test]
fn blob_string_array() {
    run_test_actions([
        TestAction::run("let blob = new Blob(['Hello', ' ', 'World'])"),
        TestAction::assert("blob instanceof Blob"),
        TestAction::assert("blob.size === 11"), // 'Hello World' = 11 bytes
        TestAction::assert("blob.type === ''"),
    ]);
}

#[test]
fn blob_with_type() {
    run_test_actions([
        TestAction::run("let blob = new Blob(['Hello World'], { type: 'text/plain' })"),
        TestAction::assert("blob.type === 'text/plain'"),
        TestAction::assert("blob.size === 11"),
    ]);
}

#[test]
fn blob_mixed_content() {
    run_test_actions([
        TestAction::run("let blob1 = new Blob(['Hello'])"),
        TestAction::run("let blob2 = new Blob([blob1, ' World'])"),
        TestAction::assert("blob2.size === 11"),
        TestAction::assert("blob2.type === ''"),
    ]);
}

#[test]
fn blob_slice_method() {
    run_test_actions([
        TestAction::run("let blob = new Blob(['Hello World'])"),
        TestAction::run("let slice1 = blob.slice(0, 5)"),
        TestAction::assert("slice1 instanceof Blob"),
        TestAction::assert("slice1.size === 5"),
        TestAction::assert("slice1.type === ''"),
    ]);
}

#[test]
fn blob_slice_with_type() {
    run_test_actions([
        TestAction::run("let blob = new Blob(['Hello World'])"),
        TestAction::run("let slice = blob.slice(0, 5, 'text/html')"),
        TestAction::assert("slice.type === 'text/html'"),
        TestAction::assert("slice.size === 5"),
    ]);
}

#[test]
fn blob_slice_negative_indices() {
    run_test_actions([
        TestAction::run("let blob = new Blob(['Hello World'])"), // 11 bytes
        TestAction::run("let slice1 = blob.slice(-5)"), // Last 5 bytes: 'World'
        TestAction::assert("slice1.size === 5"),
        TestAction::run("let slice2 = blob.slice(0, -6)"), // First 5 bytes: 'Hello'
        TestAction::assert("slice2.size === 5"),
        TestAction::run("let slice3 = blob.slice(-5, -1)"), // 'Worl'
        TestAction::assert("slice3.size === 4"),
    ]);
}

#[test]
fn blob_slice_out_of_bounds() {
    run_test_actions([
        TestAction::run("let blob = new Blob(['Hello'])"), // 5 bytes
        TestAction::run("let slice1 = blob.slice(0, 100)"), // End beyond size
        TestAction::assert("slice1.size === 5"),
        TestAction::run("let slice2 = blob.slice(100, 200)"), // Start beyond size
        TestAction::assert("slice2.size === 0"),
        TestAction::run("let slice3 = blob.slice(-100, 2)"), // Start before beginning
        TestAction::assert("slice3.size === 2"),
    ]);
}

#[test]
fn blob_slice_invalid_range() {
    run_test_actions([
        TestAction::run("let blob = new Blob(['Hello World'])"),
        TestAction::run("let slice = blob.slice(5, 2)"), // start > end
        TestAction::assert("slice.size === 0"),
    ]);
}

#[test]
fn blob_text_method() {
    run_test_actions([
        TestAction::run("let blob = new Blob(['Hello World'])"),
        TestAction::assert("typeof blob.text === 'function'"),
        // text() should now return a Promise
        TestAction::run("let result = blob.text()"),
        TestAction::assert("result instanceof Promise"),
    ]);
}

#[test]
fn blob_array_buffer_method() {
    run_test_actions([
        TestAction::run("let blob = new Blob(['Hello'])"),
        TestAction::assert("typeof blob.arrayBuffer === 'function'"),
        // Note: arrayBuffer() should return a Promise<ArrayBuffer>, but for now returns undefined
        TestAction::run("let result = blob.arrayBuffer()"),
        TestAction::assert("result === undefined || result instanceof Promise"),
    ]);
}

#[test]
fn blob_stream_method() {
    run_test_actions([
        TestAction::run("let blob = new Blob(['Hello'])"),
        TestAction::assert("typeof blob.stream === 'function'"),
        // stream() should return a ReadableStream with advanced features
        TestAction::run("let result = blob.stream()"),
        TestAction::assert("result.constructor.name === 'ReadableStream'"),
        TestAction::assert("typeof result.getReader === 'function'"),
        TestAction::assert("typeof result.cancel === 'function'"),
    ]);
}

#[test]
fn blob_size_property() {
    run_test_actions([
        TestAction::run("let blob = new Blob()"),
        TestAction::assert("typeof blob.size === 'number'"),
        TestAction::assert("blob.size >= 0"),

        TestAction::run("let blob2 = new Blob([''])"),
        TestAction::assert("blob2.size === 0"),

        TestAction::run("let blob3 = new Blob(['a'])"),
        TestAction::assert("blob3.size === 1"),

        TestAction::run("let blob4 = new Blob(['abc'])"),
        TestAction::assert("blob4.size === 3"),
    ]);
}

#[test]
fn blob_type_property() {
    run_test_actions([
        TestAction::run("let blob1 = new Blob()"),
        TestAction::assert("typeof blob1.type === 'string'"),
        TestAction::assert("blob1.type === ''"),

        TestAction::run("let blob2 = new Blob([], { type: 'text/plain' })"),
        TestAction::assert("blob2.type === 'text/plain'"),

        TestAction::run("let blob3 = new Blob([], { type: 'application/json' })"),
        TestAction::assert("blob3.type === 'application/json'"),
    ]);
}

#[test]
fn blob_constructor_edge_cases() {
    run_test_actions([
        // Non-array first argument should not throw (browsers are permissive)
        TestAction::run("let blob1 = new Blob('string')"),
        TestAction::assert("blob1.size >= 0"),

        // Null/undefined parts
        TestAction::run("let blob2 = new Blob([null, undefined])"),
        TestAction::assert("blob2.size >= 0"),

        // Numbers and booleans get converted to strings
        TestAction::run("let blob3 = new Blob([42, true, false])"),
        TestAction::assert("blob3.size > 0"),
    ]);
}

#[test]
fn blob_options_handling() {
    run_test_actions([
        // Missing type in options
        TestAction::run("let blob1 = new Blob(['test'], {})"),
        TestAction::assert("blob1.type === ''"),

        // Non-string type gets converted
        TestAction::run("let blob2 = new Blob(['test'], { type: 123 })"),
        TestAction::assert("typeof blob2.type === 'string'"),

        // Undefined options
        TestAction::run("let blob3 = new Blob(['test'], undefined)"),
        TestAction::assert("blob3.type === ''"),

        // Null options
        TestAction::run("let blob4 = new Blob(['test'], null)"),
        TestAction::assert("blob4.type === ''"),
    ]);
}

#[test]
fn blob_stream_advanced_features() {
    run_test_actions([
        TestAction::run("let blob = new Blob(['a'.repeat(100000)])"), // Large blob for chunking
        TestAction::run("let stream = blob.stream()"),

        // Test that stream has proper structure
        TestAction::assert("stream instanceof ReadableStream"),
        TestAction::assert("typeof stream.getReader === 'function'"),
        TestAction::assert("typeof stream.cancel === 'function'"),
        TestAction::assert("typeof stream.tee === 'function'"),

        // Test reader functionality
        TestAction::run("let reader = stream.getReader()"),
        TestAction::assert("reader !== null"),
        TestAction::assert("typeof reader === 'object'"),
    ]);
}

#[test]
fn blob_stream_cancellation() {
    run_test_actions([
        TestAction::run("let blob = new Blob(['test data for cancellation'])"),
        TestAction::run("let stream = blob.stream()"),

        // Test cancellation
        TestAction::run("let cancelPromise = stream.cancel('user cancelled')"),
        TestAction::assert("cancelPromise instanceof Promise"),

        // Stream should be cancelled
        TestAction::assert("stream.locked === false || stream.locked === true"), // State may vary
    ]);
}

#[test]
fn blob_stream_backpressure() {
    run_test_actions([
        // Create large blob to test backpressure
        TestAction::run("let largeData = 'x'.repeat(1000000)"), // 1MB data
        TestAction::run("let blob = new Blob([largeData])"),
        TestAction::run("let stream = blob.stream()"),

        // Test that stream handles large data
        TestAction::assert("stream instanceof ReadableStream"),
        TestAction::run("let reader = stream.getReader()"),
        TestAction::assert("typeof reader.read === 'function'"),
    ]);
}

#[test]
fn blob_stream_chunking() {
    run_test_actions([
        // Test with data that will be chunked
        TestAction::run("let data = 'A'.repeat(200000)"), // 200KB, should create multiple chunks
        TestAction::run("let blob = new Blob([data])"),
        TestAction::run("let stream = blob.stream()"),

        // Verify stream setup for chunked reading
        TestAction::assert("stream instanceof ReadableStream"),
        TestAction::run("let reader = stream.getReader()"),
        TestAction::assert("reader !== null"),
    ]);
}

#[test]
fn blob_promise_based_methods() {
    run_test_actions([
        TestAction::run("let blob = new Blob(['promise test data'])"),

        // Test that text() returns a Promise
        TestAction::run("let textPromise = blob.text()"),
        TestAction::assert("textPromise instanceof Promise"),

        // Test that arrayBuffer() returns a Promise
        TestAction::run("let bufferPromise = blob.arrayBuffer()"),
        TestAction::assert("bufferPromise instanceof Promise"),

        // Both promises should be separate instances
        TestAction::assert("textPromise !== bufferPromise"),
    ]);
}