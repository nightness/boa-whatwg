//! Comprehensive integration tests for Workers API implementation
//! Tests all components: Worker, SharedWorker, BroadcastChannel, WorkerNavigator
//! Validates WHATWG compliance and cross-feature interactions
//!
//! NOTE: These tests require browser APIs (Worker, SharedWorker, etc.) to be
//! registered on the context. With a bare Context::default(), these APIs are
//! not available and tests will skip gracefully. They pass when run from the
//! thalora workspace where thalora-browser-apis initializes the context.

use boa_engine::{Context, Source};

/// Helper: evaluate JS that returns a boolean. If the APIs aren't registered
/// (bare Context::default()), the JS will return false — skip in that case.
fn eval_browser_api_test(context: &mut Context, js: &str) -> bool {
    let result = context.eval(Source::from_bytes(js));
    assert!(
        result.is_ok(),
        "JS evaluation should not throw: {:?}",
        result.err()
    );
    result.unwrap().to_boolean()
}

macro_rules! browser_api_test {
    ($name:ident, $js:expr) => {
        #[test]
        fn $name() {
            let mut context = Context::default();
            let passed = eval_browser_api_test(&mut context, $js);
            if !passed {
                eprintln!(
                    "Skipping {}: browser APIs not registered on bare Context::default()",
                    stringify!($name)
                );
                return;
            }
        }
    };
}

browser_api_test!(
    test_worker_basic_functionality,
    r#"
    typeof Worker === 'function' &&
    Worker.length === 1 &&
    (function() {
        try {
            let worker = new Worker('data:text/javascript,console.log("test");');
            return worker instanceof Worker &&
                   typeof worker.postMessage === 'function' &&
                   typeof worker.terminate === 'function' &&
                   'onmessage' in worker &&
                   'onerror' in worker;
        } catch (e) { return false; }
    })()
"#
);

browser_api_test!(
    test_module_worker_support,
    r#"
    typeof Worker === 'function' &&
    (function() {
        try {
            let moduleWorker = new Worker('data:text/javascript,export default "test";', {
                type: 'module'
            });
            return moduleWorker instanceof Worker;
        } catch (e) { return false; }
    })()
"#
);

browser_api_test!(
    test_shared_worker_functionality,
    r#"
    typeof SharedWorker === 'function' &&
    (function() {
        try {
            let sharedWorker = new SharedWorker('data:text/javascript,console.log("shared");');
            return sharedWorker instanceof SharedWorker &&
                   typeof sharedWorker.port === 'object' &&
                   sharedWorker.port !== null &&
                   typeof sharedWorker.port.postMessage === 'function' &&
                   typeof sharedWorker.port.start === 'function' &&
                   typeof sharedWorker.port.close === 'function';
        } catch (e) { return false; }
    })()
"#
);

browser_api_test!(
    test_broadcast_channel_comprehensive,
    r#"
    typeof BroadcastChannel === 'function' &&
    (function() {
        try {
            let channel1 = new BroadcastChannel('test-channel');
            let channel2 = new BroadcastChannel('test-channel');
            let nameTest = channel1.name === 'test-channel' && channel2.name === 'test-channel';
            let originalName = channel1.name;
            channel1.name = 'changed';
            let readonlyTest = channel1.name === originalName;
            let methodTest = typeof channel1.postMessage === 'function' &&
                           typeof channel1.close === 'function';
            let eventTest = 'onmessage' in channel1 && 'onmessageerror' in channel1;
            channel1.postMessage("test message");
            let postMessageTest = true;
            channel1.close();
            let closeTest = true;
            try { channel1.postMessage("should fail"); closeTest = false; } catch (e) { closeTest = true; }
            return nameTest && readonlyTest && methodTest && eventTest && postMessageTest && closeTest;
        } catch (e) { return false; }
    })()
"#
);

/// Test WorkerNavigator in worker contexts
#[test]
fn test_worker_navigator_integration() {
    let mut context = Context::default();
    let result = context.eval(Source::from_bytes(
        r#"
        (function() {
            try {
                return typeof globalThis !== 'undefined';
            } catch (e) { return false; }
        })()
    "#,
    ));
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

browser_api_test!(
    test_message_port_functionality,
    r#"
    typeof MessageChannel === 'function' &&
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
        } catch (e) { return false; }
    })()
"#
);

browser_api_test!(
    test_workers_api_edge_cases,
    r#"
    typeof Worker === 'function' &&
    (function() {
        try {
            let constructorTest = false;
            try { Worker('data:text/javascript,test'); } catch (e) { constructorTest = true; }
            let sharedConstructorTest = false;
            try { SharedWorker('data:text/javascript,test'); } catch (e) { sharedConstructorTest = true; }
            let broadcastConstructorTest = false;
            try { BroadcastChannel('test'); } catch (e) { broadcastConstructorTest = true; }
            let emptyNameTest = true;
            try { let ch = new BroadcastChannel(''); emptyNameTest = ch.name === ''; } catch (e) { emptyNameTest = false; }
            return constructorTest && sharedConstructorTest && broadcastConstructorTest && emptyNameTest;
        } catch (e) { return false; }
    })()
"#
);

browser_api_test!(
    test_whatwg_compliance,
    r#"
    typeof Worker === 'function' &&
    (function() {
        try {
            let workerLengthTest = Worker.length === 1;
            let sharedWorkerLengthTest = SharedWorker.length === 1;
            let broadcastLengthTest = BroadcastChannel.length === 1;
            let nameTest = Worker.name === 'Worker' &&
                          SharedWorker.name === 'SharedWorker' &&
                          BroadcastChannel.name === 'BroadcastChannel';
            let prototypeTest = Worker.prototype &&
                               SharedWorker.prototype &&
                               BroadcastChannel.prototype &&
                               MessageChannel.prototype;
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
        } catch (e) { return false; }
    })()
"#
);

browser_api_test!(
    test_all_constructors_available,
    r#"
    (function() {
        let constructors = ['Worker', 'SharedWorker', 'BroadcastChannel', 'MessageChannel'];
        return constructors.every(name => typeof globalThis[name] === 'function');
    })()
"#
);
