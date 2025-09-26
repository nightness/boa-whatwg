//! Declarative Shadow DOM parsing support
//!
//! Implementation of declarative Shadow DOM parsing according to WHATWG HTML spec
//! https://html.spec.whatwg.org/multipage/scripting.html#declarative-shadow-roots

use crate::{
    builtins::{
        element::ElementData,
        shadow_root::{ShadowRootData, ShadowRootMode},
        html_slot_element::HTMLSlotElementData,
    },
    object::JsObject,
    value::JsValue,
    Context, JsResult, JsNativeError,
};
use boa_gc::{Finalize, Trace, GcRefCell};
use std::collections::HashMap;

/// Declarative Shadow DOM parser for template elements
pub struct DeclarativeShadowDOMParser;

impl DeclarativeShadowDOMParser {
    /// Parse HTML content and process declarative shadow roots
    /// This implements the WHATWG algorithm for processing declarative shadow roots
    pub fn parse_and_process(
        html: &str,
        context: &mut Context,
    ) -> JsResult<Vec<JsObject>> {
        let mut elements = Vec::new();

        // Parse HTML and identify template elements with shadowrootmode
        let template_elements = Self::find_declarative_shadow_templates(html);

        for template_info in template_elements {
            let element = Self::process_declarative_shadow_template(template_info, context)?;
            elements.push(element);
        }

        Ok(elements)
    }

    /// Find template elements with shadowrootmode attributes
    fn find_declarative_shadow_templates(html: &str) -> Vec<DeclarativeShadowTemplateInfo> {
        let mut templates = Vec::new();
        let mut search_start = 0;

        while let Some(template_start) = html[search_start..].find("<template") {
            let absolute_start = search_start + template_start;

            // Find the end of the opening tag
            if let Some(tag_end) = html[absolute_start..].find('>') {
                let tag_end_absolute = absolute_start + tag_end;
                let opening_tag = &html[absolute_start..tag_end_absolute + 1];

                // Check for shadowrootmode attribute
                if let Some(shadow_mode) = Self::extract_shadow_root_mode(opening_tag) {
                    // Find the closing template tag
                    if let Some(template_end) = html[tag_end_absolute..].find("</template>") {
                        let template_end_absolute = tag_end_absolute + template_end;
                        let template_content = &html[tag_end_absolute + 1..template_end_absolute];

                        let template_info = DeclarativeShadowTemplateInfo {
                            shadow_mode,
                            content: template_content.to_string(),
                            attributes: Self::parse_template_attributes(opening_tag),
                            start_pos: absolute_start,
                            end_pos: template_end_absolute + 11, // +11 for "</template>"
                        };

                        templates.push(template_info);
                        search_start = template_end_absolute + 11;
                    } else {
                        // Malformed template, skip
                        search_start = tag_end_absolute + 1;
                    }
                } else {
                    // No shadowrootmode attribute, skip this template
                    search_start = tag_end_absolute + 1;
                }
            } else {
                // Malformed template start
                break;
            }
        }

        templates
    }

    /// Extract shadowrootmode attribute value from template tag
    fn extract_shadow_root_mode(opening_tag: &str) -> Option<ShadowRootMode> {
        // Look for shadowrootmode attribute
        let patterns = [
            r#"shadowrootmode="open""#,
            r#"shadowrootmode='open'"#,
            r#"shadowrootmode=open"#,
            r#"shadowrootmode="closed""#,
            r#"shadowrootmode='closed'"#,
            r#"shadowrootmode=closed"#,
        ];

        for pattern in &patterns {
            if opening_tag.to_lowercase().contains(&pattern.to_lowercase()) {
                if pattern.contains("open") {
                    return Some(ShadowRootMode::Open);
                } else if pattern.contains("closed") {
                    return Some(ShadowRootMode::Closed);
                }
            }
        }

        // Also handle cases with extra whitespace
        if let Some(attr_start) = opening_tag.to_lowercase().find("shadowrootmode") {
            let attr_part = &opening_tag[attr_start..];
            if let Some(equals_pos) = attr_part.find('=') {
                let value_part = &attr_part[equals_pos + 1..].trim();
                let cleaned_value = value_part
                    .trim_start_matches('"')
                    .trim_start_matches('\'')
                    .trim_end_matches('"')
                    .trim_end_matches('\'')
                    .trim();

                match cleaned_value.to_lowercase().as_str() {
                    "open" => return Some(ShadowRootMode::Open),
                    "closed" => return Some(ShadowRootMode::Closed),
                    _ => return None,
                }
            }
        }

        None
    }

