//! Document Web API implementation for Boa
//!
//! Native implementation of Document standard
//! https://dom.spec.whatwg.org/#interface-document

use crate::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    NativeFunction,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::{Attribute, PropertyDescriptorBuilder}
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// JavaScript `Document` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Document;

impl IntrinsicObject for Document {
    fn init(realm: &Realm) {
        let ready_state_func = BuiltInBuilder::callable(realm, get_ready_state)
            .name(js_string!("get readyState"))
            .build();

        let url_func = BuiltInBuilder::callable(realm, get_url)
            .name(js_string!("get URL"))
            .build();

        let title_func = BuiltInBuilder::callable(realm, get_title)
            .name(js_string!("get title"))
            .build();

        let title_setter_func = BuiltInBuilder::callable(realm, set_title)
            .name(js_string!("set title"))
            .build();

        let body_func = BuiltInBuilder::callable(realm, get_body)
            .name(js_string!("get body"))
            .build();

        let head_func = BuiltInBuilder::callable(realm, get_head)
            .name(js_string!("get head"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("readyState"),
                Some(ready_state_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("URL"),
                Some(url_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("title"),
                Some(title_func),
                Some(title_setter_func),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("body"),
                Some(body_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("head"),
                Some(head_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(create_element, js_string!("createElement"), 1)
            .method(get_element_by_id, js_string!("getElementById"), 1)
            .method(query_selector, js_string!("querySelector"), 1)
            .method(query_selector_all, js_string!("querySelectorAll"), 1)
            .method(add_event_listener, js_string!("addEventListener"), 2)
            .method(remove_event_listener, js_string!("removeEventListener"), 2)
            .method(dispatch_event, js_string!("dispatchEvent"), 1)
            .method(start_view_transition, js_string!("startViewTransition"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Document {
    const NAME: JsString = StaticJsStrings::DOCUMENT;
}

impl BuiltInConstructor for Document {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::document;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::document,
            context,
        )?;

        let document_data = DocumentData::new();

        let document = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            document_data,
        );

        Ok(document.into())
    }
}

/// Internal data for Document objects
#[derive(Debug, Trace, Finalize, JsData)]
pub struct DocumentData {
    #[unsafe_ignore_trace]
    ready_state: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    url: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    title: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    elements: Arc<Mutex<HashMap<String, JsObject>>>,
    #[unsafe_ignore_trace]
    event_listeners: Arc<Mutex<HashMap<String, Vec<JsValue>>>>,
    #[unsafe_ignore_trace]
    html_content: Arc<Mutex<String>>,
}

impl DocumentData {
    fn new() -> Self {
        let doc_data = Self {
            ready_state: Arc::new(Mutex::new("loading".to_string())),
            url: Arc::new(Mutex::new("about:blank".to_string())),
            title: Arc::new(Mutex::new("".to_string())),
            elements: Arc::new(Mutex::new(HashMap::new())),
            event_listeners: Arc::new(Mutex::new(HashMap::new())),
            html_content: Arc::new(Mutex::new("".to_string())),
        };

        // Set up DOM sync bridge - connect Element changes to Document updates
        use crate::builtins::element::GLOBAL_DOM_SYNC;
        let html_content_ref = doc_data.html_content.clone();
        GLOBAL_DOM_SYNC.get_or_init(|| crate::builtins::element::DomSync::new())
            .set_updater(Box::new(move |html| {
                *html_content_ref.lock().unwrap() = html.to_string();
            }));

        doc_data
    }

    pub fn set_ready_state(&self, state: &str) {
        *self.ready_state.lock().unwrap() = state.to_string();
    }

    pub fn set_url(&self, url: &str) {
        *self.url.lock().unwrap() = url.to_string();
    }

    pub fn set_title(&self, title: &str) {
        *self.title.lock().unwrap() = title.to_string();
    }

    pub fn set_html_content(&self, html: &str) {
        *self.html_content.lock().unwrap() = html.to_string();

        // Process all forms in the HTML and prepare them for DOM access
        self.process_forms_in_html(html);
    }

    pub fn update_html_from_dom(&self, html: &str) {
        *self.html_content.lock().unwrap() = html.to_string();
    }

    pub fn get_html_content(&self) -> String {
        self.html_content.lock().unwrap().clone()
    }

    pub fn get_ready_state(&self) -> String {
        self.ready_state.lock().unwrap().clone()
    }

    pub fn get_url(&self) -> String {
        self.url.lock().unwrap().clone()
    }

    pub fn get_title(&self) -> String {
        self.title.lock().unwrap().clone()
    }

    pub fn add_element(&self, id: String, element: JsObject) {
        self.elements.lock().unwrap().insert(id, element);
    }

    pub fn get_element(&self, id: &str) -> Option<JsObject> {
        self.elements.lock().unwrap().get(id).cloned()
    }

    /// Process all forms in HTML content and prepare elements collections
    /// This ensures that forms accessed via DOM events have proper elements collections
    fn process_forms_in_html(&self, html_content: &str) {
        eprintln!("🔍 DEBUG: process_forms_in_html called with {} characters of HTML", html_content.len());

        // Parse the HTML content to find all forms
        let document = scraper::Html::parse_document(html_content);

        // Find all form elements
        if let Ok(form_selector) = scraper::Selector::parse("form") {
            let form_count = document.select(&form_selector).count();
            eprintln!("🔍 DEBUG: Found {} forms in HTML", form_count);

            for (form_index, form_element) in document.select(&form_selector).enumerate() {
                // Create a unique ID for this form if it doesn't have one
                let form_id = if let Some(id) = form_element.value().attr("id") {
                    id.to_string()
                } else {
                    format!("auto_form_{}", form_index)
                };

                // Store form metadata for later DOM access
                let mut form_inputs = Vec::new();

                // Parse form's inner HTML to find input elements
                let form_inner_html = form_element.inner_html();
                let form_doc = scraper::Html::parse_fragment(&form_inner_html);

                if let Ok(input_selector) = scraper::Selector::parse("input") {
                    for input_element in form_doc.select(&input_selector) {
                        if let Some(input_name) = input_element.value().attr("name") {
                            let input_value = input_element.value().attr("value").unwrap_or("").to_string();
                            let input_type = input_element.value().attr("type").unwrap_or("text").to_string();

                            form_inputs.push((input_name.to_string(), input_value, input_type));
                        }
                    }
                }

                // Store the form metadata for later JavaScript access
                // We'll use this when DOM queries ask for this form
                self.add_form_metadata(form_id, form_inputs);
            }
        }
    }

    /// Add form metadata that can be used when creating form elements in JavaScript
    fn add_form_metadata(&self, form_id: String, inputs: Vec<(String, String, String)>) {
        // Create an HTMLFormElement with proper elements collection
        use crate::builtins::form::{HTMLFormElement, HTMLInputElement, HTMLFormControlsCollection};
        use boa_engine::{Context, object::ObjectInitializer, js_string};

        // For now, store the metadata - we'll need a context to create the actual objects
        // This processing happens at document level so all forms are known before JavaScript queries them
        // TODO: This needs to be enhanced to create actual JavaScript objects when we have a context
        eprintln!("🔍 DEBUG: Found form '{}' with {} inputs", form_id, inputs.len());
        for (name, value, input_type) in &inputs {
            eprintln!("🔍 DEBUG: - Input '{}' = '{}' (type: {})", name, value, input_type);
        }
    }

    pub fn add_event_listener(&self, event_type: String, listener: JsValue) {
        self.event_listeners.lock().unwrap()
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(listener);
    }

    pub fn remove_event_listener(&self, event_type: &str, listener: &JsValue) {
        if let Some(listeners) = self.event_listeners.lock().unwrap().get_mut(event_type) {
            listeners.retain(|l| !JsValue::same_value(l, listener));
        }
    }

    pub fn get_event_listeners(&self, event_type: &str) -> Vec<JsValue> {
        self.event_listeners.lock().unwrap()
            .get(event_type)
            .cloned()
            .unwrap_or_default()
    }
}

/// `Document.prototype.readyState` getter
fn get_ready_state(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.readyState called on non-object")
    })?;

    if let Some(document) = this_obj.downcast_ref::<DocumentData>() {
        Ok(JsString::from(document.get_ready_state()).into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Document.prototype.readyState called on non-Document object")
            .into())
    }
}

/// `Document.prototype.URL` getter
fn get_url(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.URL called on non-object")
    })?;

    if let Some(document) = this_obj.downcast_ref::<DocumentData>() {
        Ok(JsString::from(document.get_url()).into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Document.prototype.URL called on non-Document object")
            .into())
    }
}

/// `Document.prototype.title` getter
fn get_title(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.title called on non-object")
    })?;

    if let Some(document) = this_obj.downcast_ref::<DocumentData>() {
        Ok(JsString::from(document.get_title()).into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Document.prototype.title called on non-Document object")
            .into())
    }
}

/// `Document.prototype.title` setter
fn set_title(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.title setter called on non-object")
    })?;

    if let Some(document) = this_obj.downcast_ref::<DocumentData>() {
        let title = args.get_or_undefined(0).to_string(context)?;
        document.set_title(&title.to_std_string_escaped());
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("Document.prototype.title setter called on non-Document object")
            .into())
    }
}

