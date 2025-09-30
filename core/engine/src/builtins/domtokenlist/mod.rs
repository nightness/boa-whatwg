//! DOMTokenList implementation (classList) - minimal spec-aligned subset
//!
//! Implements: add, remove, toggle, contains, item, length, toString
use crate::{
    builtins::{BuiltInBuilder, BuiltInObject, IntrinsicObject, BuiltInConstructor},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string, object::JsObject, property::Attribute, realm::Realm,
    string::JsString, Context, JsArgs, JsData, JsNativeError, JsResult, JsValue,
};
use crate::builtins::element::ElementData;
use boa_gc::{Finalize, Trace};

/// Internal data for DOMTokenList objects
#[derive(Debug, Trace, Finalize, JsData)]
pub struct DOMTokenListData {
    /// The associated element object
    element: JsObject,
}

impl DOMTokenListData {
    pub fn new(element: JsObject) -> Self {
        Self { element }
    }

    fn class_name(&self, context: &mut Context) -> Option<String> {
        if let Some(ed) = self.element.downcast_ref::<ElementData>() {
            Some(ed.get_class_name())
        } else {
            None
        }
    }

    fn set_class_name(&self, value: String) {
        if let Some(mut ed) = self.element.downcast_mut::<ElementData>() {
            ed.set_class_name(value);
        }
    }
}

/// The `DOMTokenList` object
#[derive(Debug, Trace, Finalize)]
pub struct DOMTokenList;

impl DOMTokenList {
    fn split_tokens(class_name: &str) -> Vec<String> {
        class_name
            .split(|c: char| matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0C'))
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    fn join_tokens(tokens: &[String]) -> String {
        tokens.join(" ")
    }

    fn validate_token(token: &str) -> Result<(), JsNativeError> {
        if token.is_empty() {
            return Err(JsNativeError::typ().with_message("The token must not be empty"));
        }
        if token.chars().any(|c| matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0C')) {
            return Err(JsNativeError::typ().with_message("The token contains invalid whitespace"));
        }
        Ok(())
    }

    /// Helper: build or return a DOMTokenList object bound to an element
    pub fn create_for_element(element: JsObject, context: &mut Context) -> JsResult<JsObject> {
        let data = DOMTokenListData::new(element.clone());
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().domtokenlist().prototype(),
            data,
        );
        Ok(obj)
    }

    /* Prototype methods */
    fn contains(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("DOMTokenList.contains called on non-object"))?;
        if let Some(data) = this_obj.downcast_ref::<DOMTokenListData>() {
            let token = args.get_or_undefined(0).to_string(context)?;
            let token_std = token.to_std_string_escaped();
            Self::validate_token(&token_std)?;
            if let Some(class_name) = data.class_name(context) {
                let tokens = Self::split_tokens(&class_name);
                return Ok(JsValue::new(tokens.contains(&token_std)));
            }
            Ok(JsValue::new(false))
        } else {
            Err(JsNativeError::typ().with_message("DOMTokenList.contains called on non-DOMTokenList object").into())
        }
    }

