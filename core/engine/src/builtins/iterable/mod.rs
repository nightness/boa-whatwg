//! Boa's implementation of ECMAScript's `IteratorRecord` and iterator prototype objects.

use crate::{
    Context, JsArgs, JsResult, JsValue,
    builtins::{BuiltInBuilder, IntrinsicObject, Array},
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    js_string,
    object::JsObject,
    property::PropertyDescriptor,
    realm::Realm,
    symbol::JsSymbol,
};
use boa_gc::{Finalize, Trace};

mod async_from_sync_iterator;
pub(crate) use async_from_sync_iterator::AsyncFromSyncIterator;

/// `IfAbruptCloseIterator ( value, iteratorRecord )`
///
/// `IfAbruptCloseIterator` is a shorthand for a sequence of algorithm steps that use an `Iterator`
/// Record.
///
/// More information:
///  - [ECMA reference][spec]
///
///  [spec]: https://tc39.es/ecma262/#sec-ifabruptcloseiterator
macro_rules! if_abrupt_close_iterator {
    ($value:expr, $iterator_record:expr, $context:expr) => {
        match $value {
            // 1. If value is an abrupt completion, return ? IteratorClose(iteratorRecord, value).
            Err(err) => return $iterator_record.close(Err(err), $context),
            // 2. Else if value is a Completion Record, set value to value.
            Ok(value) => value,
        }
    };
}

// Export macro to crate level
pub(crate) use if_abrupt_close_iterator;

use super::OrdinaryObject;

/// The built-in iterator prototypes.
#[derive(Debug, Trace, Finalize)]
pub struct IteratorPrototypes {
    /// The `IteratorPrototype` object.
    iterator: JsObject,

    /// The `AsyncIteratorPrototype` object.
    async_iterator: JsObject,

    /// The `AsyncFromSyncIteratorPrototype` prototype object.
    async_from_sync_iterator: JsObject,

    /// The `ArrayIteratorPrototype` prototype object.
    array: JsObject,

    /// The `SetIteratorPrototype` prototype object.
    set: JsObject,

    /// The `StringIteratorPrototype` prototype object.
    string: JsObject,

    /// The `RegExpStringIteratorPrototype` prototype object.
    regexp_string: JsObject,

    /// The `MapIteratorPrototype` prototype object.
    map: JsObject,

    /// The `ForInIteratorPrototype` prototype object.
    for_in: JsObject,

    /// The `%SegmentIteratorPrototype%` prototype object.
    #[cfg(feature = "intl")]
    segment: JsObject,
}

impl Default for IteratorPrototypes {
    fn default() -> Self {
        Self {
            iterator: JsObject::with_null_proto(),
            async_iterator: JsObject::with_null_proto(),
            async_from_sync_iterator: JsObject::with_null_proto(),
            array: JsObject::with_null_proto(),
            set: JsObject::with_null_proto(),
            string: JsObject::with_null_proto(),
            regexp_string: JsObject::with_null_proto(),
            map: JsObject::with_null_proto(),
            for_in: JsObject::with_null_proto(),
            #[cfg(feature = "intl")]
            segment: JsObject::with_null_proto(),
        }
    }
}

impl IteratorPrototypes {
    /// Returns the `ArrayIteratorPrototype` object.
    #[inline]
    #[must_use]
    pub fn array(&self) -> JsObject {
        self.array.clone()
    }

    /// Returns the `IteratorPrototype` object.
    #[inline]
    #[must_use]
    pub fn iterator(&self) -> JsObject {
        self.iterator.clone()
    }

    /// Returns the `AsyncIteratorPrototype` object.
    #[inline]
    #[must_use]
    pub fn async_iterator(&self) -> JsObject {
        self.async_iterator.clone()
    }

    /// Returns the `AsyncFromSyncIteratorPrototype` object.
    #[inline]
    #[must_use]
    pub fn async_from_sync_iterator(&self) -> JsObject {
        self.async_from_sync_iterator.clone()
    }

    /// Returns the `SetIteratorPrototype` object.
    #[inline]
    #[must_use]
    pub fn set(&self) -> JsObject {
        self.set.clone()
    }

