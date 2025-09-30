//! Implementation of the IDBKeyRange interface.
//!
//! IDBKeyRange represents a continuous interval over some data type that is used for keys.
//! Records can be retrieved from IndexedDB object stores and indexes using keys or a range of keys.
//!
//! More information:
//! - [WHATWG IndexedDB Specification](https://w3c.github.io/IndexedDB/#keyrange)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/IDBKeyRange)

use boa_gc::{Finalize, Trace};
use crate::{
    builtins::BuiltInBuilder,
    context::intrinsics::Intrinsics,
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, JsValue,
};
use crate::builtins::{BuiltInConstructor, BuiltInObject, IntrinsicObject};
use crate::context::intrinsics::StandardConstructor;

/// IDBKeyRange object representing a continuous interval of keys
#[derive(Debug, Clone, Finalize)]
pub struct IdbKeyRange {
    /// Lower bound of the key range (None means unbounded)
    lower: Option<JsValue>,
    /// Upper bound of the key range (None means unbounded)
    upper: Option<JsValue>,
    /// Whether the lower bound is excluded from the range
    lower_open: bool,
    /// Whether the upper bound is excluded from the range
    upper_open: bool,
}

unsafe impl Trace for IdbKeyRange {
    unsafe fn trace(&self, tracer: &mut boa_gc::Tracer) {
        unsafe {
            if let Some(ref lower) = self.lower {
                lower.trace(tracer);
            }
            if let Some(ref upper) = self.upper {
                upper.trace(tracer);
            }
        }
    }

    unsafe fn trace_non_roots(&self) {
        unsafe {
            if let Some(ref lower) = self.lower {
                lower.trace_non_roots();
            }
            if let Some(ref upper) = self.upper {
                upper.trace_non_roots();
            }
        }
    }

    fn run_finalizer(&self) {
        // No cleanup needed for IdbKeyRange
    }
}

impl JsData for IdbKeyRange {}

impl IdbKeyRange {
    /// Create a new IDBKeyRange with specified bounds
    pub fn new(lower: Option<JsValue>, upper: Option<JsValue>, lower_open: bool, upper_open: bool) -> Self {
        Self {
            lower,
            upper,
            lower_open,
            upper_open,
        }
    }

