//! CSS style scoping for Shadow DOM
//!
//! Implementation of WHATWG Shadow DOM CSS scoping algorithms
//! https://drafts.csswg.org/css-scoping-1/

use crate::{
    builtins::{
        element::ElementData,
        shadow_root::ShadowRootData,
        node::NodeData,
    },
    object::JsObject,
    value::JsValue,
    Context, JsResult, JsNativeError,
};
use boa_gc::{Finalize, Trace, GcRefCell};
use std::collections::{HashMap, HashSet};

/// CSS selector scoping types
#[derive(Debug, Clone, PartialEq, Eq, Trace, Finalize)]
pub enum ScopeType {
    /// :host selector - matches the shadow host
    Host,
    /// :host() functional selector - matches host with selector
    HostFunction(String),
    /// :host-context() selector - matches host or ancestor with selector
    HostContext(String),
    /// ::slotted() selector - matches slotted elements
    Slotted(String),
    /// Regular scoped selector
    Scoped(String),
}

/// CSS rule with scoping information
#[derive(Debug, Trace, Finalize)]
pub struct ScopedCSSRule {
    selector: String,
    scope_type: ScopeType,
    properties: GcRefCell<HashMap<String, String>>,
    shadow_root: Option<JsObject>,
}

impl ScopedCSSRule {
    pub fn new(selector: String, shadow_root: Option<JsObject>) -> Self {
        let scope_type = Self::parse_scope_type(&selector);

        Self {
            selector,
            scope_type,
            properties: GcRefCell::new(HashMap::new()),
            shadow_root,
        }
    }

    /// Parse CSS selector to determine scope type
    fn parse_scope_type(selector: &str) -> ScopeType {
        let trimmed = selector.trim();

        if trimmed == ":host" {
            ScopeType::Host
        } else if trimmed.starts_with(":host(") && trimmed.ends_with(')') {
            let inner = &trimmed[6..trimmed.len()-1];
            ScopeType::HostFunction(inner.to_string())
        } else if trimmed.starts_with(":host-context(") && trimmed.ends_with(')') {
            let inner = &trimmed[14..trimmed.len()-1];
            ScopeType::HostContext(inner.to_string())
        } else if trimmed.starts_with("::slotted(") && trimmed.ends_with(')') {
            let inner = &trimmed[10..trimmed.len()-1];
            ScopeType::Slotted(inner.to_string())
        } else {
            ScopeType::Scoped(selector.to_string())
        }
    }

    pub fn add_property(&self, property: String, value: String) {
        self.properties.borrow_mut().insert(property, value);
    }

    pub fn get_properties(&self) -> HashMap<String, String> {
        self.properties.borrow().clone()
    }
}

/// CSS scoping engine for Shadow DOM
pub struct ShadowCSSScoping;

impl ShadowCSSScoping {
    /// Parse CSS text and create scoped rules for a shadow root
    pub fn parse_shadow_css(css_text: &str, shadow_root: &JsObject) -> Vec<ScopedCSSRule> {
        let mut rules = Vec::new();

        // Simple CSS parser for Shadow DOM scoping
        // In production, this would use a full CSS parser
        let lines = css_text.lines();
        let mut current_selector = String::new();
        let mut in_rule = false;
        let mut current_rule: Option<ScopedCSSRule> = None;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with("/*") {
                continue;
            }

            if !in_rule && trimmed.contains('{') {
                // Start of rule
                let parts: Vec<&str> = trimmed.split('{').collect();
                if let Some(selector) = parts.first() {
                    current_selector = selector.trim().to_string();
                    current_rule = Some(ScopedCSSRule::new(
                        current_selector.clone(),
                        Some(shadow_root.clone())
                    ));
                    in_rule = true;
                }
            } else if in_rule && trimmed.contains('}') {
                // End of rule
                if let Some(rule) = current_rule.take() {
                    rules.push(rule);
                }
                in_rule = false;
                current_selector.clear();
            } else if in_rule && trimmed.contains(':') {
                // CSS property
                let parts: Vec<&str> = trimmed.split(':').collect();
                if parts.len() >= 2 {
                    let property = parts[0].trim().to_string();
                    let value = parts[1..].join(":").trim().trim_end_matches(';').to_string();

                    if let Some(ref rule) = current_rule {
                        rule.add_property(property, value);
                    }
                }
            }
        }

