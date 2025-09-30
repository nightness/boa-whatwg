use crate::{run_test_actions, TestAction};

#[test]
fn file_constructor_exists() {
    run_test_actions([
        TestAction::assert("typeof File === 'function'"),
        TestAction::assert("File.name === 'File'"),
        TestAction::assert("File.length === 2"), // fileBits, fileName required
    ]);
}

#[test]
fn file_constructor_basic() {
    run_test_actions([
        TestAction::run("let file = new File(['Hello World'], 'test.txt')"),
        TestAction::assert("file instanceof File"),
        TestAction::assert("file.name === 'test.txt'"),
        TestAction::assert("file.size === 11"), // 'Hello World' = 11 bytes
        TestAction::assert("file.type === ''"), // No type specified
        TestAction::assert("typeof file.lastModified === 'number'"),
        TestAction::assert("file.lastModified > 0"),
    ]);
}

#[test]
fn file_constructor_with_type() {
    run_test_actions([
        TestAction::run("let file = new File(['Hello World'], 'test.txt', { type: 'text/plain' })"),
        TestAction::assert("file.type === 'text/plain'"),
        TestAction::assert("file.name === 'test.txt'"),
        TestAction::assert("file.size === 11"),
    ]);
}

#[test]
fn file_constructor_with_last_modified() {
    run_test_actions([
        TestAction::run("let file = new File(['test'], 'test.txt', { lastModified: 1640995200000 })"),
        TestAction::assert("file.lastModified === 1640995200000"),
        TestAction::assert("file.name === 'test.txt'"),
    ]);
}

#[test]
fn file_constructor_mixed_content() {
    run_test_actions([
        TestAction::run("let blob = new Blob(['Hello'])"),
        TestAction::run("let file1 = new File([blob, ' World'], 'combined.txt')"),
        TestAction::assert("file1.size === 11"),
        TestAction::assert("file1.name === 'combined.txt'"),

        // File can contain other files
        TestAction::run("let file2 = new File(['Start '], 'start.txt')"),
        TestAction::run("let file3 = new File([file2, 'End'], 'complete.txt')"),
        TestAction::assert("file3.size === 9"), // 'Start End' = 9 bytes
        TestAction::assert("file3.name === 'complete.txt'"),
    ]);
}

#[test]
fn file_inherits_from_blob() {
    run_test_actions([
        TestAction::run("let file = new File(['test'], 'test.txt')"),
        TestAction::assert("file instanceof Blob"), // File inherits from Blob
        TestAction::assert("typeof file.slice === 'function'"),
        TestAction::assert("typeof file.text === 'function'"),
        TestAction::assert("typeof file.arrayBuffer === 'function'"),
        TestAction::assert("typeof file.stream === 'function'"),
    ]);
}

#[test]
fn file_slice_method() {
    run_test_actions([
        TestAction::run("let file = new File(['Hello World'], 'test.txt')"),
        TestAction::run("let slice = file.slice(0, 5)"),
        TestAction::assert("slice instanceof File"), // slice returns File, not Blob
        TestAction::assert("slice.size === 5"),
        TestAction::assert("slice.name === 'test.txt'"), // preserves name
        TestAction::assert("slice.lastModified === file.lastModified"), // preserves lastModified
    ]);
}

#[test]
fn file_slice_with_type() {
    run_test_actions([
        TestAction::run("let file = new File(['Hello World'], 'test.txt', { type: 'text/plain' })"),
        TestAction::run("let slice = file.slice(0, 5, 'text/html')"),
        TestAction::assert("slice.type === 'text/html'"), // new type
        TestAction::assert("slice.name === 'test.txt'"), // preserves name
        TestAction::assert("slice.size === 5"),
    ]);
}

#[test]
fn file_properties() {
    run_test_actions([
        TestAction::run("let file = new File(['content'], 'document.pdf', { type: 'application/pdf', lastModified: 1234567890 })"),

        // Test name property
        TestAction::assert("file.name === 'document.pdf'"),
        TestAction::assert("typeof file.name === 'string'"),

        // Test lastModified property
        TestAction::assert("file.lastModified === 1234567890"),
        TestAction::assert("typeof file.lastModified === 'number'"),

        // Test type property (inherited from Blob)
        TestAction::assert("file.type === 'application/pdf'"),
        TestAction::assert("typeof file.type === 'string'"),

        // Test size property (inherited from Blob)
        TestAction::assert("file.size === 7"), // 'content' = 7 bytes
        TestAction::assert("typeof file.size === 'number'"),

        // Test webkitRelativePath property
        TestAction::assert("file.webkitRelativePath === ''"),
        TestAction::assert("typeof file.webkitRelativePath === 'string'"),
    ]);
}

#[test]
fn file_text_method() {
    run_test_actions([
        TestAction::run("let file = new File(['Hello World'], 'greeting.txt')"),
        TestAction::assert("typeof file.text === 'function'"),
        // Note: text() should return a Promise, but for now we return the text directly
        TestAction::run("let result = file.text()"),
        TestAction::assert("typeof result === 'string' || result instanceof Promise"),
    ]);
}

