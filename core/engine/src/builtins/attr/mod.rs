//! Attr interface implementation for DOM Level 4
//!
//! The Attr interface represents an attribute of an element.
//! It has name, value, and ownerElement properties.
//! https://dom.spec.whatwg.org/#interface-attr

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    string::{StaticJsStrings, JsString},
    Context, JsArgs, JsData, JsNativeError, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace, GcRefCell};

/// The Attr data implementation
#[derive(Debug, Trace, Finalize, JsData)]
pub struct AttrData {
    /// The name of the attribute
    name: GcRefCell<String>,
    /// The value of the attribute
    value: GcRefCell<String>,
    /// The element that owns this attribute (optional)
    owner_element: GcRefCell<Option<JsObject>>,
    /// Whether this is a namespace-aware attribute
    namespace_uri: GcRefCell<Option<String>>,
    /// Local name (for namespace-aware attributes)
    local_name: GcRefCell<Option<String>>,
    /// Prefix (for namespace-aware attributes)
    prefix: GcRefCell<Option<String>>,
}

impl AttrData {
    /// Create a new Attr
    pub fn new(name: String, value: String) -> Self {
        Self {
            name: GcRefCell::new(name),
            value: GcRefCell::new(value),
            owner_element: GcRefCell::new(None),
            namespace_uri: GcRefCell::new(None),
            local_name: GcRefCell::new(None),
            prefix: GcRefCell::new(None),
        }
    }

    /// Get the name of the attribute
    pub fn name(&self) -> String {
        self.name.borrow().clone()
    }

    /// Get the value of the attribute
    pub fn value(&self) -> String {
        self.value.borrow().clone()
    }

    /// Set the value of the attribute
    pub fn set_value(&self, value: String) {
        *self.value.borrow_mut() = value;
    }

    /// Get the owner element
    pub fn owner_element(&self) -> Option<JsObject> {
        self.owner_element.borrow().clone()
    }

    /// Set the owner element
    pub fn set_owner_element(&self, element: Option<JsObject>) {
        *self.owner_element.borrow_mut() = element;
    }

    /// Get the namespace URI
    pub fn namespace_uri(&self) -> Option<String> {
        self.namespace_uri.borrow().clone()
    }

    /// Set the namespace URI
    pub fn set_namespace_uri(&self, uri: Option<String>) {
        *self.namespace_uri.borrow_mut() = uri;
    }

    /// Get the local name
    pub fn local_name(&self) -> Option<String> {
        self.local_name.borrow().clone()
    }

    /// Set the local name
    pub fn set_local_name(&self, name: Option<String>) {
        *self.local_name.borrow_mut() = name;
    }

    /// Get the prefix
    pub fn prefix(&self) -> Option<String> {
        self.prefix.borrow().clone()
    }

    /// Set the prefix
    pub fn set_prefix(&self, prefix: Option<String>) {
        *self.prefix.borrow_mut() = prefix;
    }

    /// Check if this attribute is specified (always true for DOM Level 4)
    pub fn specified(&self) -> bool {
        true // Always true in DOM Level 4
    }
}

/// The `Attr` object
#[derive(Debug, Trace, Finalize)]
pub struct Attr;

impl Attr {
    /// Create a new Attr
    pub fn create(context: &mut Context, name: String, value: String) -> JsResult<JsObject> {
        let attr_data = AttrData::new(name, value);

        let attr_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().attr().prototype(),
            attr_data,
        );