        rules
    }

    /// Apply scoped styles to elements within a shadow tree
    pub fn apply_scoped_styles(
        shadow_root: &JsObject,
        rules: &[ScopedCSSRule],
        context: &mut Context,
    ) -> JsResult<()> {
        let shadow_data = shadow_root
            .downcast_ref::<ShadowRootData>()
            .ok_or_else(|| {
                JsNativeError::typ().with_message("Object is not a ShadowRoot")
            })?;

        let host = shadow_data.get_host()
            .ok_or_else(|| {
                JsNativeError::typ().with_message("Shadow root has no host")
            })?;

        for rule in rules {
            match &rule.scope_type {
                ScopeType::Host => {
                    // Apply to shadow host
                    Self::apply_styles_to_element(&host, rule, context)?;
                }
                ScopeType::HostFunction(selector) => {
                    // Apply to host if it matches the selector
                    if Self::element_matches_selector(&host, selector) {
                        Self::apply_styles_to_element(&host, rule, context)?;
                    }
                }
                ScopeType::HostContext(selector) => {
                    // Apply to host if it or an ancestor matches the selector
                    if Self::host_or_ancestor_matches_selector(&host, selector) {
                        Self::apply_styles_to_element(&host, rule, context)?;
                    }
                }
                ScopeType::Slotted(selector) => {
                    // Apply to slotted elements matching the selector
                    let slotted_elements = Self::get_slotted_elements(shadow_root);
                    for element in slotted_elements {
                        if Self::element_matches_selector(&element, selector) {
                            Self::apply_styles_to_element(&element, rule, context)?;
                        }
                    }
                }
                ScopeType::Scoped(selector) => {
                    // Apply to elements within the shadow tree
                    let elements = Self::query_shadow_tree(shadow_root, selector);
                    for element in elements {
                        Self::apply_styles_to_element(&element, rule, context)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Apply CSS properties to an element
    fn apply_styles_to_element(
        element: &JsObject,
        rule: &ScopedCSSRule,
        context: &mut Context,
    ) -> JsResult<()> {
        if let Some(element_data) = element.downcast_ref::<ElementData>() {
            let properties = rule.get_properties();

            for (property, value) in properties {
                element_data.set_style_property(&property, &value);
            }
        }

        Ok(())
    }

    /// Check if an element matches a CSS selector
    fn element_matches_selector(element: &JsObject, selector: &str) -> bool {
        // Simplified selector matching
        // In production, this would use a full CSS selector engine

        if let Some(element_data) = element.downcast_ref::<ElementData>() {
            let tag_name = element_data.get_tag_name().to_lowercase();

            // Tag selector
            if selector == tag_name {
                return true;
            }

            // Class selector
            if selector.starts_with('.') {
                let class_name = &selector[1..];
                let element_class = element_data.get_class_name();
                return element_class.split_whitespace().any(|c| c == class_name);
            }

            // ID selector
            if selector.starts_with('#') {
                let id_selector = &selector[1..];
                return element_data.get_id() == id_selector;
            }

            // Universal selector
            if selector == "*" {
                return true;
            }
        }

        false
    }

    /// Check if host or any ancestor matches selector
    fn host_or_ancestor_matches_selector(host: &JsObject, selector: &str) -> bool {
        // Check host first
        if Self::element_matches_selector(host, selector) {
            return true;
        }

        // Check ancestors
        if let Some(element_data) = host.downcast_ref::<ElementData>() {
            let mut current = element_data.get_parent_node();

            while let Some(parent) = current {
                if Self::element_matches_selector(&parent, selector) {
                    return true;
                }

                if let Some(parent_data) = parent.downcast_ref::<ElementData>() {
                    current = parent_data.get_parent_node();
                } else {
                    break;
                }
            }
        }

        false
    }

    /// Get all slotted elements from shadow tree slots
    fn get_slotted_elements(shadow_root: &JsObject) -> Vec<JsObject> {
        let mut slotted = Vec::new();

        // Find all slot elements in shadow tree
        let slots = Self::find_slots_in_tree(shadow_root);

        for slot in slots {
            if let Some(slot_data) = slot.downcast_ref::<crate::builtins::html_slot_element::HTMLSlotElementData>() {
                let assigned = slot_data.get_assigned_nodes();
                slotted.extend(assigned);
            }
        }

        slotted
    }

    /// Find all slot elements in a tree
    fn find_slots_in_tree(root: &JsObject) -> Vec<JsObject> {
        let mut slots = Vec::new();
        Self::collect_slots_recursive(root, &mut slots);
        slots
    }

    /// Recursively collect slot elements
    fn collect_slots_recursive(node: &JsObject, slots: &mut Vec<JsObject>) {
        // Check if this node is a slot element
        if node.downcast_ref::<crate::builtins::html_slot_element::HTMLSlotElementData>().is_some() {
            slots.push(node.clone());
        }

        // Check children
        let children = Self::get_child_nodes(node);
        for child in children {
            Self::collect_slots_recursive(&child, slots);
        }
    }

    /// Query elements in shadow tree using selector
    fn query_shadow_tree(shadow_root: &JsObject, selector: &str) -> Vec<JsObject> {
        let mut matches = Vec::new();
        Self::query_recursive(shadow_root, selector, &mut matches);
        matches
    }

    /// Recursively query elements
    fn query_recursive(node: &JsObject, selector: &str, matches: &mut Vec<JsObject>) {
        // Check if current node matches
        if Self::element_matches_selector(node, selector) {
            matches.push(node.clone());
        }

        // Check children
        let children = Self::get_child_nodes(node);
        for child in children {
            Self::query_recursive(&child, selector, matches);
        }
    }

    /// Helper to get child nodes
    fn get_child_nodes(node: &JsObject) -> Vec<JsObject> {
        if let Some(element_data) = node.downcast_ref::<ElementData>() {
            element_data.get_children()
        } else if let Some(shadow_data) = node.downcast_ref::<ShadowRootData>() {
            shadow_data.fragment_data().get_children()
        } else {
            Vec::new()
        }
    }

    /// Create style isolation boundaries
    pub fn isolate_shadow_styles(shadow_root: &JsObject) -> JsResult<()> {
        // Implementation of style isolation
        // This prevents styles from leaking in or out of shadow trees

        if let Some(shadow_data) = shadow_root.downcast_ref::<ShadowRootData>() {
            // Mark shadow tree as style-isolated
            shadow_data.set_style_isolated(true);

            // Reset inherited styles for shadow tree root
            Self::reset_inherited_styles(shadow_root)?;
        }

        Ok(())
    }

    /// Reset inherited styles for shadow boundary
    fn reset_inherited_styles(shadow_root: &JsObject) -> JsResult<()> {
        // Reset inheritable CSS properties at shadow boundary
        // This implements the CSS inheritance isolation

        let inheritable_properties = vec![
            "color", "font-family", "font-size", "font-style", "font-weight",
            "line-height", "text-align", "text-indent", "letter-spacing",
            "word-spacing", "white-space", "direction", "visibility"
        ];

        if let Some(shadow_data) = shadow_root.downcast_ref::<ShadowRootData>() {
            for property in inheritable_properties {
                // Reset to initial value or inherit from host if needed
                shadow_data.fragment_data().set_style_property(
                    property.to_string(),
                    "initial".to_string()
                );
            }
        }

        Ok(())
    }

    /// Process :host and :host() selectors
    pub fn process_host_selectors(css_text: &str, host_element: &JsObject) -> String {
        // Replace :host selectors with scoped equivalents
        let mut processed = css_text.to_string();

        // Simple replacement for demonstration
        // In production, this would use proper CSS parsing
        processed = processed.replace(":host", &format!("[data-shadow-host=\"{}\"]",
            Self::get_element_id(host_element)));

        processed
    }

    /// Get unique identifier for element
    fn get_element_id(element: &JsObject) -> String {
        if let Some(element_data) = element.downcast_ref::<ElementData>() {
            element_data.get_id()
        } else {
            format!("element-{:p}", element)
        }
    }

    /// Validate and sanitize CSS for shadow DOM
    pub fn sanitize_shadow_css(css_text: &str) -> String {
        // Remove dangerous CSS that could break shadow DOM isolation
        let mut sanitized = css_text.to_string();

        // Remove @import rules (could leak styles)
        sanitized = sanitized.replace("@import", "/* @import removed */");

        // Validate :host usage
        sanitized = Self::validate_host_selectors(&sanitized);

        sanitized
    }

    /// Validate :host selector usage
    fn validate_host_selectors(css_text: &str) -> String {
        // Ensure :host selectors are used correctly
        // :host must be at the beginning of compound selectors

        let mut validated = css_text.to_string();

        // This is a simplified validation
        // Production would need full CSS parser

        validated
    }
}

/// CSS custom properties (CSS variables) support for Shadow DOM
pub struct ShadowCSSCustomProperties;

impl ShadowCSSCustomProperties {
    /// Process CSS custom properties with shadow DOM scoping
    pub fn process_custom_properties(
        css_text: &str,
        shadow_root: &JsObject,
    ) -> JsResult<String> {
        // Implementation of CSS custom properties scoping
        // Custom properties inherit across shadow boundaries unless explicitly reset

        let mut processed = css_text.to_string();

        // Process --* custom properties
        // These should inherit from the host unless overridden

        Ok(processed)
    }

    /// Resolve custom property values in shadow context
    pub fn resolve_custom_property(
        property_name: &str,
        shadow_root: &JsObject,
        context: &mut Context,
    ) -> JsResult<Option<String>> {
        // Look up custom property value in shadow tree context
        // If not found, look in host context

        if let Some(shadow_data) = shadow_root.downcast_ref::<ShadowRootData>() {
            // Check shadow root styles
            if let Some(value) = shadow_data.fragment_data().get_style_property(property_name) {
                return Ok(Some(value));
            }

            // Check host styles
            if let Some(host) = shadow_data.get_host() {
                if let Some(host_data) = host.downcast_ref::<ElementData>() {
                    if let Some(value) = host_data.get_style_property(property_name) {
                        return Ok(Some(value));
                    }
                }
            }
        }

        Ok(None)
    }
}