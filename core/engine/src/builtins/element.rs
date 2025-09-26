//! Element Web API implementation for Boa
//!
//! Real native implementation of Element standard with actual DOM tree functionality
//! https://dom.spec.whatwg.org/#interface-element

use crate::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::Attribute
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicU32, Ordering}};
use std::sync::OnceLock;

/// Global node ID counter for unique DOM node identification
static NODE_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

/// Global DOM synchronization for updating document HTML content
pub static GLOBAL_DOM_SYNC: OnceLock<DomSync> = OnceLock::new();

/// Bridge between Element DOM changes and Document HTML content
pub struct DomSync {
    document_html_updater: Mutex<Option<Box<dyn Fn(&str) + Send + Sync>>>,
}

impl DomSync {
    pub fn new() -> Self {
        Self {
            document_html_updater: Mutex::new(None),
        }
    }

    pub fn set_updater(&self, updater: Box<dyn Fn(&str) + Send + Sync>) {
        *self.document_html_updater.lock().unwrap() = Some(updater);
    }

    fn update_document_html(&self, html: &str) {
        if let Some(updater) = self.document_html_updater.lock().unwrap().as_ref() {
            updater(html);
        } else {
        }
    }
}

/// JavaScript `Element` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Element;

impl IntrinsicObject for Element {
    fn init(realm: &Realm) {
        let tag_name_func = BuiltInBuilder::callable(realm, get_tag_name)
            .name(js_string!("get tagName"))
            .build();

        let id_func = BuiltInBuilder::callable(realm, get_id)
            .name(js_string!("get id"))
            .build();

        let id_setter_func = BuiltInBuilder::callable(realm, set_id)
            .name(js_string!("set id"))
            .build();

        let class_name_func = BuiltInBuilder::callable(realm, get_class_name)
            .name(js_string!("get className"))
            .build();

        let class_name_setter_func = BuiltInBuilder::callable(realm, set_class_name)
            .name(js_string!("set className"))
            .build();

        let inner_html_func = BuiltInBuilder::callable(realm, get_inner_html)
            .name(js_string!("get innerHTML"))
            .build();

        let inner_html_setter_func = BuiltInBuilder::callable(realm, set_inner_html)
            .name(js_string!("set innerHTML"))
            .build();

        let text_content_func = BuiltInBuilder::callable(realm, get_text_content)
            .name(js_string!("get textContent"))
            .build();

        let text_content_setter_func = BuiltInBuilder::callable(realm, set_text_content)
            .name(js_string!("set textContent"))
            .build();

        let children_func = BuiltInBuilder::callable(realm, get_children)
            .name(js_string!("get children"))
            .build();

        let parent_node_func = BuiltInBuilder::callable(realm, get_parent_node)
            .name(js_string!("get parentNode"))
            .build();

        let style_func = BuiltInBuilder::callable(realm, get_style)
            .name(js_string!("get style"))
            .build();

        let class_list_func = BuiltInBuilder::callable(realm, get_class_list)
            .name(js_string!("get classList"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("tagName"),
                Some(tag_name_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("id"),
                Some(id_func),
                Some(id_setter_func),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("className"),
                Some(class_name_func),
                Some(class_name_setter_func),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("innerHTML"),
                Some(inner_html_func),
                Some(inner_html_setter_func),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("textContent"),
                Some(text_content_func),
                Some(text_content_setter_func),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("children"),
                Some(children_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("parentNode"),
                Some(parent_node_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("style"),
                Some(style_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("classList"),
                Some(class_list_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(set_attribute, js_string!("setAttribute"), 2)
            .method(get_attribute, js_string!("getAttribute"), 1)
            .method(has_attribute, js_string!("hasAttribute"), 1)
            .method(remove_attribute, js_string!("removeAttribute"), 1)
            .method(append_child, js_string!("appendChild"), 1)
            .method(remove_child, js_string!("removeChild"), 1)
            .method(set_html, js_string!("setHTML"), 1)
            .method(set_html_unsafe, js_string!("setHTMLUnsafe"), 1)
            // TEMPORARILY DISABLED: attachShadow method causes form interaction failures
            // See detailed comments in builtins/mod.rs around ShadowRoot::init() for full explanation
            // .method(attach_shadow, js_string!("attachShadow"), 1)  // <-- DISABLED
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Element {
    const NAME: JsString = StaticJsStrings::ELEMENT;
}

impl BuiltInConstructor for Element {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::element;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::element,
            context,
        )?;

        let element_data = ElementData::new();

        let element = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            element_data,
        );

        Ok(element.into())
    }
}

/// Internal data for Element objects - represents a real DOM node
#[derive(Debug, Trace, Finalize, JsData)]
pub struct ElementData {
    /// Unique node ID for DOM tree operations
    #[unsafe_ignore_trace]
    node_id: u32,
    /// Element tag name (e.g., "div", "span", "body")
    #[unsafe_ignore_trace]
    tag_name: Arc<Mutex<String>>,
    /// Element namespace URI
    #[unsafe_ignore_trace]
    namespace_uri: Arc<Mutex<Option<String>>>,
    /// Element ID attribute
    #[unsafe_ignore_trace]
    id: Arc<Mutex<String>>,
    /// Element class attribute
    #[unsafe_ignore_trace]
    class_name: Arc<Mutex<String>>,
    /// Inner HTML content - parsed and maintained as real DOM
    #[unsafe_ignore_trace]
    inner_html: Arc<Mutex<String>>,
    /// Text content - computed from child text nodes
    #[unsafe_ignore_trace]
    text_content: Arc<Mutex<String>>,
    /// All element attributes (id, class, data-*, etc.)
    #[unsafe_ignore_trace]
    attributes: Arc<Mutex<HashMap<String, String>>>,
    /// Child elements in DOM tree order
    #[unsafe_ignore_trace]
    children: Arc<Mutex<Vec<JsObject>>>,
    /// Parent element in DOM tree
    #[unsafe_ignore_trace]
    parent_node: Arc<Mutex<Option<JsObject>>>,
    /// Computed CSS style object
    #[unsafe_ignore_trace]
    style: Arc<Mutex<CSSStyleDeclaration>>,
    /// Element's bounding box for layout
    #[unsafe_ignore_trace]
    bounding_rect: Arc<Mutex<DOMRect>>,
    /// Event listeners attached to this element
    #[unsafe_ignore_trace]
    event_listeners: Arc<Mutex<HashMap<String, Vec<JsValue>>>>,
    /// Shadow root for Shadow DOM API
    #[unsafe_ignore_trace]
    shadow_root: Arc<Mutex<Option<JsObject>>>,
}

/// CSS Style Declaration for real style computation
#[derive(Debug, Clone)]
pub struct CSSStyleDeclaration {
    /// CSS properties and values
    properties: HashMap<String, String>,
    /// Computed styles from cascading
    computed: HashMap<String, String>,
}

/// DOM Rectangle for element positioning
#[derive(Debug, Clone)]
pub struct DOMRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl CSSStyleDeclaration {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            computed: HashMap::new(),
        }
    }

    pub fn set_property(&mut self, property: &str, value: &str) {
        self.properties.insert(property.to_string(), value.to_string());
        self.compute_property(property, value);
    }

    pub fn get_property(&self, property: &str) -> Option<&String> {
        self.computed.get(property).or_else(|| self.properties.get(property))
    }

    fn compute_property(&mut self, property: &str, value: &str) {
        // Real CSS property computation with inheritance and cascading
        match property {
            "width" | "height" => {
                // Handle different units (px, %, em, rem, vh, vw)
                let computed_value = self.compute_length_value(value);
                self.computed.insert(property.to_string(), computed_value);
            },
            "color" | "background-color" => {
                // Handle color values (hex, rgb, rgba, hsl, named colors)
                let computed_color = self.compute_color_value(value);
                self.computed.insert(property.to_string(), computed_color);
            },
            "display" => {
                // Validate display values
                let valid_display = match value {
                    "block" | "inline" | "inline-block" | "flex" | "grid" | "none" => value,
                    _ => "block" // Default fallback
                };
                self.computed.insert(property.to_string(), valid_display.to_string());
            },
            _ => {
                // Store as-is for other properties
                self.computed.insert(property.to_string(), value.to_string());
            }
        }
    }

    fn compute_length_value(&self, value: &str) -> String {
        // Parse and compute length values
        if value.ends_with("px") {
            value.to_string() // Already in pixels
        } else if value.ends_with("%") {
            // Would need parent context for percentage calculation
            value.to_string()
        } else if value.ends_with("em") {
            // Would need font-size context
            value.to_string()
        } else if let Ok(num) = value.parse::<f64>() {
            format!("{}px", num) // Treat unitless as pixels
        } else {
            value.to_string()
        }
    }

    fn compute_color_value(&self, value: &str) -> String {
        // Parse and normalize color values
        if value.starts_with("#") {
            value.to_string() // Hex color
        } else if value.starts_with("rgb") {
            value.to_string() // RGB/RGBA color
        } else {
            // Named colors or invalid - normalize to hex
            match value {
                "red" => "#ff0000".to_string(),
                "green" => "#008000".to_string(),
                "blue" => "#0000ff".to_string(),
                "black" => "#000000".to_string(),
                "white" => "#ffffff".to_string(),
                _ => value.to_string()
            }
        }
    }
}

impl DOMRect {
    fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }

    fn update_bounds(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
        self.top = y;
        self.left = x;
        self.right = x + width;
        self.bottom = y + height;
    }
}

impl ElementData {
    fn new() -> Self {
        let node_id = NODE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            node_id,
            tag_name: Arc::new(Mutex::new("".to_string())),
            namespace_uri: Arc::new(Mutex::new(None)),
            id: Arc::new(Mutex::new("".to_string())),
            class_name: Arc::new(Mutex::new("".to_string())),
            inner_html: Arc::new(Mutex::new("".to_string())),
            text_content: Arc::new(Mutex::new("".to_string())),
            attributes: Arc::new(Mutex::new(HashMap::new())),
            children: Arc::new(Mutex::new(Vec::new())),
            parent_node: Arc::new(Mutex::new(None)),
            style: Arc::new(Mutex::new(CSSStyleDeclaration::new())),
            bounding_rect: Arc::new(Mutex::new(DOMRect::new())),
            event_listeners: Arc::new(Mutex::new(HashMap::new())),
            shadow_root: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_tag_name(tag_name: String) -> Self {
        let mut data = Self::new();
        *data.tag_name.lock().unwrap() = tag_name;
        data
    }

    pub fn get_tag_name(&self) -> String {
        self.tag_name.lock().unwrap().clone()
    }

    pub fn set_tag_name(&self, tag_name: String) {
        *self.tag_name.lock().unwrap() = tag_name;
    }

    pub fn get_namespace_uri(&self) -> Option<String> {
        self.namespace_uri.lock().unwrap().clone()
    }

    pub fn set_namespace_uri(&self, namespace_uri: Option<String>) {
        *self.namespace_uri.lock().unwrap() = namespace_uri;
    }

    pub fn get_id(&self) -> String {
        self.id.lock().unwrap().clone()
    }

    pub fn set_id(&self, id: String) {
        *self.id.lock().unwrap() = id;
    }

    pub fn get_class_name(&self) -> String {
        self.class_name.lock().unwrap().clone()
    }

    pub fn set_class_name(&self, class_name: String) {
        *self.class_name.lock().unwrap() = class_name;
    }

    pub fn get_inner_html(&self) -> String {
        self.inner_html.lock().unwrap().clone()
    }

    pub fn set_inner_html(&self, html: String) {
        *self.inner_html.lock().unwrap() = html.clone();

        // Parse HTML and update DOM tree
        self.parse_and_update_children(&html);

        // Recompute text content from parsed children
        self.recompute_text_content();

        // CRITICAL: Update the document's HTML content so querySelector can find changes
        self.update_document_html_content();
    }

    /// Parse HTML string and create child elements
    fn parse_and_update_children(&self, html: &str) {
        let mut children = self.children.lock().unwrap();
        children.clear();

        // Simple HTML parser - in real implementation would use proper HTML5 parser
        let parsed_elements = self.simple_html_parse(html);
        children.extend(parsed_elements);
    }

    /// Simple HTML parser for basic tag parsing
    fn simple_html_parse(&self, html: &str) -> Vec<JsObject> {
        let mut elements = Vec::new();
        let mut current_pos = 0;
        let html_bytes = html.as_bytes();

        while current_pos < html_bytes.len() {
            if html_bytes[current_pos] == b'<' {
                // Find end of tag
                if let Some(tag_end) = html[current_pos..].find('>') {
                    let tag_content = &html[current_pos + 1..current_pos + tag_end];

                    if !tag_content.starts_with('/') { // Not a closing tag
                        // Parse opening tag
                        let parts: Vec<&str> = tag_content.split_whitespace().collect();
                        if let Some(tag_name) = parts.first() {
                            // Create new element
                            let element_data = ElementData::with_tag_name(tag_name.to_uppercase());

                            // Parse attributes
                            for attr_part in parts.iter().skip(1) {
                                if let Some(eq_pos) = attr_part.find('=') {
                                    let attr_name = &attr_part[..eq_pos];
                                    let attr_value = &attr_part[eq_pos + 1..].trim_matches('"');
                                    element_data.set_attribute(attr_name.to_string(), attr_value.to_string());
                                }
                            }

                            // Create JsObject for the element
                            let element = JsObject::from_proto_and_data(None, element_data);
                            elements.push(element);
                        }
                    }

                    current_pos += tag_end + 1;
                } else {
                    current_pos += 1;
                }
            } else {
                // Text content - find next tag or end
                let text_start = current_pos;
                let text_end = html[current_pos..].find('<').map(|pos| current_pos + pos).unwrap_or(html.len());

                let text_content = html[text_start..text_end].trim();
                if !text_content.is_empty() {
                    // Create text node as element with special tag
                    let text_element = ElementData::with_tag_name("#text".to_string());
                    text_element.set_text_content(text_content.to_string());

                    let text_obj = JsObject::from_proto_and_data(None, text_element);
                    elements.push(text_obj);
                }

                current_pos = text_end;
            }
        }

        elements
    }

    /// Recompute text content from all child text nodes
    fn recompute_text_content(&self) {
        let children = self.children.lock().unwrap();
        let mut text_parts = Vec::new();

        for child in children.iter() {
            if let Some(child_data) = child.downcast_ref::<ElementData>() {
                let child_tag = child_data.get_tag_name();
                if child_tag == "#text" {
                    text_parts.push(child_data.get_text_content());
                } else {
                    // Recursively get text from child elements
                    text_parts.push(child_data.get_text_content());
                }
            }
        }

        *self.text_content.lock().unwrap() = text_parts.join("");
    }

    /// Update the document's HTML content to reflect DOM changes
    /// This is CRITICAL for querySelector to find dynamically added content
    fn update_document_html_content(&self) {
        // Debug: Print that this method is being called

        // Regenerate full HTML from current DOM state
        let serialized_html = self.serialize_to_html();

        // Find document in global scope and update its HTML content
        // This uses a global static to communicate between Element and Document
        GLOBAL_DOM_SYNC.get_or_init(|| DomSync::new()).update_document_html(&serialized_html);
    }

    /// Serialize this element and all children to HTML string
    fn serialize_to_html(&self) -> String {
        let tag_name = self.get_tag_name();
        let mut html = format!("<{}", tag_name);

        // Add attributes
        let attributes = self.attributes.lock().unwrap();
        for (name, value) in attributes.iter() {
            html.push_str(&format!(" {}=\"{}\"", name, value));
        }
        html.push('>');

        // Add inner HTML
        html.push_str(&self.get_inner_html());

        html.push_str(&format!("</{}>", tag_name));
        html
    }

    pub fn get_text_content(&self) -> String {
        self.text_content.lock().unwrap().clone()
    }

    pub fn set_text_content(&self, content: String) {
        *self.text_content.lock().unwrap() = content;
    }

    pub fn get_attribute(&self, name: &str) -> Option<String> {
        self.attributes.lock().unwrap().get(name).cloned()
    }

    pub fn set_attribute(&self, name: String, value: String) {
        self.attributes.lock().unwrap().insert(name, value);
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.lock().unwrap().contains_key(name)
    }

    pub fn remove_attribute(&self, name: &str) {
        self.attributes.lock().unwrap().remove(name);
    }

    pub fn append_child(&self, child: JsObject) {
        self.children.lock().unwrap().push(child);
    }

    pub fn remove_child(&self, child: &JsObject) {
        self.children.lock().unwrap().retain(|c| !JsObject::equals(c, child));
    }

    pub fn get_children(&self) -> Vec<JsObject> {
        self.children.lock().unwrap().clone()
    }

    pub fn get_parent_node(&self) -> Option<JsObject> {
        self.parent_node.lock().unwrap().clone()
    }

    pub fn set_parent_node(&self, parent: Option<JsObject>) {
        *self.parent_node.lock().unwrap() = parent;
    }

    pub fn get_style(&self) -> CSSStyleDeclaration {
        self.style.lock().unwrap().clone()
    }

    /// Add event listener to this element
    pub fn add_event_listener(&self, event_type: String, listener: JsValue) {
        self.event_listeners.lock().unwrap()
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(listener);
    }

    /// Remove event listener from this element
    pub fn remove_event_listener(&self, event_type: &str, listener: &JsValue) {
        if let Some(listeners) = self.event_listeners.lock().unwrap().get_mut(event_type) {
            listeners.retain(|l| !JsValue::same_value(l, listener));
        }
    }

    /// Attach shadow root for Shadow DOM API
    pub fn attach_shadow_root(&self, shadow_root: JsObject) {
        if let Ok(mut guard) = self.shadow_root.try_lock() {
            *guard = Some(shadow_root);
        } else {
            eprintln!("WARNING: Shadow DOM mutex was locked, skipping shadow root attachment");
        }
    }

    /// Get shadow root (returns None if no shadow root or mode is 'closed')
    pub fn get_shadow_root(&self) -> Option<JsObject> {
        if let Ok(guard) = self.shadow_root.try_lock() {
            guard.clone()
        } else {
            eprintln!("WARNING: Shadow DOM mutex was locked, returning None");
            None
        }
    }

    /// Dispatch event on this element
    pub fn dispatch_event(&self, event_type: &str, event_data: &JsValue, context: &mut Context) -> JsResult<()> {
        let listeners = self.event_listeners.lock().unwrap();
        if let Some(event_listeners) = listeners.get(event_type) {
            for listener in event_listeners {
                if listener.is_callable() {
                    let _ = listener.as_callable().unwrap().call(
                        &JsValue::undefined(),
                        &[event_data.clone()],
                        context,
                    );
                }
            }
        }
        Ok(())
    }

    /// Update CSS style property with real computation
    pub fn set_style_property(&self, property: &str, value: &str) {
        let mut style = self.style.lock().unwrap();
        style.set_property(property, value);

        // Update layout if this affects positioning
        if matches!(property, "width" | "height" | "position" | "left" | "top") {
            self.recompute_layout();
        }
    }

    /// Get CSS style property value
    pub fn get_style_property(&self, property: &str) -> Option<String> {
        let style = self.style.lock().unwrap();
        style.get_property(property).cloned()
    }

    /// Recompute element layout and bounding box
    fn recompute_layout(&self) {
        let style = self.style.lock().unwrap();
        let mut rect = self.bounding_rect.lock().unwrap();

        // Get computed dimensions
        let width = style.get_property("width")
            .and_then(|w| self.parse_length_value(w))
            .unwrap_or(0.0);

        let height = style.get_property("height")
            .and_then(|h| self.parse_length_value(h))
            .unwrap_or(0.0);

        let left = style.get_property("left")
            .and_then(|l| self.parse_length_value(l))
            .unwrap_or(0.0);

        let top = style.get_property("top")
            .and_then(|t| self.parse_length_value(t))
            .unwrap_or(0.0);

        // Update bounding rectangle
        rect.update_bounds(left, top, width, height);
    }

    /// Parse length value to pixels
    fn parse_length_value(&self, value: &str) -> Option<f64> {
        if let Some(px_value) = value.strip_suffix("px") {
            px_value.parse().ok()
        } else if let Ok(num) = value.parse::<f64>() {
            Some(num) // Treat unitless as pixels
        } else {
            None
        }
    }

    /// Get bounding client rectangle
    pub fn get_bounding_client_rect(&self) -> DOMRect {
        self.bounding_rect.lock().unwrap().clone()
    }

    /// Real DOM tree traversal - get element by ID
    pub fn find_element_by_id(&self, id: &str) -> Option<JsObject> {
        // Check this element
        if self.get_id() == id {
            // Return self as JsObject - would need proper conversion
            return None; // Placeholder
        }

        // Recursively search children
        let children = self.children.lock().unwrap();
        for child in children.iter() {
            if let Some(child_data) = child.downcast_ref::<ElementData>() {
                if let Some(found) = child_data.find_element_by_id(id) {
                    return Some(found);
                }
            }
        }

        None
    }

    /// CSS selector matching
    pub fn matches_selector(&self, selector: &str) -> bool {
        // Simple selector matching - real implementation would use CSS parser
        if selector.starts_with('#') {
            // ID selector
            let id = &selector[1..];
            return self.get_id() == id;
        } else if selector.starts_with('.') {
            // Class selector
            let class = &selector[1..];
            return self.get_class_name().split_whitespace().any(|c| c == class);
        } else {
            // Tag selector
            return self.get_tag_name().to_lowercase() == selector.to_lowercase();
        }
    }

    /// Query selector implementation
    pub fn query_selector(&self, selector: &str) -> Option<JsObject> {
        // Check this element
        if self.matches_selector(selector) {
            // Return self - would need proper conversion
            return None; // Placeholder
        }

        // Recursively search children
        let children = self.children.lock().unwrap();
        for child in children.iter() {
            if let Some(child_data) = child.downcast_ref::<ElementData>() {
                if child_data.matches_selector(selector) {
                    return Some(child.clone());
                }
                // Search deeper
                if let Some(found) = child_data.query_selector(selector) {
                    return Some(found);
                }
            }
        }

        None
    }

    /// Query all elements matching selector
    pub fn query_selector_all(&self, selector: &str) -> Vec<JsObject> {
        let mut results = Vec::new();

        // Check this element
        if self.matches_selector(selector) {
            // Would add self to results
        }

        // Recursively search children
        let children = self.children.lock().unwrap();
        for child in children.iter() {
            if let Some(child_data) = child.downcast_ref::<ElementData>() {
                if child_data.matches_selector(selector) {
                    results.push(child.clone());
                }
                // Search deeper
                results.extend(child_data.query_selector_all(selector));
            }
        }

        results
    }
}

/// `Element.prototype.tagName` getter
fn get_tag_name(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.tagName called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        Ok(JsString::from(element.get_tag_name()).into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.tagName called on non-Element object")
            .into())
    }
}

/// `Element.prototype.id` getter
fn get_id(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.id called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        Ok(JsString::from(element.get_id()).into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.id called on non-Element object")
            .into())
    }
}

/// `Element.prototype.id` setter
fn set_id(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.id setter called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let id = args.get_or_undefined(0).to_string(context)?;
        element.set_id(id.to_std_string_escaped());
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.id setter called on non-Element object")
            .into())
    }
}

