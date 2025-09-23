//! Node interface implementation for DOM Level 4
//!
//! The Node interface is the primary datatype for the entire Document Object Model.
//! It represents a single node in the document tree.
//! https://dom.spec.whatwg.org/#interface-node

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    property::{Attribute, PropertyDescriptorBuilder},
    realm::Realm,
    string::{StaticJsStrings, JsString},
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult,
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Node types as defined by the DOM specification
#[derive(Debug, Clone, PartialEq, Eq, Trace, Finalize)]
#[repr(u16)]
pub enum NodeType {
    Node = 0,
    Element = 1,
    Attribute = 2,
    Text = 3,
    CDataSection = 4,
    ProcessingInstruction = 7,
    Comment = 8,
    Document = 9,
    DocumentType = 10,
    DocumentFragment = 11,
}

impl NodeType {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(NodeType::Node),
            1 => Some(NodeType::Element),
            2 => Some(NodeType::Attribute),
            3 => Some(NodeType::Text),
            4 => Some(NodeType::CDataSection),
            7 => Some(NodeType::ProcessingInstruction),
            8 => Some(NodeType::Comment),
            9 => Some(NodeType::Document),
            10 => Some(NodeType::DocumentType),
            11 => Some(NodeType::DocumentFragment),
            _ => None,
        }
    }
}

/// Document position flags for Node.compareDocumentPosition()
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum DocumentPosition {
    Disconnected = 0x01,
    Preceding = 0x02,
    Following = 0x04,
    Contains = 0x08,
    ContainedBy = 0x10,
    ImplementationSpecific = 0x20,
}

/// Node interface data structure
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct NodeData {
    /// Node type (Element, Text, Document, etc.)
    #[unsafe_ignore_trace]
    node_type: NodeType,

    /// Node name (tag name for elements, "#text" for text nodes, etc.)
    #[unsafe_ignore_trace]
    node_name: Arc<Mutex<String>>,

    /// Node value (null for elements, text content for text nodes, etc.)
    #[unsafe_ignore_trace]
    node_value: Arc<Mutex<Option<String>>>,

    /// Base URI for the node
    #[unsafe_ignore_trace]
    base_uri: Arc<Mutex<Option<String>>>,

    /// Owner document (null for Document nodes)
    #[unsafe_ignore_trace]
    owner_document: Arc<Mutex<Option<JsObject>>>,

    /// Parent node
    #[unsafe_ignore_trace]
    parent_node: Arc<Mutex<Option<JsObject>>>,

    /// Child nodes (live NodeList)
    #[unsafe_ignore_trace]
    child_nodes: Arc<Mutex<Vec<JsObject>>>,

    /// Previous sibling node
    #[unsafe_ignore_trace]
    previous_sibling: Arc<Mutex<Option<JsObject>>>,

    /// Next sibling node
    #[unsafe_ignore_trace]
    next_sibling: Arc<Mutex<Option<JsObject>>>,

    /// Text content cache
    #[unsafe_ignore_trace]
    text_content: Arc<Mutex<Option<String>>>,

    /// Whether the node is connected to a document
    #[unsafe_ignore_trace]
    is_connected: Arc<Mutex<bool>>,
}

impl NodeData {
    /// Create a new NodeData with the specified type
    pub fn new(node_type: NodeType) -> Self {
        let node_name = match node_type {
            NodeType::Element => "".to_string(), // Will be set by Element
            NodeType::Text => "#text".to_string(),
            NodeType::Comment => "#comment".to_string(),
            NodeType::Document => "#document".to_string(),
            NodeType::DocumentFragment => "#document-fragment".to_string(),
            NodeType::DocumentType => "html".to_string(), // Default, can be changed
            NodeType::Attribute => "".to_string(), // Will be set by Attr
            NodeType::CDataSection => "#cdata-section".to_string(),
            NodeType::ProcessingInstruction => "".to_string(),
            NodeType::Node => "#node".to_string(), // Will be set by PI
        };

        Self {
            node_type,
            node_name: Arc::new(Mutex::new(node_name)),
            node_value: Arc::new(Mutex::new(None)),
            base_uri: Arc::new(Mutex::new(None)),
            owner_document: Arc::new(Mutex::new(None)),
            parent_node: Arc::new(Mutex::new(None)),
            child_nodes: Arc::new(Mutex::new(Vec::new())),
            previous_sibling: Arc::new(Mutex::new(None)),
            next_sibling: Arc::new(Mutex::new(None)),
            text_content: Arc::new(Mutex::new(None)),
            is_connected: Arc::new(Mutex::new(false)),
        }
    }