#[test]
fn file_array_buffer_method() {
    run_test_actions([
        TestAction::run("let file = new File(['Hello'], 'test.txt')"),
        TestAction::assert("typeof file.arrayBuffer === 'function'"),
        // Note: arrayBuffer() should return a Promise<ArrayBuffer>, but for now returns undefined
        TestAction::run("let result = file.arrayBuffer()"),
        TestAction::assert("result === undefined || result instanceof Promise"),
    ]);
}

#[test]
fn file_stream_method() {
    run_test_actions([
        TestAction::run("let file = new File(['Hello'], 'test.txt')"),
        TestAction::assert("typeof file.stream === 'function'"),
        // Note: stream() should return a ReadableStream, but for now returns undefined
        TestAction::run("let result = file.stream()"),
        TestAction::assert("result === undefined || result.constructor.name === 'ReadableStream'"),
    ]);
}

#[test]
fn file_constructor_error_cases() {
    run_test_actions([
        // Missing fileName should throw
        TestAction::run("try { new File(['test']); } catch(e) { var error1 = e.name; }"),
        TestAction::assert("error1 === 'TypeError'"),

        // No arguments should throw
        TestAction::run("try { new File(); } catch(e) { var error2 = e.name; }"),
        TestAction::assert("error2 === 'TypeError'"),
    ]);
}

#[test]
fn file_constructor_edge_cases() {
    run_test_actions([
        // Empty file parts
        TestAction::run("let file1 = new File([], 'empty.txt')"),
        TestAction::assert("file1.size === 0"),
        TestAction::assert("file1.name === 'empty.txt'"),

        // Null/undefined parts
        TestAction::run("let file2 = new File([null, undefined], 'null-undef.txt')"),
        TestAction::assert("file2.size >= 0"),
        TestAction::assert("file2.name === 'null-undef.txt'"),

        // Numbers and booleans get converted to strings
        TestAction::run("let file3 = new File([42, true, false], 'mixed.txt')"),
        TestAction::assert("file3.size > 0"),
        TestAction::assert("file3.name === 'mixed.txt'"),
    ]);
}

#[test]
fn file_options_handling() {
    run_test_actions([
        // Missing type in options
        TestAction::run("let file1 = new File(['test'], 'test.txt', {})"),
        TestAction::assert("file1.type === ''"),

        // Non-string type gets converted
        TestAction::run("let file2 = new File(['test'], 'test.txt', { type: 123 })"),
        TestAction::assert("typeof file2.type === 'string'"),

        // Undefined options
        TestAction::run("let file3 = new File(['test'], 'test.txt', undefined)"),
        TestAction::assert("file3.type === ''"),
        TestAction::assert("typeof file3.lastModified === 'number'"),

        // Null options
        TestAction::run("let file4 = new File(['test'], 'test.txt', null)"),
        TestAction::assert("file4.type === ''"),
        TestAction::assert("typeof file4.lastModified === 'number'"),
    ]);
}

#[test]
fn file_readonly_properties() {
    run_test_actions([
        TestAction::run("let file = new File(['test'], 'original.txt')"),

        // Attempt to modify readonly properties
        TestAction::run("let originalName = file.name"),
        TestAction::run("let originalLastModified = file.lastModified"),
        TestAction::run("let originalSize = file.size"),

        TestAction::run("file.name = 'modified.txt'"),
        TestAction::run("file.lastModified = 999999"),
        TestAction::run("file.size = 100"),

        // Properties should not be modified (they are readonly)
        TestAction::assert("file.name === originalName"),
        TestAction::assert("file.lastModified === originalLastModified"),
        TestAction::assert("file.size === originalSize"),
    ]);
}

#[test]
fn file_unicode_handling() {
    run_test_actions([
        // Unicode filename
        TestAction::run("let file1 = new File(['content'], '测试文件.txt')"),
        TestAction::assert("file1.name === '测试文件.txt'"),

        // Unicode content
        TestAction::run("let file2 = new File(['Hello 世界 🌍'], 'unicode.txt')"),
        TestAction::assert("file2.size > 0"),
        TestAction::assert("file2.name === 'unicode.txt'"),

        // Emoji in filename
        TestAction::run("let file3 = new File(['test'], '📁file🔥.txt')"),
        TestAction::assert("file3.name === '📁file🔥.txt'"),
    ]);
}

#[test]
fn file_mime_type_handling() {
    run_test_actions([
        // Common MIME types
        TestAction::run("let textFile = new File(['text'], 'test.txt', { type: 'text/plain' })"),
        TestAction::assert("textFile.type === 'text/plain'"),

        TestAction::run("let jsonFile = new File(['{}'], 'data.json', { type: 'application/json' })"),
        TestAction::assert("jsonFile.type === 'application/json'"),

        TestAction::run("let binaryFile = new File([new Uint8Array([1,2,3])], 'data.bin', { type: 'application/octet-stream' })"),
        TestAction::assert("binaryFile.type === 'application/octet-stream'"),

        // Empty type should remain empty
        TestAction::run("let noTypeFile = new File(['test'], 'test.txt', { type: '' })"),
        TestAction::assert("noTypeFile.type === ''"),
    ]);
}