    /// Returns the `StringIteratorPrototype` object.
    #[inline]
    #[must_use]
    pub fn string(&self) -> JsObject {
        self.string.clone()
    }

    /// Returns the `RegExpStringIteratorPrototype` object.
    #[inline]
    #[must_use]
    pub fn regexp_string(&self) -> JsObject {
        self.regexp_string.clone()
    }

    /// Returns the `MapIteratorPrototype` object.
    #[inline]
    #[must_use]
    pub fn map(&self) -> JsObject {
        self.map.clone()
    }

    /// Returns the `ForInIteratorPrototype` object.
    #[inline]
    #[must_use]
    pub fn for_in(&self) -> JsObject {
        self.for_in.clone()
    }

    /// Returns the `%SegmentIteratorPrototype%` object.
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub fn segment(&self) -> JsObject {
        self.segment.clone()
    }
}

/// `%IteratorPrototype%` object
///
/// More information:
///  - [ECMA reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-%iteratorprototype%-object
pub(crate) struct Iterator;

impl IntrinsicObject for Iterator {
    fn init(realm: &Realm) {
        let iterator_prototype = BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_method(|v, _, _| Ok(v.clone()), JsSymbol::iterator(), 0)
            .build();

        // Add ES2025 Iterator Helper methods manually
        let map_fn = BuiltInBuilder::callable(realm, Self::map)
            .name(js_string!("map"))
            .length(1)
            .build();
        iterator_prototype.insert(
            js_string!("map"),
            PropertyDescriptor::builder()
                .value(map_fn)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );

        let filter_fn = BuiltInBuilder::callable(realm, Self::filter)
            .name(js_string!("filter"))
            .length(1)
            .build();
        iterator_prototype.insert(
            js_string!("filter"),
            PropertyDescriptor::builder()
                .value(filter_fn)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );

        let take_fn = BuiltInBuilder::callable(realm, Self::take)
            .name(js_string!("take"))
            .length(1)
            .build();
        iterator_prototype.insert(
            js_string!("take"),
            PropertyDescriptor::builder()
                .value(take_fn)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );

        let drop_fn = BuiltInBuilder::callable(realm, Self::drop)
            .name(js_string!("drop"))
            .length(1)
            .build();
        iterator_prototype.insert(
            js_string!("drop"),
            PropertyDescriptor::builder()
                .value(drop_fn)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );

        let flat_map_fn = BuiltInBuilder::callable(realm, Self::flat_map)
            .name(js_string!("flatMap"))
            .length(1)
            .build();
        iterator_prototype.insert(
            js_string!("flatMap"),
            PropertyDescriptor::builder()
                .value(flat_map_fn)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );

        let reduce_fn = BuiltInBuilder::callable(realm, Self::reduce)
            .name(js_string!("reduce"))
            .length(1)
            .build();
        iterator_prototype.insert(
            js_string!("reduce"),
            PropertyDescriptor::builder()
                .value(reduce_fn)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );

        let to_array_fn = BuiltInBuilder::callable(realm, Self::to_array)
            .name(js_string!("toArray"))
            .length(0)
            .build();
        iterator_prototype.insert(
            js_string!("toArray"),
            PropertyDescriptor::builder()
                .value(to_array_fn)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );

        let for_each_fn = BuiltInBuilder::callable(realm, Self::for_each)
            .name(js_string!("forEach"))
            .length(1)
            .build();
        iterator_prototype.insert(
            js_string!("forEach"),
            PropertyDescriptor::builder()
                .value(for_each_fn)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );

        let some_fn = BuiltInBuilder::callable(realm, Self::some)
            .name(js_string!("some"))
            .length(1)
            .build();
        iterator_prototype.insert(
            js_string!("some"),
            PropertyDescriptor::builder()
                .value(some_fn)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );

        let every_fn = BuiltInBuilder::callable(realm, Self::every)
            .name(js_string!("every"))
            .length(1)
            .build();
        iterator_prototype.insert(
            js_string!("every"),
            PropertyDescriptor::builder()
                .value(every_fn)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );

        let find_fn = BuiltInBuilder::callable(realm, Self::find)
            .name(js_string!("find"))
            .length(1)
            .build();
        iterator_prototype.insert(
            js_string!("find"),
            PropertyDescriptor::builder()
                .value(find_fn)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().iterator()
    }
}

impl Iterator {
    /// `Iterator.prototype.map ( mapper )`
    ///
    /// More information:
    ///  - [ES2025 spec][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-iteratorprototype.map
    fn map(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If O is not an Object, throw a TypeError exception.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.map called on non-object")
        })?;