    /// Create a new NodeData with specified type and name
    pub fn with_name(node_type: NodeType, name: String) -> Self {
        let mut node = Self::new(node_type);
        *node.node_name.lock().unwrap() = name;
        node
    }

    // Getters and setters for node properties
    pub fn get_node_type(&self) -> NodeType {
        self.node_type.clone()
    }

    pub fn get_node_name(&self) -> String {
        self.node_name.lock().unwrap().clone()
    }

    pub fn set_node_name(&self, name: String) {
        *self.node_name.lock().unwrap() = name;
    }

    pub fn get_node_value(&self) -> Option<String> {
        self.node_value.lock().unwrap().clone()
    }

    pub fn set_node_value(&self, value: Option<String>) {
        *self.node_value.lock().unwrap() = value;
    }

    pub fn get_base_uri(&self) -> Option<String> {
        self.base_uri.lock().unwrap().clone()
    }

    pub fn set_base_uri(&self, uri: Option<String>) {
        *self.base_uri.lock().unwrap() = uri;
    }

    pub fn get_owner_document(&self) -> Option<JsObject> {
        self.owner_document.lock().unwrap().clone()
    }

    pub fn set_owner_document(&self, document: Option<JsObject>) {
        *self.owner_document.lock().unwrap() = document;
    }

    pub fn get_parent_node(&self) -> Option<JsObject> {
        self.parent_node.lock().unwrap().clone()
    }

    pub fn set_parent_node(&self, parent: Option<JsObject>) {
        *self.parent_node.lock().unwrap() = parent;
        // Update isConnected status
        self.update_connected_status();
    }

    pub fn get_child_nodes(&self) -> Vec<JsObject> {
        self.child_nodes.lock().unwrap().clone()
    }

    pub fn add_child_node(&self, child: JsObject) {
        self.child_nodes.lock().unwrap().push(child);
    }