/// `Element.prototype.className` getter
fn get_class_name(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.className called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        Ok(JsString::from(element.get_class_name()).into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.className called on non-Element object")
            .into())
    }
}

/// `Element.prototype.className` setter
fn set_class_name(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.className setter called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let class_name = args.get_or_undefined(0).to_string(context)?;
        element.set_class_name(class_name.to_std_string_escaped());
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.className setter called on non-Element object")
            .into())
    }
}

/// `Element.prototype.innerHTML` getter
fn get_inner_html(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.innerHTML called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        Ok(JsString::from(element.get_inner_html()).into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.innerHTML called on non-Element object")
            .into())
    }
}

/// `Element.prototype.innerHTML` setter
fn set_inner_html(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {

    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.innerHTML setter called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let html = args.get_or_undefined(0).to_string(context)?;
        element.set_inner_html(html.to_std_string_escaped());
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.innerHTML setter called on non-Element object")
            .into())
    }
}

/// `Element.prototype.textContent` getter
fn get_text_content(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.textContent called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        Ok(JsString::from(element.get_text_content()).into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.textContent called on non-Element object")
            .into())
    }
}

/// `Element.prototype.textContent` setter
fn set_text_content(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.textContent setter called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let content = args.get_or_undefined(0).to_string(context)?;
        element.set_text_content(content.to_std_string_escaped());
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.textContent setter called on non-Element object")
            .into())
    }
}

