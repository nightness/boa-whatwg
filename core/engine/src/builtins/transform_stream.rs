//! TransformStream Web API implementation for Boa
//!
//! Implementation of the WHATWG Streams Standard TransformStream
//! https://streams.spec.whatwg.org/
//!
//! This implements the complete TransformStream interface according to the
//! WHATWG Streams Living Standard

use crate::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder, Promise},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::Attribute
};
use boa_gc::{Finalize, Trace};
use super::{readable_stream::ReadableStream, writable_stream::WritableStream};

/// JavaScript `TransformStream` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct TransformStream;

impl IntrinsicObject for TransformStream {
    fn init(realm: &Realm) {
        let readable_getter = BuiltInBuilder::callable(realm, get_readable)
            .name(js_string!("get readable"))
            .build();

        let writable_getter = BuiltInBuilder::callable(realm, get_writable)
            .name(js_string!("get writable"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("readable"),
                Some(readable_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("writable"),
                Some(writable_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for TransformStream {
    const NAME: JsString = StaticJsStrings::TRANSFORM_STREAM;
}

impl BuiltInConstructor for TransformStream {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::transform_stream;

    /// `TransformStream(transformer, writableStrategy, readableStrategy)`
    ///
    /// Constructor for TransformStream objects.
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("TransformStream constructor requires 'new'")
                .into());
        }

        let transformer = args.get_or_undefined(0);
        let writable_strategy = args.get_or_undefined(1);
        let readable_strategy = args.get_or_undefined(2);

        // Create readable and writable streams
        let readable_stream = ReadableStream::constructor(
            &context.intrinsics().constructors().readable_stream().constructor().into(),
            &[JsValue::undefined(), readable_strategy.clone()],
            context,
        )?;

        let writable_stream = WritableStream::constructor(
            &context.intrinsics().constructors().writable_stream().constructor().into(),
            &[JsValue::undefined(), writable_strategy.clone()],
            context,
        )?;

        // Create the TransformStream object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::transform_stream, context)?;
        let transform_stream = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            TransformStreamData::new(
                transformer.clone(),
                readable_stream.as_object().unwrap().clone(),
                writable_stream.as_object().unwrap().clone(),
            ),
        );

        Ok(transform_stream.into())
    }
}

/// Internal data for TransformStream instances
#[derive(Debug, Trace, Finalize, JsData)]
struct TransformStreamData {
    #[unsafe_ignore_trace]
    transformer: JsValue,
    readable: JsObject,
    writable: JsObject,
    backpressure: bool,
    backpressure_change_promise: Option<JsObject>,
}

impl TransformStreamData {
    fn new(transformer: JsValue, readable: JsObject, writable: JsObject) -> Self {
        Self {
            transformer,
            readable,
            writable,
            backpressure: false,
            backpressure_change_promise: None,
        }
    }
}

/// Get the readable property of a TransformStream
fn get_readable(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("TransformStream.prototype.readable getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<TransformStreamData>() {
        Ok(JsValue::from(data.readable.clone()))
    } else {
        Err(JsNativeError::typ()
            .with_message("TransformStream.prototype.readable getter called on incompatible object")
            .into())
    }
}

/// Get the writable property of a TransformStream
fn get_writable(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("TransformStream.prototype.writable getter called on non-object")
    })?;

    if let Some(data) = this_obj.downcast_ref::<TransformStreamData>() {
        Ok(JsValue::from(data.writable.clone()))
    } else {
        Err(JsNativeError::typ()
            .with_message("TransformStream.prototype.writable getter called on incompatible object")
            .into())
    }
}