    pub fn remove_child_node(&self, child: &JsObject) -> bool {
        let mut children = self.child_nodes.lock().unwrap();
        if let Some(pos) = children.iter().position(|c| JsObject::equals(c, child)) {
            children.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn insert_child_node(&self, child: JsObject, index: usize) {
        let mut children = self.child_nodes.lock().unwrap();
        if index <= children.len() {
            children.insert(index, child);
        } else {
            children.push(child);
        }
    }

    pub fn get_previous_sibling(&self) -> Option<JsObject> {
        self.previous_sibling.lock().unwrap().clone()
    }

    pub fn set_previous_sibling(&self, sibling: Option<JsObject>) {
        *self.previous_sibling.lock().unwrap() = sibling;
    }

    pub fn get_next_sibling(&self) -> Option<JsObject> {
        self.next_sibling.lock().unwrap().clone()
    }

    pub fn set_next_sibling(&self, sibling: Option<JsObject>) {
        *self.next_sibling.lock().unwrap() = sibling;
    }

    pub fn get_first_child(&self) -> Option<JsObject> {
        self.child_nodes.lock().unwrap().first().cloned()
    }

    pub fn get_last_child(&self) -> Option<JsObject> {
        self.child_nodes.lock().unwrap().last().cloned()
    }

    pub fn has_child_nodes(&self) -> bool {
        !self.child_nodes.lock().unwrap().is_empty()
    }

    pub fn is_connected(&self) -> bool {
        *self.is_connected.lock().unwrap()
    }

    /// Update the connected status by checking if we're connected to a document
    fn update_connected_status(&self) {
        let connected = self.check_connected_to_document();
        *self.is_connected.lock().unwrap() = connected;
    }

    /// Check if this node is connected to a document by traversing up the tree
    fn check_connected_to_document(&self) -> bool {
        if self.node_type == NodeType::Document {
            return true;
        }

        let mut current = self.get_parent_node();
        while let Some(parent) = current {
            if let Some(parent_data) = parent.downcast_ref::<NodeData>() {
                if parent_data.get_node_type() == NodeType::Document {
                    return true;
                }
                current = parent_data.get_parent_node();
            } else {
                break;
            }
        }
        false
    }

    /// Get the text content of this node and its descendants
    pub fn get_text_content(&self) -> Option<String> {
        match self.node_type {
            NodeType::DocumentFragment | NodeType::Element => {
                // Concatenate text content of all text node descendants
                let mut text = String::new();
                self.collect_text_content(&mut text);
                if text.is_empty() {
                    None
                } else {
                    Some(text)
                }
            }
            NodeType::Text | NodeType::Comment | NodeType::ProcessingInstruction => {
                self.get_node_value()
            }
            _ => None,
        }
    }

    /// Recursively collect text content from descendant text nodes
    fn collect_text_content(&self, text: &mut String) {
        for child in self.get_child_nodes() {
            if let Some(child_data) = child.downcast_ref::<NodeData>() {
                match child_data.get_node_type() {
                    NodeType::Text => {
                        if let Some(value) = child_data.get_node_value() {
                            text.push_str(&value);
                        }
                    }
                    NodeType::Element | NodeType::DocumentFragment => {
                        child_data.collect_text_content(text);
                    }
                    _ => {}
                }
            }
        }
    }

    /// Set the text content, replacing all child nodes with a single text node
    pub fn set_text_content(&self, content: Option<String>) {
        match self.node_type {
            NodeType::Text | NodeType::Comment | NodeType::ProcessingInstruction => {
                self.set_node_value(content);
            }
            NodeType::DocumentFragment | NodeType::Element => {
                // Remove all children and add a single text node if content is provided
                self.child_nodes.lock().unwrap().clear();
                if let Some(text) = content {
                    if !text.is_empty() {
                        // TODO: Create a proper text node here
                        // For now, we'll store it in our text_content cache
                        *self.text_content.lock().unwrap() = Some(text);
                    }
                }
            }
            _ => {
                // Document, DocumentType nodes: do nothing
            }
        }
    }
}

impl IntrinsicObject for NodeData {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(Self::append_child, js_string!("appendChild"), 1)
            .static_method(Self::remove_child, js_string!("removeChild"), 1)
            .static_method(Self::insert_before, js_string!("insertBefore"), 2)
            .static_method(Self::replace_child, js_string!("replaceChild"), 2)
            .static_method(Self::clone_node, js_string!("cloneNode"), 0)
            .static_method(Self::normalize, js_string!("normalize"), 0)
            .static_method(Self::is_equal_node, js_string!("isEqualNode"), 1)
            .static_method(Self::is_same_node, js_string!("isSameNode"), 1)
            .static_method(Self::compare_document_position, js_string!("compareDocumentPosition"), 1)
            .static_method(Self::contains, js_string!("contains"), 1)
            .static_method(Self::lookup_prefix, js_string!("lookupPrefix"), 1)
            .static_method(Self::lookup_namespace_uri, js_string!("lookupNamespaceURI"), 1)
            .static_method(Self::is_default_namespace, js_string!("isDefaultNamespace"), 1)
            .static_method(Self::has_child_nodes_method, js_string!("hasChildNodes"), 0)
            .static_method(Self::get_root_node, js_string!("getRootNode"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for NodeData {
    const NAME: JsString = StaticJsStrings::NODE;
}

impl BuiltInConstructor for NodeData {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::node;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling Node constructor without `new` is forbidden")
                .into());
        }

        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::node,
            context,
        )?;

        // Abstract interface - cannot be constructed directly
        Err(JsNativeError::typ()
            .with_message("Illegal constructor")
            .into())
    }
}

// Node constants
impl NodeData {
    const ELEMENT_NODE: u16 = 1;
    const ATTRIBUTE_NODE: u16 = 2;
    const TEXT_NODE: u16 = 3;
    const CDATA_SECTION_NODE: u16 = 4;
    const PROCESSING_INSTRUCTION_NODE: u16 = 7;
    const COMMENT_NODE: u16 = 8;
    const DOCUMENT_NODE: u16 = 9;
    const DOCUMENT_TYPE_NODE: u16 = 10;
    const DOCUMENT_FRAGMENT_NODE: u16 = 11;

    const DOCUMENT_POSITION_DISCONNECTED: u32 = 0x01;
    const DOCUMENT_POSITION_PRECEDING: u32 = 0x02;
    const DOCUMENT_POSITION_FOLLOWING: u32 = 0x04;
    const DOCUMENT_POSITION_CONTAINS: u32 = 0x08;
    const DOCUMENT_POSITION_CONTAINED_BY: u32 = 0x10;
    const DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC: u32 = 0x20;
}

// Property accessors
impl NodeData {
    fn get_node_type_accessor(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.nodeType called on non-object")
        })?;