    /// Parse other attributes from template tag
    fn parse_template_attributes(opening_tag: &str) -> HashMap<String, String> {
        let mut attributes = HashMap::new();

        // Simple attribute parsing - in production would use proper HTML parser
        let attr_patterns = [
            "shadowrootclonable",
            "shadowrootserializable",
            "shadowrootdelegatesfocus",
        ];

        for pattern in &attr_patterns {
            if opening_tag.to_lowercase().contains(pattern) {
                // Check if it's a boolean attribute or has a value
                if let Some(attr_pos) = opening_tag.to_lowercase().find(pattern) {
                    let after_attr = &opening_tag[attr_pos + pattern.len()..];
                    if after_attr.trim_start().starts_with('=') {
                        // Has a value
                        if let Some(value) = Self::extract_attribute_value(after_attr) {
                            attributes.insert(pattern.to_string(), value);
                        } else {
                            attributes.insert(pattern.to_string(), "true".to_string());
                        }
                    } else {
                        // Boolean attribute
                        attributes.insert(pattern.to_string(), "true".to_string());
                    }
                }
            }
        }

        attributes
    }

    /// Extract attribute value from attribute string
    fn extract_attribute_value(attr_string: &str) -> Option<String> {
        let trimmed = attr_string.trim_start();
        if !trimmed.starts_with('=') {
            return None;
        }

        let value_part = trimmed[1..].trim_start();
        if value_part.starts_with('"') {
            // Double-quoted value
            if let Some(end_quote) = value_part[1..].find('"') {
                return Some(value_part[1..end_quote + 1].to_string());
            }
        } else if value_part.starts_with('\'') {
            // Single-quoted value
            if let Some(end_quote) = value_part[1..].find('\'') {
                return Some(value_part[1..end_quote + 1].to_string());
            }
        } else {
            // Unquoted value - find next space or tag end
            let end_pos = value_part
                .find(' ')
                .or_else(|| value_part.find('\t'))
                .or_else(|| value_part.find('\n'))
                .or_else(|| value_part.find('\r'))
                .or_else(|| value_part.find('>'))
                .unwrap_or(value_part.len());
            return Some(value_part[..end_pos].to_string());
        }

        None
    }

    /// Process a declarative shadow template and create shadow root
    fn process_declarative_shadow_template(
        template_info: DeclarativeShadowTemplateInfo,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        // Create host element (usually the element containing the template)
        let host_element = Self::create_host_element_for_template(context)?;

        // Determine shadow root options from attributes
        let clonable = template_info.attributes.get("shadowrootclonable")
            .map(|v| v == "true" || v.is_empty())
            .unwrap_or(false);

        let serializable = template_info.attributes.get("shadowrootserializable")
            .map(|v| v == "true" || v.is_empty())
            .unwrap_or(false);

        let delegates_focus = template_info.attributes.get("shadowrootdelegatesfocus")
            .map(|v| v == "true" || v.is_empty())
            .unwrap_or(false);

        // Create shadow root with specified mode and options
        let shadow_root = Self::create_shadow_root_from_template(
            template_info.shadow_mode,
            clonable,
            serializable,
            delegates_focus,
            context,
        )?;

        // Set the host relationship
        if let Some(shadow_data) = shadow_root.downcast_ref::<ShadowRootData>() {
            shadow_data.set_host(host_element.clone());
        }

        if let Some(host_data) = host_element.downcast_ref::<ElementData>() {
            host_data.attach_shadow_root(shadow_root.clone());
        }

        // Parse and set the shadow tree content
        Self::populate_shadow_tree_content(&shadow_root, &template_info.content, context)?;

        // Process any nested declarative shadow roots
        Self::process_nested_declarative_shadows(&shadow_root, &template_info.content, context)?;

        Ok(host_element)
    }

    /// Create a host element for the declarative shadow root
    fn create_host_element_for_template(context: &mut Context) -> JsResult<JsObject> {
        // Create a generic element to serve as the shadow host
        // In a real implementation, this would be the element that contained the template
        let element_data = ElementData::with_tag_name("div".to_string());

        let realm = context.realm().clone();
        let prototype = realm.intrinsics().constructors().object().prototype();

        let element = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            element_data,
        );