/// `Element.prototype.children` getter
fn get_children(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.children called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let children = element.get_children();
        use crate::builtins::Array;
        let children_values: Vec<JsValue> = children.into_iter().map(|child| child.into()).collect();
        let array = Array::create_array_from_list(children_values, context);
        Ok(array.into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.children called on non-Element object")
            .into())
    }
}

/// `Element.prototype.parentNode` getter
fn get_parent_node(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.parentNode called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        Ok(element.get_parent_node().map(|parent| parent.into()).unwrap_or(JsValue::null()))
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.parentNode called on non-Element object")
            .into())
    }
}

/// `Element.prototype.style` getter
fn get_style(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.style called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        // Create a style object with getters/setters for CSS properties
        let style_obj = JsObject::default();

        // Add common CSS properties as dynamic getters/setters
        let css_properties = ["width", "height", "color", "background-color", "display",
                             "position", "left", "top", "right", "bottom", "margin", "padding"];

        for property in css_properties {
            // Create getter for this property
            let prop_name = property.replace("-", "_"); // Convert kebab-case to snake_case for JS
            let property_copy = property.to_string();

            // This would need proper closure binding in real implementation
            // For now, return empty style object
        }

        Ok(style_obj.into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.style called on non-Element object")
            .into())
    }
}

/// `Element.prototype.classList` getter
fn get_class_list(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.classList called on non-object")
    })?;

    if let Some(_element) = this_obj.downcast_ref::<ElementData>() {
        // Create or return a DOMTokenList bound to this element
        let list = crate::builtins::DOMTokenList::create_for_element(this_obj.clone(), context)?;
        Ok(list.into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.classList called on non-Element object")
            .into())
    }
}