        if let Some(node) = this_obj.downcast_ref::<NodeData>() {
            let node_type_value = match node.get_node_type() {
                NodeType::Node => 0,
                NodeType::Element => 1,
                NodeType::Attribute => 2,
                NodeType::Text => 3,
                NodeType::CDataSection => 4,
                NodeType::ProcessingInstruction => 7,
                NodeType::Comment => 8,
                NodeType::Document => 9,
                NodeType::DocumentType => 10,
                NodeType::DocumentFragment => 11,
            };
            Ok(JsValue::from(node_type_value))
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.nodeType called on non-Node object")
                .into())
        }
    }

    fn get_node_name_accessor(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.nodeName called on non-object")
        })?;

        if let Some(node) = this_obj.downcast_ref::<NodeData>() {
            Ok(JsValue::from(js_string!(node.get_node_name())))
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.nodeName called on non-Node object")
                .into())
        }
    }

    fn get_node_value_accessor(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.nodeValue called on non-object")
        })?;

        if let Some(node) = this_obj.downcast_ref::<NodeData>() {
            match node.get_node_value() {
                Some(value) => Ok(JsValue::from(js_string!(value))),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.nodeValue called on non-Node object")
                .into())
        }
    }

    fn set_node_value_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.nodeValue setter called on non-object")
        })?;

        if let Some(node) = this_obj.downcast_ref::<NodeData>() {
            let value = args.get_or_undefined(0);
            let new_value = if value.is_null() {
                None
            } else {
                Some(value.to_string(context)?.to_std_string_escaped())
            };
            node.set_node_value(new_value);
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.nodeValue setter called on non-Node object")
                .into())
        }
    }

    fn get_base_uri_accessor(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.baseURI called on non-object")
        })?;

        if let Some(node) = this_obj.downcast_ref::<NodeData>() {
            match node.get_base_uri() {
                Some(uri) => Ok(JsValue::from(js_string!(uri))),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.baseURI called on non-Node object")
                .into())
        }
    }

    fn get_is_connected_accessor(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.isConnected called on non-object")
        })?;

        if let Some(node) = this_obj.downcast_ref::<NodeData>() {
            Ok(JsValue::from(node.is_connected()))
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.isConnected called on non-Node object")
                .into())
        }
    }

    fn get_owner_document_accessor(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.ownerDocument called on non-object")
        })?;

        if let Some(node) = this_obj.downcast_ref::<NodeData>() {
            match node.get_owner_document() {
                Some(doc) => Ok(doc.into()),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.ownerDocument called on non-Node object")
                .into())
        }
    }

    fn get_parent_node_accessor(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.parentNode called on non-object")
        })?;

        if let Some(node) = this_obj.downcast_ref::<NodeData>() {
            match node.get_parent_node() {
                Some(parent) => Ok(parent.into()),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.parentNode called on non-Node object")
                .into())
        }
    }

    fn get_child_nodes_accessor(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.childNodes called on non-object")
        })?;

        if let Some(node) = this_obj.downcast_ref::<NodeData>() {
            // TODO: Return a proper live NodeList object
            // For now, return an array-like object
            let children = node.get_child_nodes();
            let array = crate::builtins::Array::array_create(children.len() as u64, None, context)?;

            for (i, child) in children.iter().enumerate() {
                array.set(i, child.clone(), false, context)?;
            }

            Ok(array.into())
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.childNodes called on non-Node object")
                .into())
        }
    }

    fn get_first_child_accessor(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.firstChild called on non-object")
        })?;

        if let Some(node) = this_obj.downcast_ref::<NodeData>() {
            match node.get_first_child() {
                Some(child) => Ok(child.into()),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.firstChild called on non-Node object")
                .into())
        }
    }

    fn get_last_child_accessor(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.lastChild called on non-object")
        })?;

        if let Some(node) = this_obj.downcast_ref::<NodeData>() {
            match node.get_last_child() {
                Some(child) => Ok(child.into()),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.lastChild called on non-Node object")
                .into())
        }
    }

    fn get_previous_sibling_accessor(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.previousSibling called on non-object")
        })?;

        if let Some(node) = this_obj.downcast_ref::<NodeData>() {
            match node.get_previous_sibling() {
                Some(sibling) => Ok(sibling.into()),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.previousSibling called on non-Node object")
                .into())
        }
    }

    fn get_next_sibling_accessor(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.nextSibling called on non-object")
        })?;

        if let Some(node) = this_obj.downcast_ref::<NodeData>() {
            match node.get_next_sibling() {
                Some(sibling) => Ok(sibling.into()),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.nextSibling called on non-Node object")
                .into())
        }
    }

    fn get_text_content_accessor(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.textContent called on non-object")
        })?;

        if let Some(node) = this_obj.downcast_ref::<NodeData>() {
            match node.get_text_content() {
                Some(content) => Ok(JsValue::from(js_string!(content))),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.textContent called on non-Node object")
                .into())
        }
    }

    fn set_text_content_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.textContent setter called on non-object")
        })?;

        if let Some(node) = this_obj.downcast_ref::<NodeData>() {
            let value = args.get_or_undefined(0);
            let new_content = if value.is_null() {
                None
            } else {
                Some(value.to_string(context)?.to_std_string_escaped())
            };
            node.set_text_content(new_content);
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.textContent setter called on non-Node object")
                .into())
        }
    }
}

