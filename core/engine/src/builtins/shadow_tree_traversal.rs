//! Shadow-including tree traversal algorithms for Shadow DOM
//!
//! Implementation of WHATWG DOM shadow-including tree traversal algorithms
//! https://dom.spec.whatwg.org/#concept-shadow-including-tree-order

use crate::{
    builtins::{
        element::ElementData,
        shadow_root::ShadowRootData,
        html_slot_element::HTMLSlotElementData,
        node::NodeData,
    },
    object::JsObject,
    value::JsValue,
    Context, JsResult,
};
use boa_gc::{Finalize, Trace};
use std::collections::{VecDeque, HashSet};

/// Shadow-including tree traversal order types
#[derive(Debug, Clone, PartialEq, Eq, Trace, Finalize)]
pub enum TraversalOrder {
    /// Pre-order traversal (parent before children)
    PreOrder,
    /// Post-order traversal (children before parent)
    PostOrder,
}

/// Shadow-including tree traversal algorithms
pub struct ShadowTreeTraversal;

impl ShadowTreeTraversal {
    /// Get shadow-including root of a node
    /// https://dom.spec.whatwg.org/#concept-shadow-including-root
    pub fn shadow_including_root(node: &JsObject) -> JsObject {
        let mut current = node.clone();

        loop {
            let parent = Self::shadow_including_parent(&current);
            if parent.is_none() {
                break;
            }
            current = parent.unwrap();
        }

        current
    }

    /// Get shadow-including parent of a node
    /// https://dom.spec.whatwg.org/#concept-shadow-including-parent
    pub fn shadow_including_parent(node: &JsObject) -> Option<JsObject> {
        // If node is a shadow root, return its host
        if let Some(_shadow_data) = node.downcast_ref::<ShadowRootData>() {
            return Self::get_shadow_host(node);
        }

        // Otherwise, return regular parent
        Self::get_parent_node(node)
    }

    /// Get shadow-including ancestors of a node
    /// https://dom.spec.whatwg.org/#concept-shadow-including-ancestor
    pub fn shadow_including_ancestors(node: &JsObject) -> Vec<JsObject> {
        let mut ancestors = Vec::new();
        let mut current = Self::shadow_including_parent(node);

        while let Some(parent) = current {
            ancestors.push(parent.clone());
            current = Self::shadow_including_parent(&parent);
        }

        ancestors
    }

    /// Get shadow-including descendants of a node
    /// https://dom.spec.whatwg.org/#concept-shadow-including-descendant
    pub fn shadow_including_descendants(node: &JsObject) -> Vec<JsObject> {
        let mut descendants = Vec::new();
        Self::collect_shadow_including_descendants(node, &mut descendants);
        descendants
    }

    /// Helper to recursively collect shadow-including descendants
    fn collect_shadow_including_descendants(node: &JsObject, descendants: &mut Vec<JsObject>) {
        let children = Self::get_child_nodes(node);

        for child in children {
            descendants.push(child.clone());
            Self::collect_shadow_including_descendants(&child, descendants);

            // If child is an element with a shadow root, traverse shadow tree
            if let Some(shadow_root) = Self::get_shadow_root(&child) {
                descendants.push(shadow_root.clone());
                Self::collect_shadow_including_descendants(&shadow_root, descendants);
            }
        }
    }

    /// Shadow-including tree order traversal
    /// https://dom.spec.whatwg.org/#concept-shadow-including-tree-order
    pub fn shadow_including_tree_order(
        root: &JsObject,
        order: TraversalOrder,
    ) -> Vec<JsObject> {
        let mut result = Vec::new();
        Self::traverse_shadow_including_tree(root, &mut result, order);
        result
    }

