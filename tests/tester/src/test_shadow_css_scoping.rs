//! Tests for Shadow DOM CSS scoping implementation

use boa_engine::{
    builtins::{
        shadow_css_scoping::{ShadowCSSScoping, ScopeType, ScopedCSSRule},
        element::ElementData,
        shadow_root::ShadowRootData,
    },
    object::JsObject,
    value::JsValue,
    Context, JsResult,
};

#[test]
fn test_css_selector_parsing() {
    // Test parsing of different CSS selector types
    let rule1 = ScopedCSSRule::new(":host".to_string(), None);
    assert_eq!(rule1.scope_type, ScopeType::Host);

    let rule2 = ScopedCSSRule::new(":host(.active)".to_string(), None);
    if let ScopeType::HostFunction(selector) = &rule2.scope_type {
        assert_eq!(selector, ".active");
    } else {
        panic!("Expected HostFunction scope type");
    }

    let rule3 = ScopedCSSRule::new(":host-context(.dark-theme)".to_string(), None);
    if let ScopeType::HostContext(selector) = &rule3.scope_type {
        assert_eq!(selector, ".dark-theme");
    } else {
        panic!("Expected HostContext scope type");
    }

    let rule4 = ScopedCSSRule::new("::slotted(p)".to_string(), None);
    if let ScopeType::Slotted(selector) = &rule4.scope_type {
        assert_eq!(selector, "p");
    } else {
        panic!("Expected Slotted scope type");
    }

    let rule5 = ScopedCSSRule::new("div".to_string(), None);
    if let ScopeType::Scoped(selector) = &rule5.scope_type {
        assert_eq!(selector, "div");
    } else {
        panic!("Expected Scoped scope type");
    }
}

#[test]
fn test_css_parsing() {
    let css_text = r#"
        :host {
            display: block;
            background: white;
        }

        :host(.active) {
            background: blue;
        }

        ::slotted(p) {
            color: red;
        }

        div {
            margin: 10px;
        }
    "#;

    let rules = ShadowCSSScoping::parse_shadow_css(css_text, &create_mock_shadow_root());

    // Should have parsed 4 rules
    assert_eq!(rules.len(), 4);

    // Check first rule (:host)
    assert_eq!(rules[0].scope_type, ScopeType::Host);
    let properties = rules[0].get_properties();
    assert_eq!(properties.get("display"), Some(&"block".to_string()));
    assert_eq!(properties.get("background"), Some(&"white".to_string()));

    // Check second rule (:host(.active))
    if let ScopeType::HostFunction(selector) = &rules[1].scope_type {
        assert_eq!(selector, ".active");
    } else {
        panic!("Expected HostFunction scope type");
    }

    // Check third rule (::slotted(p))
    if let ScopeType::Slotted(selector) = &rules[2].scope_type {
        assert_eq!(selector, "p");
    } else {
        panic!("Expected Slotted scope type");
    }

    // Check fourth rule (div)
    if let ScopeType::Scoped(selector) = &rules[3].scope_type {
        assert_eq!(selector, "div");
    } else {
        panic!("Expected Scoped scope type");
    }
}

#[test]
fn test_element_selector_matching() {
    let element = create_mock_element("div", Some("test-id".to_string()), Some("active highlight".to_string()));

    // Test tag selector
    assert!(ShadowCSSScoping::element_matches_selector(&element, "div"));
    assert!(!ShadowCSSScoping::element_matches_selector(&element, "span"));

    // Test ID selector
    assert!(ShadowCSSScoping::element_matches_selector(&element, "#test-id"));
    assert!(!ShadowCSSScoping::element_matches_selector(&element, "#other-id"));

    // Test class selector
    assert!(ShadowCSSScoping::element_matches_selector(&element, ".active"));
    assert!(ShadowCSSScoping::element_matches_selector(&element, ".highlight"));
    assert!(!ShadowCSSScoping::element_matches_selector(&element, ".inactive"));

    // Test universal selector
    assert!(ShadowCSSScoping::element_matches_selector(&element, "*"));
}

#[test]
fn test_css_sanitization() {
    let malicious_css = r#"
        @import url("http://evil.com/styles.css");

        :host {
            background: red;
        }

        div {
            color: blue;
        }
    "#;

    let sanitized = ShadowCSSScoping::sanitize_shadow_css(malicious_css);

    // @import should be removed
    assert!(sanitized.contains("/* @import removed */"));
    assert!(!sanitized.contains("@import url"));

    // Other CSS should remain
    assert!(sanitized.contains(":host"));
    assert!(sanitized.contains("background: red"));
    assert!(sanitized.contains("div"));
}

