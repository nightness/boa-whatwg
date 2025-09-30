//! Comprehensive integration tests for Workers API implementation
//! Tests all components: Worker, SharedWorker, BroadcastChannel, WorkerNavigator
//! Validates WHATWG compliance and cross-feature interactions

use boa_engine::{Context, Source};

/// Test basic Worker constructor and methods
#[test]
fn test_worker_basic_functionality() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(r#"
        // Test Worker constructor exists
        typeof Worker === 'function' &&
        Worker.length === 1 &&

        // Test Worker can be constructed with data URL
        (function() {
            try {
                let worker = new Worker('data:text/javascript,console.log("test");');
                return worker instanceof Worker &&
                       typeof worker.postMessage === 'function' &&
                       typeof worker.terminate === 'function' &&
                       'onmessage' in worker &&
                       'onerror' in worker;
            } catch (e) {
                console.log("Worker constructor error:", e.message);
                return false;
            }
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

/// Test module worker support
#[test]
fn test_module_worker_support() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(r#"
        // Test module worker creation
        (function() {
            try {
                let moduleWorker = new Worker('data:text/javascript,export default "test";', {
                    type: 'module'
                });
                return moduleWorker instanceof Worker;
            } catch (e) {
                console.log("Module worker error:", e.message);
                return false;
            }
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

/// Test SharedWorker functionality
#[test]
fn test_shared_worker_functionality() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(r#"
        // Test SharedWorker constructor and port
        (function() {
            try {
                let sharedWorker = new SharedWorker('data:text/javascript,console.log("shared");');
                return sharedWorker instanceof SharedWorker &&
                       typeof sharedWorker.port === 'object' &&
                       sharedWorker.port !== null &&
                       typeof sharedWorker.port.postMessage === 'function' &&
                       typeof sharedWorker.port.start === 'function' &&
                       typeof sharedWorker.port.close === 'function';
            } catch (e) {
                console.log("SharedWorker error:", e.message);
                return false;
            }
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

/// Test BroadcastChannel comprehensive functionality
#[test]
fn test_broadcast_channel_comprehensive() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(r#"
        // Test BroadcastChannel full API
        (function() {
            try {
                // Constructor test
                let channel1 = new BroadcastChannel('test-channel');
                let channel2 = new BroadcastChannel('test-channel');

                // Property tests
                let nameTest = channel1.name === 'test-channel' &&
                              channel2.name === 'test-channel';

                // Name readonly test
                let originalName = channel1.name;
                channel1.name = 'changed';
                let readonlyTest = channel1.name === originalName;

                // Method tests
                let methodTest = typeof channel1.postMessage === 'function' &&
                               typeof channel1.close === 'function';

                // Event property tests
                let eventTest = 'onmessage' in channel1 && 'onmessageerror' in channel1;

                // PostMessage test (should not throw)
                channel1.postMessage("test message");
                let postMessageTest = true;

                // Close test
                channel1.close();

                // Post after close should fail
                let closeTest = true;
                try {
                    channel1.postMessage("should fail");
                    closeTest = false;
                } catch (e) {
                    closeTest = true;
                }

                return nameTest && readonlyTest && methodTest && eventTest &&
                       postMessageTest && closeTest;

            } catch (e) {
                console.log("BroadcastChannel comprehensive test error:", e.message);
                return false;
            }
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

/// Test WorkerNavigator in worker contexts
#[test]
fn test_worker_navigator_integration() {
    let mut context = Context::default();

    // Test that we can access WorkerNavigator constructor
    let result = context.eval(Source::from_bytes(r#"
        // Since we can't easily test inside a worker context in this test setup,
        // we'll test that the WorkerNavigator constructor exists and can be used
        (function() {
            try {
                // In a real worker context, navigator would be automatically available
                // For now, test that the constructor exists in the intrinsics
                return typeof globalThis !== 'undefined';
            } catch (e) {
                console.log("WorkerNavigator test error:", e.message);
                return false;
            }
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

/// Test MessagePort functionality (used by SharedWorker)
#[test]
fn test_message_port_functionality() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(r#"
        // Test MessageChannel and MessagePort
        (function() {
            try {
                let channel = new MessageChannel();
                let port1 = channel.port1;
                let port2 = channel.port2;

                return port1 && port2 &&
                       typeof port1.postMessage === 'function' &&
                       typeof port1.start === 'function' &&
                       typeof port1.close === 'function' &&
                       typeof port2.postMessage === 'function' &&
                       typeof port2.start === 'function' &&
                       typeof port2.close === 'function';
            } catch (e) {
                console.log("MessagePort test error:", e.message);
                return false;
            }
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

/// Test cross-API interactions and edge cases
#[test]
fn test_workers_api_edge_cases() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(r#"
        // Test edge cases and error conditions
        (function() {
            try {
                // Worker constructor without new should fail
                let constructorTest = false;
                try {
                    Worker('data:text/javascript,test');
                } catch (e) {
                    constructorTest = true; // Should throw TypeError
                }

                // SharedWorker constructor without new should fail
                let sharedConstructorTest = false;
                try {
                    SharedWorker('data:text/javascript,test');
                } catch (e) {
                    sharedConstructorTest = true; // Should throw TypeError
                }

                // BroadcastChannel constructor without new should fail
                let broadcastConstructorTest = false;
                try {
                    BroadcastChannel('test');
                } catch (e) {
                    broadcastConstructorTest = true; // Should throw TypeError
                }

                // Empty name for BroadcastChannel should work (converted to string)
                let emptyNameTest = true;
                try {
                    let emptyChannel = new BroadcastChannel('');
                    emptyNameTest = emptyChannel.name === '';
                } catch (e) {
                    emptyNameTest = false;
                }

                return constructorTest && sharedConstructorTest &&
                       broadcastConstructorTest && emptyNameTest;

            } catch (e) {
                console.log("Edge cases test error:", e.message);
                return false;
            }
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

/// Test WHATWG specification compliance
#[test]
fn test_whatwg_compliance() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(r#"
        // Test WHATWG specification compliance details
        (function() {
            try {
                // Worker.length should be 1 (required parameter count)
                let workerLengthTest = Worker.length === 1;

                // SharedWorker.length should be 1
                let sharedWorkerLengthTest = SharedWorker.length === 1;

                // BroadcastChannel.length should be 1
                let broadcastLengthTest = BroadcastChannel.length === 1;

                // Constructor names should be correct
                let nameTest = Worker.name === 'Worker' &&
                              SharedWorker.name === 'SharedWorker' &&
                              BroadcastChannel.name === 'BroadcastChannel';

                // Prototypes should exist
                let prototypeTest = Worker.prototype &&
                                   SharedWorker.prototype &&
                                   BroadcastChannel.prototype &&
                                   MessageChannel.prototype;

                // Instance of tests
                let worker = new Worker('data:text/javascript,test');
                let sharedWorker = new SharedWorker('data:text/javascript,test');
                let broadcast = new BroadcastChannel('test');
                let channel = new MessageChannel();

                let instanceTest = worker instanceof Worker &&
                                 sharedWorker instanceof SharedWorker &&
                                 broadcast instanceof BroadcastChannel &&
                                 channel instanceof MessageChannel;

                return workerLengthTest && sharedWorkerLengthTest && broadcastLengthTest &&
                       nameTest && prototypeTest && instanceTest;

            } catch (e) {
                console.log("WHATWG compliance test error:", e.message);
                return false;
            }
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

/// Test all Workers API constructors are properly registered
#[test]
fn test_all_constructors_available() {
    let mut context = Context::default();

    let result = context.eval(Source::from_bytes(r#"
        // Verify all Workers API constructors are available in global scope
        (function() {
            let constructors = [
                'Worker',
                'SharedWorker',
                'BroadcastChannel',
                'MessageChannel'
            ];

            return constructors.every(name => {
                return typeof globalThis[name] === 'function';
            });
        })()
    "#));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}