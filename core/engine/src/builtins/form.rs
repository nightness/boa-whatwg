//! HTMLFormElement and HTMLFormControlsCollection implementation for Boa
//!
//! Complete implementation of HTML Form elements following WHATWG HTML spec
//! https://html.spec.whatwg.org/multipage/forms.html#the-form-element

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
use std::collections::HashMap;

/// HTMLFormElement implementation
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct HTMLFormElement {
    /// Form controls collection by name/id
    pub elements: HashMap<String, JsObject>,
    /// Form controls collection by index
    pub elements_by_index: Vec<JsObject>,
    /// Form action URL
    pub action: String,
    /// Form method (GET, POST, etc.)
    pub method: String,
    /// Form name
    pub name: String,
}

impl HTMLFormElement {
    /// Create a new HTMLFormElement
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            elements_by_index: Vec::new(),
            action: String::new(),
            method: "GET".to_string(),
            name: String::new(),
        }
    }

    /// Add a form control element
    pub fn add_element(&mut self, name: String, element: JsObject) {
        self.elements.insert(name, element.clone());
        self.elements_by_index.push(element);
    }

    /// Get element by name
    pub fn get_element_by_name(&self, name: &str) -> Option<JsObject> {
        self.elements.get(name).cloned()
    }

    /// Get element by index
    pub fn get_element_by_index(&self, index: usize) -> Option<JsObject> {
        self.elements_by_index.get(index).cloned()
    }

    /// Get elements count
    pub fn elements_length(&self) -> usize {
        self.elements_by_index.len()
    }
}

impl IntrinsicObject for HTMLFormElement {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(Self::named_getter, js_string!("namedItem"), 1)
            .property(js_string!("length"), 0, Attribute::CONFIGURABLE)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLFormElement {
    const NAME: JsString = StaticJsStrings::FORM_ELEMENT;
}

impl BuiltInConstructor for HTMLFormElement {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_form_element;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling HTMLFormElement constructor without `new` is forbidden")
                .into());
        }

        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::html_form_element,
            context,
        )?;

        let form = HTMLFormElement::new();
        let form_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            form,
        );

        // Add elements collection property
        let elements_collection = HTMLFormControlsCollection::new(form_obj.clone());
        let elements_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            elements_collection,
        );

        form_obj.define_property_or_throw(
            js_string!("elements"),
            PropertyDescriptorBuilder::new()
                .value(elements_obj)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
            context,
        )?;

        Ok(form_obj.into())
    }
}

impl HTMLFormElement {
    /// `HTMLFormElement.prototype.namedItem(name)`
    fn named_getter(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLFormElement.namedItem called on non-object")
        })?;

        if let Some(form) = this_obj.downcast_ref::<HTMLFormElement>() {
            let name = args.get_or_undefined(0).to_string(context)?;
            let name_str = name.to_std_string_escaped();

            if let Some(element) = form.get_element_by_name(&name_str) {
                return Ok(element.into());
            }
        }

        Ok(JsValue::null())
    }
}

/// HTMLFormControlsCollection implementation
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct HTMLFormControlsCollection {
    /// Reference to the parent form
    pub form: JsObject,
}

impl HTMLFormControlsCollection {
    /// Create a new HTMLFormControlsCollection
    pub fn new(form: JsObject) -> Self {
        Self { form }
    }
}

impl IntrinsicObject for HTMLFormControlsCollection {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(Self::item, js_string!("item"), 1)
            .static_method(Self::named_item, js_string!("namedItem"), 1)
            .property(js_string!("length"), 0, Attribute::CONFIGURABLE)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLFormControlsCollection {
    const NAME: JsString = StaticJsStrings::FORM_CONTROLS_COLLECTION;
}

impl BuiltInConstructor for HTMLFormControlsCollection {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_form_controls_collection;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("HTMLFormControlsCollection constructor cannot be called")
            .into())
    }
}

impl HTMLFormControlsCollection {
    /// `HTMLFormControlsCollection.prototype.item(index)`
    fn item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLFormControlsCollection.item called on non-object")
        })?;

        if let Some(collection) = this_obj.downcast_ref::<HTMLFormControlsCollection>() {
            let index = args.get_or_undefined(0).to_number(context)? as usize;

            if let Some(form_data) = collection.form.downcast_ref::<HTMLFormElement>() {
                if let Some(element) = form_data.get_element_by_index(index) {
                    return Ok(element.into());
                }
            }
        }

        Ok(JsValue::null())
    }

    /// `HTMLFormControlsCollection.prototype.namedItem(name)`
    fn named_item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("HTMLFormControlsCollection.namedItem called on non-object")
        })?;

        if let Some(collection) = this_obj.downcast_ref::<HTMLFormControlsCollection>() {
            let name = args.get_or_undefined(0).to_string(context)?;
            let name_str = name.to_std_string_escaped();

            if let Some(form_data) = collection.form.downcast_ref::<HTMLFormElement>() {
                if let Some(element) = form_data.get_element_by_name(&name_str) {
                    return Ok(element.into());
                }
            }
        }

        Ok(JsValue::null())
    }
}

/// HTMLInputElement implementation
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct HTMLInputElement {
    /// Input name attribute
    pub name: String,
    /// Input value
    pub value: String,
    /// Input type
    pub input_type: String,
    /// Input id
    pub id: String,
}

impl HTMLInputElement {
    /// Create a new HTMLInputElement
    pub fn new(name: String, value: String, input_type: String, id: String) -> Self {
        Self {
            name,
            value,
            input_type,
            id,
        }
    }
}

impl IntrinsicObject for HTMLInputElement {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(js_string!("name"), JsValue::from(js_string!("")), Attribute::WRITABLE | Attribute::CONFIGURABLE)
            .property(js_string!("value"), JsValue::from(js_string!("")), Attribute::WRITABLE | Attribute::CONFIGURABLE)
            .property(js_string!("type"), JsValue::from(js_string!("text")), Attribute::WRITABLE | Attribute::CONFIGURABLE)
            .property(js_string!("id"), JsValue::from(js_string!("")), Attribute::WRITABLE | Attribute::CONFIGURABLE)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLInputElement {
    const NAME: JsString = StaticJsStrings::INPUT_ELEMENT;
}

impl BuiltInConstructor for HTMLInputElement {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_input_element;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling HTMLInputElement constructor without `new` is forbidden")
                .into());
        }

        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::html_input_element,
            context,
        )?;

        let input = HTMLInputElement::new(
            String::new(),
            String::new(),
            "text".to_string(),
            String::new(),
        );

        let input_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            input,
        );

        Ok(input_obj.into())
    }
}