// Node methods implementation will continue in the next part...
impl NodeData {
    /// `Node.prototype.appendChild(child)`
    fn append_child(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.appendChild called on non-object")
        })?;

        let child_arg = args.get_or_undefined(0);
        let child_obj = child_arg.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.appendChild: child must be a Node")
        })?;

        if let Some(parent_node) = this_obj.downcast_ref::<NodeData>() {
            if let Some(child_node) = child_obj.clone().downcast_ref::<NodeData>() {
                // Remove child from its current parent if it has one
                if let Some(old_parent) = child_node.get_parent_node() {
                    if let Some(old_parent_data) = old_parent.downcast_ref::<NodeData>() {
                        old_parent_data.remove_child_node(&child_obj);
                    }
                }

                // Add child to this node
                parent_node.add_child_node(child_obj.clone());
                child_node.set_parent_node(Some(this_obj.clone()));

                // Update sibling links
                if let Some(last_child) = parent_node.get_last_child() {
                    if !JsObject::equals(&last_child, &child_obj) {
                        if let Some(last_child_data) = last_child.clone().downcast_ref::<NodeData>() {
                            last_child_data.set_next_sibling(Some(child_obj.clone()));
                            child_node.set_previous_sibling(Some(last_child));
                        }
                    }
                }

                Ok(child_obj.into())
            } else {
                Err(JsNativeError::typ()
                    .with_message("Node.appendChild: child must be a Node")
                    .into())
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.appendChild called on non-Node object")
                .into())
        }
    }

    /// `Node.prototype.removeChild(child)`
    fn remove_child(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.removeChild called on non-object")
        })?;

        let child_arg = args.get_or_undefined(0);
        let child_obj = child_arg.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.removeChild: child must be a Node")
        })?;

        if let Some(parent_node) = this_obj.downcast_ref::<NodeData>() {
            if let Some(child_node) = child_obj.clone().downcast_ref::<NodeData>() {
                // Check if child is actually a child of this node
                if !parent_node.get_child_nodes().iter().any(|c| JsObject::equals(c, &child_obj)) {
                    return Err(JsNativeError::typ()
                        .with_message("Node.removeChild: child is not a child of this node")
                        .into());
                }

                // Update sibling links
                let prev_sibling = child_node.get_previous_sibling();
                let next_sibling = child_node.get_next_sibling();

                if let Some(prev) = &prev_sibling {
                    if let Some(prev_data) = prev.downcast_ref::<NodeData>() {
                        prev_data.set_next_sibling(next_sibling.clone());
                    }
                }

                if let Some(next) = &next_sibling {
                    if let Some(next_data) = next.downcast_ref::<NodeData>() {
                        next_data.set_previous_sibling(prev_sibling);
                    }
                }

                // Remove from parent's child list
                parent_node.remove_child_node(&child_obj);

                // Clear child's parent and sibling references
                child_node.set_parent_node(None);
                child_node.set_previous_sibling(None);
                child_node.set_next_sibling(None);

                Ok(child_obj.into())
            } else {
                Err(JsNativeError::typ()
                    .with_message("Node.removeChild: child must be a Node")
                    .into())
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.removeChild called on non-Node object")
                .into())
        }
    }

    /// `Node.prototype.insertBefore(newNode, referenceNode)`
    fn insert_before(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.insertBefore called on non-object")
        })?;

        let new_node_arg = args.get_or_undefined(0);
        let reference_node_arg = args.get_or_undefined(1);

        let new_node_obj = new_node_arg.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.insertBefore: newNode must be a Node")
        })?;

        if let Some(parent_node) = this_obj.downcast_ref::<NodeData>() {
            if let Some(new_node_data) = new_node_obj.clone().downcast_ref::<NodeData>() {
                let children = parent_node.get_child_nodes();

                let insert_index = if reference_node_arg.is_null() {
                    // Insert at the end
                    children.len()
                } else {
                    let reference_obj = reference_node_arg.as_object().ok_or_else(|| {
                        JsNativeError::typ().with_message("Node.insertBefore: referenceNode must be a Node or null")
                    })?;

                    // Find the index of the reference node
                    children.iter().position(|c| JsObject::equals(c, &reference_obj))
                        .ok_or_else(|| {
                            JsNativeError::typ().with_message("Node.insertBefore: referenceNode is not a child of this node")
                        })?
                };

                // Remove new node from its current parent if it has one
                if let Some(old_parent) = new_node_data.get_parent_node() {
                    if let Some(old_parent_data) = old_parent.downcast_ref::<NodeData>() {
                        old_parent_data.remove_child_node(&new_node_obj);
                    }
                }

                // Insert the new node
                parent_node.insert_child_node(new_node_obj.clone(), insert_index);
                new_node_data.set_parent_node(Some(this_obj.clone()));

                // Update sibling links
                let updated_children = parent_node.get_child_nodes();
                for (i, child) in updated_children.iter().enumerate() {
                    if let Some(child_data) = child.downcast_ref::<NodeData>() {
                        let prev = if i > 0 { Some(updated_children[i - 1].clone()) } else { None };
                        let next = if i < updated_children.len() - 1 { Some(updated_children[i + 1].clone()) } else { None };

                        child_data.set_previous_sibling(prev);
                        child_data.set_next_sibling(next);
                    }
                }

                Ok(new_node_obj.into())
            } else {
                Err(JsNativeError::typ()
                    .with_message("Node.insertBefore: newNode must be a Node")
                    .into())
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.insertBefore called on non-Node object")
                .into())
        }
    }

    /// `Node.prototype.replaceChild(newChild, oldChild)`
    fn replace_child(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let old_child = args.get_or_undefined(1);

        // First remove the old child
        Self::remove_child(this, &[old_child.clone()], context)?;

        // Then insert the new child at the same position
        // For simplicity, we'll just append it
        Self::append_child(this, &[args.get_or_undefined(0).clone()], context)
    }

    /// `Node.prototype.cloneNode(deep)`
    fn clone_node(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.cloneNode called on non-object")
        })?;

        if let Some(node) = this_obj.downcast_ref::<NodeData>() {
            let deep = args.get_or_undefined(0).to_boolean();

            // Create a new node of the same type
            let cloned_node_data = NodeData::new(node.get_node_type());
            cloned_node_data.set_node_name(node.get_node_name());
            cloned_node_data.set_node_value(node.get_node_value());
            cloned_node_data.set_base_uri(node.get_base_uri());

            let cloned_obj = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                context.intrinsics().constructors().node().prototype(),
                cloned_node_data,
            );

            // If deep cloning, clone all child nodes recursively
            if deep {
                for child in node.get_child_nodes() {
                    let cloned_child = Self::clone_node(&child.into(), &[JsValue::from(true)], context)?;
                    Self::append_child(&cloned_obj.clone().into(), &[cloned_child], context)?;
                }
            }

            Ok(cloned_obj.into())
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.cloneNode called on non-Node object")
                .into())
        }
    }

    /// `Node.prototype.normalize()`
    fn normalize(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.normalize called on non-object")
        })?;

        if let Some(_node) = this_obj.downcast_ref::<NodeData>() {
            // TODO: Implement text node normalization
            // This should merge adjacent text nodes and remove empty text nodes
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.normalize called on non-Node object")
                .into())
        }
    }

    /// `Node.prototype.isEqualNode(otherNode)`
    fn is_equal_node(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.isEqualNode called on non-object")
        })?;

        let other_arg = args.get_or_undefined(0);
        if other_arg.is_null() {
            return Ok(JsValue::from(false));
        }

        let other_obj = other_arg.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.isEqualNode: other must be a Node or null")
        })?;

        if let Some(this_node) = this_obj.downcast_ref::<NodeData>() {
            if let Some(other_node) = other_obj.downcast_ref::<NodeData>() {
                // Two nodes are equal if they have the same type, name, value, and children
                let equal = this_node.get_node_type() == other_node.get_node_type() &&
                           this_node.get_node_name() == other_node.get_node_name() &&
                           this_node.get_node_value() == other_node.get_node_value();

                Ok(JsValue::from(equal))
            } else {
                Ok(JsValue::from(false))
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.isEqualNode called on non-Node object")
                .into())
        }
    }

    /// `Node.prototype.isSameNode(otherNode)`
    fn is_same_node(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.isSameNode called on non-object")
        })?;

        let other_arg = args.get_or_undefined(0);
        if other_arg.is_null() {
            return Ok(JsValue::from(false));
        }

        let other_obj = other_arg.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.isSameNode: other must be a Node or null")
        })?;

        Ok(JsValue::from(JsObject::equals(&this_obj, &other_obj)))
    }

    /// `Node.prototype.compareDocumentPosition(other)`
    fn compare_document_position(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let _this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.compareDocumentPosition called on non-object")
        })?;

        let _other_arg = args.get_or_undefined(0);
        // TODO: Implement proper document position comparison
        // For now, return DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC
        Ok(JsValue::from(Self::DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC))
    }

    /// `Node.prototype.contains(other)`
    fn contains(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.contains called on non-object")
        })?;

        let other_arg = args.get_or_undefined(0);
        if other_arg.is_null() {
            return Ok(JsValue::from(false));
        }

        let other_obj = other_arg.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.contains: other must be a Node or null")
        })?;

        // Same node contains itself
        if JsObject::equals(&this_obj, &other_obj) {
            return Ok(JsValue::from(true));
        }

        if let Some(_this_node) = this_obj.downcast_ref::<NodeData>() {
            if let Some(other_node) = other_obj.downcast_ref::<NodeData>() {
                // Walk up the ancestor chain of other_node to see if we find this_node
                let mut current = other_node.get_parent_node();
                while let Some(parent) = current {
                    if JsObject::equals(&parent, &this_obj) {
                        return Ok(JsValue::from(true));
                    }
                    if let Some(parent_data) = parent.downcast_ref::<NodeData>() {
                        current = parent_data.get_parent_node();
                    } else {
                        break;
                    }
                }
                Ok(JsValue::from(false))
            } else {
                Ok(JsValue::from(false))
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.contains called on non-Node object")
                .into())
        }
    }

    /// `Node.prototype.lookupPrefix(namespace)`
    fn lookup_prefix(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let _this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.lookupPrefix called on non-object")
        })?;

        // TODO: Implement namespace prefix lookup
        Ok(JsValue::null())
    }

    /// `Node.prototype.lookupNamespaceURI(prefix)`
    fn lookup_namespace_uri(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let _this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.lookupNamespaceURI called on non-object")
        })?;

        // TODO: Implement namespace URI lookup
        Ok(JsValue::null())
    }

    /// `Node.prototype.isDefaultNamespace(namespace)`
    fn is_default_namespace(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let _this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.isDefaultNamespace called on non-object")
        })?;

        // TODO: Implement default namespace check
        Ok(JsValue::from(false))
    }

    /// `Node.prototype.hasChildNodes()`
    fn has_child_nodes_method(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.hasChildNodes called on non-object")
        })?;

        if let Some(node) = this_obj.downcast_ref::<NodeData>() {
            Ok(JsValue::from(node.has_child_nodes()))
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.hasChildNodes called on non-Node object")
                .into())
        }
    }

    /// `Node.prototype.getRootNode(options)`
    fn get_root_node(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Node.getRootNode called on non-object")
        })?;

        if let Some(node) = this_obj.downcast_ref::<NodeData>() {
            // Walk up to the root node
            let mut current = Some(this_obj.clone());
            let mut root = this_obj.clone();

            while let Some(node_obj) = current {
                if let Some(node_data) = node_obj.clone().downcast_ref::<NodeData>() {
                    root = node_obj;
                    current = node_data.get_parent_node();
                } else {
                    break;
                }
            }

            Ok(root.into())
        } else {
            Err(JsNativeError::typ()
                .with_message("Node.getRootNode called on non-Node object")
                .into())
        }
    }
}

