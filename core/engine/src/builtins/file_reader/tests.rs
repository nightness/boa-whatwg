use crate::{run_test_actions, TestAction};

#[test]
fn file_reader_constructor_exists() {
    run_test_actions([
        TestAction::assert("typeof FileReader === 'function'"),
        TestAction::assert("FileReader.name === 'FileReader'"),
        TestAction::assert("FileReader.length === 0"), // No required parameters
    ]);
}

#[test]
fn file_reader_constructor_basic() {
    run_test_actions([
        TestAction::run("let reader = new FileReader()"),
        TestAction::assert("reader instanceof FileReader"),
        TestAction::assert("typeof reader === 'object'"),
    ]);
}

#[test]
fn file_reader_initial_state() {
    run_test_actions([
        TestAction::run("let reader = new FileReader()"),

        // Initial ready state should be EMPTY (0)
        TestAction::assert("reader.readyState === FileReader.EMPTY"),
        TestAction::assert("reader.readyState === 0"),

        // Initial result should be null
        TestAction::assert("reader.result === null"),

        // Initial error should be null
        TestAction::assert("reader.error === null"),
    ]);
}

#[test]
fn file_reader_constants() {
    run_test_actions([
        // Test FileReader constants
        TestAction::assert("FileReader.EMPTY === 0"),
        TestAction::assert("FileReader.LOADING === 1"),
        TestAction::assert("FileReader.DONE === 2"),

        // Constants should be accessible on instances
        TestAction::run("let reader = new FileReader()"),
        TestAction::assert("reader.constructor.EMPTY === 0"),
        TestAction::assert("reader.constructor.LOADING === 1"),
        TestAction::assert("reader.constructor.DONE === 2"),
    ]);
}

#[test]
fn file_reader_methods_exist() {
    run_test_actions([
        TestAction::run("let reader = new FileReader()"),

        // Read methods
        TestAction::assert("typeof reader.readAsArrayBuffer === 'function'"),
        TestAction::assert("typeof reader.readAsBinaryString === 'function'"),
        TestAction::assert("typeof reader.readAsDataURL === 'function'"),
        TestAction::assert("typeof reader.readAsText === 'function'"),
        TestAction::assert("typeof reader.abort === 'function'"),

        // Method lengths
        TestAction::assert("reader.readAsArrayBuffer.length === 1"),
        TestAction::assert("reader.readAsBinaryString.length === 1"),
        TestAction::assert("reader.readAsDataURL.length === 1"),
        TestAction::assert("reader.readAsText.length === 1"),
        TestAction::assert("reader.abort.length === 0"),
    ]);
}

#[test]
fn file_reader_event_handlers() {
    run_test_actions([
        TestAction::run("let reader = new FileReader()"),

        // Event handlers should initially be null
        TestAction::assert("reader.onloadstart === null"),
        TestAction::assert("reader.onprogress === null"),
        TestAction::assert("reader.onload === null"),
        TestAction::assert("reader.onloadend === null"),
        TestAction::assert("reader.onerror === null"),
        TestAction::assert("reader.onabort === null"),

        // Should be able to set event handlers
        TestAction::run("reader.onload = function() { console.log('loaded'); }"),
        TestAction::assert("typeof reader.onload === 'function'"),

        TestAction::run("reader.onerror = function() { console.log('error'); }"),
        TestAction::assert("typeof reader.onerror === 'function'"),

        // Setting to null should work
        TestAction::run("reader.onload = null"),
        TestAction::assert("reader.onload === null"),
    ]);
}

#[test]
fn file_reader_read_as_text() {
    run_test_actions([
        // Create a test file
        TestAction::run("let file = new File(['Hello World'], 'test.txt', { type: 'text/plain' })"),
        TestAction::run("let reader = new FileReader()"),

        // Test readAsText method
        TestAction::run("reader.readAsText(file)"),

        // State should change to LOADING
        TestAction::assert("reader.readyState === FileReader.LOADING"),
        TestAction::assert("reader.readyState === 1"),

        // Result should still be null during loading
        TestAction::assert("reader.result === null"),
    ]);
}