    /// Helper for shadow-including tree order traversal
    fn traverse_shadow_including_tree(
        node: &JsObject,
        result: &mut Vec<JsObject>,
        order: TraversalOrder,
    ) {
        match order {
            TraversalOrder::PreOrder => {
                result.push(node.clone());
            }
            TraversalOrder::PostOrder => {
                // Process children first, then current node
            }
        }

        // Get children and shadow roots
        let children = Self::get_child_nodes(node);

        for child in children {
            Self::traverse_shadow_including_tree(&child, result, order.clone());

            // If child is an element with a shadow root, traverse shadow tree
            if let Some(shadow_root) = Self::get_shadow_root(&child) {
                Self::traverse_shadow_including_tree(&shadow_root, result, order.clone());
            }
        }

        if order == TraversalOrder::PostOrder {
            result.push(node.clone());
        }
    }

    /// Get shadow-including inclusive ancestors
    /// https://dom.spec.whatwg.org/#concept-shadow-including-inclusive-ancestor
    pub fn shadow_including_inclusive_ancestors(node: &JsObject) -> Vec<JsObject> {
        let mut ancestors = vec![node.clone()];
        ancestors.extend(Self::shadow_including_ancestors(node));
        ancestors
    }

    /// Get shadow-including inclusive descendants
    /// https://dom.spec.whatwg.org/#concept-shadow-including-inclusive-descendant
    pub fn shadow_including_inclusive_descendants(node: &JsObject) -> Vec<JsObject> {
        let mut descendants = vec![node.clone()];
        descendants.extend(Self::shadow_including_descendants(node));
        descendants
    }

    /// Check if node A is a shadow-including ancestor of node B
    pub fn is_shadow_including_ancestor(ancestor: &JsObject, node: &JsObject) -> bool {
        let ancestors = Self::shadow_including_ancestors(node);
        ancestors.iter().any(|a| JsObject::equals(a, ancestor))
    }

    /// Check if node A is a shadow-including descendant of node B
    pub fn is_shadow_including_descendant(descendant: &JsObject, node: &JsObject) -> bool {
        let descendants = Self::shadow_including_descendants(node);
        descendants.iter().any(|d| JsObject::equals(d, descendant))
    }

    /// Find the shadow-including first child
    pub fn shadow_including_first_child(node: &JsObject) -> Option<JsObject> {
        let children = Self::get_child_nodes(node);
        children.into_iter().next()
    }

    /// Find the shadow-including last child
    pub fn shadow_including_last_child(node: &JsObject) -> Option<JsObject> {
        let children = Self::get_child_nodes(node);
        children.into_iter().last()
    }

    /// Find the shadow-including next sibling
    pub fn shadow_including_next_sibling(node: &JsObject) -> Option<JsObject> {
        let parent = Self::shadow_including_parent(node)?;
        let siblings = Self::get_child_nodes(&parent);

        let mut found_current = false;
        for sibling in siblings {
            if found_current {
                return Some(sibling);
            }
            if JsObject::equals(&sibling, node) {
                found_current = true;
            }
        }
        None
    }

    /// Find the shadow-including previous sibling
    pub fn shadow_including_previous_sibling(node: &JsObject) -> Option<JsObject> {
        let parent = Self::shadow_including_parent(node)?;
        let siblings = Self::get_child_nodes(&parent);

        let mut previous = None;
        for sibling in siblings {
            if JsObject::equals(&sibling, node) {
                return previous;
            }
            previous = Some(sibling);
        }
        None
    }

    /// Get flattened slottables for an element
    /// This is used for slot assignment in shadow DOM
    pub fn get_flattened_slottables(element: &JsObject) -> Vec<JsObject> {
        // Implementation of flattened slottables algorithm
        // https://dom.spec.whatwg.org/#concept-slotable-flattened-slotable

        if Self::is_slot_element(element) {
            // For slot elements, return assigned nodes or fallback content
            if let Some(slot_data) = element.downcast_ref::<HTMLSlotElementData>() {
                let assigned = slot_data.get_assigned_nodes();
                if assigned.is_empty() {
                    // Return fallback content (child nodes of the slot)
                    Self::get_child_nodes(element)
                } else {
                    assigned
                }
            } else {
                Vec::new()
            }
        } else {
            // For non-slot elements, return the element itself
            vec![element.clone()]
        }
    }