#[test]
fn test_host_selector_processing() {
    let host_element = create_mock_element("my-component", Some("host-1".to_string()), None);

    let css_with_host = r#"
        :host {
            display: block;
        }

        :host(.theme-dark) {
            background: black;
        }
    "#;

    let processed = ShadowCSSScoping::process_host_selectors(css_with_host, &host_element);

    // :host should be replaced with scoped equivalent
    assert!(processed.contains("[data-shadow-host=\"host-1\"]"));
    assert!(!processed.contains(":host "));
}

#[test]
fn test_shadow_css_isolation() {
    let shadow_root = create_mock_shadow_root();

    // Test style isolation
    let result = ShadowCSSScoping::isolate_shadow_styles(&shadow_root);
    assert!(result.is_ok());

    // In a full implementation, this would verify that styles are properly isolated
}

#[test]
fn test_custom_property_resolution() {
    let shadow_root = create_mock_shadow_root();
    let mut context = Context::default();

    // Test custom property resolution
    let result = ShadowCSSScoping::resolve_custom_property(
        "--main-color",
        &shadow_root,
        &mut context,
    );

    assert!(result.is_ok());
    // In this test case, no custom property is set, so should return None
    assert_eq!(result.unwrap(), None);
}

#[test]
fn test_css_rule_properties() {
    let mut rule = ScopedCSSRule::new(":host".to_string(), None);

    // Test adding properties
    rule.add_property("color".to_string(), "red".to_string());
    rule.add_property("background".to_string(), "blue".to_string());

    let properties = rule.get_properties();
    assert_eq!(properties.get("color"), Some(&"red".to_string()));
    assert_eq!(properties.get("background"), Some(&"blue".to_string()));
    assert_eq!(properties.len(), 2);
}

#[test]
fn test_shadow_tree_css_extraction() {
    let html_with_styles = r#"
        <div>
            <style>
                :host { display: block; }
                .content { color: red; }
            </style>
            <p>Content</p>
            <style>
                ::slotted(span) { font-weight: bold; }
            </style>
        </div>
    "#;

    let css_rules = ShadowRootData::extract_css_from_html(html_with_styles);

    assert_eq!(css_rules.len(), 2);
    assert!(css_rules[0].contains(":host { display: block; }"));
    assert!(css_rules[0].contains(".content { color: red; }"));
    assert!(css_rules[1].contains("::slotted(span) { font-weight: bold; }"));
}

// Helper functions to create mock objects for testing

fn create_mock_shadow_root() -> JsObject {
    // In a real implementation, this would create a proper JsObject with ShadowRootData
    // For testing purposes, we create a mock
    use boa_gc::GcRefCell;
    use crate::builtins::document_fragment::DocumentFragmentData;

    let shadow_data = ShadowRootData::new(
        crate::builtins::shadow_root::ShadowRootMode::Open,
        false, // clonable
        false, // serializable
        false, // delegates_focus
    );

    // This is simplified for testing - in production would use proper object construction
    let mut context = Context::default();
    let realm = context.realm().clone();
    let prototype = realm.intrinsics().constructors().object().prototype();

    JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        prototype,
        shadow_data,
    )
}

fn create_mock_element(tag_name: &str, id: Option<String>, class_name: Option<String>) -> JsObject {
    // Create a mock element for testing
    let element_data = ElementData::with_tag_name(tag_name.to_string());

    if let Some(id) = id {
        element_data.set_id(id);
    }

    if let Some(class_name) = class_name {
        element_data.set_class_name(class_name);
    }

    // This is simplified for testing
    let mut context = Context::default();
    let realm = context.realm().clone();
    let prototype = realm.intrinsics().constructors().object().prototype();

    JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        prototype,
        element_data,
    )
}

#[test]
fn test_comprehensive_shadow_css_workflow() {
    // Test the complete CSS scoping workflow
    let html_content = r#"
        <style>
            :host {
                display: block;
                background: white;
            }

            :host(.dark) {
                background: black;
                color: white;
            }

            ::slotted(p) {
                margin: 1em;
            }

            .content {
                padding: 16px;
            }

            #title {
                font-size: 2em;
            }
        </style>

        <div class="content">
            <h1 id="title">Shadow Content</h1>
            <slot></slot>
        </div>
    "#;

    // Test CSS extraction
    let css_rules = ShadowRootData::extract_css_from_html(html_content);
    assert_eq!(css_rules.len(), 1);

    // Test CSS parsing
    let shadow_root = create_mock_shadow_root();
    let parsed_rules = ShadowCSSScoping::parse_shadow_css(&css_rules[0], &shadow_root);

    // Should parse all 5 rules
    assert_eq!(parsed_rules.len(), 5);

    // Test CSS sanitization
    let sanitized = ShadowCSSScoping::sanitize_shadow_css(&css_rules[0]);
    assert!(!sanitized.contains("@import"));

    // Test scoping application would happen here with real DOM elements
    // In a full implementation, this would apply the scoped styles to elements
}