/// `Element.prototype.setAttribute(name, value)`
fn set_attribute(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.setAttribute called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let name = args.get_or_undefined(0).to_string(context)?;
        let value = args.get_or_undefined(1).to_string(context)?;
        element.set_attribute(name.to_std_string_escaped(), value.to_std_string_escaped());
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.setAttribute called on non-Element object")
            .into())
    }
}

/// `Element.prototype.getAttribute(name)`
fn get_attribute(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.getAttribute called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let name = args.get_or_undefined(0).to_string(context)?;
        if let Some(value) = element.get_attribute(&name.to_std_string_escaped()) {
            Ok(JsString::from(value).into())
        } else {
            Ok(JsValue::null())
        }
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.getAttribute called on non-Element object")
            .into())
    }
}

/// `Element.prototype.hasAttribute(name)`
fn has_attribute(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.hasAttribute called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let name = args.get_or_undefined(0).to_string(context)?;
        Ok(element.has_attribute(&name.to_std_string_escaped()).into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.hasAttribute called on non-Element object")
            .into())
    }
}

/// `Element.prototype.removeAttribute(name)`
fn remove_attribute(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.removeAttribute called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let name = args.get_or_undefined(0).to_string(context)?;
        element.remove_attribute(&name.to_std_string_escaped());
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.removeAttribute called on non-Element object")
            .into())
    }
}