        // 3. If IsCallable(mapper) is false, throw a TypeError exception.
        let mapper = args.get_or_undefined(0).as_callable().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.map mapper must be callable")
        })?;

        // Get the iterator's next method
        let next_method = o.get(js_string!("next"), context)?;

        // Create an IteratorRecord
        let mut iter = IteratorRecord::new(o.clone(), next_method);

        let mut result = Vec::new();
        let mut counter = 0u64;

        loop {
            // Get next value
            let next_result = iter.next(None, context)?;
            if next_result.complete(context)? {
                break;
            }

            let value = next_result.value(context)?;
            let mapped = mapper.call(&JsValue::undefined(), &[value, counter.into()], context)?;
            result.push(mapped);
            counter += 1;
        }

        // Return as array
        Ok(Array::create_array_from_list(result, context).into())
    }

    /// `Iterator.prototype.filter ( predicate )`
    fn filter(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.filter called on non-object")
        })?;

        let predicate = args.get_or_undefined(0).as_callable().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.filter predicate must be callable")
        })?;

        let next_method = o.get(js_string!("next"), context)?;
        let mut iter = IteratorRecord::new(o.clone(), next_method);

        let mut result = Vec::new();
        let mut counter = 0u64;

        loop {
            let next_result = iter.next(None, context)?;
            if next_result.complete(context)? {
                break;
            }

            let value = next_result.value(context)?;
            let test = predicate.call(&JsValue::undefined(), &[value.clone(), counter.into()], context)?;

            if test.to_boolean() {
                result.push(value);
            }
            counter += 1;
        }

        Ok(Array::create_array_from_list(result, context).into())
    }

    /// `Iterator.prototype.take ( limit )`
    fn take(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.take called on non-object")
        })?;

        let limit = args.get_or_undefined(0).to_number(context)? as usize;

        let next_method = o.get(js_string!("next"), context)?;
        let mut iter = IteratorRecord::new(o.clone(), next_method);

        let mut result = Vec::new();

        for _ in 0..limit {
            let next_result = iter.next(None, context)?;
            if next_result.complete(context)? {
                break;
            }

            result.push(next_result.value(context)?);
        }

        Ok(Array::create_array_from_list(result, context).into())
    }

    /// `Iterator.prototype.drop ( limit )`
    fn drop(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.drop called on non-object")
        })?;

        let limit = args.get_or_undefined(0).to_number(context)? as usize;

        let next_method = o.get(js_string!("next"), context)?;
        let mut iter = IteratorRecord::new(o.clone(), next_method);

        // Skip the first `limit` items
        for _ in 0..limit {
            let next_result = iter.next(None, context)?;
            if next_result.complete(context)? {
                return Ok(Array::create_array_from_list(Vec::new(), context).into());
            }
        }

        // Collect remaining items
        let mut result = Vec::new();
        loop {
            let next_result = iter.next(None, context)?;
            if next_result.complete(context)? {
                break;
            }

            result.push(next_result.value(context)?);
        }

        Ok(Array::create_array_from_list(result, context).into())
    }

    /// `Iterator.prototype.flatMap ( mapper )`
    fn flat_map(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.flatMap called on non-object")
        })?;

        let mapper = args.get_or_undefined(0).as_callable().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.flatMap mapper must be callable")
        })?;

        let next_method = o.get(js_string!("next"), context)?;
        let mut iter = IteratorRecord::new(o.clone(), next_method);

        let mut result = Vec::new();
        let mut counter = 0u64;

        loop {
            let next_result = iter.next(None, context)?;
            if next_result.complete(context)? {
                break;
            }

            let value = next_result.value(context)?;
            let mapped = mapper.call(&JsValue::undefined(), &[value, counter.into()], context)?;

            // Flatten one level - check if mapped value is iterable
            if let Some(mapped_obj) = mapped.as_object() {
                if let Ok(iter_fn) = mapped_obj.get(JsSymbol::iterator(), context) {
                    if let Some(iter_callable) = iter_fn.as_callable() {
                        let inner_iter_value = iter_callable.call(&mapped, &[], context)?;
                        if let Some(inner_obj) = inner_iter_value.as_object() {
                            if let Ok(inner_next) = inner_obj.get(js_string!("next"), context) {
                                let mut inner_record = IteratorRecord::new(inner_obj.clone(), inner_next);
                                // Iterate and collect from inner iterator
                                loop {
                                    let inner_result = inner_record.next(None, context)?;
                                    if inner_result.complete(context)? {
                                        break;
                                    }
                                    result.push(inner_result.value(context)?);
                                }
                                counter += 1;
                                continue;
                            }
                        }
                    }
                }
            }

            // Not iterable or failed to get iterator, just push the value
            result.push(mapped);
            counter += 1;
        }

        Ok(Array::create_array_from_list(result, context).into())
    }

    /// `Iterator.prototype.reduce ( reducer [, initialValue] )`
    fn reduce(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.reduce called on non-object")
        })?;

        let reducer = args.get_or_undefined(0).as_callable().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.reduce reducer must be callable")
        })?;

        let next_method = o.get(js_string!("next"), context)?;
        let mut iter = IteratorRecord::new(o.clone(), next_method);

        let mut accumulator = if args.len() > 1 {
            args[1].clone()
        } else {
            // Get first value as initial accumulator
            let next_result = iter.next(None, context)?;
            if next_result.complete(context)? {
                return Err(JsNativeError::typ().with_message("Reduce of empty iterator with no initial value").into());
            }
            next_result.value(context)?
        };

        let mut counter = 0u64;

        loop {
            let next_result = iter.next(None, context)?;
            if next_result.complete(context)? {
                break;
            }

            let value = next_result.value(context)?;
            accumulator = reducer.call(&JsValue::undefined(), &[accumulator, value, counter.into()], context)?;
            counter += 1;
        }

        Ok(accumulator)
    }

    /// `Iterator.prototype.toArray ()`
    fn to_array(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.toArray called on non-object")
        })?;

        let next_method = o.get(js_string!("next"), context)?;
        let mut iter = IteratorRecord::new(o.clone(), next_method);

        let mut result = Vec::new();

        loop {
            let next_result = iter.next(None, context)?;
            if next_result.complete(context)? {
                break;
            }

            result.push(next_result.value(context)?);
        }

        Ok(Array::create_array_from_list(result, context).into())
    }

    /// `Iterator.prototype.forEach ( fn )`
    fn for_each(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.forEach called on non-object")
        })?;

        let func = args.get_or_undefined(0).as_callable().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.forEach fn must be callable")
        })?;

        let next_method = o.get(js_string!("next"), context)?;
        let mut iter = IteratorRecord::new(o.clone(), next_method);

        let mut counter = 0u64;

        loop {
            let next_result = iter.next(None, context)?;
            if next_result.complete(context)? {
                break;
            }

            let value = next_result.value(context)?;
            func.call(&JsValue::undefined(), &[value, counter.into()], context)?;
            counter += 1;
        }

        Ok(JsValue::undefined())
    }

    /// `Iterator.prototype.some ( predicate )`
    fn some(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.some called on non-object")
        })?;

        let predicate = args.get_or_undefined(0).as_callable().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.some predicate must be callable")
        })?;

        let next_method = o.get(js_string!("next"), context)?;
        let mut iter = IteratorRecord::new(o.clone(), next_method);

        let mut counter = 0u64;

        loop {
            let next_result = iter.next(None, context)?;
            if next_result.complete(context)? {
                break;
            }

            let value = next_result.value(context)?;
            let test = predicate.call(&JsValue::undefined(), &[value, counter.into()], context)?;

            if test.to_boolean() {
                return Ok(JsValue::from(true));
            }
            counter += 1;
        }

        Ok(JsValue::from(false))
    }

    /// `Iterator.prototype.every ( predicate )`
    fn every(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.every called on non-object")
        })?;

        let predicate = args.get_or_undefined(0).as_callable().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.every predicate must be callable")
        })?;

        let next_method = o.get(js_string!("next"), context)?;
        let mut iter = IteratorRecord::new(o.clone(), next_method);

        let mut counter = 0u64;

        loop {
            let next_result = iter.next(None, context)?;
            if next_result.complete(context)? {
                break;
            }

            let value = next_result.value(context)?;
            let test = predicate.call(&JsValue::undefined(), &[value, counter.into()], context)?;

            if !test.to_boolean() {
                return Ok(JsValue::from(false));
            }
            counter += 1;
        }

        Ok(JsValue::from(true))
    }

    /// `Iterator.prototype.find ( predicate )`
    fn find(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.find called on non-object")
        })?;

        let predicate = args.get_or_undefined(0).as_callable().ok_or_else(|| {
            JsNativeError::typ().with_message("Iterator.prototype.find predicate must be callable")
        })?;

        let next_method = o.get(js_string!("next"), context)?;
        let mut iter = IteratorRecord::new(o.clone(), next_method);

        let mut counter = 0u64;

        loop {
            let next_result = iter.next(None, context)?;
            if next_result.complete(context)? {
                break;
            }

            let value = next_result.value(context)?;
            let test = predicate.call(&JsValue::undefined(), &[value.clone(), counter.into()], context)?;

            if test.to_boolean() {
                return Ok(value);
            }
            counter += 1;
        }

        Ok(JsValue::undefined())
    }
}

