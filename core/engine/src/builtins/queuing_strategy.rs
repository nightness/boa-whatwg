//! Queuing Strategy Web API implementation for Boa
//!
//! Implementation of WHATWG Streams Standard Queuing Strategies:
//! - CountQueuingStrategy
//! - ByteLengthQueuingStrategy
//!
//! https://streams.spec.whatwg.org/#qs

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

/// JavaScript `CountQueuingStrategy` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct CountQueuingStrategy;

impl IntrinsicObject for CountQueuingStrategy {
    fn init(realm: &Realm) {
        let high_water_mark_getter = BuiltInBuilder::callable(realm, get_high_water_mark)
            .name(js_string!("get highWaterMark"))
            .build();

        let size_getter = BuiltInBuilder::callable(realm, get_size)
            .name(js_string!("get size"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("highWaterMark"),
                Some(high_water_mark_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("size"),
                Some(size_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for CountQueuingStrategy {
    const NAME: JsString = StaticJsStrings::COUNT_QUEUING_STRATEGY;
}

impl BuiltInConstructor for CountQueuingStrategy {
    const LENGTH: usize = 1;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::count_queuing_strategy;

    /// `CountQueuingStrategy(options)`
    ///
    /// Constructor for CountQueuingStrategy objects.
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("CountQueuingStrategy constructor requires 'new'")
                .into());
        }

        let options = args.get_or_undefined(0);

        // Extract highWaterMark from options
        let high_water_mark = if let Some(options_obj) = options.as_object() {
            let hwm = options_obj.get(js_string!("highWaterMark"), context)?;
            hwm.to_number(context)?
        } else {
            return Err(JsNativeError::typ()
                .with_message("CountQueuingStrategy constructor requires an options object")
                .into());
        };

        if high_water_mark < 0.0 {
            return Err(JsNativeError::range()
                .with_message("CountQueuingStrategy highWaterMark must be non-negative")
                .into());
        }

        // Create the CountQueuingStrategy object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::count_queuing_strategy, context)?;
        let strategy = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            CountQueuingStrategyData::new(high_water_mark),
        );

        Ok(strategy.into())
    }
}

/// JavaScript `ByteLengthQueuingStrategy` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct ByteLengthQueuingStrategy;

impl IntrinsicObject for ByteLengthQueuingStrategy {
    fn init(realm: &Realm) {
        let high_water_mark_getter = BuiltInBuilder::callable(realm, get_byte_high_water_mark)
            .name(js_string!("get highWaterMark"))
            .build();

        let size_getter = BuiltInBuilder::callable(realm, get_byte_size)
            .name(js_string!("get size"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("highWaterMark"),
                Some(high_water_mark_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("size"),
                Some(size_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for ByteLengthQueuingStrategy {
    const NAME: JsString = StaticJsStrings::BYTE_LENGTH_QUEUING_STRATEGY;
}

impl BuiltInConstructor for ByteLengthQueuingStrategy {
    const LENGTH: usize = 1;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::byte_length_queuing_strategy;

    /// `ByteLengthQueuingStrategy(options)`
    ///
    /// Constructor for ByteLengthQueuingStrategy objects.
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("ByteLengthQueuingStrategy constructor requires 'new'")
                .into());
        }

        let options = args.get_or_undefined(0);

        // Extract highWaterMark from options
        let high_water_mark = if let Some(options_obj) = options.as_object() {
            let hwm = options_obj.get(js_string!("highWaterMark"), context)?;
            hwm.to_number(context)?
        } else {
            return Err(JsNativeError::typ()
                .with_message("ByteLengthQueuingStrategy constructor requires an options object")
                .into());
        };

        if high_water_mark < 0.0 {
            return Err(JsNativeError::range()
                .with_message("ByteLengthQueuingStrategy highWaterMark must be non-negative")
                .into());
        }

        // Create the ByteLengthQueuingStrategy object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::byte_length_queuing_strategy, context)?;
        let strategy = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            ByteLengthQueuingStrategyData::new(high_water_mark),
        );

        Ok(strategy.into())
    }
}

/// Internal data for CountQueuingStrategy instances
#[derive(Debug, Trace, Finalize, JsData)]
struct CountQueuingStrategyData {
    high_water_mark: f64,
}

impl CountQueuingStrategyData {
    fn new(high_water_mark: f64) -> Self {
        Self { high_water_mark }
    }
}

/// Internal data for ByteLengthQueuingStrategy instances
#[derive(Debug, Trace, Finalize, JsData)]
struct ByteLengthQueuingStrategyData {
    high_water_mark: f64,
}

impl ByteLengthQueuingStrategyData {
    fn new(high_water_mark: f64) -> Self {
        Self { high_water_mark }
    }
}

/// Get the highWaterMark property of a CountQueuingStrategy
fn get_high_water_mark(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("CountQueuingStrategy.prototype.highWaterMark getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<CountQueuingStrategyData>() {
        Ok(data.high_water_mark.into())
    } else {
        Err(JsNativeError::typ()
            .with_message("CountQueuingStrategy.prototype.highWaterMark getter called on incompatible object")
            .into())
    }
}

/// Get the size property of a CountQueuingStrategy
fn get_size(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("CountQueuingStrategy.prototype.size getter called on non-object")
    })?;

    if this_obj.downcast_ref::<CountQueuingStrategyData>().is_some() {
        // Return a function that always returns 1
        let size_fn = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
            Ok(JsValue::from(1))
        })
        .name(js_string!("size"))
        .length(0)
        .build();

        Ok(JsValue::from(size_fn))
    } else {
        Err(JsNativeError::typ()
            .with_message("CountQueuingStrategy.prototype.size getter called on incompatible object")
            .into())
    }
}

/// Get the highWaterMark property of a ByteLengthQueuingStrategy
fn get_byte_high_water_mark(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("ByteLengthQueuingStrategy.prototype.highWaterMark getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<ByteLengthQueuingStrategyData>() {
        Ok(data.high_water_mark.into())
    } else {
        Err(JsNativeError::typ()
            .with_message("ByteLengthQueuingStrategy.prototype.highWaterMark getter called on incompatible object")
            .into())
    }
}

/// Get the size property of a ByteLengthQueuingStrategy
fn get_byte_size(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("ByteLengthQueuingStrategy.prototype.size getter called on non-object")
    })?;

    if this_obj.downcast_ref::<ByteLengthQueuingStrategyData>().is_some() {
        // Return a function that returns the byteLength property
        let size_fn = BuiltInBuilder::callable(context.realm(), |_this, args, context| {
            let chunk = args.get_or_undefined(0);

            // Try to get byteLength property
            if let Some(chunk_obj) = chunk.as_object() {
                if let Ok(byte_length) = chunk_obj.get(js_string!("byteLength"), context) {
                    return byte_length.to_number(context).map(JsValue::from);
                }
            }

            // Fallback: convert to string and get byte length
            let str_value = chunk.to_string(context)?;
            Ok(JsValue::from(str_value.len() as f64))
        })
        .name(js_string!("size"))
        .length(1)
        .build();

        Ok(JsValue::from(size_fn))
    } else {
        Err(JsNativeError::typ()
            .with_message("ByteLengthQueuingStrategy.prototype.size getter called on incompatible object")
            .into())
    }
}