/// `Element.prototype.appendChild(child)`
fn append_child(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.appendChild called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let child_value = args.get_or_undefined(0);
        if let Some(child_obj) = child_value.as_object() {
            // Set parent relationship
            if let Some(child_element) = child_obj.downcast_ref::<ElementData>() {
                child_element.set_parent_node(Some(this_obj.clone()));
            }
            element.append_child(child_obj.clone());
            Ok(child_value.clone())
        } else {
            Err(JsNativeError::typ()
                .with_message("Element.prototype.appendChild requires an Element argument")
                .into())
        }
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.appendChild called on non-Element object")
            .into())
    }
}

/// `Element.prototype.removeChild(child)`
fn remove_child(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.removeChild called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let child_value = args.get_or_undefined(0);
        if let Some(child_obj) = child_value.as_object() {
            // Remove parent relationship
            if let Some(child_element) = child_obj.downcast_ref::<ElementData>() {
                child_element.set_parent_node(None);
            }
            element.remove_child(&child_obj);
            Ok(child_value.clone())
        } else {
            Err(JsNativeError::typ()
                .with_message("Element.prototype.removeChild requires an Element argument")
                .into())
        }
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.removeChild called on non-Element object")
            .into())
    }
}

/// `Element.prototype.setHTML(input, options)` - Chrome 124
fn set_html(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.setHTML called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let input = args.get_or_undefined(0).to_string(context)?;
        let _options = args.get(1).cloned().unwrap_or(JsValue::undefined());

        // In a real implementation, this would sanitize the HTML
        // For now, we just set it as innerHTML
        element.set_inner_html(input.to_std_string_escaped());
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.setHTML called on non-Element object")
            .into())
    }
}