/// `%AsyncIteratorPrototype%` object
///
/// More information:
///  - [ECMA reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-asynciteratorprototype
pub(crate) struct AsyncIterator;

impl IntrinsicObject for AsyncIterator {
    fn init(realm: &Realm) {
        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_method(|v, _, _| Ok(v.clone()), JsSymbol::async_iterator(), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().async_iterator()
    }
}

/// `CreateIterResultObject( value, done )`
///
/// Generates an object supporting the `IteratorResult` interface.
pub fn create_iter_result_object(value: JsValue, done: bool, context: &mut Context) -> JsValue {
    // 1. Assert: Type(done) is Boolean.
    // 2. Let obj be ! OrdinaryObjectCreate(%Object.prototype%).
    // 3. Perform ! CreateDataPropertyOrThrow(obj, "value", value).
    // 4. Perform ! CreateDataPropertyOrThrow(obj, "done", done).
    let obj = context
        .intrinsics()
        .templates()
        .iterator_result()
        .create(OrdinaryObject, vec![value, done.into()]);

    // 5. Return obj.
    obj.into()
}

/// Iterator hint for `GetIterator`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IteratorHint {
    /// Hints that the iterator should be sync.
    Sync,

    /// Hints that the iterator should be async.
    Async,
}

impl JsValue {
    /// `GetIteratorFromMethod ( obj, method )`
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getiteratorfrommethod
    pub fn get_iterator_from_method(
        &self,
        method: &JsObject,
        context: &mut Context,
    ) -> JsResult<IteratorRecord> {
        // 1. Let iterator be ? Call(method, obj).
        let iterator = method.call(self, &[], context)?;
        // 2. If iterator is not an Object, throw a TypeError exception.
        let iterator_obj = iterator.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("returned iterator is not an object")
        })?;
        // 3. Let nextMethod be ? Get(iterator, "next").
        let next_method = iterator_obj.get(js_string!("next"), context)?;
        // 4. Let iteratorRecord be the Iterator Record { [[Iterator]]: iterator, [[NextMethod]]: nextMethod, [[Done]]: false }.
        // 5. Return iteratorRecord.
        Ok(IteratorRecord::new(iterator_obj.clone(), next_method))
    }

    /// `GetIterator ( obj, kind )`
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getiterator
    pub fn get_iterator(
        &self,
        hint: IteratorHint,
        context: &mut Context,
    ) -> JsResult<IteratorRecord> {
        let method = match hint {
            // 1. If kind is async, then
            IteratorHint::Async => {
                // a. Let method be ? GetMethod(obj, %Symbol.asyncIterator%).
                let Some(method) = self.get_method(JsSymbol::async_iterator(), context)? else {
                    // b. If method is undefined, then
                    //     i. Let syncMethod be ? GetMethod(obj, %Symbol.iterator%).
                    let sync_method =
                        self.get_method(JsSymbol::iterator(), context)?
                            .ok_or_else(|| {
                                // ii. If syncMethod is undefined, throw a TypeError exception.
                                JsNativeError::typ().with_message(format!(
                                    "value with type `{}` is not iterable",
                                    self.type_of()
                                ))
                            })?;
                    // iii. Let syncIteratorRecord be ? GetIteratorFromMethod(obj, syncMethod).
                    let sync_iterator_record =
                        self.get_iterator_from_method(&sync_method, context)?;
                    // iv. Return CreateAsyncFromSyncIterator(syncIteratorRecord).
                    return Ok(AsyncFromSyncIterator::create(sync_iterator_record, context));
                };

                Some(method)
            }
            // 2. Else,
            IteratorHint::Sync => {
                // a. Let method be ? GetMethod(obj, %Symbol.iterator%).
                self.get_method(JsSymbol::iterator(), context)?
            }
        };

        let method = method.ok_or_else(|| {
            // 3. If method is undefined, throw a TypeError exception.
            JsNativeError::typ().with_message(format!(
                "value with type `{}` is not iterable",
                self.type_of()
            ))
        })?;

        // 4. Return ? GetIteratorFromMethod(obj, method).
        self.get_iterator_from_method(&method, context)
    }
}

