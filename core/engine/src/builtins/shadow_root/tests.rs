//! Tests for Shadow DOM implementation

use crate::{js_string, run_test_actions, TestAction, JsNativeErrorKind, JsValue};

#[test]
fn shadow_root_basic_creation() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run("
            let div = document.createElement('div');
            let shadowRoot = div.attachShadow({mode: 'open'});
        "),
        TestAction::assert_eq("typeof shadowRoot", js_string!("object")),
    ]);
}

#[test]
fn shadow_root_mode_property() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run("
            let div = document.createElement('div');
            let openShadow = div.attachShadow({mode: 'open'});
        "),
        TestAction::assert_eq("openShadow.mode", js_string!("open")),
        TestAction::run("
            let div2 = document.createElement('div');
            let closedShadow = div2.attachShadow({mode: 'closed'});
        "),
        TestAction::assert_eq("closedShadow.mode", js_string!("closed")),
    ]);
}

#[test]
fn shadow_root_host_property() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run("
            let div = document.createElement('div');
            let shadowRoot = div.attachShadow({mode: 'open'});
        "),
        TestAction::assert_eq("shadowRoot.host === div", true),
    ]);
}

#[test]
fn shadow_root_open_shadow_root_property() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run("
            let div = document.createElement('div');
            let shadowRoot = div.attachShadow({mode: 'open'});
        "),
        TestAction::assert_eq("div.shadowRoot === shadowRoot", true),
    ]);
}

#[test]
fn shadow_root_closed_shadow_root_property() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run("
            let div = document.createElement('div');
            let shadowRoot = div.attachShadow({mode: 'closed'});
        "),
        TestAction::assert_eq("div.shadowRoot", JsValue::null()),
    ]);
}

#[test]
fn shadow_root_clonable_property() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run("
            let div = document.createElement('div');
            let shadowRoot = div.attachShadow({mode: 'open', clonable: true});
        "),
        TestAction::assert_eq("shadowRoot.clonable", true),
        TestAction::run("
            let div2 = document.createElement('div');
            let shadowRoot2 = div2.attachShadow({mode: 'open', clonable: false});
        "),
        TestAction::assert_eq("shadowRoot2.clonable", false),
    ]);
}

#[test]
fn shadow_root_serializable_property() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run("
            let div = document.createElement('div');
            let shadowRoot = div.attachShadow({mode: 'open', serializable: true});
        "),
        TestAction::assert_eq("shadowRoot.serializable", true),
        TestAction::run("
            let div2 = document.createElement('div');
            let shadowRoot2 = div2.attachShadow({mode: 'open', serializable: false});
        "),
        TestAction::assert_eq("shadowRoot2.serializable", false),
    ]);
}

#[test]
fn shadow_root_delegates_focus_property() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run("
            let div = document.createElement('div');
            let shadowRoot = div.attachShadow({mode: 'open', delegatesFocus: true});
        "),
        TestAction::assert_eq("shadowRoot.delegatesFocus", true),
        TestAction::run("
            let div2 = document.createElement('div');
            let shadowRoot2 = div2.attachShadow({mode: 'open', delegatesFocus: false});
        "),
        TestAction::assert_eq("shadowRoot2.delegatesFocus", false),
    ]);
}

#[test]
fn shadow_root_invalid_mode_error() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run("
            let div = document.createElement('div');
            try {
                div.attachShadow({mode: 'invalid'});
            } catch (e) {
                globalThis.error = e;
            }
        "),
        TestAction::assert_eq("globalThis.error instanceof TypeError", true),
    ]);
}

#[test]
fn shadow_root_missing_mode_error() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run("
            let div = document.createElement('div');
            try {
                div.attachShadow({});
            } catch (e) {
                globalThis.error = e;
            }
        "),
        TestAction::assert_eq("globalThis.error instanceof TypeError", true),
    ]);
}

#[test]
fn shadow_root_no_options_error() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run("
            let div = document.createElement('div');
            try {
                div.attachShadow();
            } catch (e) {
                globalThis.error = e;
            }
        "),
        TestAction::assert_eq("globalThis.error instanceof TypeError", true),
    ]);
}

#[test]
fn shadow_root_duplicate_shadow_root_error() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run("
            let div = document.createElement('div');
            div.attachShadow({mode: 'open'});
            try {
                div.attachShadow({mode: 'open'});
            } catch (e) {
                globalThis.error = e;
            }
        "),
        TestAction::assert_eq("globalThis.error instanceof Error", true),
    ]);
}

#[test]
fn shadow_root_unsupported_element_error() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run("
            let img = document.createElement('img');
            try {
                img.attachShadow({mode: 'open'});
            } catch (e) {
                globalThis.error = e;
            }
        "),
        TestAction::assert_eq("globalThis.error instanceof Error", true),
    ]);
}

#[test]
fn shadow_root_inheritance_from_document_fragment() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run("
            let div = document.createElement('div');
            let shadowRoot = div.attachShadow({mode: 'open'});
        "),
        TestAction::assert_eq("shadowRoot instanceof DocumentFragment", true),
        TestAction::assert_eq("typeof shadowRoot.querySelector", js_string!("function")),
        TestAction::assert_eq("typeof shadowRoot.querySelectorAll", js_string!("function")),
        TestAction::assert_eq("typeof shadowRoot.getElementById", js_string!("function")),
        TestAction::assert_eq("typeof shadowRoot.append", js_string!("function")),
    ]);
}

#[test]
fn shadow_root_get_html_method() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run("
            let div = document.createElement('div');
            let shadowRoot = div.attachShadow({mode: 'open'});
        "),
        TestAction::assert_eq("typeof shadowRoot.getHTML", js_string!("function")),
        TestAction::run("let html = shadowRoot.getHTML();"),
        TestAction::assert_eq("typeof html", js_string!("string")),
    ]);
}

#[test]
fn shadow_root_constructor_illegal() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run("
            try {
                new ShadowRoot();
            } catch (e) {
                globalThis.error = e;
            }
        "),
        TestAction::assert_eq("globalThis.error instanceof TypeError", true),
    ]);
}

#[test]
fn shadow_root_supported_elements() {
    // Test that supported elements can have shadow roots
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run("
            const supportedElements = ['div', 'span', 'section', 'article', 'aside', 'header', 'footer', 'main', 'nav', 'blockquote', 'body', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'p'];
            globalThis.results = [];
            for (const tag of supportedElements) {
                try {
                    const elem = document.createElement(tag);
                    elem.attachShadow({mode: 'open'});
                    globalThis.results.push(true);
                } catch (e) {
                    globalThis.results.push(false);
                }
            }
        "),
        TestAction::assert_eq("globalThis.results.every(r => r)", true),
    ]);
}

#[test]
fn shadow_root_properties_read_only() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run("
            let div = document.createElement('div');
            let shadowRoot = div.attachShadow({mode: 'open'});
            // Try to modify read-only properties
            shadowRoot.mode = 'closed';
            shadowRoot.host = null;
            shadowRoot.clonable = !shadowRoot.clonable;
        "),
        // Properties should remain unchanged (they're read-only)
        TestAction::assert_eq("shadowRoot.mode", js_string!("open")),
        TestAction::assert_eq("shadowRoot.host === div", true),
        TestAction::assert_eq("shadowRoot.clonable", false), // default value
    ]);
}