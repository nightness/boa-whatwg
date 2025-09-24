//! Tests for DOMTokenList (classList)
use crate::run_test_actions;
use crate::TestAction;

#[test]
fn domtokenlist_basic() {
    let actions = vec![
        TestAction::run("var d = document.createElement('div')"),
        TestAction::assert_eq("typeof d.classList", js_string!("object")),
        TestAction::run("d.classList.add('a')"),
        TestAction::assert_eq("d.classList.contains('a')", true),
        TestAction::run("d.classList.add('b', 'c')"),
        TestAction::assert_eq("d.className", js_string!("a b c")),
        TestAction::run("d.classList.remove('b')"),
        TestAction::assert_eq("d.className", js_string!("a c")),
        TestAction::run("var toggled = d.classList.toggle('c')"),
        TestAction::assert_eq("toggled", false),
        TestAction::assert_eq("d.className", js_string!("a")),
        TestAction::run("d.classList.toggle('x')"),
        TestAction::assert_eq("d.classList.contains('x')", true),
    ];
    run_test_actions(actions);
}