/// The result of the iteration process.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct IteratorResult {
    object: JsObject,
}

impl IteratorResult {
    /// Gets a new `IteratorResult` from a value. Returns `Err` if
    /// the value is not a [`JsObject`]
    pub(crate) fn from_value(value: JsValue) -> JsResult<Self> {
        if let Some(object) = value.into_object() {
            Ok(Self { object })
        } else {
            Err(JsNativeError::typ()
                .with_message("next value should be an object")
                .into())
        }
    }

    /// Gets the inner object of this `IteratorResult`.
    pub(crate) const fn object(&self) -> &JsObject {
        &self.object
    }

    /// `IteratorComplete ( iterResult )`
    ///
    /// The abstract operation `IteratorComplete` takes argument `iterResult` (an `Object`) and
    /// returns either a normal completion containing a `Boolean` or a throw completion.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iteratorcomplete
    #[inline]
    pub fn complete(&self, context: &mut Context) -> JsResult<bool> {
        // 1. Return ToBoolean(? Get(iterResult, "done")).
        Ok(self.object.get(js_string!("done"), context)?.to_boolean())
    }

    /// `IteratorValue ( iterResult )`
    ///
    /// The abstract operation `IteratorValue` takes argument `iterResult` (an `Object`) and
    /// returns either a normal completion containing an ECMAScript language value or a throw
    /// completion.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iteratorvalue
    #[inline]
    pub fn value(&self, context: &mut Context) -> JsResult<JsValue> {
        // 1. Return ? Get(iterResult, "value").
        self.object.get(js_string!("value"), context)
    }
}