        Ok(element)
    }

    /// Create shadow root from template attributes
    fn create_shadow_root_from_template(
        mode: ShadowRootMode,
        clonable: bool,
        serializable: bool,
        delegates_focus: bool,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        let shadow_data = ShadowRootData::new(mode, clonable, serializable, delegates_focus);

        let realm = context.realm().clone();
        let prototype = realm.intrinsics().constructors().object().prototype();

        let shadow_root = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            shadow_data,
        );

        Ok(shadow_root)
    }

    /// Populate shadow tree with parsed content from template
    fn populate_shadow_tree_content(
        shadow_root: &JsObject,
        content: &str,
        context: &mut Context,
    ) -> JsResult<()> {
        if let Some(shadow_data) = shadow_root.downcast_ref::<ShadowRootData>() {
            // Set the innerHTML which will trigger CSS scoping
            shadow_data.set_inner_html(content)
                .map_err(|e| JsNativeError::error().with_message(e))?;

            // Parse content and create actual DOM nodes
            let elements = Self::parse_content_elements(content, context)?;

            // Add elements to shadow root
            for element in elements {
                shadow_data.fragment_data().append_impl(element)
                    .map_err(|e| JsNativeError::error().with_message(e))?;
            }
        }

        Ok(())
    }

    /// Parse content string and create DOM elements
    fn parse_content_elements(content: &str, context: &mut Context) -> JsResult<Vec<JsObject>> {
        let mut elements = Vec::new();

        // Simple HTML parsing - in production would use proper HTML parser
        let element_tags = Self::extract_element_tags(content);

        for tag_info in element_tags {
            let element = Self::create_element_from_tag_info(tag_info, context)?;
            elements.push(element);
        }

        Ok(elements)
    }

    /// Extract element tag information from HTML content
    fn extract_element_tags(content: &str) -> Vec<ElementTagInfo> {
        let mut tags = Vec::new();
        let mut search_start = 0;

        while let Some(tag_start) = content[search_start..].find('<') {
            let absolute_start = search_start + tag_start;

            // Skip comments and other non-element tags
            if content[absolute_start..].starts_with("<!--") {
                if let Some(comment_end) = content[absolute_start..].find("-->") {
                    search_start = absolute_start + comment_end + 3;
                    continue;
                }
            }

            if let Some(tag_end) = content[absolute_start..].find('>') {
                let tag_end_absolute = absolute_start + tag_end;
                let tag_content = &content[absolute_start + 1..tag_end_absolute];

                // Parse tag name and attributes
                if let Some(space_pos) = tag_content.find(' ') {
                    let tag_name = tag_content[..space_pos].trim().to_string();
                    let attributes_str = tag_content[space_pos..].trim();
                    let attributes = Self::parse_element_attributes(attributes_str);

                    let tag_info = ElementTagInfo {
                        tag_name,
                        attributes,
                        is_self_closing: tag_content.ends_with('/'),
                        start_pos: absolute_start,
                        end_pos: tag_end_absolute + 1,
                    };

                    tags.push(tag_info);
                } else {
                    // Tag without attributes
                    let tag_name = tag_content.trim_end_matches('/').trim().to_string();
                    let tag_info = ElementTagInfo {
                        tag_name,
                        attributes: HashMap::new(),
                        is_self_closing: tag_content.ends_with('/'),
                        start_pos: absolute_start,
                        end_pos: tag_end_absolute + 1,
                    };

                    tags.push(tag_info);
                }

                search_start = tag_end_absolute + 1;
            } else {
                break;
            }
        }

        tags
    }

    /// Parse element attributes from attribute string
    fn parse_element_attributes(attr_str: &str) -> HashMap<String, String> {
        let mut attributes = HashMap::new();

        // Simple attribute parsing
        let parts: Vec<&str> = attr_str.split_whitespace().collect();
        for part in parts {
            if let Some(equals_pos) = part.find('=') {
                let name = part[..equals_pos].trim().to_string();
                let value = part[equals_pos + 1..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                attributes.insert(name, value);
            } else {
                // Boolean attribute
                attributes.insert(part.trim().to_string(), "true".to_string());
            }
        }

        attributes
    }

    /// Create element from tag information
    fn create_element_from_tag_info(
        tag_info: ElementTagInfo,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        // Check if this is a slot element
        if tag_info.tag_name.to_lowercase() == "slot" {
            Self::create_slot_element(tag_info, context)
        } else {
            Self::create_regular_element(tag_info, context)
        }
    }

    /// Create a slot element
    fn create_slot_element(
        tag_info: ElementTagInfo,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        let name = tag_info.attributes.get("name")
            .cloned()
            .unwrap_or_default();

        let slot_data = HTMLSlotElementData::new();
        if !name.is_empty() {
            slot_data.set_name(name);
        }

        let realm = context.realm().clone();
        let prototype = realm.intrinsics().constructors().object().prototype();

        let slot_element = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            slot_data,
        );

        Ok(slot_element)
    }

    /// Create a regular element
    fn create_regular_element(
        tag_info: ElementTagInfo,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        let element_data = ElementData::with_tag_name(tag_info.tag_name);

        // Set attributes
        for (name, value) in tag_info.attributes {
            element_data.set_attribute(name, value);
        }

        let realm = context.realm().clone();
        let prototype = realm.intrinsics().constructors().object().prototype();

        let element = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            element_data,
        );

        Ok(element)
    }

    /// Process nested declarative shadow roots within content
    fn process_nested_declarative_shadows(
        parent_shadow_root: &JsObject,
        content: &str,
        context: &mut Context,
    ) -> JsResult<()> {
        // Look for nested template elements with shadowrootmode
        let nested_templates = Self::find_declarative_shadow_templates(content);

        for template_info in nested_templates {
            let _nested_element = Self::process_declarative_shadow_template(template_info, context)?;
            // In a full implementation, would properly attach nested shadow roots
        }

        Ok(())
    }

    /// Validate declarative shadow root according to WHATWG rules
    pub fn validate_declarative_shadow_root(
        template_element: &JsObject,
        context: &mut Context,
    ) -> JsResult<bool> {
        // Check if element can have a shadow root (from element validation)
        if let Some(element_data) = template_element.downcast_ref::<ElementData>() {
            use crate::builtins::element::can_have_shadow_root;

            if !can_have_shadow_root(&element_data) {
                return Ok(false);
            }

            // Check if element already has a shadow root
            if element_data.get_shadow_root().is_some() {
                return Ok(false);
            }

            // Additional declarative shadow DOM specific validations
            // - Template must be a direct child of the host
            // - Only one template with shadowrootmode per element
            // - etc.

            return Ok(true);
        }

        Ok(false)
    }
}

