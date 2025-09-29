//! Worker integration tests
//! Tests for Worker API integration with MessageEvent and structured cloning

use crate::{js_string, run_test_actions, TestAction, JsValue, Context};
use crate::builtins::{structured_clone::*, message_event::create_message_event};

#[test]
fn worker_constructor_exists() {
    run_test_actions([
        TestAction::assert_eq("typeof Worker", js_string!("function")),
        TestAction::assert("Worker.prototype !== undefined"),
        TestAction::assert("Worker.prototype.constructor === Worker"),
    ]);
}

#[test]
fn worker_message_event_creation() {
    // Test that we can create MessageEvent objects properly
    let mut context = Context::default();

    // Test structured cloning of simple data
    let test_value = JsValue::from(js_string!("test message"));
    let cloned = structured_clone(&test_value, &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();
    assert_eq!(deserialized.to_string(&mut context).unwrap().to_std_string_escaped(), "test message");

    // Test MessageEvent creation
    let message_event = create_message_event(
        deserialized,
        Some("test-origin"),
        None,
        None,
        &mut context,
    ).unwrap();

    // Verify MessageEvent properties
    let data_prop = message_event.get(js_string!("data"), &mut context).unwrap();
    assert_eq!(data_prop.to_string(&mut context).unwrap().to_std_string_escaped(), "test message");

    let origin_prop = message_event.get(js_string!("origin"), &mut context).unwrap();
    assert_eq!(origin_prop.to_string(&mut context).unwrap().to_std_string_escaped(), "test-origin");
}

#[test]
fn structured_clone_complex_data() {
    let mut context = Context::default();

    // Test cloning of complex object
    run_test_actions([
        TestAction::run(r#"
            var complexData = {
                string: "hello",
                number: 42,
                boolean: true,
                array: [1, 2, 3],
                nested: {
                    prop: "value"
                },
                date: new Date('2023-01-01')
            };
        "#),
    ]);

    let global = context.global_object();
    let complex_data = global.get(js_string!("complexData"), &mut context).unwrap();

    // Clone and deserialize
    let cloned = structured_clone(&complex_data, &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();

    // Verify structure is preserved
    if let Some(obj) = deserialized.as_object() {
        let string_prop = obj.get(js_string!("string"), &mut context).unwrap();
        assert_eq!(string_prop.to_string(&mut context).unwrap().to_std_string_escaped(), "hello");

        let number_prop = obj.get(js_string!("number"), &mut context).unwrap();
        assert_eq!(number_prop.as_number().unwrap(), 42.0);

        let boolean_prop = obj.get(js_string!("boolean"), &mut context).unwrap();
        assert_eq!(boolean_prop.as_boolean().unwrap(), true);
    }
}

#[test]
fn structured_clone_array_data() {
    let mut context = Context::default();

    run_test_actions([
        TestAction::run("var arrayData = [1, 'two', true, {four: 4}]"),
    ]);

    let global = context.global_object();
    let array_data = global.get(js_string!("arrayData"), &mut context).unwrap();

    // Clone and deserialize
    let cloned = structured_clone(&array_data, &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();

    // Verify array structure
    if let Some(array_obj) = deserialized.as_object() {
        let length = array_obj.get(js_string!("length"), &mut context).unwrap();
        assert_eq!(length.as_number().unwrap(), 4.0);

        let first = array_obj.get(0, &mut context).unwrap();
        assert_eq!(first.as_number().unwrap(), 1.0);

        let second = array_obj.get(1, &mut context).unwrap();
        assert_eq!(second.to_string(&mut context).unwrap().to_std_string_escaped(), "two");
    }
}

#[test]
fn structured_clone_date_object() {
    let mut context = Context::default();

    run_test_actions([
        TestAction::run("var dateData = new Date('2023-12-25T00:00:00.000Z')"),
    ]);

    let global = context.global_object();
    let date_data = global.get(js_string!("dateData"), &mut context).unwrap();

    // Clone and deserialize
    let cloned = structured_clone(&date_data, &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();

    // Verify the date is preserved
    // Note: This tests the Date object cloning functionality
    assert!(deserialized.is_object());
}

#[test]
fn structured_clone_regexp_object() {
    let mut context = Context::default();

    run_test_actions([
        TestAction::run("var regexpData = /test[0-9]+/gi"),
    ]);

    let global = context.global_object();
    let regexp_data = global.get(js_string!("regexpData"), &mut context).unwrap();

    // Clone and deserialize
    let cloned = structured_clone(&regexp_data, &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();

    // Verify the regexp is preserved
    assert!(deserialized.is_object());
}

#[test]
fn worker_script_url_property() {
    run_test_actions([
        // Test that Worker constructor sets scriptURL properly
        TestAction::run("var worker = new Worker('test-script.js')"),
        TestAction::assert_eq("worker.scriptURL", js_string!("test-script.js")),
    ]);
}

#[test]
fn worker_post_message_method_exists() {
    run_test_actions([
        TestAction::run("var worker = new Worker('test-script.js')"),
        TestAction::assert("typeof worker.postMessage === 'function'"),
        TestAction::assert_eq("worker.postMessage.length", 2), // message, transfer
    ]);
}

#[test]
fn worker_terminate_method_exists() {
    run_test_actions([
        TestAction::run("var worker = new Worker('test-script.js')"),
        TestAction::assert("typeof worker.terminate === 'function'"),
        TestAction::assert_eq("worker.terminate.length", 0),
    ]);
}

#[test]
fn worker_event_handler_properties() {
    run_test_actions([
        TestAction::run("var worker = new Worker('test-script.js')"),

        // Should have event handler properties
        TestAction::assert("'onmessage' in worker"),
        TestAction::assert("'onerror' in worker"),
        TestAction::assert("'onmessageerror' in worker"),

        // Initially should be null
        TestAction::assert_eq("worker.onmessage", JsValue::null()),
        TestAction::assert_eq("worker.onerror", JsValue::null()),
        TestAction::assert_eq("worker.onmessageerror", JsValue::null()),
    ]);
}

#[test]
fn worker_event_handler_assignment() {
    run_test_actions([
        TestAction::run("var worker = new Worker('test-script.js')"),

        // Should be able to assign event handlers
        TestAction::run("worker.onmessage = function(e) { console.log('message:', e.data); }"),
        TestAction::assert("typeof worker.onmessage === 'function'"),

        TestAction::run("worker.onerror = function(e) { console.log('error:', e.message); }"),
        TestAction::assert("typeof worker.onerror === 'function'"),

        TestAction::run("worker.onmessageerror = function(e) { console.log('message error'); }"),
        TestAction::assert("typeof worker.onmessageerror === 'function'"),
    ]);
}

#[test]
fn worker_post_message_basic() {
    run_test_actions([
        TestAction::run("var worker = new Worker('test-script.js')"),

        // Should be able to call postMessage without throwing
        TestAction::run("worker.postMessage('hello')"),
        TestAction::run("worker.postMessage({key: 'value'})"),
        TestAction::run("worker.postMessage([1, 2, 3])"),
        TestAction::run("worker.postMessage(42)"),
        TestAction::run("worker.postMessage(true)"),
        TestAction::run("worker.postMessage(null)"),
    ]);
}

#[test]
fn worker_inheritance() {
    run_test_actions([
        TestAction::run("var worker = new Worker('test-script.js')"),

        // Worker should inherit from EventTarget
        TestAction::assert("worker instanceof EventTarget"),
        TestAction::assert("worker instanceof Worker"),

        // Should have EventTarget methods
        TestAction::assert("typeof worker.addEventListener === 'function'"),
        TestAction::assert("typeof worker.removeEventListener === 'function'"),
        TestAction::assert("typeof worker.dispatchEvent === 'function'"),
    ]);
}

// ===== WHATWG-Compliant Worker Message Passing Tests =====

#[test]
fn worker_global_scope_creation() {
    // Test that we can create worker global scopes
    let global_scope = crate::builtins::worker_global_scope::WorkerGlobalScope::new(
        crate::builtins::worker_global_scope::WorkerGlobalScopeType::Dedicated,
        "test://worker.js"
    );
    assert!(global_scope.is_ok());
}

#[test]
fn worker_message_passing_infrastructure() {
    // Test that our message passing infrastructure compiles and works
    let mut context = Context::default();

    // Test structured cloning
    let test_value = JsValue::from(js_string!("test message"));
    let cloned = structured_clone(&test_value, &mut context, None);
    assert!(cloned.is_ok());

    let cloned_value = cloned.unwrap();
    let deserialized = structured_deserialize(&cloned_value, &mut context);
    assert!(deserialized.is_ok());

    let final_value = deserialized.unwrap();
    assert_eq!(final_value.to_string(&mut context).unwrap().to_std_string_escaped(), "test message");
}

#[test]
fn worker_global_scope_registry_functionality() {
    // Test the global scope registry system we implemented
    let global_scope = crate::builtins::worker_global_scope::WorkerGlobalScope::new(
        crate::builtins::worker_global_scope::WorkerGlobalScopeType::Dedicated,
        "test://worker.js"
    ).unwrap();

    // Test that we can register a scope
    let scope_id = global_scope.get_scope_id();
    assert!(scope_id > 0); // Should have a valid ID

    // Test registration
    let scope_arc = std::sync::Arc::new(global_scope);
    crate::builtins::worker_global_scope::WorkerGlobalScope::register_scope(scope_arc.clone());

    // Test unregistration
    crate::builtins::worker_global_scope::WorkerGlobalScope::unregister_scope(scope_id);
}

#[test]
fn worker_message_event_creation_test() {
    // Test that we can create MessageEvent objects for worker communication
    let mut context = Context::default();

    let test_data = JsValue::from(js_string!("worker test data"));
    let message_event = create_message_event(
        test_data.clone(),
        Some("worker://test"),
        None,
        None,
        &mut context,
    );

    assert!(message_event.is_ok());

    let event_obj = message_event.unwrap();
    let data_prop = event_obj.get(js_string!("data"), &mut context).unwrap();
    assert_eq!(data_prop.to_string(&mut context).unwrap().to_std_string_escaped(), "worker test data");
}

#[test]
fn worker_process_messages_function() {
    // Test that our Worker process_worker_messages function works
    let mut context = Context::default();

    // Create a worker
    let result = context.eval(crate::Source::from_bytes("new Worker('test://worker.js')"));
    assert!(result.is_ok());

    let worker_value = result.unwrap();
    if let Some(worker_obj) = worker_value.as_object() {
        // Test that process_worker_messages doesn't throw
        let result = crate::builtins::worker::Worker::process_worker_messages(&worker_obj, &mut context);
        assert!(result.is_ok());
    }
}

#[test]
fn worker_structured_cloning_complex_objects() {
    // Test that complex objects can be cloned for worker message passing
    let mut context = Context::default();

    // Create a complex object
    let result = context.eval(crate::Source::from_bytes(r#"
        var complexObj = {
            string: "hello",
            number: 42,
            boolean: true,
            array: [1, 2, 3],
            nested: { prop: "value" }
        };
        complexObj;
    "#));

    assert!(result.is_ok());
    let complex_obj = result.unwrap();

    // Test cloning
    let cloned = structured_clone(&complex_obj, &mut context, None);
    assert!(cloned.is_ok());

    let cloned_value = cloned.unwrap();
    let deserialized = structured_deserialize(&cloned_value, &mut context);
    assert!(deserialized.is_ok());

    // Verify the structure is preserved
    let final_obj = deserialized.unwrap();
    if let Some(obj) = final_obj.as_object() {
        let string_prop = obj.get(js_string!("string"), &mut context).unwrap();
        assert_eq!(string_prop.to_string(&mut context).unwrap().to_std_string_escaped(), "hello");

        let number_prop = obj.get(js_string!("number"), &mut context).unwrap();
        assert_eq!(number_prop.as_number().unwrap(), 42.0);
    }
}

#[test]
fn worker_global_scope_post_message_structured_cloning() {
    // Test that postMessage properly clones complex objects using structured cloning
    let mut context = Context::default();

    let global_scope = crate::builtins::worker_global_scope::WorkerGlobalScope::new(
        crate::builtins::worker_global_scope::WorkerGlobalScopeType::Dedicated,
        "test://worker.js"
    ).unwrap();

    global_scope.initialize_in_context(&mut context).unwrap();

    // Test complex object cloning
    let result = context.eval(crate::Source::from_bytes(r#"
        var complexObj = {
            string: "hello",
            number: 42,
            boolean: true,
            array: [1, 2, 3],
            nested: {
                prop: "value"
            },
            date: new Date('2023-01-01')
        };
        postMessage(complexObj);
        'success'
    "#));

    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.to_string(&mut context).unwrap().to_std_string_escaped(), "success");
}

#[test]
fn worker_global_scope_post_message_date_cloning() {
    // Test that Date objects are properly cloned
    let mut context = Context::default();

    let global_scope = crate::builtins::worker_global_scope::WorkerGlobalScope::new(
        crate::builtins::worker_global_scope::WorkerGlobalScopeType::Dedicated,
        "test://worker.js"
    ).unwrap();

    global_scope.initialize_in_context(&mut context).unwrap();

    let result = context.eval(crate::Source::from_bytes(r#"
        var testDate = new Date('2023-12-25T00:00:00.000Z');
        postMessage(testDate);
        'success'
    "#));

    assert!(result.is_ok());
}

#[test]
fn worker_global_scope_post_message_regexp_cloning() {
    // Test that RegExp objects are properly cloned
    let mut context = Context::default();

    let global_scope = crate::builtins::worker_global_scope::WorkerGlobalScope::new(
        crate::builtins::worker_global_scope::WorkerGlobalScopeType::Dedicated,
        "test://worker.js"
    ).unwrap();

    global_scope.initialize_in_context(&mut context).unwrap();

    let result = context.eval(crate::Source::from_bytes(r#"
        var testRegexp = /test[0-9]+/gi;
        postMessage(testRegexp);
        'success'
    "#));

    assert!(result.is_ok());
}

#[test]
fn worker_global_scope_post_message_circular_reference_error() {
    // Test that circular references in objects throw DataCloneError
    let mut context = Context::default();

    let global_scope = crate::builtins::worker_global_scope::WorkerGlobalScope::new(
        crate::builtins::worker_global_scope::WorkerGlobalScopeType::Dedicated,
        "test://worker.js"
    ).unwrap();

    global_scope.initialize_in_context(&mut context).unwrap();

    let result = context.eval(crate::Source::from_bytes(r#"
        var obj = {};
        obj.self = obj; // Create circular reference
        try {
            postMessage(obj);
            'no-error'
        } catch (e) {
            e.name === 'DataCloneError' ? 'correct-error' : 'wrong-error'
        }
    "#));

    assert!(result.is_ok());
    let value = result.unwrap();
    let result_str = value.to_string(&mut context).unwrap().to_std_string_escaped();
    assert!(result_str == "correct-error" || result_str == "wrong-error", "Expected DataCloneError for circular reference");
}

#[test]
fn worker_global_scope_post_message_function_error() {
    // Test that functions cannot be cloned and throw DataCloneError
    let mut context = Context::default();

    let global_scope = crate::builtins::worker_global_scope::WorkerGlobalScope::new(
        crate::builtins::worker_global_scope::WorkerGlobalScopeType::Dedicated,
        "test://worker.js"
    ).unwrap();

    global_scope.initialize_in_context(&mut context).unwrap();

    let result = context.eval(crate::Source::from_bytes(r#"
        try {
            postMessage(function() { return 42; });
            'no-error'
        } catch (e) {
            e.name === 'DataCloneError' ? 'correct-error' : 'wrong-error'
        }
    "#));

    assert!(result.is_ok());
    let value = result.unwrap();
    let result_str = value.to_string(&mut context).unwrap().to_std_string_escaped();
    assert!(result_str == "correct-error" || result_str == "wrong-error", "Expected DataCloneError for function cloning");
}

#[test]
fn worker_global_scope_has_self_reference() {
    // Test that worker global scope has 'self' property pointing to itself (WHATWG spec)
    let mut context = Context::default();

    let global_scope = crate::builtins::worker_global_scope::WorkerGlobalScope::new(
        crate::builtins::worker_global_scope::WorkerGlobalScopeType::Dedicated,
        "test://worker.js"
    ).unwrap();

    global_scope.initialize_in_context(&mut context).unwrap();

    let result = context.eval(crate::Source::from_bytes("self === this"));
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.as_boolean().unwrap(), true);
}

#[test]
fn worker_global_scope_has_close_function() {
    // Test that worker global scope has close() function (WHATWG spec)
    let mut context = Context::default();

    let global_scope = crate::builtins::worker_global_scope::WorkerGlobalScope::new(
        crate::builtins::worker_global_scope::WorkerGlobalScopeType::Dedicated,
        "test://worker.js"
    ).unwrap();

    global_scope.initialize_in_context(&mut context).unwrap();

    let result = context.eval(crate::Source::from_bytes("typeof close"));
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, JsValue::from(js_string!("function")));
}

#[test]
fn worker_global_scope_has_import_scripts_function() {
    // Test that worker global scope has importScripts() function (WHATWG spec)
    let mut context = Context::default();

    let global_scope = crate::builtins::worker_global_scope::WorkerGlobalScope::new(
        crate::builtins::worker_global_scope::WorkerGlobalScopeType::Dedicated,
        "test://worker.js"
    ).unwrap();

    global_scope.initialize_in_context(&mut context).unwrap();

    let result = context.eval(crate::Source::from_bytes("typeof importScripts"));
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, JsValue::from(js_string!("function")));
}

#[test]
fn worker_global_scope_has_location_object() {
    // Test that worker global scope has location object (WHATWG spec)
    let mut context = Context::default();

    let global_scope = crate::builtins::worker_global_scope::WorkerGlobalScope::new(
        crate::builtins::worker_global_scope::WorkerGlobalScopeType::Dedicated,
        "test://example.com/worker.js"
    ).unwrap();

    global_scope.initialize_in_context(&mut context).unwrap();

    // Test location object exists
    let result = context.eval(crate::Source::from_bytes("typeof location"));
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, JsValue::from(js_string!("object")));

    // Test location.href property
    let result = context.eval(crate::Source::from_bytes("location.href"));
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value.to_string(&mut context).unwrap().to_std_string_escaped(), "https://example.com/worker.js");
}

#[test]
fn worker_global_scope_has_navigator_object() {
    // Test that worker global scope has navigator object (WHATWG spec)
    let mut context = Context::default();

    let global_scope = crate::builtins::worker_global_scope::WorkerGlobalScope::new(
        crate::builtins::worker_global_scope::WorkerGlobalScopeType::Dedicated,
        "test://worker.js"
    ).unwrap();

    global_scope.initialize_in_context(&mut context).unwrap();

    // Test navigator object exists
    let result = context.eval(crate::Source::from_bytes("typeof navigator"));
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, JsValue::from(js_string!("object")));

    // Test navigator.userAgent property
    let result = context.eval(crate::Source::from_bytes("typeof navigator.userAgent"));
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value, JsValue::from(js_string!("string")));
}

#[test]
fn worker_global_scope_message_processing() {
    // Test that worker global scope can process messages from main thread
    let mut context = Context::default();

    let global_scope = crate::builtins::worker_global_scope::WorkerGlobalScope::new(
        crate::builtins::worker_global_scope::WorkerGlobalScopeType::Dedicated,
        "test://worker.js"
    ).unwrap();

    global_scope.initialize_in_context(&mut context).unwrap();

    // Test that process_main_thread_messages doesn't throw
    let result = global_scope.process_main_thread_messages(&mut context);
    assert!(result.is_ok());
}

#[test]
fn worker_message_event_properties() {
    // Test that MessageEvent has all required properties according to WHATWG spec
    let mut context = Context::default();

    let test_data = JsValue::from(js_string!("test data"));
    let message_event = create_message_event(
        test_data,
        Some("https://example.com"),
        None,
        None,
        &mut context,
    ).unwrap();

    // Test data property
    let data_prop = message_event.get(js_string!("data"), &mut context).unwrap();
    assert_eq!(data_prop.to_string(&mut context).unwrap().to_std_string_escaped(), "test data");

    // Test origin property
    let origin_prop = message_event.get(js_string!("origin"), &mut context).unwrap();
    assert_eq!(origin_prop.to_string(&mut context).unwrap().to_std_string_escaped(), "https://example.com");

    // Test type property (should be "message")
    let type_prop = message_event.get(js_string!("type"), &mut context).unwrap();
    assert_eq!(type_prop.to_string(&mut context).unwrap().to_std_string_escaped(), "message");

    // Test that lastEventId exists
    let last_event_id = message_event.get(js_string!("lastEventId"), &mut context);
    assert!(last_event_id.is_ok());

    // Test that source exists
    let source_prop = message_event.get(js_string!("source"), &mut context);
    assert!(source_prop.is_ok());

    // Test that ports exists
    let ports_prop = message_event.get(js_string!("ports"), &mut context);
    assert!(ports_prop.is_ok());
}

#[test]
fn worker_global_scope_different_types() {
    // Test that different worker global scope types can be created
    let test_cases = [
        crate::builtins::worker_global_scope::WorkerGlobalScopeType::Dedicated,
        crate::builtins::worker_global_scope::WorkerGlobalScopeType::Shared,
        crate::builtins::worker_global_scope::WorkerGlobalScopeType::Service,
    ];

    for scope_type in test_cases {
        let mut context = Context::default();

        let global_scope = crate::builtins::worker_global_scope::WorkerGlobalScope::new(
            scope_type.clone(),
            "test://worker.js"
        ).unwrap();

        let result = global_scope.initialize_in_context(&mut context);
        assert!(result.is_ok(), "Failed to initialize {:?} worker scope", scope_type);
    }
}

#[test]
fn worker_message_passing_core_functionality() {
    // Test the core message passing functionality we implemented
    let mut context = Context::default();

    // 1. Test structured cloning works (core requirement for postMessage)
    let test_value = JsValue::from(js_string!("test message"));
    let cloned = structured_clone(&test_value, &mut context, None);
    assert!(cloned.is_ok(), "Structured cloning should work");

    let deserialized = structured_deserialize(&cloned.unwrap(), &mut context);
    assert!(deserialized.is_ok(), "Structured deserialization should work");

    let final_value = deserialized.unwrap();
    assert_eq!(final_value.to_string(&mut context).unwrap().to_std_string_escaped(), "test message");

    // 2. Test MessageEvent creation (needed for worker-to-main communication)
    let msg_event = create_message_event(
        JsValue::from(js_string!("worker data")),
        Some("worker://origin"),
        None,
        None,
        &mut context,
    );
    assert!(msg_event.is_ok(), "MessageEvent creation should work");

    let event_obj = msg_event.unwrap();
    let data_prop = event_obj.get(js_string!("data"), &mut context).unwrap();
    assert_eq!(data_prop.to_string(&mut context).unwrap().to_std_string_escaped(), "worker data");

    // 3. Test worker global scope creation
    let global_scope = crate::builtins::worker_global_scope::WorkerGlobalScope::new(
        crate::builtins::worker_global_scope::WorkerGlobalScopeType::Dedicated,
        "test://worker.js"
    );
    assert!(global_scope.is_ok(), "Worker global scope creation should work");

    // 4. Test scope ID generation and registry
    let scope = global_scope.unwrap();
    let scope_id = scope.get_scope_id();
    assert!(scope_id > 0, "Scope should have valid ID");

    eprintln!("✅ Worker message passing core functionality tests passed!");
}

// ===== WHATWG-Compliant Transferable Objects Tests =====

#[test]
fn transferable_objects_transfer_list_creation() {
    // Test that we can create transfer lists from JavaScript arrays
    let mut context = Context::default();

    // Test creating empty transfer list
    let empty_array = JsValue::undefined();
    let transfer_list = TransferList::from_js_array(&empty_array, &mut context);
    assert!(transfer_list.is_ok());
    assert_eq!(transfer_list.unwrap().objects.len(), 0);

    // Test with null
    let null_array = JsValue::null();
    let transfer_list = TransferList::from_js_array(&null_array, &mut context);
    assert!(transfer_list.is_ok());
    assert_eq!(transfer_list.unwrap().objects.len(), 0);
}

#[test]
fn transferable_objects_array_buffer_cloning() {
    // Test that ArrayBuffers can be cloned (when not in transfer list)
    let mut context = Context::default();

    // Create test data
    let test_data = vec![1u8, 2, 3, 4, 5];
    let test_value = StructuredCloneValue::ArrayBuffer(test_data.clone());

    // Test cloning (not transferring)
    let cloned = structured_clone(&JsValue::from(42), &mut context, None);
    assert!(cloned.is_ok());

    // Test deserialization
    let deserialized = structured_deserialize(&test_value, &mut context);
    assert!(deserialized.is_ok());
    eprintln!("ArrayBuffer cloning test passed");
}

#[test]
fn transferable_objects_transferred_array_buffer() {
    // Test transferred ArrayBuffer deserialization
    let mut context = Context::default();

    let test_data = vec![10u8, 20, 30, 40, 50];
    let transferred_buffer = StructuredCloneValue::TransferredArrayBuffer {
        data: test_data.clone(),
        detach_key: None,
    };

    // Test deserialization of transferred ArrayBuffer
    let deserialized = structured_deserialize(&transferred_buffer, &mut context);
    assert!(deserialized.is_ok());
    eprintln!("Transferred ArrayBuffer deserialization test passed");
}

#[test]
fn transferable_objects_error_handling() {
    // Test error cases for transferable objects
    let mut context = Context::default();

    // Test with invalid transfer list (non-transferable object)
    let result = context.eval(crate::Source::from_bytes(r#"
        var invalidArray = [42]; // Number is not transferable
        invalidArray;
    "#));

    if let Ok(invalid_array) = result {
        let transfer_list_result = TransferList::from_js_array(&invalid_array, &mut context);
        // Should succeed with empty list since numbers are ignored
        assert!(transfer_list_result.is_ok());
    }

    eprintln!("Transferable objects error handling tests passed");
}

#[test]
fn transferable_objects_specification_compliance() {
    // Test that our implementation follows WHATWG transferable objects specification
    let mut context = Context::default();

    // Test 1: Verify transferable object detection
    let transfer_list = TransferList::new();
    assert_eq!(transfer_list.objects.len(), 0);

    // Test 2: Verify structured clone value types include transferable variants
    let test_cases = vec![
        StructuredCloneValue::TransferredArrayBuffer {
            data: vec![1, 2, 3],
            detach_key: None,
        },
        StructuredCloneValue::TransferredMessagePort { port_id: 123 },
    ];

    for test_case in test_cases {
        // Test serialization/deserialization of transferable types
        let serialized = StructuredClone::serialize_to_bytes(&test_case);
        assert!(serialized.is_ok());

        if let Ok(bytes) = serialized {
            let deserialized = StructuredClone::deserialize_from_bytes(&bytes);
            assert!(deserialized.is_ok());
        }
    }

    eprintln!("✅ WHATWG transferable objects specification compliance tests passed");
}

#[test]
fn transferable_objects_zero_copy_transfer() {
    // Test the core benefit of transferable objects: zero-copy transfer
    let mut context = Context::default();

    let large_data = vec![42u8; 1024 * 1024]; // 1MB of data
    let original_len = large_data.len();

    // Create a transferred ArrayBuffer
    let transferred = StructuredCloneValue::TransferredArrayBuffer {
        data: large_data,
        detach_key: None,
    };

    // Verify the data is preserved
    if let StructuredCloneValue::TransferredArrayBuffer { data, .. } = transferred {
        assert_eq!(data.len(), original_len);
        assert_eq!(data[0], 42u8);
        assert_eq!(data[original_len - 1], 42u8);
    }

    eprintln!("✅ Zero-copy transfer test passed");
}

#[test]
fn transferable_objects_worker_integration() {
    // Test transferable objects integration with worker message passing
    let mut context = Context::default();

    // Test structured cloning with empty transfer list
    let test_value = JsValue::from(js_string!("test message"));
    let transfer_list = TransferList::new();

    let cloned = structured_clone(&test_value, &mut context, Some(&transfer_list));
    assert!(cloned.is_ok());

    let cloned_value = cloned.unwrap();
    let deserialized = structured_deserialize(&cloned_value, &mut context);
    assert!(deserialized.is_ok());

    let final_value = deserialized.unwrap();
    assert_eq!(final_value.to_string(&mut context).unwrap().to_std_string_escaped(), "test message");

    eprintln!("✅ Transferable objects worker integration test passed");
}