/// Iterator Record
///
/// An Iterator Record is a Record value used to encapsulate an
/// `Iterator` or `AsyncIterator` along with the `next` method.
///
/// More information:
///  - [ECMA reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-iterator-records
#[derive(Clone, Debug, Finalize, Trace)]
pub struct IteratorRecord {
    /// `[[Iterator]]`
    ///
    /// An object that conforms to the `Iterator` or `AsyncIterator` interface.
    iterator: JsObject,

    /// `[[NextMethod]]`
    ///
    /// The `next` method of the `[[Iterator]]` object.
    next_method: JsValue,

    /// `[[Done]]`
    ///
    /// Whether the iterator has been closed.
    done: bool,

    /// The result of the last call to `next`.
    last_result: IteratorResult,
}

impl IteratorRecord {
    /// Creates a new `IteratorRecord` with the given iterator object, next method and `done` flag.
    #[inline]
    #[must_use]
    pub fn new(iterator: JsObject, next_method: JsValue) -> Self {
        Self {
            iterator,
            next_method,
            done: false,
            last_result: IteratorResult {
                object: JsObject::with_null_proto(),
            },
        }
    }

    /// Get the `[[Iterator]]` field of the `IteratorRecord`.
    pub(crate) const fn iterator(&self) -> &JsObject {
        &self.iterator
    }