/// `Document.prototype.body` getter
fn get_body(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.body called on non-object")
    })?;

    if let Some(document) = this_obj.downcast_ref::<DocumentData>() {
        // Create body element if it doesn't exist
        if let Some(body) = document.get_element("body") {
            Ok(body.into())
        } else {
            // Create a new body element using the Element constructor
            let element_constructor = context.intrinsics().constructors().element().constructor();
            let body_element = element_constructor.construct(&[], None, context)?;

            document.add_element("body".to_string(), body_element.clone());
            Ok(body_element.into())
        }
    } else {
        Err(JsNativeError::typ()
            .with_message("Document.prototype.body called on non-Document object")
            .into())
    }
}

/// `Document.prototype.head` getter
fn get_head(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.head called on non-object")
    })?;

    if let Some(document) = this_obj.downcast_ref::<DocumentData>() {
        // Create head element if it doesn't exist
        if let Some(head) = document.get_element("head") {
            Ok(head.into())
        } else {
            // Create a new head element
            let head_element = JsObject::default();

            // Add tagName property
            head_element.define_property_or_throw(
                js_string!("tagName"),
                PropertyDescriptorBuilder::new()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(JsString::from("HEAD"))
                    .build(),
                context,
            )?;

            document.add_element("head".to_string(), head_element.clone());
            Ok(head_element.into())
        }
    } else {
        Err(JsNativeError::typ()
            .with_message("Document.prototype.head called on non-Document object")
            .into())
    }
}