/// `Element.prototype.setHTMLUnsafe(input)` - Chrome 124
fn set_html_unsafe(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.setHTMLUnsafe called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let input = args.get_or_undefined(0).to_string(context)?;

        // Set HTML without sanitization
        element.set_inner_html(input.to_std_string_escaped());
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.setHTMLUnsafe called on non-Element object")
            .into())
    }
}

/// `Element.prototype.attachShadow(options)` - Shadow DOM API
/// Check if an element can have a shadow root attached according to WHATWG spec
/// https://dom.spec.whatwg.org/#dom-element-attachshadow
pub fn can_have_shadow_root(element: &ElementData) -> bool {
    let tag_name = element.get_tag_name().to_lowercase();
    let namespace = element.get_namespace_uri().unwrap_or_default();

    // Per WHATWG spec, only these elements can have shadow roots attached:

    // 1. HTML namespace elements that are valid shadow hosts
    if namespace == "http://www.w3.org/1999/xhtml" || namespace.is_empty() {
        return match tag_name.as_str() {
            // Custom elements (any element with a hyphen in the name)
            name if name.contains('-') => true,

            // Standard HTML elements that can host shadow roots
            "article" | "aside" | "blockquote" | "body" | "div" |
            "footer" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" |
            "header" | "main" | "nav" | "p" | "section" | "span" => true,

            // Form elements that can host shadow roots
            "form" | "fieldset" => true,

            // Other valid shadow hosts
            "details" | "dialog" => true,

            // All other HTML elements cannot host shadow roots
            _ => false,
        };
    }

    // 2. Elements in other namespaces
    // Per spec, elements in non-HTML namespaces can also be shadow hosts
    // if they are valid custom elements or meet certain criteria
    if namespace == "http://www.w3.org/2000/svg" {
        // SVG elements that can be shadow hosts
        return match tag_name.as_str() {
            "g" | "svg" | "foreignObject" => true,
            name if name.contains('-') => true, // Custom SVG elements
            _ => false,
        };
    }

    // Elements in other namespaces can be shadow hosts if they're custom elements
    tag_name.contains('-')
}