    /// Gets the `[[NextMethod]]` field of the `IteratorRecord`.
    pub(crate) const fn next_method(&self) -> &JsValue {
        &self.next_method
    }

    /// Gets the last result object of the iterator record.
    pub(crate) const fn last_result(&self) -> &IteratorResult {
        &self.last_result
    }

    /// Runs `f`, setting the `done` field of this `IteratorRecord` to `true` if `f` returns
    /// an error.
    fn set_done_on_err<R, F>(&mut self, f: F) -> JsResult<R>
    where
        F: FnOnce(&mut Self) -> JsResult<R>,
    {
        let result = f(self);
        if result.is_err() {
            self.done = true;
        }
        result
    }

    /// Gets the current value of the `IteratorRecord`.
    pub(crate) fn value(&mut self, context: &mut Context) -> JsResult<JsValue> {
        self.set_done_on_err(|iter| iter.last_result.value(context))
    }

    /// Get the `[[Done]]` field of the `IteratorRecord`.
    pub(crate) const fn done(&self) -> bool {
        self.done
    }

    /// Updates the current result value of this iterator record.
    pub(crate) fn update_result(&mut self, result: JsValue, context: &mut Context) -> JsResult<()> {
        self.set_done_on_err(|iter| {
            // 3. If Type(result) is not Object, throw a TypeError exception.
            // 4. Return result.
            // `IteratorResult::from_value` does this for us.

            // `IteratorStep(iteratorRecord)`
            // https://tc39.es/ecma262/#sec-iteratorstep

            // 1. Let result be ? IteratorNext(iteratorRecord).
            let result = IteratorResult::from_value(result)?;
            // 2. Let done be ? IteratorComplete(result).
            // 3. If done is true, return false.
            iter.done = result.complete(context)?;

            iter.last_result = result;

            Ok(())
        })
    }

    /// `IteratorNext ( iteratorRecord [ , value ] )`
    ///
    /// The abstract operation `IteratorNext` takes argument `iteratorRecord` (an `Iterator`
    /// Record) and optional argument `value` (an ECMAScript language value) and returns either a
    /// normal completion containing an `Object` or a throw completion.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iteratornext
    pub(crate) fn next(
        &mut self,
        value: Option<&JsValue>,
        context: &mut Context,
    ) -> JsResult<IteratorResult> {
        // 1. If value is not present, then
        //     a. Let result be Completion(Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]])).
        // 2. Else,
        //     a. Let result be Completion(Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]], « value »)).
        // 3. If result is a throw completion, then
        //     a. Set iteratorRecord.[[Done]] to true.
        //     b. Return ? result.
        // 4. Set result to ! result.
        // 5. If result is not an Object, then
        //     a. Set iteratorRecord.[[Done]] to true.
        //     b. Throw a TypeError exception.
        // 6. Return result.
        // NOTE: In this case, `set_done_on_err` does all the heavylifting for us, which
        // simplifies the instructions below.
        self.set_done_on_err(|iter| {
            iter.next_method
                .call(
                    &iter.iterator.clone().into(),
                    value.map_or(&[], std::slice::from_ref),
                    context,
                )
                .and_then(IteratorResult::from_value)
        })
    }

