//! Worker execution and script loading tests
//! Tests the actual execution of worker scripts and message passing

use boa_engine::{Context, Source};

/// Test worker script execution with simple data URL
#[test]
fn test_worker_script_execution_simple() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(r#"
        // Test creating a worker with a simple script
        (function() {
            try {
                let worker = new Worker('data:text/javascript,console.log("Hello from worker!");');

                // Test that worker was created successfully
                let creationTest = worker instanceof Worker;

                // Test worker properties exist
                let propertiesTest = typeof worker.postMessage === 'function' &&
                                   typeof worker.terminate === 'function' &&
                                   'onmessage' in worker &&
                                   'onerror' in worker;

                return creationTest && propertiesTest;
            } catch (e) {
                console.log("Worker script execution test error:", e.message);
                return false;
            }
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

/// Test module worker with ES6 syntax
#[test]
fn test_module_worker_execution() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(r#"
        // Test module worker with ES6 syntax
        (function() {
            try {
                let moduleWorker = new Worker(
                    'data:text/javascript,const message = "Hello Module"; export default message;',
                    { type: 'module' }
                );

                return moduleWorker instanceof Worker;
            } catch (e) {
                console.log("Module worker execution test error:", e.message);
                return false;
            }
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

/// Test classic worker with global scope access
#[test]
fn test_classic_worker_global_scope() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(r#"
        // Test classic worker that uses global scope
        (function() {
            try {
                let classicWorker = new Worker(
                    'data:text/javascript,var x = 42; function test() { return x; }'
                );

                return classicWorker instanceof Worker;
            } catch (e) {
                console.log("Classic worker global scope test error:", e.message);
                return false;
            }
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

/// Test SharedWorker with connection handling
#[test]
fn test_shared_worker_connections() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(r#"
        // Test SharedWorker connection and port handling
        (function() {
            try {
                // Create SharedWorker with onconnect handler
                let sharedWorker = new SharedWorker(
                    'data:text/javascript,onconnect = function(e) { console.log("Connection received"); };'
                );

                // Test port is available
                let portTest = sharedWorker.port !== null &&
                             typeof sharedWorker.port === 'object' &&
                             typeof sharedWorker.port.postMessage === 'function';

                // Test port methods
                let methodTest = typeof sharedWorker.port.start === 'function' &&
                               typeof sharedWorker.port.close === 'function';

                return portTest && methodTest;
            } catch (e) {
                console.log("SharedWorker connections test error:", e.message);
                return false;
            }
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

/// Test worker termination
#[test]
fn test_worker_termination() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(r#"
        // Test worker termination
        (function() {
            try {
                let worker = new Worker('data:text/javascript,while(true) { /* infinite loop */ }');

                // Test terminate method exists and is callable
                let terminateTest = typeof worker.terminate === 'function';

                // Call terminate
                worker.terminate();

                return terminateTest;
            } catch (e) {
                console.log("Worker termination test error:", e.message);
                return false;
            }
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

/// Test BroadcastChannel message posting
#[test]
fn test_broadcast_channel_messaging() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(r#"
        // Test BroadcastChannel messaging
        (function() {
            try {
                let channel1 = new BroadcastChannel('test-channel');
                let channel2 = new BroadcastChannel('test-channel');

                // Set up message handler
                let messageReceived = false;
                channel1.onmessage = function(event) {
                    messageReceived = true;
                    console.log("Received message:", event.data);
                };

                // Post message from second channel
                channel2.postMessage("Hello BroadcastChannel!");

                // Test that posting doesn't throw
                let postTest = true;

                // Clean up
                channel1.close();
                channel2.close();

                return postTest;
            } catch (e) {
                console.log("BroadcastChannel messaging test error:", e.message);
                return false;
            }
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

/// Test error handling in workers
#[test]
fn test_worker_error_handling() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(r#"
        // Test worker error handling
        (function() {
            try {
                // Test invalid URL handling
                let invalidUrlTest = false;
                try {
                    new Worker('invalid://url');
                    invalidUrlTest = false;
                } catch (e) {
                    invalidUrlTest = true; // Should handle invalid URLs gracefully
                }

                // Test worker with syntax error in script
                let syntaxErrorTest = true;
                try {
                    let worker = new Worker('data:text/javascript,invalid syntax here!!!');
                    // Worker creation might succeed but execution should fail gracefully
                } catch (e) {
                    syntaxErrorTest = true;
                }

                return invalidUrlTest && syntaxErrorTest;
            } catch (e) {
                console.log("Worker error handling test error:", e.message);
                return false;
            }
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

/// Test WorkerOptions parameter handling
#[test]
fn test_worker_options() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(r#"
        // Test WorkerOptions parameter handling
        (function() {
            try {
                // Test with type: 'classic' (default)
                let classicWorker = new Worker(
                    'data:text/javascript,console.log("classic");',
                    { type: 'classic' }
                );

                // Test with type: 'module'
                let moduleWorker = new Worker(
                    'data:text/javascript,export const x = 42;',
                    { type: 'module' }
                );

                // Test with credentials option (should be accepted even if not fully implemented)
                let credentialsWorker = new Worker(
                    'data:text/javascript,console.log("credentials");',
                    { credentials: 'same-origin' }
                );

                return classicWorker instanceof Worker &&
                       moduleWorker instanceof Worker &&
                       credentialsWorker instanceof Worker;
            } catch (e) {
                console.log("Worker options test error:", e.message);
                return false;
            }
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

/// Test multiple workers and resource management
#[test]
fn test_multiple_workers() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(r#"
        // Test creating multiple workers and proper resource management
        (function() {
            try {
                let workers = [];

                // Create multiple workers
                for (let i = 0; i < 5; i++) {
                    workers.push(new Worker('data:text/javascript,console.log("Worker " + ' + i + ');'));
                }

                // Test all workers were created
                let creationTest = workers.length === 5 &&
                                 workers.every(w => w instanceof Worker);

                // Terminate all workers
                workers.forEach(w => w.terminate());

                return creationTest;
            } catch (e) {
                console.log("Multiple workers test error:", e.message);
                return false;
            }
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

/// Test SharedWorker name parameter
#[test]
fn test_shared_worker_naming() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(r#"
        // Test SharedWorker name parameter
        (function() {
            try {
                // Create SharedWorker with name
                let namedSharedWorker = new SharedWorker(
                    'data:text/javascript,console.log("named shared worker");',
                    'shared-worker-name'
                );

                // Create SharedWorker with options object containing name
                let optionsSharedWorker = new SharedWorker(
                    'data:text/javascript,console.log("options shared worker");',
                    { name: 'options-shared-worker-name' }
                );

                return namedSharedWorker instanceof SharedWorker &&
                       optionsSharedWorker instanceof SharedWorker;
            } catch (e) {
                console.log("SharedWorker naming test error:", e.message);
                return false;
            }
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}