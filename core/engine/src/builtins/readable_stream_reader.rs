//! ReadableStream Reader implementations for Boa
//!
//! Implementation of ReadableStreamDefaultReader and ReadableStreamBYOBReader
//! according to the WHATWG Streams Standard
//! https://streams.spec.whatwg.org/

use crate::{
    builtins::{BuiltInBuilder, Promise},
    object::JsObject,
    value::JsValue,
    Context, JsData, JsNativeError, JsResult, js_string, JsArgs,
    realm::Realm, property::Attribute
};
use boa_gc::{Finalize, Trace};

/// JavaScript `ReadableStreamDefaultReader` implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct ReadableStreamDefaultReader;

impl ReadableStreamDefaultReader {
    /// Create a new ReadableStreamDefaultReader instance
    pub fn create(stream: JsObject, context: &mut Context) -> JsResult<JsValue> {
        let proto = context
            .intrinsics()
            .constructors()
            .object()
            .prototype();

        let reader = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            ReadableStreamDefaultReaderData::new(stream),
        );

        // Add methods to the reader
        let read_fn = BuiltInBuilder::callable(context.realm(), Self::read)
            .name(js_string!("read"))
            .length(0)
            .build();
        reader.set(js_string!("read"), read_fn, true, context)?;

        let cancel_fn = BuiltInBuilder::callable(context.realm(), Self::cancel)
            .name(js_string!("cancel"))
            .length(1)
            .build();
        reader.set(js_string!("cancel"), cancel_fn, true, context)?;

        let release_lock_fn = BuiltInBuilder::callable(context.realm(), Self::release_lock)
            .name(js_string!("releaseLock"))
            .length(0)
            .build();
        reader.set(js_string!("releaseLock"), release_lock_fn, true, context)?;

        // Add properties
        let closed_getter = BuiltInBuilder::callable(context.realm(), Self::get_closed)
            .name(js_string!("get closed"))
            .build();
        reader.define_property_or_throw(
            js_string!("closed"),
            crate::property::PropertyDescriptorBuilder::new()
                .get(closed_getter)
                .configurable(true)
                .enumerable(true),
            context,
        )?;