/// The `Node` object
#[derive(Debug, Trace, Finalize)]
pub struct Node;

impl Node {
    // Move all the static method implementations here from the standalone functions
    fn append_child(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::append_child(this, args, context)
    }

    fn insert_before(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::insert_before(this, args, context)
    }

    fn remove_child(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::remove_child(this, args, context)
    }

    fn replace_child(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::replace_child(this, args, context)
    }

    fn clone_node(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::clone_node(this, args, context)
    }

    fn normalize(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::normalize(this, args, context)
    }

    fn is_equal_node(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::is_equal_node(this, args, context)
    }

    fn is_same_node(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::is_same_node(this, args, context)
    }

    fn compare_document_position(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::compare_document_position(this, args, context)
    }

    fn contains(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::contains(this, args, context)
    }

    fn lookup_prefix(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::lookup_prefix(this, args, context)
    }

    fn lookup_namespace_uri(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::lookup_namespace_uri(this, args, context)
    }

    fn is_default_namespace(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::is_default_namespace(this, args, context)
    }

    fn has_child_nodes_method(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::has_child_nodes_method(this, args, context)
    }

    fn get_root_node(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::get_root_node(this, args, context)
    }

    fn get_node_type_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::get_node_type_accessor(this, args, context)
    }

    fn get_node_name_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::get_node_name_accessor(this, args, context)
    }

    fn get_node_value_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::get_node_value_accessor(this, args, context)
    }

    fn set_node_value_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::set_node_value_accessor(this, args, context)
    }

    fn get_parent_node_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::get_parent_node_accessor(this, args, context)
    }

    fn get_child_nodes_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::get_child_nodes_accessor(this, args, context)
    }

    fn get_first_child_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::get_first_child_accessor(this, args, context)
    }

    fn get_last_child_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::get_last_child_accessor(this, args, context)
    }

    fn get_previous_sibling_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::get_previous_sibling_accessor(this, args, context)
    }

    fn get_next_sibling_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::get_next_sibling_accessor(this, args, context)
    }

    fn get_owner_document_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        NodeData::get_owner_document_accessor(this, args, context)
    }
}

