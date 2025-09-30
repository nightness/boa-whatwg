use crate::{run_test_actions, TestAction};

#[test]
fn debug_globals() {
    run_test_actions([
        TestAction::run("console.log('Global object keys:')"),
        TestAction::run("console.log(Object.keys(globalThis))"),
        TestAction::run("console.log('console type:', typeof console)"),
        TestAction::run("console.log('setTimeout type:', typeof setTimeout)"),
        TestAction::run("console.log('Blob type:', typeof Blob)"),
    ]);
}