    /// Find all slots in a shadow tree
    pub fn find_slots_in_shadow_tree(shadow_root: &JsObject) -> Vec<JsObject> {
        let mut slots = Vec::new();
        Self::collect_slots_recursive(shadow_root, &mut slots);
        slots
    }

    /// Recursively collect slot elements
    fn collect_slots_recursive(node: &JsObject, slots: &mut Vec<JsObject>) {
        if Self::is_slot_element(node) {
            slots.push(node.clone());
        }

        let children = Self::get_child_nodes(node);
        for child in children {
            Self::collect_slots_recursive(&child, slots);
        }
    }

    /// Shadow DOM query selector implementation
    /// This implements shadow-aware querySelector behavior
    pub fn shadow_aware_query_selector(
        root: &JsObject,
        selector: &str,
    ) -> Option<JsObject> {
        // Implementation of shadow-aware querySelector with proper shadow boundary respect
        Self::query_selector_with_shadow_boundaries(root, selector, QueryMode::First)
            .into_iter()
            .next()
    }

    /// Shadow DOM query selector all implementation
    pub fn shadow_aware_query_selector_all(
        root: &JsObject,
        selector: &str,
    ) -> Vec<JsObject> {
        // Implementation of shadow-aware querySelectorAll
        Self::query_selector_with_shadow_boundaries(root, selector, QueryMode::All)
    }

    /// Query elements with proper shadow DOM boundary handling
    fn query_selector_with_shadow_boundaries(
        root: &JsObject,
        selector: &str,
        mode: QueryMode,
    ) -> Vec<JsObject> {
        let mut matches = Vec::new();
        let parsed_selector = Self::parse_css_selector(selector);

        Self::query_recursive_with_boundaries(root, &parsed_selector, &mut matches, mode);

        matches
    }

    /// Recursive query with shadow boundary awareness
    fn query_recursive_with_boundaries(
        node: &JsObject,
        selector: &ParsedCSSSelector,
        matches: &mut Vec<JsObject>,
        mode: QueryMode,
    ) {
        // Check if current node matches
        if Self::matches_parsed_selector(node, selector) {
            matches.push(node.clone());
            if mode == QueryMode::First {
                return;
            }
        }

        // Get child nodes
        let children = Self::get_child_nodes(node);
        for child in children {
            Self::query_recursive_with_boundaries(&child, selector, matches, mode);
            if mode == QueryMode::First && !matches.is_empty() {
                return;
            }
        }

        // If this is an element with a shadow root, traverse into it
        // Shadow roots create a new search scope
        if let Some(shadow_root) = Self::get_shadow_root(node) {
            Self::query_recursive_with_boundaries(&shadow_root, selector, matches, mode);
        }
    }

    /// Helper methods for node introspection

    fn get_parent_node(node: &JsObject) -> Option<JsObject> {
        if let Some(element_data) = node.downcast_ref::<ElementData>() {
            element_data.get_parent_node()
        } else {
            None
        }
    }

    fn get_child_nodes(node: &JsObject) -> Vec<JsObject> {
        if let Some(element_data) = node.downcast_ref::<ElementData>() {
            element_data.get_children()
        } else if let Some(shadow_data) = node.downcast_ref::<ShadowRootData>() {
            shadow_data.fragment_data().get_children()
        } else {
            Vec::new()
        }
    }

    fn get_shadow_root(element: &JsObject) -> Option<JsObject> {
        if let Some(element_data) = element.downcast_ref::<ElementData>() {
            element_data.get_shadow_root()
        } else {
            None
        }
    }

    fn get_shadow_host(shadow_root: &JsObject) -> Option<JsObject> {
        if let Some(shadow_data) = shadow_root.downcast_ref::<ShadowRootData>() {
            shadow_data.get_host()
        } else {
            None
        }
    }