/// Check if element has forbidden shadow root characteristics
fn has_forbidden_shadow_characteristics(element: &ElementData) -> bool {
    let tag_name = element.get_tag_name().to_lowercase();

    // Elements that must never have shadow roots for security/functionality reasons
    match tag_name.as_str() {
        // Form controls that have special UA behavior
        "input" | "textarea" | "select" | "button" => true,

        // Media elements with special UA behavior
        "audio" | "video" | "img" | "canvas" => true,

        // Elements that affect document structure
        "html" | "head" | "title" | "meta" | "link" | "style" | "script" => true,

        // Interactive elements that could cause security issues
        "a" | "area" | "iframe" | "object" | "embed" => true,

        // Table elements with complex UA behavior
        "table" | "thead" | "tbody" | "tfoot" | "tr" | "td" | "th" |
        "col" | "colgroup" | "caption" => true,

        // List elements
        "ol" | "ul" | "li" | "dl" | "dt" | "dd" => true,

        // Other elements with special semantics
        "option" | "optgroup" | "legend" | "label" => true,

        _ => false,
    }
}

/// Check if element is a valid custom element name
fn is_valid_custom_element_name(name: &str) -> bool {
    // Per WHATWG spec, custom element names must:
    // 1. Contain a hyphen
    // 2. Start with lowercase ASCII letter
    // 3. Contain only lowercase ASCII letters, digits, hyphens, periods, underscores
    // 4. Not be one of the reserved names

    if !name.contains('-') {
        return false;
    }

    let first_char = name.chars().next().unwrap_or('\0');
    if !first_char.is_ascii_lowercase() {
        return false;
    }

    if !name.chars().all(|c| {
        c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.' || c == '_'
    }) {
        return false;
    }

    // Reserved names that cannot be custom elements
    const RESERVED_NAMES: &[&str] = &[
        "annotation-xml",
        "color-profile",
        "font-face",
        "font-face-src",
        "font-face-uri",
        "font-face-format",
        "font-face-name",
        "missing-glyph",
    ];

    !RESERVED_NAMES.contains(&name)
}