#[test]
fn file_reader_read_as_data_url() {
    run_test_actions([
        TestAction::run("let file = new File(['Hello'], 'test.txt')"),
        TestAction::run("let reader = new FileReader()"),

        TestAction::run("reader.readAsDataURL(file)"),
        TestAction::assert("reader.readyState === FileReader.LOADING"),
    ]);
}

#[test]
fn file_reader_read_as_array_buffer() {
    run_test_actions([
        TestAction::run("let file = new File(['test data'], 'test.bin')"),
        TestAction::run("let reader = new FileReader()"),

        TestAction::run("reader.readAsArrayBuffer(file)"),
        TestAction::assert("reader.readyState === FileReader.LOADING"),
    ]);
}

#[test]
fn file_reader_read_as_binary_string() {
    run_test_actions([
        TestAction::run("let file = new File([new Uint8Array([72, 101, 108, 108, 111])], 'binary.dat')"),
        TestAction::run("let reader = new FileReader()"),

        TestAction::run("reader.readAsBinaryString(file)"),
        TestAction::assert("reader.readyState === FileReader.LOADING"),
    ]);
}

#[test]
fn file_reader_read_with_blob() {
    run_test_actions([
        // FileReader should work with Blob objects too
        TestAction::run("let blob = new Blob(['Blob content'], { type: 'text/plain' })"),
        TestAction::run("let reader = new FileReader()"),

        TestAction::run("reader.readAsText(blob)"),
        TestAction::assert("reader.readyState === FileReader.LOADING"),
    ]);
}

#[test]
fn file_reader_multiple_reads_error() {
    run_test_actions([
        TestAction::run("let file = new File(['test'], 'test.txt')"),
        TestAction::run("let reader = new FileReader()"),

        // Start first read
        TestAction::run("reader.readAsText(file)"),
        TestAction::assert("reader.readyState === FileReader.LOADING"),

        // Attempting second read should throw
        TestAction::run("try { reader.readAsText(file); } catch(e) { var errorThrown = e.name; }"),
        TestAction::assert("errorThrown === 'TypeError'"),
    ]);
}

#[test]
fn file_reader_abort() {
    run_test_actions([
        TestAction::run("let file = new File(['test data'], 'test.txt')"),
        TestAction::run("let reader = new FileReader()"),

        // Start reading
        TestAction::run("reader.readAsText(file)"),
        TestAction::assert("reader.readyState === FileReader.LOADING"),

        // Abort the operation
        TestAction::run("reader.abort()"),
        TestAction::assert("reader.readyState === FileReader.DONE"),
        TestAction::assert("reader.result === null"),
        TestAction::assert("reader.error !== null"),
    ]);
}

#[test]
fn file_reader_abort_when_not_loading() {
    run_test_actions([
        TestAction::run("let reader = new FileReader()"),

        // Abort when not loading should not change state
        TestAction::assert("reader.readyState === FileReader.EMPTY"),
        TestAction::run("reader.abort()"),
        TestAction::assert("reader.readyState === FileReader.EMPTY"),
        TestAction::assert("reader.result === null"),
        TestAction::assert("reader.error === null"),
    ]);
}

#[test]
fn file_reader_read_text_with_encoding() {
    run_test_actions([
        TestAction::run("let file = new File(['test'], 'test.txt')"),
        TestAction::run("let reader = new FileReader()"),

        // Test with UTF-8 encoding
        TestAction::run("reader.readAsText(file, 'UTF-8')"),
        TestAction::assert("reader.readyState === FileReader.LOADING"),
    ]);
}

#[test]
fn file_reader_invalid_arguments() {
    run_test_actions([
        TestAction::run("let reader = new FileReader()"),

        // Reading without arguments should throw
        TestAction::run("try { reader.readAsText(); } catch(e) { var error1 = e.name; }"),
        TestAction::assert("error1 === 'TypeError'"),

        // Reading with non-File/Blob should throw
        TestAction::run("try { reader.readAsText('not a file'); } catch(e) { var error2 = e.name; }"),
        TestAction::assert("error2 === 'TypeError'"),

        TestAction::run("try { reader.readAsText({}); } catch(e) { var error3 = e.name; }"),
        TestAction::assert("error3 === 'TypeError'"),

        TestAction::run("try { reader.readAsText(null); } catch(e) { var error4 = e.name; }"),
        TestAction::assert("error4 === 'TypeError'"),
    ]);
}