    fn is_slot_element(node: &JsObject) -> bool {
        node.downcast_ref::<HTMLSlotElementData>().is_some()
    }

    fn matches_selector(node: &JsObject, selector: &str) -> bool {
        // Simplified selector matching
        // In a full implementation, this would use a CSS selector parser

        if let Some(element_data) = node.downcast_ref::<ElementData>() {
            let tag_name = element_data.get_tag_name().to_lowercase();

            // Simple tag selector
            if selector == tag_name {
                return true;
            }

            // Simple class selector
            if selector.starts_with('.') {
                let class_selector = &selector[1..];
                let class_name = element_data.get_class_name();
                return class_name.split_whitespace().any(|c| c == class_selector);
            }

            // Simple ID selector
            if selector.starts_with('#') {
                let id_selector = &selector[1..];
                return element_data.get_id() == id_selector;
            }
        }

        false
    }

    /// Parse CSS selector into structured format
    fn parse_css_selector(selector: &str) -> ParsedCSSSelector {
        // Enhanced CSS selector parsing
        let trimmed = selector.trim();

        if trimmed.is_empty() {
            return ParsedCSSSelector::Invalid;
        }

        // Handle combinator selectors (descendant, child, adjacent sibling, general sibling)
        if let Some(combinator_result) = Self::parse_combinator_selector(trimmed) {
            return combinator_result;
        }

        // Handle pseudo-class selectors
        if trimmed.contains(':') && !trimmed.starts_with(':') {
            if let Some(pseudo_result) = Self::parse_pseudo_class_selector(trimmed) {
                return pseudo_result;
            }
        }

        // Handle attribute selectors
        if trimmed.contains('[') && trimmed.contains(']') {
            if let Some(attr_result) = Self::parse_attribute_selector(trimmed) {
                return attr_result;
            }
        }

        // Handle compound selectors (multiple classes, id + class, etc.)
        if Self::is_compound_selector(trimmed) {
            return Self::parse_compound_selector(trimmed);
        }

        // Simple selectors
        if trimmed == "*" {
            ParsedCSSSelector::Universal
        } else if trimmed.starts_with('#') {
            ParsedCSSSelector::Id(trimmed[1..].to_string())
        } else if trimmed.starts_with('.') {
            ParsedCSSSelector::Class(trimmed[1..].to_string())
        } else if Self::is_valid_tag_name(trimmed) {
            ParsedCSSSelector::Element(trimmed.to_string())
        } else {
            ParsedCSSSelector::Invalid
        }
    }

    /// Parse combinator selectors (space, >, +, ~)
    fn parse_combinator_selector(selector: &str) -> Option<ParsedCSSSelector> {
        // Look for combinators in order of precedence
        let combinators = [
            (" > ", CSSCombinator::Child),
            (" + ", CSSCombinator::AdjacentSibling),
            (" ~ ", CSSCombinator::GeneralSibling),
            ("  ", CSSCombinator::Descendant), // Multiple spaces
            (" ", CSSCombinator::Descendant),  // Single space
        ];

        for (combinator_str, combinator_type) in &combinators {
            if let Some(pos) = selector.find(combinator_str) {
                let left = selector[..pos].trim();
                let right = selector[pos + combinator_str.len()..].trim();

                if !left.is_empty() && !right.is_empty() {
                    let left_selector = Self::parse_css_selector(left);
                    let right_selector = Self::parse_css_selector(right);

                    return Some(ParsedCSSSelector::Combinator {
                        left: Box::new(left_selector),
                        combinator: *combinator_type,
                        right: Box::new(right_selector),
                    });
                }
            }
        }

        None
    }