        Ok(reader.into())
    }

    /// `ReadableStreamDefaultReader.prototype.read()`
    fn read(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ReadableStreamDefaultReader.prototype.read called on non-object")
        })?;

        if let Some(reader_data) = this_obj.downcast_ref::<ReadableStreamDefaultReaderData>() {
            // Try to read from the stream
            if let Some(mut stream_data) = reader_data.stream.downcast_mut::<super::readable_stream::ReadableStreamData>() {
                if let Some(chunk) = stream_data.dequeue_chunk() {
                    // Create result object { value: chunk, done: false }
                    let result_obj = JsObject::with_object_proto(context.intrinsics());
                    result_obj.set(js_string!("value"), chunk, true, context)?;
                    result_obj.set(js_string!("done"), JsValue::from(false), true, context)?;

                    let promise_constructor = context.intrinsics().constructors().promise().constructor();
                    return crate::builtins::Promise::resolve(&promise_constructor.into(), &[JsValue::from(result_obj)], context);
                } else if stream_data.state == super::readable_stream::StreamState::Closed {
                    // Create result object { value: undefined, done: true }
                    let result_obj = JsObject::with_object_proto(context.intrinsics());
                    result_obj.set(js_string!("value"), JsValue::undefined(), true, context)?;
                    result_obj.set(js_string!("done"), JsValue::from(true), true, context)?;

                    let promise_constructor = context.intrinsics().constructors().promise().constructor();
                    return crate::builtins::Promise::resolve(&promise_constructor.into(), &[JsValue::from(result_obj)], context);
                }
            }
        }

        // Return a resolved promise for now
        let promise_constructor = context.intrinsics().constructors().promise().constructor();
        Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
    }

    /// `ReadableStreamDefaultReader.prototype.cancel(reason)`
    fn cancel(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ReadableStreamDefaultReader.prototype.cancel called on non-object")
        })?;

        let _reason = args.get_or_undefined(0);

        if let Some(reader_data) = this_obj.downcast_ref::<ReadableStreamDefaultReaderData>() {
            if let Some(mut stream_data) = reader_data.stream.downcast_mut::<super::readable_stream::ReadableStreamData>() {
                stream_data.state = super::readable_stream::StreamState::Closed;
            }
        }

        let promise_constructor = context.intrinsics().constructors().promise().constructor();
        crate::builtins::Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
    }

    /// `ReadableStreamDefaultReader.prototype.releaseLock()`
    fn release_lock(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ReadableStreamDefaultReader.prototype.releaseLock called on non-object")
        })?;

        if let Some(reader_data) = this_obj.downcast_ref::<ReadableStreamDefaultReaderData>() {
            if let Some(mut stream_data) = reader_data.stream.downcast_mut::<super::readable_stream::ReadableStreamData>() {
                stream_data.locked = false;
            }
        }

        Ok(JsValue::undefined())
    }

    /// Get the closed property of a ReadableStreamDefaultReader
    fn get_closed(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ReadableStreamDefaultReader.prototype.closed getter called on non-object")
        })?;

        if let Some(reader_data) = this_obj.downcast_ref::<ReadableStreamDefaultReaderData>() {
            if let Some(stream_data) = reader_data.stream.downcast_ref::<super::readable_stream::ReadableStreamData>() {
                match stream_data.state {
                    super::readable_stream::StreamState::Closed => {
                        let promise_constructor = context.intrinsics().constructors().promise().constructor();
                        crate::builtins::Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
                    },
                    super::readable_stream::StreamState::Errored => {
                        let promise_constructor = context.intrinsics().constructors().promise().constructor();
                        crate::builtins::Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
                    },
                    _ => {
                        let promise_constructor = context.intrinsics().constructors().promise().constructor();
                        crate::builtins::Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
                    }
                }
            } else {
                {
                    let promise_constructor = context.intrinsics().constructors().promise().constructor();
                    Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
                }
            }
        } else {
            {
                    let promise_constructor = context.intrinsics().constructors().promise().constructor();
                    Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
                }
        }
    }
}

/// JavaScript `ReadableStreamBYOBReader` implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct ReadableStreamBYOBReader;

impl ReadableStreamBYOBReader {
    /// Create a new ReadableStreamBYOBReader instance
    pub fn create(stream: JsObject, context: &mut Context) -> JsResult<JsValue> {
        let proto = context
            .intrinsics()
            .constructors()
            .object()
            .prototype();

        let reader = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            ReadableStreamBYOBReaderData::new(stream),
        );

        // Add methods to the reader
        let read_fn = BuiltInBuilder::callable(context.realm(), Self::read)
            .name(js_string!("read"))
            .length(1)
            .build();
        reader.set(js_string!("read"), read_fn, true, context)?;

        let cancel_fn = BuiltInBuilder::callable(context.realm(), Self::cancel)
            .name(js_string!("cancel"))
            .length(1)
            .build();
        reader.set(js_string!("cancel"), cancel_fn, true, context)?;

        let release_lock_fn = BuiltInBuilder::callable(context.realm(), Self::release_lock)
            .name(js_string!("releaseLock"))
            .length(0)
            .build();
        reader.set(js_string!("releaseLock"), release_lock_fn, true, context)?;

        // Add properties
        let closed_getter = BuiltInBuilder::callable(context.realm(), Self::get_closed)
            .name(js_string!("get closed"))
            .build();
        reader.define_property_or_throw(
            js_string!("closed"),
            crate::property::PropertyDescriptorBuilder::new()
                .get(closed_getter)
                .configurable(true)
                .enumerable(true),
            context,
        )?;