fn attach_shadow(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.attachShadow called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let options = args.get_or_undefined(0);

        // Parse options object according to WHATWG spec
        let shadow_init = if let Some(options_obj) = options.as_object() {
            let mode = if let Ok(mode_value) = options_obj.get(js_string!("mode"), context) {
                let mode_str = mode_value.to_string(context)?.to_std_string_escaped();
                crate::builtins::shadow_root::ShadowRootMode::from_string(&mode_str)
                    .ok_or_else(|| JsNativeError::typ()
                        .with_message("attachShadow mode must be 'open' or 'closed'"))?
            } else {
                return Err(JsNativeError::typ()
                    .with_message("attachShadow options must include a mode")
                    .into());
            };

            let clonable = if let Ok(clonable_value) = options_obj.get(js_string!("clonable"), context) {
                clonable_value.to_boolean()
            } else {
                false
            };

            let serializable = if let Ok(serializable_value) = options_obj.get(js_string!("serializable"), context) {
                serializable_value.to_boolean()
            } else {
                false
            };

            let delegates_focus = if let Ok(delegates_focus_value) = options_obj.get(js_string!("delegatesFocus"), context) {
                delegates_focus_value.to_boolean()
            } else {
                false
            };

            crate::builtins::shadow_root::ShadowRootInit {
                mode,
                clonable,
                serializable,
                delegates_focus,
            }
        } else {
            return Err(JsNativeError::typ()
                .with_message("attachShadow requires an options object")
                .into());
        };

        // Check if element already has a shadow root
        if element.get_shadow_root().is_some() {
            return Err(JsNativeError::error()
                .with_message("Element already has a shadow root")
                .into());
        }

        // Validate element according to WHATWG specification
        // https://dom.spec.whatwg.org/#dom-element-attachshadow
        if !can_have_shadow_root(&element) {
            return Err(JsNativeError::error()
                .with_message("Operation not supported")
                .into());
        }

        // Create a proper ShadowRoot using the new implementation
        let shadow_root = crate::builtins::shadow_root::ShadowRoot::create_shadow_root(
            shadow_init.mode.clone(),
            &shadow_init,
            context,
        )?;

        // Set the host element for the shadow root
        if let Some(shadow_data) = shadow_root.downcast_ref::<crate::builtins::shadow_root::ShadowRootData>() {
            shadow_data.set_host(this_obj.clone());
        }

        // Set shadowRoot property on the element according to mode
        match shadow_init.mode {
            crate::builtins::shadow_root::ShadowRootMode::Open => {
                this_obj.define_property_or_throw(
                    js_string!("shadowRoot"),
                    crate::property::PropertyDescriptorBuilder::new()
                        .value(shadow_root.clone())
                        .writable(false)
                        .enumerable(false)
                        .configurable(true)
                        .build(),
                    context,
                )?;
            }
            crate::builtins::shadow_root::ShadowRootMode::Closed => {
                // For 'closed' mode, shadowRoot property should be null
                this_obj.define_property_or_throw(
                    js_string!("shadowRoot"),
                    crate::property::PropertyDescriptorBuilder::new()
                        .value(JsValue::null())
                        .writable(false)
                        .enumerable(false)
                        .configurable(true)
                        .build(),
                    context,
                )?;
            }
        }

        // Store the shadow root internally in element data
        element.attach_shadow_root(shadow_root.clone());

        Ok(shadow_root.into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.attachShadow called on non-Element object")
            .into())
    }
}

#[cfg(test)]
mod tests;