    /// Parse pseudo-class selectors (:hover, :first-child, etc.)
    fn parse_pseudo_class_selector(selector: &str) -> Option<ParsedCSSSelector> {
        if let Some(colon_pos) = selector.find(':') {
            let base = &selector[..colon_pos];
            let pseudo = &selector[colon_pos + 1..];

            let base_selector = if base.is_empty() {
                ParsedCSSSelector::Universal
            } else {
                Self::parse_css_selector(base)
            };

            return Some(ParsedCSSSelector::PseudoClass {
                base: Box::new(base_selector),
                pseudo_class: pseudo.to_string(),
            });
        }

        None
    }

    /// Parse attribute selectors [attr], [attr=value], [attr~=value], etc.
    fn parse_attribute_selector(selector: &str) -> Option<ParsedCSSSelector> {
        if let Some(bracket_start) = selector.find('[') {
            if let Some(bracket_end) = selector.rfind(']') {
                let base = &selector[..bracket_start];
                let attr_part = &selector[bracket_start + 1..bracket_end];

                let base_selector = if base.is_empty() {
                    ParsedCSSSelector::Universal
                } else {
                    Self::parse_css_selector(base)
                };

                // Parse attribute conditions
                let attr_condition = Self::parse_attribute_condition(attr_part);

                return Some(ParsedCSSSelector::Attribute {
                    base: Box::new(base_selector),
                    attribute: attr_condition,
                });
            }
        }

        None
    }

    /// Parse attribute condition (name, name=value, name~=value, etc.)
    fn parse_attribute_condition(attr_str: &str) -> AttributeCondition {
        let attr_str = attr_str.trim();

        // Look for different attribute operators
        let operators = [
            ("~=", AttributeOperator::Contains),
            ("|=", AttributeOperator::Prefix),
            ("^=", AttributeOperator::StartsWith),
            ("$=", AttributeOperator::EndsWith),
            ("*=", AttributeOperator::Substring),
            ("=", AttributeOperator::Exact),
        ];

        for (op_str, op_type) in &operators {
            if let Some(pos) = attr_str.find(op_str) {
                let name = attr_str[..pos].trim().to_string();
                let value = attr_str[pos + op_str.len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();

                return AttributeCondition {
                    name,
                    operator: Some(*op_type),
                    value: Some(value),
                };
            }
        }

        // No operator found, just attribute existence
        AttributeCondition {
            name: attr_str.to_string(),
            operator: None,
            value: None,
        }
    }

    /// Check if selector is a compound selector (multiple simple selectors)
    fn is_compound_selector(selector: &str) -> bool {
        let has_id = selector.contains('#');
        let has_class = selector.contains('.');
        let has_element = selector.chars().any(|c| c.is_alphabetic());

        (has_id as u8) + (has_class as u8) + (has_element as u8) > 1
    }

    /// Parse compound selector (e.g., "div.class#id")
    fn parse_compound_selector(selector: &str) -> ParsedCSSSelector {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut chars = selector.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '#' | '.' => {
                    // Save previous part if any
                    if !current.is_empty() {
                        if Self::is_valid_tag_name(&current) {
                            parts.push(ParsedCSSSelector::Element(current.clone()));
                        }
                        current.clear();
                    }

                    // Start new part
                    current.push(ch);
                }
                _ => {
                    current.push(ch);

                    // Check if we're at the end or next char is a delimiter
                    if chars.peek().map_or(true, |&next| next == '#' || next == '.') {
                        // Process current part
                        if current.starts_with('#') {
                            parts.push(ParsedCSSSelector::Id(current[1..].to_string()));
                        } else if current.starts_with('.') {
                            parts.push(ParsedCSSSelector::Class(current[1..].to_string()));
                        } else if Self::is_valid_tag_name(&current) {
                            parts.push(ParsedCSSSelector::Element(current.clone()));
                        }
                        current.clear();
                    }
                }
            }
        }