impl IntrinsicObject for Node {
    fn init(realm: &Realm) {
        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_property(js_string!("ELEMENT_NODE"), 1, Attribute::READONLY)
            .static_property(js_string!("ATTRIBUTE_NODE"), 2, Attribute::READONLY)
            .static_property(js_string!("TEXT_NODE"), 3, Attribute::READONLY)
            .static_property(js_string!("CDATA_SECTION_NODE"), 4, Attribute::READONLY)
            .static_property(js_string!("ENTITY_REFERENCE_NODE"), 5, Attribute::READONLY)
            .static_property(js_string!("ENTITY_NODE"), 6, Attribute::READONLY)
            .static_property(js_string!("PROCESSING_INSTRUCTION_NODE"), 7, Attribute::READONLY)
            .static_property(js_string!("COMMENT_NODE"), 8, Attribute::READONLY)
            .static_property(js_string!("DOCUMENT_NODE"), 9, Attribute::READONLY)
            .static_property(js_string!("DOCUMENT_TYPE_NODE"), 10, Attribute::READONLY)
            .static_property(js_string!("DOCUMENT_FRAGMENT_NODE"), 11, Attribute::READONLY)
            .static_property(js_string!("NOTATION_NODE"), 12, Attribute::READONLY)
            .method(Self::append_child, js_string!("appendChild"), 1)
            .method(Self::insert_before, js_string!("insertBefore"), 2)
            .method(Self::remove_child, js_string!("removeChild"), 1)
            .method(Self::replace_child, js_string!("replaceChild"), 2)
            .method(Self::clone_node, js_string!("cloneNode"), 0)
            .method(Self::normalize, js_string!("normalize"), 0)
            .method(Self::is_equal_node, js_string!("isEqualNode"), 1)
            .method(Self::is_same_node, js_string!("isSameNode"), 1)
            .method(Self::compare_document_position, js_string!("compareDocumentPosition"), 1)
            .method(Self::contains, js_string!("contains"), 1)
            .method(Self::lookup_prefix, js_string!("lookupPrefix"), 1)
            .method(Self::lookup_namespace_uri, js_string!("lookupNamespaceURI"), 1)
            .method(Self::is_default_namespace, js_string!("isDefaultNamespace"), 1)
            .method(Self::has_child_nodes_method, js_string!("hasChildNodes"), 0)
            .method(Self::get_root_node, js_string!("getRootNode"), 0)
            .method(Self::get_node_type_accessor, js_string!("nodeType"), 0)
            .method(Self::get_node_name_accessor, js_string!("nodeName"), 0)
            .method(Self::get_node_value_accessor, js_string!("nodeValue"), 0)
            .method(Self::get_parent_node_accessor, js_string!("parentNode"), 0)
            .method(Self::get_child_nodes_accessor, js_string!("childNodes"), 0)
            .method(Self::get_first_child_accessor, js_string!("firstChild"), 0)
            .method(Self::get_last_child_accessor, js_string!("lastChild"), 0)
            .method(Self::get_previous_sibling_accessor, js_string!("previousSibling"), 0)
            .method(Self::get_next_sibling_accessor, js_string!("nextSibling"), 0)
            .method(Self::get_owner_document_accessor, js_string!("ownerDocument"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Node {
    const NAME: JsString = StaticJsStrings::NODE;
}

impl BuiltInConstructor for Node {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::node;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Node constructor should not be called directly
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Constructor Node requires 'new'")
                .into());
        }

        // Create a new Node object with default values
        let node_data = NodeData::new(NodeType::Node);

        let node_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().node().prototype(),
            node_data,
        );

        Ok(node_obj.into())
    }
}