/// Information about a declarative shadow template found in HTML
#[derive(Debug, Clone)]
struct DeclarativeShadowTemplateInfo {
    shadow_mode: ShadowRootMode,
    content: String,
    attributes: HashMap<String, String>,
    start_pos: usize,
    end_pos: usize,
}

/// Information about an element tag found during parsing
#[derive(Debug, Clone)]
struct ElementTagInfo {
    tag_name: String,
    attributes: HashMap<String, String>,
    is_self_closing: bool,
    start_pos: usize,
    end_pos: usize,
}

/// HTML processing utilities for declarative Shadow DOM
pub struct DeclarativeShadowDOMHTMLProcessor;

impl DeclarativeShadowDOMHTMLProcessor {
    /// Process complete HTML document and handle all declarative shadow roots
    pub fn process_document_html(
        html: &str,
        context: &mut Context,
    ) -> JsResult<String> {
        // Find all declarative shadow templates
        let templates = DeclarativeShadowDOMParser::find_declarative_shadow_templates(html);

        if templates.is_empty() {
            return Ok(html.to_string());
        }

        let mut processed_html = html.to_string();

        // Process templates in reverse order to maintain position indices
        for template in templates.into_iter().rev() {
            // Process the template and create shadow root
            let _host_element = DeclarativeShadowDOMParser::process_declarative_shadow_template(template.clone(), context)?;

            // Replace the template element in HTML with processed content
            // In a real implementation, this would modify the DOM rather than string manipulation
            processed_html.replace_range(template.start_pos..template.end_pos, "<!-- declarative shadow root processed -->");
        }

        Ok(processed_html)
    }

    /// Check if HTML contains declarative shadow DOM
    pub fn contains_declarative_shadow_dom(html: &str) -> bool {
        html.to_lowercase().contains("shadowrootmode")
    }

    /// Extract shadow root templates for server-side processing
    pub fn extract_shadow_templates(html: &str) -> Vec<String> {
        let templates = DeclarativeShadowDOMParser::find_declarative_shadow_templates(html);
        templates.into_iter().map(|t| t.content).collect()
    }
}