    /// `IDBKeyRange.bound(lower, upper, lowerOpen, upperOpen)`
    /// Creates a key range with both lower and upper bounds
    fn bound(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let lower = args.get_or_undefined(0);
        let upper = args.get_or_undefined(1);
        let lower_open = args.get_or_undefined(2).to_boolean();
        let upper_open = args.get_or_undefined(3).to_boolean();

        // Validate bounds
        if !Self::is_valid_key(lower) {
            return Err(JsNativeError::error()
                .with_message("Lower bound is not a valid key")
                .into());
        }
        if !Self::is_valid_key(upper) {
            return Err(JsNativeError::error()
                .with_message("Upper bound is not a valid key")
                .into());
        }

        // Check that lower <= upper
        let cmp_result = Self::compare_keys(lower, upper, context)?;
        if cmp_result > 0 {
            return Err(JsNativeError::error()
                .with_message("Lower bound must be less than or equal to upper bound")
                .into());
        }
        if cmp_result == 0 && (lower_open || upper_open) {
            return Err(JsNativeError::error()
                .with_message("Lower and upper bounds cannot both be open when they are equal")
                .into());
        }

        let key_range = IdbKeyRange::new(
            Some(lower.clone()),
            Some(upper.clone()),
            lower_open,
            upper_open,
        );

        let range_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().object().prototype()),
            key_range
        );

        Self::add_range_properties(&range_obj, context)?;

        Ok(JsValue::from(range_obj))
    }

    /// `IDBKeyRange.only(value)`
    /// Creates a key range containing only a single key
    fn only(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let value = args.get_or_undefined(0);

        if !Self::is_valid_key(value) {
            return Err(JsNativeError::error()
                .with_message("Value is not a valid key")
                .into());
        }

        let key_range = IdbKeyRange::new(
            Some(value.clone()),
            Some(value.clone()),
            false,
            false,
        );

        let range_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().object().prototype()),
            key_range
        );

        Self::add_range_properties(&range_obj, context)?;

        Ok(JsValue::from(range_obj))
    }

    /// `IDBKeyRange.lowerBound(bound, open)`
    /// Creates a key range with only a lower bound
    fn lower_bound(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let lower = args.get_or_undefined(0);
        let open = args.get_or_undefined(1).to_boolean();

        if !Self::is_valid_key(lower) {
            return Err(JsNativeError::error()
                .with_message("Lower bound is not a valid key")
                .into());
        }

        let key_range = IdbKeyRange::new(
            Some(lower.clone()),
            None,
            open,
            false,
        );

        let range_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().object().prototype()),
            key_range
        );

        Self::add_range_properties(&range_obj, context)?;

        Ok(JsValue::from(range_obj))
    }

    /// `IDBKeyRange.upperBound(bound, open)`
    /// Creates a key range with only an upper bound
    fn upper_bound(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let upper = args.get_or_undefined(0);
        let open = args.get_or_undefined(1).to_boolean();

        if !Self::is_valid_key(upper) {
            return Err(JsNativeError::error()
                .with_message("Upper bound is not a valid key")
                .into());
        }

        let key_range = IdbKeyRange::new(
            None,
            Some(upper.clone()),
            false,
            open,
        );

        let range_obj = JsObject::from_proto_and_data(
            Some(context.intrinsics().constructors().object().prototype()),
            key_range
        );

        Self::add_range_properties(&range_obj, context)?;

        Ok(JsValue::from(range_obj))
    }

    /// `keyRange.includes(key)`
    /// Checks whether a key is included in the key range
    fn includes(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let key = args.get_or_undefined(0);

        if !Self::is_valid_key(key) {
            return Err(JsNativeError::error()
                .with_message("Key is not a valid key")
                .into());
        }

        let range_obj = this.as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

        let range_data = range_obj.downcast_ref::<IdbKeyRange>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an IDBKeyRange"))?;

        let includes = Self::key_in_range(key, &range_data, context)?;

        Ok(JsValue::from(includes))
    }

    /// Check if a key is within the specified range
    pub fn key_in_range(key: &JsValue, range: &IdbKeyRange, context: &mut Context) -> JsResult<bool> {
        // Check lower bound
        if let Some(ref lower) = range.lower {
            let cmp = Self::compare_keys(key, lower, context)?;
            if cmp < 0 || (cmp == 0 && range.lower_open) {
                return Ok(false);
            }
        }

        // Check upper bound
        if let Some(ref upper) = range.upper {
            let cmp = Self::compare_keys(key, upper, context)?;
            if cmp > 0 || (cmp == 0 && range.upper_open) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Compare two keys according to IndexedDB key comparison algorithm
    pub fn compare_keys(a: &JsValue, b: &JsValue, context: &mut Context) -> JsResult<i32> {
        // For now, use a simplified comparison
        // In a full implementation, this would follow the complete IndexedDB key comparison algorithm

        // Numbers
        if a.is_number() && b.is_number() {
            let a_num = a.to_number(context)?;
            let b_num = b.to_number(context)?;
            return Ok(if a_num < b_num { -1 } else if a_num > b_num { 1 } else { 0 });
        }

        // Strings
        if a.is_string() && b.is_string() {
            let a_str = a.to_string(context)?.to_std_string_escaped();
            let b_str = b.to_string(context)?.to_std_string_escaped();
            return Ok(if a_str < b_str { -1 } else if a_str > b_str { 1 } else { 0 });
        }

        // Different types - use type ordering (number < string < date < array)
        let a_type = Self::get_key_type(a);
        let b_type = Self::get_key_type(b);

        if a_type != b_type {
            return Ok(a_type.cmp(&b_type) as i32);
        }

        // Default to string comparison for other cases
        let a_str = a.to_string(context)?.to_std_string_escaped();
        let b_str = b.to_string(context)?.to_std_string_escaped();
        Ok(if a_str < b_str { -1 } else if a_str > b_str { 1 } else { 0 })
    }

    /// Get the type ordering for IndexedDB key comparison
    fn get_key_type(value: &JsValue) -> u8 {
        if value.is_number() { 0 }
        else if value.is_string() { 1 }
        else if value.is_object() {
            // Would check for Date, Array here in full implementation
            2
        }
        else { 3 }
    }

    /// Check if a value is a valid IndexedDB key
    pub fn is_valid_key(value: &JsValue) -> bool {
        // Valid keys: number, string, date, array (in full implementation)
        value.is_number() || value.is_string() || value.is_object()
    }

    /// Add properties to a key range object
    fn add_range_properties(range_obj: &JsObject, context: &mut Context) -> JsResult<()> {
        let range_data = range_obj.downcast_ref::<IdbKeyRange>().unwrap();

        // Add lower property
        let lower_value = range_data.lower.clone().unwrap_or(JsValue::undefined());
        range_obj.set(js_string!("lower"), lower_value, false, context)?;

        // Add upper property
        let upper_value = range_data.upper.clone().unwrap_or(JsValue::undefined());
        range_obj.set(js_string!("upper"), upper_value, false, context)?;

        // Add lowerOpen property
        range_obj.set(js_string!("lowerOpen"), JsValue::from(range_data.lower_open), false, context)?;

        // Add upperOpen property
        range_obj.set(js_string!("upperOpen"), JsValue::from(range_data.upper_open), false, context)?;

        // Add includes method
        let includes_fn = BuiltInBuilder::callable(context.realm(), Self::includes)
            .name(js_string!("includes"))
            .length(1)
            .build();

        range_obj.set(js_string!("includes"), includes_fn, true, context)?;

        Ok(())
    }

    /// Create a global IDBKeyRange constructor
    pub fn create_constructor(context: &mut Context) -> JsObject {
        let key_range_constructor = BuiltInBuilder::callable(context.realm(), |_, _, _| {
            Err(JsNativeError::typ()
                .with_message("IDBKeyRange constructor cannot be called directly")
                .into())
        })
        .name(js_string!("IDBKeyRange"))
        .length(0)
        .build();

        // Add static methods
        let bound_fn = BuiltInBuilder::callable(context.realm(), Self::bound)
            .name(js_string!("bound"))
            .length(4)
            .build();
        key_range_constructor.set(js_string!("bound"), bound_fn, true, context).unwrap();

        let only_fn = BuiltInBuilder::callable(context.realm(), Self::only)
            .name(js_string!("only"))
            .length(1)
            .build();
        key_range_constructor.set(js_string!("only"), only_fn, true, context).unwrap();

        let lower_bound_fn = BuiltInBuilder::callable(context.realm(), Self::lower_bound)
            .name(js_string!("lowerBound"))
            .length(2)
            .build();
        key_range_constructor.set(js_string!("lowerBound"), lower_bound_fn, true, context).unwrap();

        let upper_bound_fn = BuiltInBuilder::callable(context.realm(), Self::upper_bound)
            .name(js_string!("upperBound"))
            .length(2)
            .build();
        key_range_constructor.set(js_string!("upperBound"), upper_bound_fn, true, context).unwrap();

        key_range_constructor.into()
    }
}