#[test]
fn file_reader_properties_readonly() {
    run_test_actions([
        TestAction::run("let reader = new FileReader()"),

        // Attempt to modify readonly properties
        TestAction::run("let originalState = reader.readyState"),
        TestAction::run("reader.readyState = 999"),
        TestAction::assert("reader.readyState === originalState"),

        TestAction::run("let originalResult = reader.result"),
        TestAction::run("reader.result = 'modified'"),
        TestAction::assert("reader.result === originalResult"),

        TestAction::run("let originalError = reader.error"),
        TestAction::run("reader.error = 'modified'"),
        TestAction::assert("reader.error === originalError"),
    ]);
}

#[test]
fn file_reader_event_handler_types() {
    run_test_actions([
        TestAction::run("let reader = new FileReader()"),

        // Valid function assignment
        TestAction::run("reader.onload = function() {}"),
        TestAction::assert("typeof reader.onload === 'function'"),

        // Arrow function assignment
        TestAction::run("reader.onload = () => {}"),
        TestAction::assert("typeof reader.onload === 'function'"),

        // Null assignment
        TestAction::run("reader.onload = null"),
        TestAction::assert("reader.onload === null"),

        // Undefined assignment
        TestAction::run("reader.onload = undefined"),
        TestAction::assert("reader.onload === null"),

        // Invalid assignments should be ignored
        TestAction::run("reader.onload = 'not a function'"),
        TestAction::assert("reader.onload === null"),

        TestAction::run("reader.onload = 42"),
        TestAction::assert("reader.onload === null"),

        TestAction::run("reader.onload = {}"),
        TestAction::assert("reader.onload === null"),
    ]);
}

#[test]
fn file_reader_empty_file() {
    run_test_actions([
        TestAction::run("let emptyFile = new File([], 'empty.txt')"),
        TestAction::run("let reader = new FileReader()"),

        TestAction::assert("emptyFile.size === 0"),
        TestAction::run("reader.readAsText(emptyFile)"),
        TestAction::assert("reader.readyState === FileReader.LOADING"),
    ]);
}

#[test]
fn file_reader_large_file_simulation() {
    run_test_actions([
        // Create a larger file to test
        TestAction::run("let content = 'a'.repeat(1000)"),
        TestAction::run("let largeFile = new File([content], 'large.txt')"),
        TestAction::run("let reader = new FileReader()"),

        TestAction::assert("largeFile.size === 1000"),
        TestAction::run("reader.readAsText(largeFile)"),
        TestAction::assert("reader.readyState === FileReader.LOADING"),
    ]);
}

#[test]
fn file_reader_unicode_content() {
    run_test_actions([
        TestAction::run("let unicodeFile = new File(['Hello 世界 🌍'], 'unicode.txt', { type: 'text/plain; charset=UTF-8' })"),
        TestAction::run("let reader = new FileReader()"),

        TestAction::run("reader.readAsText(unicodeFile)"),
        TestAction::assert("reader.readyState === FileReader.LOADING"),

        // Test with different encodings
        TestAction::run("let reader2 = new FileReader()"),
        TestAction::run("reader2.readAsText(unicodeFile, 'UTF-8')"),
        TestAction::assert("reader2.readyState === FileReader.LOADING"),
    ]);
}

#[test]
fn file_reader_multiple_instances() {
    run_test_actions([
        TestAction::run("let file = new File(['shared content'], 'shared.txt')"),
        TestAction::run("let reader1 = new FileReader()"),
        TestAction::run("let reader2 = new FileReader()"),

        // Multiple readers should work independently
        TestAction::run("reader1.readAsText(file)"),
        TestAction::run("reader2.readAsDataURL(file)"),

        TestAction::assert("reader1.readyState === FileReader.LOADING"),
        TestAction::assert("reader2.readyState === FileReader.LOADING"),

        // They should be independent objects
        TestAction::assert("reader1 !== reader2"),
        TestAction::assert("reader1.result !== reader2.result"),
    ]);
}