        Ok(reader.into())
    }

    /// `ReadableStreamBYOBReader.prototype.read(view)`
    fn read(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ReadableStreamBYOBReader.prototype.read called on non-object")
        })?;

        let _view = args.get_or_undefined(0);

        if let Some(reader_data) = this_obj.downcast_ref::<ReadableStreamBYOBReaderData>() {
            if let Some(stream_data) = reader_data.stream.downcast_ref::<super::readable_stream::ReadableStreamData>() {
                match stream_data.state {
                    super::readable_stream::StreamState::Closed => {
                        let result_obj = JsObject::with_object_proto(context.intrinsics());
                        result_obj.set(js_string!("value"), JsValue::undefined(), true, context)?;
                        result_obj.set(js_string!("done"), JsValue::from(true), true, context)?;
                        let promise_constructor = context.intrinsics().constructors().promise().constructor();
                    return crate::builtins::Promise::resolve(&promise_constructor.into(), &[JsValue::from(result_obj)], context);
                    },
                    _ => {
                        // For BYOB reader, we'd need to handle the view parameter properly
                        // This is a simplified implementation
                        let result_obj = JsObject::with_object_proto(context.intrinsics());
                        result_obj.set(js_string!("value"), _view.clone(), true, context)?;
                        result_obj.set(js_string!("done"), JsValue::from(false), true, context)?;
                        let promise_constructor = context.intrinsics().constructors().promise().constructor();
                    return crate::builtins::Promise::resolve(&promise_constructor.into(), &[JsValue::from(result_obj)], context);
                    }
                }
            }
        }

        let promise_constructor = context.intrinsics().constructors().promise().constructor();
        Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
    }

    /// `ReadableStreamBYOBReader.prototype.cancel(reason)`
    fn cancel(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ReadableStreamBYOBReader.prototype.cancel called on non-object")
        })?;

        let _reason = args.get_or_undefined(0);

        if let Some(reader_data) = this_obj.downcast_ref::<ReadableStreamBYOBReaderData>() {
            if let Some(mut stream_data) = reader_data.stream.downcast_mut::<super::readable_stream::ReadableStreamData>() {
                stream_data.state = super::readable_stream::StreamState::Closed;
            }
        }

        let promise_constructor = context.intrinsics().constructors().promise().constructor();
        crate::builtins::Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
    }

    /// `ReadableStreamBYOBReader.prototype.releaseLock()`
    fn release_lock(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ReadableStreamBYOBReader.prototype.releaseLock called on non-object")
        })?;

        if let Some(reader_data) = this_obj.downcast_ref::<ReadableStreamBYOBReaderData>() {
            if let Some(mut stream_data) = reader_data.stream.downcast_mut::<super::readable_stream::ReadableStreamData>() {
                stream_data.locked = false;
            }
        }

        Ok(JsValue::undefined())
    }

    /// Get the closed property of a ReadableStreamBYOBReader
    fn get_closed(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ReadableStreamBYOBReader.prototype.closed getter called on non-object")
        })?;

        if let Some(reader_data) = this_obj.downcast_ref::<ReadableStreamBYOBReaderData>() {
            if let Some(stream_data) = reader_data.stream.downcast_ref::<super::readable_stream::ReadableStreamData>() {
                match stream_data.state {
                    super::readable_stream::StreamState::Closed => {
                        let promise_constructor = context.intrinsics().constructors().promise().constructor();
                        crate::builtins::Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
                    },
                    super::readable_stream::StreamState::Errored => {
                        let promise_constructor = context.intrinsics().constructors().promise().constructor();
                        crate::builtins::Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
                    },
                    _ => {
                        let promise_constructor = context.intrinsics().constructors().promise().constructor();
                        crate::builtins::Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
                    }
                }
            } else {
                {
                    let promise_constructor = context.intrinsics().constructors().promise().constructor();
                    Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
                }
            }
        } else {
            {
                    let promise_constructor = context.intrinsics().constructors().promise().constructor();
                    Promise::resolve(&promise_constructor.into(), &[JsValue::undefined()], context)
                }
        }
    }
}

/// Internal data for ReadableStreamDefaultReader instances
#[derive(Debug, Trace, Finalize, JsData)]
struct ReadableStreamDefaultReaderData {
    stream: JsObject,
}

impl ReadableStreamDefaultReaderData {
    fn new(stream: JsObject) -> Self {
        Self { stream }
    }
}

/// Internal data for ReadableStreamBYOBReader instances
#[derive(Debug, Trace, Finalize, JsData)]
struct ReadableStreamBYOBReaderData {
    stream: JsObject,
}

impl ReadableStreamBYOBReaderData {
    fn new(stream: JsObject) -> Self {
        Self { stream }
    }
}