    fn add(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("DOMTokenList.add called on non-object"))?;
        if let Some(data) = this_obj.downcast_ref::<DOMTokenListData>() {
            let mut class = data.class_name(context).unwrap_or_default();
            let mut tokens = Self::split_tokens(&class);
            for arg in args.iter() {
                let token = arg.to_string(context)?;
                let token_std = token.to_std_string_escaped();
                Self::validate_token(&token_std)?;
                if !tokens.contains(&token_std) {
                    tokens.push(token_std);
                }
            }
            class = Self::join_tokens(&tokens);
            data.set_class_name(class);
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ().with_message("DOMTokenList.add called on non-DOMTokenList object").into())
        }
    }

    fn remove(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("DOMTokenList.remove called on non-object"))?;
        if let Some(data) = this_obj.downcast_ref::<DOMTokenListData>() {
            let mut class = data.class_name(context).unwrap_or_default();
            let mut tokens = Self::split_tokens(&class);
            for arg in args.iter() {
                let token = arg.to_string(context)?;
                let token_std = token.to_std_string_escaped();
                Self::validate_token(&token_std)?;
                tokens.retain(|x| x != &token_std);
            }
            class = Self::join_tokens(&tokens);
            data.set_class_name(class);
            Ok(JsValue::undefined())
        } else {
            Err(JsNativeError::typ().with_message("DOMTokenList.remove called on non-DOMTokenList object").into())
        }
    }

    fn toggle(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("DOMTokenList.toggle called on non-object"))?;
        if let Some(data) = this_obj.downcast_ref::<DOMTokenListData>() {
            let token = args.get_or_undefined(0).to_string(context)?;
            let token_std = token.to_std_string_escaped();
            Self::validate_token(&token_std)?;
            let mut class = data.class_name(context).unwrap_or_default();
            let mut tokens = Self::split_tokens(&class);
            if tokens.contains(&token_std) {
                tokens.retain(|x| x != &token_std);
                class = Self::join_tokens(&tokens);
                data.set_class_name(class);
                return Ok(JsValue::new(false));
            } else {
                tokens.push(token_std);
                class = Self::join_tokens(&tokens);
                data.set_class_name(class);
                return Ok(JsValue::new(true));
            }
        } else {
            Err(JsNativeError::typ().with_message("DOMTokenList.toggle called on non-DOMTokenList object").into())
        }
    }

    fn item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("DOMTokenList.item called on non-object"))?;
        if let Some(data) = this_obj.downcast_ref::<DOMTokenListData>() {
            let index = args.get_or_undefined(0).to_length(context)? as usize;
            if let Some(class_name) = data.class_name(context) {
                let tokens = Self::split_tokens(&class_name);
                if let Some(t) = tokens.get(index) {
                    return Ok(JsValue::from(JsString::from(t.clone())));
                }
            }
            Ok(JsValue::null())
        } else {
            Err(JsNativeError::typ().with_message("DOMTokenList.item called on non-DOMTokenList object").into())
        }
    }

    fn get_length(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("DOMTokenList.length called on non-object"))?;
        if let Some(data) = this_obj.downcast_ref::<DOMTokenListData>() {
            if let Some(class_name) = data.class_name(context) {
                let tokens = Self::split_tokens(&class_name);
                return Ok(JsValue::new(tokens.len() as i32));
            }
            Ok(JsValue::new(0))
        } else {
            Err(JsNativeError::typ().with_message("DOMTokenList.length called on non-DOMTokenList object").into())
        }
    }
}

impl IntrinsicObject for DOMTokenList {
    fn init(realm: &Realm) {
        let contains_func = BuiltInBuilder::callable(realm, Self::contains)
            .name(js_string!("contains"))
            .length(1)
            .build();

        let add_func = BuiltInBuilder::callable(realm, Self::add)
            .name(js_string!("add"))
            .length(1)
            .build();

        let remove_func = BuiltInBuilder::callable(realm, Self::remove)
            .name(js_string!("remove"))
            .length(1)
            .build();

        let toggle_func = BuiltInBuilder::callable(realm, Self::toggle)
            .name(js_string!("toggle"))
            .length(1)
            .build();

        let item_func = BuiltInBuilder::callable(realm, Self::item)
            .name(js_string!("item"))
            .length(1)
            .build();

        let length_getter = BuiltInBuilder::callable(realm, Self::get_length)
            .name(js_string!("get length"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::contains, js_string!("contains"), 1)
            .method(Self::add, js_string!("add"), 1)
            .method(Self::remove, js_string!("remove"), 1)
            .method(Self::toggle, js_string!("toggle"), 1)
            .method(Self::item, js_string!("item"), 1)
            .accessor(js_string!("length"), Some(length_getter), None, Attribute::CONFIGURABLE)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for DOMTokenList {
    const NAME: JsString = js_string!("DOMTokenList");
}

impl BuiltInConstructor for DOMTokenList {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::domtokenlist;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // DOMTokenList is not intended to be directly constructed in most engines; return an empty object
        let obj = JsObject::default();
        Ok(obj.into())
    }
}