        Ok(attr_obj)
    }


    /// Get the name property
    fn name(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Attr.prototype.name called on non-object")
        })?;

        if let Some(attr_data) = this_obj.downcast_ref::<AttrData>() {
            Ok(JsString::from(attr_data.name()).into())
        } else {
            Err(JsNativeError::typ()
                .with_message("Attr.prototype.name called on non-Attr object")
                .into())
        }
    }

    /// Get the value property
    fn value(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Attr.prototype.value called on non-object")
        })?;

        if let Some(attr_data) = this_obj.downcast_ref::<AttrData>() {
            Ok(JsString::from(attr_data.value()).into())
        } else {
            Err(JsNativeError::typ()
                .with_message("Attr.prototype.value called on non-Attr object")
                .into())
        }
    }

    /// Set the value property
    fn set_value(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Attr.prototype.value setter called on non-object")
        })?;

        if let Some(attr_data) = this_obj.downcast_ref::<AttrData>() {
            let new_value = args.get_or_undefined(0).to_string(context)?;
            // Don't escape - preserve the raw string value
            attr_data.set_value(new_value.to_std_string().unwrap_or_default());
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ()
                .with_message("Attr.prototype.value setter called on non-Attr object")
                .into())
        }
    }

    /// Get the ownerElement property
    fn owner_element(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Attr.prototype.ownerElement called on non-object")
        })?;

        if let Some(attr_data) = this_obj.downcast_ref::<AttrData>() {
            match attr_data.owner_element() {
                Some(element) => Ok(element.into()),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Attr.prototype.ownerElement called on non-Attr object")
                .into())
        }
    }

    /// Get the namespaceURI property
    fn namespace_uri(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Attr.prototype.namespaceURI called on non-object")
        })?;

        if let Some(attr_data) = this_obj.downcast_ref::<AttrData>() {
            match attr_data.namespace_uri() {
                Some(uri) => Ok(JsString::from(uri).into()),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Attr.prototype.namespaceURI called on non-Attr object")
                .into())
        }
    }

    /// Get the localName property
    fn local_name(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Attr.prototype.localName called on non-object")
        })?;

        if let Some(attr_data) = this_obj.downcast_ref::<AttrData>() {
            match attr_data.local_name() {
                Some(name) => Ok(JsString::from(name).into()),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Attr.prototype.localName called on non-Attr object")
                .into())
        }
    }

    /// Get the prefix property
    fn prefix(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Attr.prototype.prefix called on non-object")
        })?;

        if let Some(attr_data) = this_obj.downcast_ref::<AttrData>() {
            match attr_data.prefix() {
                Some(prefix) => Ok(JsString::from(prefix).into()),
                None => Ok(JsValue::null()),
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("Attr.prototype.prefix called on non-Attr object")
                .into())
        }
    }

    /// Get the specified property (always true in DOM Level 4)
    fn specified(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Attr.prototype.specified called on non-object")
        })?;

        if let Some(_attr_data) = this_obj.downcast_ref::<AttrData>() {
            Ok(JsValue::new(true))
        } else {
            Err(JsNativeError::typ()
                .with_message("Attr.prototype.specified called on non-Attr object")
                .into())
        }
    }
}

impl IntrinsicObject for Attr {
    fn init(realm: &Realm) {
        // Create getter functions following Element pattern exactly
        let name_getter = BuiltInBuilder::callable(realm, Self::name)
            .name(js_string!("get name"))
            .build();

        let value_getter = BuiltInBuilder::callable(realm, Self::value)
            .name(js_string!("get value"))
            .build();

        let value_setter = BuiltInBuilder::callable(realm, Self::set_value)
            .name(js_string!("set value"))
            .build();

        let owner_element_getter = BuiltInBuilder::callable(realm, Self::owner_element)
            .name(js_string!("get ownerElement"))
            .build();

        let namespace_uri_getter = BuiltInBuilder::callable(realm, Self::namespace_uri)
            .name(js_string!("get namespaceURI"))
            .build();

        let local_name_getter = BuiltInBuilder::callable(realm, Self::local_name)
            .name(js_string!("get localName"))
            .build();

        let prefix_getter = BuiltInBuilder::callable(realm, Self::prefix)
            .name(js_string!("get prefix"))
            .build();

        let specified_getter = BuiltInBuilder::callable(realm, Self::specified)
            .name(js_string!("get specified"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("name"),
                Some(name_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("value"),
                Some(value_getter),
                Some(value_setter),
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("ownerElement"),
                Some(owner_element_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("namespaceURI"),
                Some(namespace_uri_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("localName"),
                Some(local_name_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("prefix"),
                Some(prefix_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("specified"),
                Some(specified_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Attr {
    const NAME: JsString = js_string!("Attr");
}

impl BuiltInConstructor for Attr {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::attr;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        use crate::object::internal_methods::get_prototype_from_constructor;

        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::attr,
            context,
        )?;

        // Create a default Attr object
        let attr_data = AttrData::new(String::new(), String::new());

        let attr_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            attr_data,
        );

        Ok(attr_obj.into())
    }
}

#[cfg(test)]
mod tests;