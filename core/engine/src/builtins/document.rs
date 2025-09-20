//! Document Web API implementation for Boa
//!
//! Native implementation of Document standard
//! https://dom.spec.whatwg.org/#interface-document

use crate::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
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

        // Create a new element
        let element = JsObject::default();

        // Add tagName property
        element.define_property_or_throw(
            js_string!("tagName"),
            PropertyDescriptorBuilder::new()
                .configurable(false)
                .enumerable(true)
                .writable(false)
                .value(JsString::from(tag_name_upper.as_str()))
                .build(),
            context,
        )?;

        // Add style property as empty object
        let style_obj = JsObject::default();
        element.define_property_or_throw(
            js_string!("style"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(style_obj)
                .build(),
            context,
        )?;

        // Add Canvas-specific functionality for <canvas> elements
        if tag_name_upper == "CANVAS" {
            // Add width and height properties with default values
            element.define_property_or_throw(
                js_string!("width"),
                PropertyDescriptorBuilder::new()
                    .configurable(true)
                    .enumerable(true)
                    .writable(true)
                    .value(300) // Default canvas width
                    .build(),
                context,
            )?;

            element.define_property_or_throw(
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

            element.define_property_or_throw(
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

            element.define_property_or_throw(
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

        Ok(element.into())
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

    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.querySelector called on non-object")
    })?;

    if let Some(document) = this_obj.downcast_ref::<DocumentData>() {
        let selector = args.get_or_undefined(0).to_string(context)?;
        let selector_str = selector.to_std_string_escaped();

        // Get the HTML content from the document
        let html_content = document.get_html_content();

        // Use real DOM implementation with scraper library
        if let Some(element) = create_real_element_from_html(context, &selector_str, &html_content)? {
            return Ok(element.into());
        }

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
            let element_obj = context.intrinsics().constructors().object().constructor();

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
            let element_obj = context.intrinsics().constructors().object().constructor();

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
            // TODO: Implement WebGL context
            Ok(JsValue::null())
        }
        "webgl2" | "experimental-webgl2" => {
            // TODO: Implement WebGL2 context
            Ok(JsValue::null())
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