        if parts.len() == 1 {
            parts.into_iter().next().unwrap()
        } else {
            ParsedCSSSelector::Compound(parts)
        }
    }

    /// Check if a string is a valid HTML tag name
    fn is_valid_tag_name(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    /// Match element against parsed selector
    fn matches_parsed_selector(element: &JsObject, selector: &ParsedCSSSelector) -> bool {
        match selector {
            ParsedCSSSelector::Universal => true,
            ParsedCSSSelector::Element(tag) => Self::matches_element_selector(element, tag),
            ParsedCSSSelector::Id(id) => Self::matches_id_selector(element, id),
            ParsedCSSSelector::Class(class) => Self::matches_class_selector(element, class),
            ParsedCSSSelector::Attribute { base, attribute } => {
                Self::matches_parsed_selector(element, base)
                    && Self::matches_attribute_condition(element, attribute)
            }
            ParsedCSSSelector::PseudoClass { base, pseudo_class } => {
                Self::matches_parsed_selector(element, base)
                    && Self::matches_pseudo_class(element, pseudo_class)
            }
            ParsedCSSSelector::Combinator { left, combinator, right } => {
                Self::matches_combinator_selector(element, left, *combinator, right)
            }
            ParsedCSSSelector::Compound(selectors) => {
                selectors.iter().all(|s| Self::matches_parsed_selector(element, s))
            }
            ParsedCSSSelector::Invalid => false,
        }
    }

    /// Match element tag name
    fn matches_element_selector(element: &JsObject, tag: &str) -> bool {
        if let Some(element_data) = element.downcast_ref::<ElementData>() {
            element_data.get_tag_name().to_lowercase() == tag.to_lowercase()
        } else {
            false
        }
    }

    /// Match element ID
    fn matches_id_selector(element: &JsObject, id: &str) -> bool {
        if let Some(element_data) = element.downcast_ref::<ElementData>() {
            element_data.get_id() == id
        } else {
            false
        }
    }

    /// Match element class
    fn matches_class_selector(element: &JsObject, class: &str) -> bool {
        if let Some(element_data) = element.downcast_ref::<ElementData>() {
            let class_name = element_data.get_class_name();
            class_name.split_whitespace().any(|c| c == class)
        } else {
            false
        }
    }

    /// Match attribute condition
    fn matches_attribute_condition(element: &JsObject, condition: &AttributeCondition) -> bool {
        if let Some(element_data) = element.downcast_ref::<ElementData>() {
            let attr_value = element_data.get_attribute(&condition.name);

            match (&condition.operator, &condition.value) {
                (None, None) => attr_value.is_some(), // Just existence
                (Some(op), Some(expected_value)) => {
                    if let Some(actual_value) = attr_value {
                        Self::matches_attribute_operator(&actual_value, expected_value, *op)
                    } else {
                        false
                    }
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// Match attribute operator
    fn matches_attribute_operator(actual: &str, expected: &str, op: AttributeOperator) -> bool {
        match op {
            AttributeOperator::Exact => actual == expected,
            AttributeOperator::Contains => {
                actual.split_whitespace().any(|word| word == expected)
            }
            AttributeOperator::Prefix => {
                actual == expected || actual.starts_with(&format!("{}-", expected))
            }
            AttributeOperator::StartsWith => actual.starts_with(expected),
            AttributeOperator::EndsWith => actual.ends_with(expected),
            AttributeOperator::Substring => actual.contains(expected),
        }
    }

    /// Match pseudo-class
    fn matches_pseudo_class(element: &JsObject, pseudo_class: &str) -> bool {
        // Basic pseudo-class matching
        match pseudo_class {
            "first-child" => Self::is_first_child(element),
            "last-child" => Self::is_last_child(element),
            "only-child" => Self::is_only_child(element),
            "empty" => Self::is_empty_element(element),
            _ => false, // Unknown pseudo-classes don't match
        }
    }

    /// Match combinator selector
    fn matches_combinator_selector(
        element: &JsObject,
        left: &ParsedCSSSelector,
        combinator: CSSCombinator,
        right: &ParsedCSSSelector,
    ) -> bool {
        // The right selector must match the current element
        if !Self::matches_parsed_selector(element, right) {
            return false;
        }

        match combinator {
            CSSCombinator::Descendant => Self::has_ancestor_matching(element, left),
            CSSCombinator::Child => Self::has_parent_matching(element, left),
            CSSCombinator::AdjacentSibling => Self::has_adjacent_sibling_matching(element, left),
            CSSCombinator::GeneralSibling => Self::has_preceding_sibling_matching(element, left),
        }
    }

    /// Check if element has ancestor matching selector
    fn has_ancestor_matching(element: &JsObject, selector: &ParsedCSSSelector) -> bool {
        let mut current = Self::get_parent_node(element);

        while let Some(parent) = current {
            if Self::matches_parsed_selector(&parent, selector) {
                return true;
            }
            current = Self::get_parent_node(&parent);
        }

        false
    }

    /// Check if element has parent matching selector
    fn has_parent_matching(element: &JsObject, selector: &ParsedCSSSelector) -> bool {
        if let Some(parent) = Self::get_parent_node(element) {
            Self::matches_parsed_selector(&parent, selector)
        } else {
            false
        }
    }

    /// Check if element has adjacent sibling matching selector
    fn has_adjacent_sibling_matching(element: &JsObject, selector: &ParsedCSSSelector) -> bool {
        if let Some(prev_sibling) = Self::get_previous_sibling(element) {
            Self::matches_parsed_selector(&prev_sibling, selector)
        } else {
            false
        }
    }

    /// Check if element has preceding sibling matching selector
    fn has_preceding_sibling_matching(element: &JsObject, selector: &ParsedCSSSelector) -> bool {
        if let Some(parent) = Self::get_parent_node(element) {
            let siblings = Self::get_child_nodes(&parent);
            let mut found_current = false;

            for sibling in siblings.iter().rev() {
                if JsObject::equals(sibling, element) {
                    found_current = true;
                    continue;
                }

                if found_current && Self::matches_parsed_selector(sibling, selector) {
                    return true;
                }
            }
        }

        false
    }

    /// Helper pseudo-class checks
    fn is_first_child(element: &JsObject) -> bool {
        if let Some(parent) = Self::get_parent_node(element) {
            let children = Self::get_child_nodes(&parent);
            children.first().map_or(false, |first| JsObject::equals(first, element))
        } else {
            false
        }
    }

    fn is_last_child(element: &JsObject) -> bool {
        if let Some(parent) = Self::get_parent_node(element) {
            let children = Self::get_child_nodes(&parent);
            children.last().map_or(false, |last| JsObject::equals(last, element))
        } else {
            false
        }
    }

    fn is_only_child(element: &JsObject) -> bool {
        if let Some(parent) = Self::get_parent_node(element) {
            let children = Self::get_child_nodes(&parent);
            children.len() == 1 && JsObject::equals(&children[0], element)
        } else {
            false
        }
    }

    fn is_empty_element(element: &JsObject) -> bool {
        let children = Self::get_child_nodes(element);
        children.is_empty()
    }

    /// Get previous sibling
    fn get_previous_sibling(element: &JsObject) -> Option<JsObject> {
        if let Some(parent) = Self::get_parent_node(element) {
            let siblings = Self::get_child_nodes(&parent);
            let mut prev = None;

            for sibling in siblings {
                if JsObject::equals(&sibling, element) {
                    return prev;
                }
                prev = Some(sibling);
            }
        }

        None
    }
}

/// Query mode for selector matching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QueryMode {
    First,
    All,
}

/// Parsed CSS selector representation
#[derive(Debug, Clone)]
enum ParsedCSSSelector {
    Universal,
    Element(String),
    Id(String),
    Class(String),
    Attribute {
        base: Box<ParsedCSSSelector>,
        attribute: AttributeCondition,
    },
    PseudoClass {
        base: Box<ParsedCSSSelector>,
        pseudo_class: String,
    },
    Combinator {
        left: Box<ParsedCSSSelector>,
        combinator: CSSCombinator,
        right: Box<ParsedCSSSelector>,
    },
    Compound(Vec<ParsedCSSSelector>),
    Invalid,
}

/// CSS combinator types
#[derive(Debug, Clone, Copy)]
enum CSSCombinator {
    Descendant, // space
    Child,      // >
    AdjacentSibling, // +
    GeneralSibling,  // ~
}

/// Attribute condition for attribute selectors
#[derive(Debug, Clone)]
struct AttributeCondition {
    name: String,
    operator: Option<AttributeOperator>,
    value: Option<String>,
}

/// Attribute matching operators
#[derive(Debug, Clone, Copy)]
enum AttributeOperator {
    Exact,      // =
    Contains,   // ~=
    Prefix,     // |=
    StartsWith, // ^=
    EndsWith,   // $=
    Substring,  // *=
}

/// Event path computation for Shadow DOM events
pub struct EventPath;

impl EventPath {
    /// Compute composed event path according to WHATWG spec
    /// https://dom.spec.whatwg.org/#concept-event-path
    pub fn compute_composed_path(target: &JsObject, composed: bool) -> Vec<JsObject> {
        let mut path = Vec::new();

        // Start from the target and walk up the shadow-including tree
        let mut current = Some(target.clone());

        while let Some(node) = current {
            // Check if we should include this node in the path
            if Self::should_include_in_path(&node, composed) {
                path.push(node.clone());
            }

            current = ShadowTreeTraversal::shadow_including_parent(&node);
        }

        path
    }

    /// Check if a node should be included in the event path
    fn should_include_in_path(node: &JsObject, composed: bool) -> bool {
        // If the event is composed, include all nodes
        if composed {
            return true;
        }

        // For non-composed events, check shadow root boundaries
        if let Some(shadow_data) = node.downcast_ref::<ShadowRootData>() {
            // Closed shadow roots block non-composed events
            match shadow_data.mode() {
                crate::builtins::shadow_root::ShadowRootMode::Closed => false,
                crate::builtins::shadow_root::ShadowRootMode::Open => true,
            }
        } else {
            true
        }
    }

    /// Retarget event for shadow DOM boundaries
    pub fn retarget_event(
        original_target: &JsObject,
        current_target: &JsObject,
    ) -> JsObject {
        // Implementation of event retargeting algorithm
        // This ensures events crossing shadow boundaries appear to come from the host

        let target_ancestors = ShadowTreeTraversal::shadow_including_ancestors(original_target);
        let current_ancestors = ShadowTreeTraversal::shadow_including_ancestors(current_target);

        // Find the lowest common shadow-including ancestor
        for target_ancestor in &target_ancestors {
            for current_ancestor in &current_ancestors {
                if JsObject::equals(target_ancestor, current_ancestor) {
                    // Found common ancestor, check if it's a shadow boundary
                    if Self::crosses_shadow_boundary(original_target, target_ancestor) {
                        return Self::find_shadow_host_for_retargeting(target_ancestor);
                    }
                }
            }
        }

        original_target.clone()
    }

    /// Check if event path crosses shadow boundary
    fn crosses_shadow_boundary(target: &JsObject, ancestor: &JsObject) -> bool {
        let ancestors = ShadowTreeTraversal::shadow_including_ancestors(target);

        for anc in ancestors {
            if let Some(_shadow_data) = anc.downcast_ref::<ShadowRootData>() {
                // Found a shadow root in the path
                if JsObject::equals(&anc, ancestor) {
                    return true;
                }
            }
        }

        false
    }

    /// Find shadow host for event retargeting
    fn find_shadow_host_for_retargeting(shadow_root: &JsObject) -> JsObject {
        if let Some(shadow_data) = shadow_root.downcast_ref::<ShadowRootData>() {
            shadow_data.get_host().unwrap_or_else(|| shadow_root.clone())
        } else {
            shadow_root.clone()
        }
    }
}