/// `Document.prototype.createElement(tagName)`
fn create_element(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.createElement called on non-object")
    })?;

    if let Some(_document) = this_obj.downcast_ref::<DocumentData>() {
        let tag_name = args.get_or_undefined(0).to_string(context)?;
        let tag_name_upper = tag_name.to_std_string_escaped().to_uppercase();

        // Create a proper Element object using Element constructor pattern
        let element_constructor = context.intrinsics().constructors().element().constructor();
        let element = crate::builtins::element::Element::constructor(
            &element_constructor.clone().into(),
            &[],
            context,
        )?;

        // Get the Element object from the JsValue
        let element_obj = element.as_object().unwrap();

        // Add tagName property (this should be done by ElementData, but make it explicit)
        element_obj.define_property_or_throw(
            js_string!("tagName"),
            PropertyDescriptorBuilder::new()
                .configurable(false)
                .enumerable(true)
                .writable(false)
                .value(JsString::from(tag_name_upper.as_str()))
                .build(),
            context,
        )?;

        // Set the tag name in the element data
        if let Some(element_data) = element_obj.downcast_ref::<crate::builtins::element::ElementData>() {
            element_data.set_tag_name(tag_name_upper.clone());
        }

        // Add style property as empty object
        let style_obj = JsObject::default();
        element_obj.define_property_or_throw(
            js_string!("style"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(style_obj)
                .build(),
            context,
        )?;

        // Add Form-specific functionality for <form> elements
        if tag_name_upper == "FORM" {
            // Create elements collection that Google's code expects
            let elements_collection = JsObject::default();

            // Add common form controls as properties of elements collection
            // Google often checks for elements like 'q' (search query)
            let q_element = JsObject::default();
            q_element.define_property_or_throw(
                js_string!("value"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(js_string!(""))
                    .build(),
                context,
            )?;

            elements_collection.define_property_or_throw(
                js_string!("q"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(q_element)
                    .build(),
                context,
            )?;

            // Add elements collection to form
            element_obj.define_property_or_throw(
                js_string!("elements"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(elements_collection)
                    .build(),
                context,
            )?;

            // Add getAttribute method that Google's code uses
            let get_attribute_func = BuiltInBuilder::callable(context.realm(), |this, args, ctx| {
                let attr_name = args.get_or_undefined(0).to_string(ctx)?;
                let attr_name_str = attr_name.to_std_string_escaped();

                // Return common attributes that Google checks
                match attr_name_str.as_str() {
                    "data-submitfalse" => Ok(JsValue::null()), // Google checks this
                    _ => Ok(JsValue::null())
                }
            })
            .name(js_string!("getAttribute"))
            .build();

            element_obj.define_property_or_throw(
                js_string!("getAttribute"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(get_attribute_func)
                    .build(),
                context,
            )?;
        }

        // Add Button-specific functionality for <button> elements
        if tag_name_upper == "BUTTON" {
            // Add button-specific properties
            element_obj.define_property_or_throw(
                js_string!("type"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(js_string!("button"))
                    .build(),
                context,
            )?;

            element_obj.define_property_or_throw(
                js_string!("value"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(js_string!(""))
                    .build(),
                context,
            )?;
        }

        // Add Canvas-specific functionality for <canvas> elements
        if tag_name_upper == "CANVAS" {
            // Add width and height properties with default values
            element_obj.define_property_or_throw(
                js_string!("width"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(300) // Default canvas width
                    .build(),
                context,
            )?;

            element_obj.define_property_or_throw(
                js_string!("height"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(150) // Default canvas height
                    .build(),
                context,
            )?;

            // Add getContext method
            let get_context_func = BuiltInBuilder::callable(context.realm(), canvas_get_context)
                .name(js_string!("getContext"))
                .build();

            element_obj.define_property_or_throw(
                js_string!("getContext"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(get_context_func)
                    .build(),
                context,
            )?;

            // Add toDataURL method
            let to_data_url_func = BuiltInBuilder::callable(context.realm(), canvas_to_data_url)
                .name(js_string!("toDataURL"))
                .build();

            element_obj.define_property_or_throw(
                js_string!("toDataURL"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(to_data_url_func)
                    .build(),
                context,
            )?;
        }

        Ok(element)
    } else {
        Err(JsNativeError::typ()
            .with_message("Document.prototype.createElement called on non-Document object")
            .into())
    }
}

/// `Document.prototype.getElementById(id)`
fn get_element_by_id(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.getElementById called on non-object")
    })?;

    if let Some(document) = this_obj.downcast_ref::<DocumentData>() {
        let id = args.get_or_undefined(0).to_string(context)?;

        if let Some(element) = document.get_element(&id.to_std_string_escaped()) {
            Ok(element.into())
        } else {
            Ok(JsValue::null())
        }
    } else {
        Err(JsNativeError::typ()
            .with_message("Document.prototype.getElementById called on non-Document object")
            .into())
    }
}

/// `Document.prototype.querySelector(selector)`
fn query_selector(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    eprintln!("DEBUG: query_selector called!");

    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.querySelector called on non-object")
    })?;

    if let Some(document) = this_obj.downcast_ref::<DocumentData>() {
        let selector = args.get_or_undefined(0).to_string(context)?;
        let selector_str = selector.to_std_string_escaped();
        eprintln!("DEBUG: query_selector selector: {}", selector_str);

        // Get the HTML content from the document
        let html_content = document.get_html_content();
        eprintln!("DEBUG: query_selector HTML content length: {}", html_content.len());

        // Use real DOM implementation with scraper library
        if let Some(element) = create_real_element_from_html(context, &selector_str, &html_content)? {
            return Ok(element.into());
        }

        eprintln!("DEBUG: query_selector returning null - no element found");
        Ok(JsValue::null())
    } else {
        Err(JsNativeError::typ()
            .with_message("Document.prototype.querySelector called on non-Document object")
            .into())
    }
}

/// Real DOM element creation using scraper library and actual HTML content
fn create_real_element_from_html(context: &mut Context, selector: &str, html_content: &str) -> JsResult<Option<JsObject>> {
    // Use the scraper crate to parse real HTML and find elements
    let document = scraper::Html::parse_document(html_content);

    if let Ok(css_selector) = scraper::Selector::parse(selector) {
        if let Some(element_ref) = document.select(&css_selector).next() {
            eprintln!("DEBUG: querySelector creating element using Element constructor");

            // Actually construct a new Element instance using the Element constructor
            let element_constructor = context.intrinsics().constructors().element().constructor();
            let element_obj = element_constructor.construct(&[], Some(&element_constructor), context)?;

            eprintln!("DEBUG: Element created, checking for dispatchEvent...");
            if let Ok(dispatch_event) = element_obj.get(js_string!("dispatchEvent"), context) {
                eprintln!("DEBUG: dispatchEvent found on created element: {:?}", dispatch_event.type_of());
            } else {
                eprintln!("DEBUG: dispatchEvent NOT found on created element!");
            }

            // Set real properties from the actual HTML element
            let tag_name = element_ref.value().name().to_uppercase();
            element_obj.set(js_string!("tagName"), js_string!(tag_name.clone()), false, context)?;
            element_obj.set(js_string!("nodeType"), 1, false, context)?; // ELEMENT_NODE

            // Set real attributes from the HTML
            for (attr_name, attr_value) in element_ref.value().attrs() {
                element_obj.set(js_string!(attr_name), js_string!(attr_value), false, context)?;
            }

            // Set text content
            let text_content: String = element_ref.text().collect();
            element_obj.set(js_string!("textContent"), js_string!(text_content), false, context)?;

            // Set innerHTML
            let inner_html = element_ref.inner_html();
            element_obj.set(js_string!("innerHTML"), js_string!(inner_html), false, context)?;

            // Add common DOM methods
            let focus_fn = context.intrinsics().constructors().function().constructor();
            element_obj.set(js_string!("focus"), focus_fn, false, context)?;

            let click_fn = context.intrinsics().constructors().function().constructor();
            element_obj.set(js_string!("click"), click_fn, false, context)?;

            // Add value property for input elements
            if tag_name == "INPUT" {
                if let Some(value) = element_ref.value().attr("value") {
                    element_obj.set(js_string!("value"), js_string!(value), false, context)?;
                } else {
                    element_obj.set(js_string!("value"), js_string!(""), false, context)?;
                }

                // Add name property for input elements (needed for form.elements access)
                if let Some(name) = element_ref.value().attr("name") {
                    element_obj.set(js_string!("name"), js_string!(name), false, context)?;
                }
            }

            // Add form-specific functionality for FORM elements from HTML
            if tag_name == "FORM" {
                // Create elements collection
                let elements_collection = context.intrinsics().constructors().object().constructor();

                // Find all input elements within this form using the HTML content
                let form_selector = scraper::Selector::parse("input").unwrap();

                // Parse the inner HTML of this form to find inputs
                let form_inner_html = element_ref.inner_html();
                let form_doc = scraper::Html::parse_fragment(&form_inner_html);

                for input_element in form_doc.select(&form_selector) {
                    if let Some(input_name) = input_element.value().attr("name") {
                        // Create input element object
                        let input_obj = context.intrinsics().constructors().object().constructor();

                        // Add value property
                        if let Some(input_value) = input_element.value().attr("value") {
                            input_obj.set(js_string!("value"), js_string!(input_value), false, context)?;
                        } else {
                            input_obj.set(js_string!("value"), js_string!(""), false, context)?;
                        }

                        // Add name property
                        input_obj.set(js_string!("name"), js_string!(input_name), false, context)?;

                        // Add input type
                        if let Some(input_type) = input_element.value().attr("type") {
                            input_obj.set(js_string!("type"), js_string!(input_type), false, context)?;
                        } else {
                            input_obj.set(js_string!("type"), js_string!("text"), false, context)?;
                        }

                        // Add this input to the elements collection by name
                        elements_collection.set(js_string!(input_name), input_obj, false, context)?;
                    }
                }

                // Add elements collection to the form
                element_obj.set(js_string!("elements"), elements_collection, false, context)?;

                // Add getAttribute method that Google's code needs
                let get_attribute_func = BuiltInBuilder::callable(context.realm(), |this, args, ctx| {
                    let attr_name = args.get_or_undefined(0).to_string(ctx)?;
                    let attr_name_str = attr_name.to_std_string_escaped();

                    // Return common attributes that Google checks
                    match attr_name_str.as_str() {
                        "data-submitfalse" => Ok(JsValue::null()), // Google checks this
                        _ => Ok(JsValue::null())
                    }
                })
                .name(js_string!("getAttribute"))
                .build();

                element_obj.set(js_string!("getAttribute"), get_attribute_func, false, context)?;
            }

            return Ok(Some(element_obj));
        }
    }

    Ok(None)
}

/// Real DOM elements creation using scraper library to find all matching elements
fn create_all_real_elements_from_html(context: &mut Context, selector: &str, html_content: &str) -> JsResult<Vec<JsValue>> {
    let mut elements = Vec::new();

    // Use the scraper crate to parse real HTML and find all elements
    let document = scraper::Html::parse_document(html_content);

    if let Ok(css_selector) = scraper::Selector::parse(selector) {
        for element_ref in document.select(&css_selector) {
            let element_obj = context.intrinsics().constructors().element().constructor();

            // Set real properties from the actual HTML element
            let tag_name = element_ref.value().name().to_uppercase();
            element_obj.set(js_string!("tagName"), js_string!(tag_name.clone()), false, context)?;
            element_obj.set(js_string!("nodeType"), 1, false, context)?; // ELEMENT_NODE

            // Set real attributes from the HTML
            for (attr_name, attr_value) in element_ref.value().attrs() {
                element_obj.set(js_string!(attr_name), js_string!(attr_value), false, context)?;
            }

            // Set text content
            let text_content: String = element_ref.text().collect();
            element_obj.set(js_string!("textContent"), js_string!(text_content), false, context)?;

            // Set innerHTML
            let inner_html = element_ref.inner_html();
            element_obj.set(js_string!("innerHTML"), js_string!(inner_html), false, context)?;

            // Add common DOM methods
            let focus_fn = context.intrinsics().constructors().function().constructor();
            element_obj.set(js_string!("focus"), focus_fn, false, context)?;

            let click_fn = context.intrinsics().constructors().function().constructor();
            element_obj.set(js_string!("click"), click_fn, false, context)?;

            // Add value property for input elements
            if tag_name == "INPUT" {
                if let Some(value) = element_ref.value().attr("value") {
                    element_obj.set(js_string!("value"), js_string!(value), false, context)?;
                } else {
                    element_obj.set(js_string!("value"), js_string!(""), false, context)?;
                }
            }

            elements.push(element_obj.into());
        }
    }

    Ok(elements)
}

/// `Document.prototype.querySelectorAll(selector)`
fn query_selector_all(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.querySelectorAll called on non-object")
    })?;

    if let Some(document) = this_obj.downcast_ref::<DocumentData>() {
        let selector = args.get_or_undefined(0).to_string(context)?;
        let selector_str = selector.to_std_string_escaped();

        // Get the HTML content from the document
        let html_content = document.get_html_content();

        // Use real DOM implementation with scraper library to find all matching elements
        let elements = create_all_real_elements_from_html(context, &selector_str, &html_content)?;

        use crate::builtins::Array;
        let array = Array::create_array_from_list(elements, context);
        Ok(array.into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Document.prototype.querySelectorAll called on non-Document object")
            .into())
    }
}

/// `Document.prototype.addEventListener(type, listener)`
fn add_event_listener(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.addEventListener called on non-object")
    })?;

    if let Some(document) = this_obj.downcast_ref::<DocumentData>() {
        let event_type = args.get_or_undefined(0).to_string(context)?;
        let listener = args.get_or_undefined(1).clone();

        document.add_event_listener(event_type.to_std_string_escaped(), listener);
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("Document.prototype.addEventListener called on non-Document object")
            .into())
    }
}

/// `Document.prototype.removeEventListener(type, listener)`
fn remove_event_listener(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.removeEventListener called on non-object")
    })?;

    if let Some(document) = this_obj.downcast_ref::<DocumentData>() {
        let event_type = args.get_or_undefined(0).to_string(context)?;
        let listener = args.get_or_undefined(1);

        document.remove_event_listener(&event_type.to_std_string_escaped(), listener);
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("Document.prototype.removeEventListener called on non-Document object")
            .into())
    }
}

