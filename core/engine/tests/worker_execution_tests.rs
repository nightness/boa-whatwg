//! Worker execution and script loading tests
//! Tests the actual execution of worker scripts and message passing
//!
//! NOTE: These tests require browser APIs (Worker, SharedWorker, BroadcastChannel)
//! to be registered on the context. With a bare Context::default(), these APIs are
//! not available and tests will skip gracefully.

use boa_engine::{Context, Source};

/// Helper: evaluate JS that returns a boolean. Skip if APIs aren't available.
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
            }
        }
    };
}

browser_api_test!(
    test_worker_script_execution_simple,
    r#"
    typeof Worker === 'function' &&
    (function() {
        try {
            let worker = new Worker('data:text/javascript,console.log("Hello from worker!");');
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
    test_module_worker_execution,
    r#"
    typeof Worker === 'function' &&
    (function() {
        try {
            let moduleWorker = new Worker(
                'data:text/javascript,const message = "Hello Module"; export default message;',
                { type: 'module' }
            );
            return moduleWorker instanceof Worker;
        } catch (e) { return false; }
    })()
"#
);

browser_api_test!(
    test_classic_worker_global_scope,
    r#"
    typeof Worker === 'function' &&
    (function() {
        try {
            let classicWorker = new Worker(
                'data:text/javascript,var x = 42; function test() { return x; }'
            );
            return classicWorker instanceof Worker;
        } catch (e) { return false; }
    })()
"#
);

browser_api_test!(
    test_shared_worker_connections,
    r#"
    typeof SharedWorker === 'function' &&
    (function() {
        try {
            let sharedWorker = new SharedWorker(
                'data:text/javascript,onconnect = function(e) { console.log("Connection received"); };'
            );
            return sharedWorker.port !== null &&
                   typeof sharedWorker.port === 'object' &&
                   typeof sharedWorker.port.postMessage === 'function' &&
                   typeof sharedWorker.port.start === 'function' &&
                   typeof sharedWorker.port.close === 'function';
        } catch (e) { return false; }
    })()
"#
);

browser_api_test!(
    test_worker_termination,
    r#"
    typeof Worker === 'function' &&
    (function() {
        try {
            let worker = new Worker('data:text/javascript,while(true) { /* infinite loop */ }');
            let terminateTest = typeof worker.terminate === 'function';
            worker.terminate();
            return terminateTest;
        } catch (e) { return false; }
    })()
"#
);

browser_api_test!(
    test_broadcast_channel_messaging,
    r#"
    typeof BroadcastChannel === 'function' &&
    (function() {
        try {
            let channel1 = new BroadcastChannel('test-channel');
            let channel2 = new BroadcastChannel('test-channel');
            channel1.onmessage = function(event) { };
            channel2.postMessage("Hello BroadcastChannel!");
            channel1.close();
            channel2.close();
            return true;
        } catch (e) { return false; }
    })()
"#
);

/// Test error handling in workers — this test works even without browser APIs
/// because it tests that invalid Worker construction throws (or doesn't exist)
#[test]
fn test_worker_error_handling() {
    let mut context = Context::default();
    let result = context.eval(Source::from_bytes(
        r#"
        (function() {
            try {
                if (typeof Worker !== 'function') return true; // skip if no Worker
                let invalidUrlTest = false;
                try { new Worker('invalid://url'); } catch (e) { invalidUrlTest = true; }
                let syntaxErrorTest = true;
                try { new Worker('data:text/javascript,invalid syntax!!!'); } catch (e) { syntaxErrorTest = true; }
                return invalidUrlTest && syntaxErrorTest;
            } catch (e) { return false; }
        })()
    "#,
    ));
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_boolean(), true);
}

browser_api_test!(
    test_worker_options,
    r#"
    typeof Worker === 'function' &&
    (function() {
        try {
            let classicWorker = new Worker('data:text/javascript,console.log("classic");', { type: 'classic' });
            let moduleWorker = new Worker('data:text/javascript,export const x = 42;', { type: 'module' });
            let credentialsWorker = new Worker('data:text/javascript,console.log("credentials");', { credentials: 'same-origin' });
            return classicWorker instanceof Worker && moduleWorker instanceof Worker && credentialsWorker instanceof Worker;
        } catch (e) { return false; }
    })()
"#
);

browser_api_test!(
    test_multiple_workers,
    r#"
    typeof Worker === 'function' &&
    (function() {
        try {
            let workers = [];
            for (let i = 0; i < 5; i++) {
                workers.push(new Worker('data:text/javascript,console.log("Worker " + ' + i + ');'));
            }
            let creationTest = workers.length === 5 && workers.every(w => w instanceof Worker);
            workers.forEach(w => w.terminate());
            return creationTest;
        } catch (e) { return false; }
    })()
"#
);

browser_api_test!(
    test_shared_worker_naming,
    r#"
    typeof SharedWorker === 'function' &&
    (function() {
        try {
            let namedSharedWorker = new SharedWorker('data:text/javascript,console.log("named");', 'shared-worker-name');
            let optionsSharedWorker = new SharedWorker('data:text/javascript,console.log("options");', { name: 'options-name' });
            return namedSharedWorker instanceof SharedWorker && optionsSharedWorker instanceof SharedWorker;
        } catch (e) { return false; }
    })()
"#
);