    /// `IteratorStep ( iteratorRecord )`
    ///
    /// Updates the `IteratorRecord` and returns `true` if the next result record returned
    /// `done: true`, otherwise returns `false`. This differs slightly from the spec, but also
    /// simplifies some logic around iterators.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iteratorstep
    pub(crate) fn step(&mut self, context: &mut Context) -> JsResult<bool> {
        self.set_done_on_err(|iter| {
            // 1. Let result be ? IteratorNext(iteratorRecord).
            let result = iter.next(None, context)?;

            // 2. Let done be Completion(IteratorComplete(result)).
            // 3. If done is a throw completion, then
            //     a. Set iteratorRecord.[[Done]] to true.
            //     b. Return ? done.
            // 4. Set done to ! done.
            // 5. If done is true, then
            //     a. Set iteratorRecord.[[Done]] to true.
            //     b. Return done.
            iter.done = result.complete(context)?;

            iter.last_result = result;

            // 6. Return result.
            Ok(iter.done)
        })
    }

    /// `IteratorStepValue ( iteratorRecord )`
    ///
    /// Updates the `IteratorRecord` and returns `Some(value)` if the next result record returned
    /// `done: true`, otherwise returns `None`.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iteratorstepvalue
    pub(crate) fn step_value(&mut self, context: &mut Context) -> JsResult<Option<JsValue>> {
        // 1. Let result be ? IteratorStep(iteratorRecord).
        if self.step(context)? {
            // 2. If result is done, then
            //     a. Return done.
            Ok(None)
        } else {
            // 3. Let value be Completion(IteratorValue(result)).
            // 4. If value is a throw completion, then
            //     a. Set iteratorRecord.[[Done]] to true.
            // 5. Return ? value.
            self.value(context).map(Some)
        }
    }

    /// `IteratorClose ( iteratorRecord, completion )`
    ///
    /// The abstract operation `IteratorClose` takes arguments `iteratorRecord` (an
    /// [Iterator Record][Self]) and `completion` (a `Completion` Record) and returns a
    /// `Completion` Record. It is used to notify an iterator that it should perform any actions it
    /// would normally perform when it has reached its completed state.
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    ///  [spec]: https://tc39.es/ecma262/#sec-iteratorclose
    pub(crate) fn close(
        &self,
        completion: JsResult<JsValue>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Assert: Type(iteratorRecord.[[Iterator]]) is Object.

        // 2. Let iterator be iteratorRecord.[[Iterator]].
        let iterator = &self.iterator;

        // 3. Let innerResult be Completion(GetMethod(iterator, "return")).
        let inner_result = iterator.get_method(js_string!("return"), context);

        // 4. If innerResult.[[Type]] is normal, then
        let inner_result = match inner_result {
            Ok(inner_result) => {
                // a. Let return be innerResult.[[Value]].
                let r#return = inner_result;

                if let Some(r#return) = r#return {
                    // c. Set innerResult to Completion(Call(return, iterator)).
                    r#return.call(&iterator.clone().into(), &[], context)
                } else {
                    // b. If return is undefined, return ? completion.
                    return completion;
                }
            }
            Err(inner_result) => {
                // 5. If completion.[[Type]] is throw, return ? completion.
                completion?;

                // 6. If innerResult.[[Type]] is throw, return ? innerResult.
                return Err(inner_result);
            }
        };

        // 5. If completion.[[Type]] is throw, return ? completion.
        let completion = completion?;

        // 6. If innerResult.[[Type]] is throw, return ? innerResult.
        let inner_result = inner_result?;

        if inner_result.is_object() {
            // 8. Return ? completion.
            Ok(completion)
        } else {
            // 7. If Type(innerResult.[[Value]]) is not Object, throw a TypeError exception.
            Err(JsNativeError::typ()
                .with_message("inner result was not an object")
                .into())
        }
    }

    /// `IteratorToList ( iteratorRecord )`
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    ///  [spec]: https://tc39.es/ecma262/#sec-iteratortolist
    pub(crate) fn into_list(mut self, context: &mut Context) -> JsResult<Vec<JsValue>> {
        // 1. Let values be a new empty List.
        let mut values = Vec::new();

        // 2. Repeat,
        //     a. Let next be ? IteratorStepValue(iteratorRecord).
        while let Some(value) = self.step_value(context)? {
            // c. Append next to values.
            values.push(value);
        }

        //     b. If next is done, then
        //         i. Return values.
        Ok(values)
    }
}