/// `Document.prototype.dispatchEvent(event)`
fn dispatch_event(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.dispatchEvent called on non-object")
    })?;

    if let Some(document) = this_obj.downcast_ref::<DocumentData>() {
        let event = args.get_or_undefined(0);

        // Get event type from event object
        if event.is_object() {
            if let Some(event_obj) = event.as_object() {
                if let Ok(type_val) = event_obj.get(js_string!("type"), context) {
                    let event_type = type_val.to_string(context)?;
                    let listeners = document.get_event_listeners(&event_type.to_std_string_escaped());

                    // Call each listener
                    for listener in listeners {
                        if listener.is_callable() {
                            let _ = listener.as_callable().unwrap().call(
                                &this_obj.clone().into(),
                                &[event.clone()],
                                context,
                            );
                        }
                    }
                }
            }
        }

        Ok(true.into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Document.prototype.dispatchEvent called on non-Document object")
            .into())
    }
}

/// `Document.prototype.startViewTransition(callback)`
fn start_view_transition(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.startViewTransition called on non-object")
    })?;

    if let Some(_document) = this_obj.downcast_ref::<DocumentData>() {
        let callback = args.get_or_undefined(0);

        // Create transition object
        let transition = JsObject::default();

        // Add finished property as resolved Promise
        let finished_promise = context.eval(boa_engine::Source::from_bytes("Promise.resolve()"))?;
        transition.define_property_or_throw(
            js_string!("finished"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(false)
                .value(finished_promise)
                .build(),
            context,
        )?;

        // Add ready property as resolved Promise
        let ready_promise = context.eval(boa_engine::Source::from_bytes("Promise.resolve()"))?;
        transition.define_property_or_throw(
            js_string!("ready"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(false)
                .value(ready_promise)
                .build(),
            context,
        )?;

        // Handle callback if provided
        let mut callback_promise = context.eval(boa_engine::Source::from_bytes("Promise.resolve()"))?;
        if !callback.is_undefined() && callback.is_callable() {
            // Call the callback function
            if let Ok(result) = callback.as_callable()
                .unwrap()
                .call(&JsValue::undefined(), &[], context) {

                // Check if result is a promise
                if result.is_object() {
                    if let Some(obj) = result.as_object() {
                        if obj.has_property(js_string!("then"), context)? {
                            callback_promise = result;
                        }
                    }
                }
            }
        }

        // Add updateCallbackDone property
        transition.define_property_or_throw(
            js_string!("updateCallbackDone"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(false)
                .value(callback_promise)
                .build(),
            context,
        )?;

        // Add skipTransition method
        let skip_function = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
            Ok(JsValue::undefined())
        })
        .name(js_string!("skipTransition"))
        .build();

        transition.define_property_or_throw(
            js_string!("skipTransition"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(false)
                .value(skip_function)
                .build(),
            context,
        )?;

        Ok(transition.into())
    } else {
        Err(JsNativeError::typ()
            .with_message("Document.prototype.startViewTransition called on non-Document object")
            .into())
    }
}

/// Canvas `getContext(contextType)` method implementation
fn canvas_get_context(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let context_type = args.get_or_undefined(0).to_string(context)?;
    let context_type_str = context_type.to_std_string_escaped();

    match context_type_str.as_str() {
        "2d" => {
            // Create a Canvas 2D rendering context object
            let context_2d = JsObject::default();

            // Add Canvas 2D methods
            // Drawing rectangles
            let fill_rect_func = BuiltInBuilder::callable(context.realm(), canvas_2d_fill_rect)
                .name(js_string!("fillRect"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("fillRect"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(fill_rect_func)
                    .build(),
                context,
            )?;

            let stroke_rect_func = BuiltInBuilder::callable(context.realm(), canvas_2d_stroke_rect)
                .name(js_string!("strokeRect"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("strokeRect"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(stroke_rect_func)
                    .build(),
                context,
            )?;

            let clear_rect_func = BuiltInBuilder::callable(context.realm(), canvas_2d_clear_rect)
                .name(js_string!("clearRect"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("clearRect"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(clear_rect_func)
                    .build(),
                context,
            )?;

            // Text rendering
            let fill_text_func = BuiltInBuilder::callable(context.realm(), canvas_2d_fill_text)
                .name(js_string!("fillText"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("fillText"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(fill_text_func)
                    .build(),
                context,
            )?;

            let stroke_text_func = BuiltInBuilder::callable(context.realm(), canvas_2d_stroke_text)
                .name(js_string!("strokeText"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("strokeText"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(stroke_text_func)
                    .build(),
                context,
            )?;

            let measure_text_func = BuiltInBuilder::callable(context.realm(), canvas_2d_measure_text)
                .name(js_string!("measureText"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("measureText"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(measure_text_func)
                    .build(),
                context,
            )?;

            // Path methods
            let begin_path_func = BuiltInBuilder::callable(context.realm(), canvas_2d_begin_path)
                .name(js_string!("beginPath"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("beginPath"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(begin_path_func)
                    .build(),
                context,
            )?;

            let move_to_func = BuiltInBuilder::callable(context.realm(), canvas_2d_move_to)
                .name(js_string!("moveTo"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("moveTo"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(move_to_func)
                    .build(),
                context,
            )?;

            let line_to_func = BuiltInBuilder::callable(context.realm(), canvas_2d_line_to)
                .name(js_string!("lineTo"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("lineTo"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(line_to_func)
                    .build(),
                context,
            )?;

            let stroke_func = BuiltInBuilder::callable(context.realm(), canvas_2d_stroke)
                .name(js_string!("stroke"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("stroke"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(stroke_func)
                    .build(),
                context,
            )?;

            let fill_func = BuiltInBuilder::callable(context.realm(), canvas_2d_fill)
                .name(js_string!("fill"))
                .build();
            context_2d.define_property_or_throw(
                js_string!("fill"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(fill_func)
                    .build(),
                context,
            )?;

            // Style properties
            context_2d.define_property_or_throw(
                js_string!("fillStyle"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(js_string!("#000000"))
                    .build(),
                context,
            )?;

            context_2d.define_property_or_throw(
                js_string!("strokeStyle"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(js_string!("#000000"))
                    .build(),
                context,
            )?;

            context_2d.define_property_or_throw(
                js_string!("lineWidth"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(1.0)
                    .build(),
                context,
            )?;

            context_2d.define_property_or_throw(
                js_string!("font"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(js_string!("10px sans-serif"))
                    .build(),
                context,
            )?;

            Ok(context_2d.into())
        }
        "webgl" | "experimental-webgl" => {
            create_webgl_context(context, false)
        }
        "webgl2" | "experimental-webgl2" => {
            create_webgl_context(context, true)
        }
        _ => Ok(JsValue::null())
    }
}

/// Canvas `toDataURL(type, quality)` method implementation
fn canvas_to_data_url(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _mime_type = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    let _quality = args.get_or_undefined(1).to_number(context)?;

    // For now, return a minimal empty PNG data URL
    // TODO: Implement actual image generation
    Ok(js_string!("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==").into())
}

// Canvas 2D context method implementations
fn canvas_2d_fill_rect(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _x = args.get_or_undefined(0).to_number(context)?;
    let _y = args.get_or_undefined(1).to_number(context)?;
    let _width = args.get_or_undefined(2).to_number(context)?;
    let _height = args.get_or_undefined(3).to_number(context)?;

    // TODO: Implement actual rectangle drawing
    eprintln!("Canvas fillRect({}, {}, {}, {})", _x, _y, _width, _height);

    Ok(JsValue::undefined())
}

fn canvas_2d_stroke_rect(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _x = args.get_or_undefined(0).to_number(context)?;
    let _y = args.get_or_undefined(1).to_number(context)?;
    let _width = args.get_or_undefined(2).to_number(context)?;
    let _height = args.get_or_undefined(3).to_number(context)?;

    // TODO: Implement actual rectangle outlining
    eprintln!("Canvas strokeRect({}, {}, {}, {})", _x, _y, _width, _height);

    Ok(JsValue::undefined())
}

fn canvas_2d_clear_rect(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _x = args.get_or_undefined(0).to_number(context)?;
    let _y = args.get_or_undefined(1).to_number(context)?;
    let _width = args.get_or_undefined(2).to_number(context)?;
    let _height = args.get_or_undefined(3).to_number(context)?;

    // TODO: Implement actual rectangle clearing
    eprintln!("Canvas clearRect({}, {}, {}, {})", _x, _y, _width, _height);

    Ok(JsValue::undefined())
}

fn canvas_2d_fill_text(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let text = args.get_or_undefined(0).to_string(context)?;
    let _x = args.get_or_undefined(1).to_number(context)?;
    let _y = args.get_or_undefined(2).to_number(context)?;
    let _max_width = args.get_or_undefined(3);

    // TODO: Implement actual text rendering
    eprintln!("Canvas fillText('{}', {}, {})", text.to_std_string_escaped(), _x, _y);

    Ok(JsValue::undefined())
}

fn canvas_2d_stroke_text(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let text = args.get_or_undefined(0).to_string(context)?;
    let _x = args.get_or_undefined(1).to_number(context)?;
    let _y = args.get_or_undefined(2).to_number(context)?;
    let _max_width = args.get_or_undefined(3);

    // TODO: Implement actual text stroking
    eprintln!("Canvas strokeText('{}', {}, {})", text.to_std_string_escaped(), _x, _y);

    Ok(JsValue::undefined())
}

fn canvas_2d_measure_text(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let text = args.get_or_undefined(0).to_string(context)?;

    // Create TextMetrics object
    let metrics = JsObject::default();

    // Calculate approximate width (very basic implementation)
    let text_width = text.to_std_string_escaped().len() as f64 * 6.0; // Rough estimate

    metrics.define_property_or_throw(
        js_string!("width"),
        PropertyDescriptorBuilder::new()
            .configurable(true)
            .enumerable(true)
            .writable(false)
            .value(text_width)
            .build(),
        context,
    )?;

    // TODO: Add other TextMetrics properties (actualBoundingBoxLeft, etc.)

    Ok(metrics.into())
}

fn canvas_2d_begin_path(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // TODO: Implement path state management
    eprintln!("Canvas beginPath()");
    Ok(JsValue::undefined())
}

fn canvas_2d_move_to(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _x = args.get_or_undefined(0).to_number(context)?;
    let _y = args.get_or_undefined(1).to_number(context)?;

    // TODO: Implement path cursor movement
    eprintln!("Canvas moveTo({}, {})", _x, _y);

    Ok(JsValue::undefined())
}

fn canvas_2d_line_to(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _x = args.get_or_undefined(0).to_number(context)?;
    let _y = args.get_or_undefined(1).to_number(context)?;

    // TODO: Implement line drawing to path
    eprintln!("Canvas lineTo({}, {})", _x, _y);

    Ok(JsValue::undefined())
}

fn canvas_2d_stroke(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // TODO: Implement path stroking
    eprintln!("Canvas stroke()");
    Ok(JsValue::undefined())
}

fn canvas_2d_fill(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // TODO: Implement path filling
    eprintln!("Canvas fill()");
    Ok(JsValue::undefined())
}

/// Create WebGL context with comprehensive method support
fn create_webgl_context(context: &mut Context, is_webgl2: bool) -> JsResult<JsValue> {
    let gl_context = JsObject::default();

    // WebGL constants (subset of most commonly used)
    gl_context.set(js_string!("VERTEX_SHADER"), JsValue::from(35633), false, context)?;
    gl_context.set(js_string!("FRAGMENT_SHADER"), JsValue::from(35632), false, context)?;
    gl_context.set(js_string!("ARRAY_BUFFER"), JsValue::from(34962), false, context)?;
    gl_context.set(js_string!("STATIC_DRAW"), JsValue::from(35044), false, context)?;
    gl_context.set(js_string!("COLOR_BUFFER_BIT"), JsValue::from(16384), false, context)?;
    gl_context.set(js_string!("TRIANGLES"), JsValue::from(4), false, context)?;
    gl_context.set(js_string!("FLOAT"), JsValue::from(5126), false, context)?;

    // Core WebGL methods
    let create_shader_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
        let shader_obj = JsObject::default();
        Ok(JsValue::from(shader_obj))
    }) };
    gl_context.set(js_string!("createShader"), JsValue::from(create_shader_fn.to_js_function(context.realm())), false, context)?;

    let create_program_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
        let program_obj = JsObject::default();
        Ok(JsValue::from(program_obj))
    }) };
    gl_context.set(js_string!("createProgram"), JsValue::from(create_program_fn.to_js_function(context.realm())), false, context)?;

    let create_buffer_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
        let buffer_obj = JsObject::default();
        Ok(JsValue::from(buffer_obj))
    }) };
    gl_context.set(js_string!("createBuffer"), JsValue::from(create_buffer_fn.to_js_function(context.realm())), false, context)?;

    // Shader operations
    let shader_source_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("shaderSource"), JsValue::from(shader_source_fn.to_js_function(context.realm())), false, context)?;

    let compile_shader_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("compileShader"), JsValue::from(compile_shader_fn.to_js_function(context.realm())), false, context)?;

    // Program operations
    let attach_shader_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("attachShader"), JsValue::from(attach_shader_fn.to_js_function(context.realm())), false, context)?;

    let link_program_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("linkProgram"), JsValue::from(link_program_fn.to_js_function(context.realm())), false, context)?;

    let use_program_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("useProgram"), JsValue::from(use_program_fn.to_js_function(context.realm())), false, context)?;

    // Buffer operations
    let bind_buffer_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("bindBuffer"), JsValue::from(bind_buffer_fn.to_js_function(context.realm())), false, context)?;

    let buffer_data_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("bufferData"), JsValue::from(buffer_data_fn.to_js_function(context.realm())), false, context)?;

    // Rendering operations
    let viewport_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("viewport"), JsValue::from(viewport_fn.to_js_function(context.realm())), false, context)?;

    let clear_color_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("clearColor"), JsValue::from(clear_color_fn.to_js_function(context.realm())), false, context)?;

    let clear_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("clear"), JsValue::from(clear_fn.to_js_function(context.realm())), false, context)?;

    let draw_arrays_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| Ok(JsValue::undefined())) };
    gl_context.set(js_string!("drawArrays"), JsValue::from(draw_arrays_fn.to_js_function(context.realm())), false, context)?;

    // Critical fingerprinting methods
    let get_parameter_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
        if args.is_empty() {
            return Ok(JsValue::null());
        }

        let param = args[0].to_i32(context)?;
        match param {
            7936 => Ok(JsValue::from(js_string!("WebKit"))), // GL_VENDOR
            7937 => Ok(JsValue::from(js_string!("WebKit WebGL"))), // GL_RENDERER
            7938 => Ok(JsValue::from(js_string!("WebGL 1.0 (OpenGL ES 2.0 Chromium)"))), // GL_VERSION
            34921 => Ok(JsValue::from(js_string!("WebGL GLSL ES 1.0 (OpenGL ES GLSL ES 1.0 Chromium)"))), // GL_SHADING_LANGUAGE_VERSION
            34930 => Ok(JsValue::from(16)), // GL_MAX_TEXTURE_SIZE
            3379 => Ok(JsValue::from(16384)), // GL_MAX_VIEWPORT_DIMS
            _ => Ok(JsValue::from(0))
        }
    }) };
    gl_context.set(js_string!("getParameter"), JsValue::from(get_parameter_fn.to_js_function(context.realm())), false, context)?;

    // Extensions support
    let get_extension_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
        if args.is_empty() {
            return Ok(JsValue::null());
        }

        let ext_name = args[0].to_string(context)?.to_std_string_escaped();
        match ext_name.as_str() {
            "WEBKIT_EXT_texture_filter_anisotropic" |
            "EXT_texture_filter_anisotropic" |
            "OES_element_index_uint" |
            "OES_standard_derivatives" => {
                let ext_obj = JsObject::default();
                Ok(JsValue::from(ext_obj))
            },
            _ => Ok(JsValue::null())
        }
    }) };
    gl_context.set(js_string!("getExtension"), JsValue::from(get_extension_fn.to_js_function(context.realm())), false, context)?;

    let get_supported_extensions_fn = unsafe { NativeFunction::from_closure(|_, _args, context| {
        let extensions = vec![
            "WEBKIT_EXT_texture_filter_anisotropic",
            "EXT_texture_filter_anisotropic",
            "OES_element_index_uint",
            "OES_standard_derivatives",
            "WEBGL_debug_renderer_info"
        ];

        let js_array = boa_engine::object::builtins::JsArray::new(context);
        for (i, ext) in extensions.iter().enumerate() {
            js_array.set(i, js_string!(*ext), true, context).ok();
        }
        Ok(JsValue::from(js_array))
    }) };
    gl_context.set(js_string!("getSupportedExtensions"), JsValue::from(get_supported_extensions_fn.to_js_function(context.realm())), false, context)?;

    // WebGL2 specific methods
    if is_webgl2 {
        let create_vertex_array_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
            let vao_obj = JsObject::default();
            Ok(JsValue::from(vao_obj))
        }) };
        gl_context.set(js_string!("createVertexArray"), JsValue::from(create_vertex_array_fn.to_js_function(context.realm())), false, context)?;
    }

    Ok(JsValue::from(gl_context))
}

#[cfg(test)]
mod tests;