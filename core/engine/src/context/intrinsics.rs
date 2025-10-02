//! Data structures that contain intrinsic objects and constructors.

use boa_gc::{Finalize, Trace};

use crate::{
    JsSymbol,
    builtins::{Array, OrdinaryObject, iterable::IteratorPrototypes, uri::UriFunctions},
    js_string,
    object::{
        CONSTRUCTOR, JsFunction, JsObject, Object, PROTOTYPE,
        internal_methods::immutable_prototype::IMMUTABLE_PROTOTYPE_EXOTIC_INTERNAL_METHODS,
        shape::{RootShape, shared_shape::template::ObjectTemplate},
    },
    property::{Attribute, PropertyKey},
};

#[cfg(feature = "intl")]
use crate::builtins::intl::Intl;

/// The intrinsic objects and constructors.
///
/// `Intrinsics` is internally stored using a `Gc`, which makes it cheapily clonable
/// for multiple references to the same set of intrinsic objects.
#[derive(Debug, Trace, Finalize)]
pub struct Intrinsics {
    /// Cached standard constructors
    pub(super) constructors: StandardConstructors,
    /// Cached intrinsic objects
    pub(super) objects: IntrinsicObjects,
    /// Cached object templates.
    pub(super) templates: ObjectTemplates,
}

impl Intrinsics {
    /// Creates a new set of uninitialized intrinsics.
    ///
    /// Creates all the required empty objects for every intrinsic in this realm.
    ///
    /// To initialize all the intrinsics with their spec properties, see [`Realm::initialize`].
    ///
    /// [`Realm::initialize`]: crate::realm::Realm::initialize
    pub(crate) fn uninit(root_shape: &RootShape) -> Option<Self> {
        let constructors = StandardConstructors::default();
        let templates = ObjectTemplates::new(root_shape, &constructors);

        Some(Self {
            constructors,
            objects: IntrinsicObjects::uninit()?,
            templates,
        })
    }

    /// Return the cached intrinsic objects.
    #[inline]
    #[must_use]
    pub const fn objects(&self) -> &IntrinsicObjects {
        &self.objects
    }

    /// Return the cached standard constructors.
    #[inline]
    #[must_use]
    pub const fn constructors(&self) -> &StandardConstructors {
        &self.constructors
    }

    /// Returns a mutable reference to the standard constructors.
    ///
    /// This is needed for external crates (like thalora-browser-apis) to register
    /// their constructors in the intrinsics after extraction from Boa.
    pub fn constructors_mut(&mut self) -> &mut StandardConstructors {
        &mut self.constructors
    }

    pub(crate) const fn templates(&self) -> &ObjectTemplates {
        &self.templates
    }

    /// Initialize all intrinsic objects by calling their `init()` methods.
    ///
    /// This populates each intrinsic object with its methods and properties according to
    /// the ECMAScript specification.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createintrinsics
    pub(crate) fn initialize(&self, realm: &crate::realm::Realm) {
        use crate::builtins::{self, IntrinsicObject};

        builtins::function::BuiltInFunctionObject::init(realm);
        builtins::object::OrdinaryObject::init(realm);
        builtins::math::Math::init(realm);
        builtins::json::Json::init(realm);
        builtins::array::Array::init(realm);
        builtins::proxy::Proxy::init(realm);
        builtins::array_buffer::ArrayBuffer::init(realm);
        builtins::array_buffer::SharedArrayBuffer::init(realm);
        builtins::bigint::BigInt::init(realm);
        builtins::boolean::Boolean::init(realm);
        builtins::date::Date::init(realm);
        builtins::dataview::DataView::init(realm);
        builtins::map::Map::init(realm);
        builtins::number::Number::init(realm);
        builtins::number::IsFinite::init(realm);
        builtins::number::IsNaN::init(realm);
        builtins::number::ParseInt::init(realm);
        builtins::number::ParseFloat::init(realm);
        builtins::eval::Eval::init(realm);
        builtins::set::Set::init(realm);
        builtins::string::String::init(realm);
        builtins::regexp::RegExp::init(realm);
        builtins::symbol::Symbol::init(realm);
        builtins::error::Error::init(realm);
        builtins::error::RangeError::init(realm);
        builtins::error::ReferenceError::init(realm);
        builtins::error::TypeError::init(realm);
        builtins::error::SyntaxError::init(realm);
        builtins::error::EvalError::init(realm);
        builtins::error::UriError::init(realm);
        builtins::error::AggregateError::init(realm);
        builtins::reflect::Reflect::init(realm);
        builtins::promise::Promise::init(realm);
        builtins::uri::EncodeUri::init(realm);
        builtins::uri::EncodeUriComponent::init(realm);
        builtins::uri::DecodeUri::init(realm);
        builtins::uri::DecodeUriComponent::init(realm);
        builtins::weak::WeakRef::init(realm);
        builtins::weak_map::WeakMap::init(realm);
        builtins::weak_set::WeakSet::init(realm);
        builtins::atomics::Atomics::init(realm);

        // Typed arrays
        builtins::typed_array::Int8Array::init(realm);
        builtins::typed_array::Uint8Array::init(realm);
        builtins::typed_array::Uint8ClampedArray::init(realm);
        builtins::typed_array::Int16Array::init(realm);
        builtins::typed_array::Uint16Array::init(realm);
        builtins::typed_array::Int32Array::init(realm);
        builtins::typed_array::Uint32Array::init(realm);
        builtins::typed_array::BigInt64Array::init(realm);
        builtins::typed_array::BigUint64Array::init(realm);
        builtins::typed_array::Float32Array::init(realm);
        builtins::typed_array::Float64Array::init(realm);

        #[cfg(feature = "annex-b")]
        {
            builtins::escape::Escape::init(realm);
            builtins::escape::Unescape::init(realm);
        }

        #[cfg(feature = "intl")]
        {
            builtins::intl::Intl::init(realm);
        }

        #[cfg(feature = "temporal")]
        {
            builtins::temporal::Temporal::init(realm);
        }
    }
}

/// Stores a constructor (such as `Object`) and its corresponding prototype.
#[derive(Debug, Trace, Finalize, Clone)]
pub struct StandardConstructor {
    constructor: JsFunction,
    prototype: JsObject,
}

impl Default for StandardConstructor {
    fn default() -> Self {
        Self {
            constructor: JsFunction::empty_intrinsic_function(true),
            prototype: JsObject::with_null_proto(),
        }
    }
}

impl StandardConstructor {
    /// Creates a new `StandardConstructor` from the constructor and the prototype.
    ///
    /// Made public for external crates (like thalora-browser-apis) to register their constructors.
    pub fn new(constructor: JsFunction, prototype: JsObject) -> Self {
        Self {
            constructor,
            prototype,
        }
    }

    /// Build a constructor with a defined prototype.
    fn with_prototype(prototype: JsObject) -> Self {
        Self {
            constructor: JsFunction::empty_intrinsic_function(true),
            prototype,
        }
    }

    /// Return the prototype of the constructor object.
    ///
    /// This is the same as `Object.prototype`, `Array.prototype`, etc.
    #[inline]
    #[must_use]
    pub fn prototype(&self) -> JsObject {
        self.prototype.clone()
    }

    /// Return the constructor object.
    ///
    /// This is the same as `Object`, `Array`, etc.
    #[inline]
    #[must_use]
    pub fn constructor(&self) -> JsObject {
        self.constructor.clone().into()
    }
}

/// Cached core standard constructors.
#[derive(Debug, Trace, Finalize)]
pub struct StandardConstructors {
    object: StandardConstructor,
    proxy: StandardConstructor,
    date: StandardConstructor,
    function: StandardConstructor,
    async_function: StandardConstructor,
    generator_function: StandardConstructor,
    async_generator_function: StandardConstructor,
    array: StandardConstructor,
    bigint: StandardConstructor,
    number: StandardConstructor,
    boolean: StandardConstructor,
    string: StandardConstructor,
    regexp: StandardConstructor,
    symbol: StandardConstructor,
    error: StandardConstructor,
    type_error: StandardConstructor,
    reference_error: StandardConstructor,
    range_error: StandardConstructor,
    syntax_error: StandardConstructor,
    eval_error: StandardConstructor,
    uri_error: StandardConstructor,
    aggregate_error: StandardConstructor,
    map: StandardConstructor,
    set: StandardConstructor,
    typed_array: StandardConstructor,
    typed_int8_array: StandardConstructor,
    typed_uint8_array: StandardConstructor,
    typed_uint8clamped_array: StandardConstructor,
    typed_int16_array: StandardConstructor,
    typed_uint16_array: StandardConstructor,
    typed_int32_array: StandardConstructor,
    typed_uint32_array: StandardConstructor,
    typed_bigint64_array: StandardConstructor,
    typed_biguint64_array: StandardConstructor,
    #[cfg(feature = "float16")]
    typed_float16_array: StandardConstructor,
    typed_float32_array: StandardConstructor,
    typed_float64_array: StandardConstructor,
    array_buffer: StandardConstructor,
    shared_array_buffer: StandardConstructor,
    data_view: StandardConstructor,
    date_time_format: StandardConstructor,
    promise: StandardConstructor,
    readable_stream: StandardConstructor,
    writable_stream: StandardConstructor,
    transform_stream: StandardConstructor,
    count_queuing_strategy: StandardConstructor,
    byte_length_queuing_strategy: StandardConstructor,
    websocket: StandardConstructor,
    websocket_stream: StandardConstructor,
    // WebAssembly API constructors
    webassembly_module: StandardConstructor,
    webassembly_instance: StandardConstructor,
    webassembly_memory: StandardConstructor,
    webassembly_table: StandardConstructor,
    webassembly_global: StandardConstructor,
    worker: StandardConstructor,
    shared_worker: StandardConstructor,
    service_worker: StandardConstructor,
    service_worker_container: StandardConstructor,
    worklet: StandardConstructor,
    message_channel: StandardConstructor,
    message_port: StandardConstructor,
    broadcast_channel: StandardConstructor,
    crypto: StandardConstructor,
    request: StandardConstructor,
    response: StandardConstructor,
    headers: StandardConstructor,
    abort_controller: StandardConstructor,
    xmlhttprequest: StandardConstructor,
    mutation_observer: StandardConstructor,
    intersection_observer: StandardConstructor,
    resize_observer: StandardConstructor,
    document: StandardConstructor,
    window: StandardConstructor,
    history: StandardConstructor,
    pageswap_event: StandardConstructor,
    node: StandardConstructor,
    character_data: StandardConstructor,
    text: StandardConstructor,
    document_fragment: StandardConstructor,
    shadow_root: StandardConstructor,
    html_slot_element: StandardConstructor,
    nodelist: StandardConstructor,
    element: StandardConstructor,
    attr: StandardConstructor,
    comment: StandardConstructor,
    domtokenlist: StandardConstructor,
    processing_instruction: StandardConstructor,
    cdata_section: StandardConstructor,
    html_form_element: StandardConstructor,
    html_form_controls_collection: StandardConstructor,
    html_input_element: StandardConstructor,
    selection: StandardConstructor,
    range: StandardConstructor,
    event: StandardConstructor,
    event_target: StandardConstructor,
    custom_event: StandardConstructor,
    message_event: StandardConstructor,
    console: StandardConstructor,
    blob: StandardConstructor,
    file: StandardConstructor,
    file_reader: StandardConstructor,
    event_source: StandardConstructor,
    rtc_peer_connection: StandardConstructor,
    rtc_data_channel: StandardConstructor,
    rtc_ice_candidate: StandardConstructor,
    rtc_session_description: StandardConstructor,
    weak_ref: StandardConstructor,
    weak_map: StandardConstructor,
    weak_set: StandardConstructor,
    storage: StandardConstructor,
    storage_event: StandardConstructor,
    storage_manager: StandardConstructor,
    cache: StandardConstructor,
    cache_storage: StandardConstructor,
    cookie_store: StandardConstructor,
    file_system_handle: StandardConstructor,
    file_system_file_handle: StandardConstructor,
    file_system_directory_handle: StandardConstructor,
    lock_manager: StandardConstructor,
    idb_factory: StandardConstructor,
    navigator: StandardConstructor,
    performance: StandardConstructor,
    #[cfg(feature = "intl")]
    collator: StandardConstructor,
    #[cfg(feature = "intl")]
    list_format: StandardConstructor,
    #[cfg(feature = "intl")]
    locale: StandardConstructor,
    #[cfg(feature = "intl")]
    segmenter: StandardConstructor,
    #[cfg(feature = "intl")]
    plural_rules: StandardConstructor,
    #[cfg(feature = "intl")]
    number_format: StandardConstructor,
    #[cfg(feature = "temporal")]
    instant: StandardConstructor,
    #[cfg(feature = "temporal")]
    plain_date_time: StandardConstructor,
    #[cfg(feature = "temporal")]
    plain_date: StandardConstructor,
    #[cfg(feature = "temporal")]
    plain_time: StandardConstructor,
    #[cfg(feature = "temporal")]
    plain_year_month: StandardConstructor,
    #[cfg(feature = "temporal")]
    plain_month_day: StandardConstructor,
    #[cfg(feature = "temporal")]
    time_zone: StandardConstructor,
    #[cfg(feature = "temporal")]
    duration: StandardConstructor,
    #[cfg(feature = "temporal")]
    zoned_date_time: StandardConstructor,
    #[cfg(feature = "temporal")]
    calendar: StandardConstructor,
}

impl Default for StandardConstructors {
    fn default() -> Self {
        Self {
            object: StandardConstructor::with_prototype(JsObject::from_object_and_vtable(
                Object::<OrdinaryObject>::default(),
                &IMMUTABLE_PROTOTYPE_EXOTIC_INTERNAL_METHODS,
            )),
            async_generator_function: StandardConstructor::default(),
            proxy: StandardConstructor::default(),
            date: StandardConstructor::default(),
            function: StandardConstructor {
                constructor: JsFunction::empty_intrinsic_function(true),
                prototype: JsFunction::empty_intrinsic_function(false).into(),
            },
            async_function: StandardConstructor::default(),
            generator_function: StandardConstructor::default(),
            array: StandardConstructor::with_prototype(JsObject::from_proto_and_data(None, Array)),
            bigint: StandardConstructor::default(),
            number: StandardConstructor::with_prototype(JsObject::from_proto_and_data(None, 0.0)),
            boolean: StandardConstructor::with_prototype(JsObject::from_proto_and_data(
                None, false,
            )),
            string: StandardConstructor::with_prototype(JsObject::from_proto_and_data(
                None,
                js_string!(),
            )),
            regexp: StandardConstructor::default(),
            symbol: StandardConstructor::default(),
            error: StandardConstructor::default(),
            type_error: StandardConstructor::default(),
            reference_error: StandardConstructor::default(),
            range_error: StandardConstructor::default(),
            syntax_error: StandardConstructor::default(),
            eval_error: StandardConstructor::default(),
            uri_error: StandardConstructor::default(),
            aggregate_error: StandardConstructor::default(),
            map: StandardConstructor::default(),
            set: StandardConstructor::default(),
            typed_array: StandardConstructor::default(),
            typed_int8_array: StandardConstructor::default(),
            typed_uint8_array: StandardConstructor::default(),
            typed_uint8clamped_array: StandardConstructor::default(),
            typed_int16_array: StandardConstructor::default(),
            typed_uint16_array: StandardConstructor::default(),
            typed_int32_array: StandardConstructor::default(),
            typed_uint32_array: StandardConstructor::default(),
            typed_bigint64_array: StandardConstructor::default(),
            typed_biguint64_array: StandardConstructor::default(),
            #[cfg(feature = "float16")]
            typed_float16_array: StandardConstructor::default(),
            typed_float32_array: StandardConstructor::default(),
            typed_float64_array: StandardConstructor::default(),
            array_buffer: StandardConstructor::default(),
            shared_array_buffer: StandardConstructor::default(),
            data_view: StandardConstructor::default(),
            date_time_format: StandardConstructor::default(),
            promise: StandardConstructor::default(),
            readable_stream: StandardConstructor::default(),
            writable_stream: StandardConstructor::default(),
            transform_stream: StandardConstructor::default(),
            count_queuing_strategy: StandardConstructor::default(),
            byte_length_queuing_strategy: StandardConstructor::default(),
            websocket: StandardConstructor::default(),
            websocket_stream: StandardConstructor::default(),
            // WebAssembly API constructors
            webassembly_module: StandardConstructor::default(),
            webassembly_instance: StandardConstructor::default(),
            webassembly_memory: StandardConstructor::default(),
            webassembly_table: StandardConstructor::default(),
            webassembly_global: StandardConstructor::default(),
            worker: StandardConstructor::default(),
            shared_worker: StandardConstructor::default(),
            service_worker: StandardConstructor::default(),
            service_worker_container: StandardConstructor::default(),
            worklet: StandardConstructor::default(),
            message_channel: StandardConstructor::default(),
            message_port: StandardConstructor::default(),
            broadcast_channel: StandardConstructor::default(),
            crypto: StandardConstructor::default(),
            request: StandardConstructor::default(),
            response: StandardConstructor::default(),
            headers: StandardConstructor::default(),
            abort_controller: StandardConstructor::default(),
            xmlhttprequest: StandardConstructor::default(),
            mutation_observer: StandardConstructor::default(),
            intersection_observer: StandardConstructor::default(),
            resize_observer: StandardConstructor::default(),
            document: StandardConstructor::default(),
            window: StandardConstructor::default(),
            history: StandardConstructor::default(),
            pageswap_event: StandardConstructor::default(),
            node: StandardConstructor::default(),
            character_data: StandardConstructor::default(),
            text: StandardConstructor::default(),
            document_fragment: StandardConstructor::default(),
            shadow_root: StandardConstructor::default(),
            html_slot_element: StandardConstructor::default(),
            nodelist: StandardConstructor::default(),
            element: StandardConstructor::default(),
            attr: StandardConstructor::default(),
            comment: StandardConstructor::default(),
            domtokenlist: StandardConstructor::default(),
            processing_instruction: StandardConstructor::default(),
            cdata_section: StandardConstructor::default(),
            html_form_element: StandardConstructor::default(),
            html_form_controls_collection: StandardConstructor::default(),
            html_input_element: StandardConstructor::default(),
            selection: StandardConstructor::default(),
            range: StandardConstructor::default(),
            event: StandardConstructor::default(),
            event_target: StandardConstructor::default(),
            custom_event: StandardConstructor::default(),
            message_event: StandardConstructor::default(),
            console: StandardConstructor::default(),
            blob: StandardConstructor::default(),
            file: StandardConstructor::default(),
            file_reader: StandardConstructor::default(),
            event_source: StandardConstructor::default(),
            rtc_peer_connection: StandardConstructor::default(),
            rtc_data_channel: StandardConstructor::default(),
            rtc_ice_candidate: StandardConstructor::default(),
            rtc_session_description: StandardConstructor::default(),
            weak_ref: StandardConstructor::default(),
            weak_map: StandardConstructor::default(),
            weak_set: StandardConstructor::default(),
            storage: StandardConstructor::default(),
            storage_event: StandardConstructor::default(),
            storage_manager: StandardConstructor::default(),
            cache: StandardConstructor::default(),
            cache_storage: StandardConstructor::default(),
            cookie_store: StandardConstructor::default(),
            file_system_handle: StandardConstructor::default(),
            file_system_file_handle: StandardConstructor::default(),
            file_system_directory_handle: StandardConstructor::default(),
            lock_manager: StandardConstructor::default(),
            idb_factory: StandardConstructor::default(),
            navigator: StandardConstructor::default(),
            performance: StandardConstructor::default(),
            #[cfg(feature = "intl")]
            collator: StandardConstructor::default(),
            #[cfg(feature = "intl")]
            list_format: StandardConstructor::default(),
            #[cfg(feature = "intl")]
            locale: StandardConstructor::default(),
            #[cfg(feature = "intl")]
            segmenter: StandardConstructor::default(),
            #[cfg(feature = "intl")]
            plural_rules: StandardConstructor::default(),
            #[cfg(feature = "intl")]
            number_format: StandardConstructor::default(),
            #[cfg(feature = "temporal")]
            instant: StandardConstructor::default(),
            #[cfg(feature = "temporal")]
            plain_date_time: StandardConstructor::default(),
            #[cfg(feature = "temporal")]
            plain_date: StandardConstructor::default(),
            #[cfg(feature = "temporal")]
            plain_time: StandardConstructor::default(),
            #[cfg(feature = "temporal")]
            plain_year_month: StandardConstructor::default(),
            #[cfg(feature = "temporal")]
            plain_month_day: StandardConstructor::default(),
            #[cfg(feature = "temporal")]
            time_zone: StandardConstructor::default(),
            #[cfg(feature = "temporal")]
            duration: StandardConstructor::default(),
            #[cfg(feature = "temporal")]
            zoned_date_time: StandardConstructor::default(),
            #[cfg(feature = "temporal")]
            calendar: StandardConstructor::default(),
        }
    }
}

impl StandardConstructors {
    /// Returns the `AsyncGeneratorFunction` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgeneratorfunction-constructor
    #[inline]
    #[must_use]
    pub const fn async_generator_function(&self) -> &StandardConstructor {
        &self.async_generator_function
    }

    /// Returns the `Object` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object-constructor
    #[inline]
    #[must_use]
    pub const fn object(&self) -> &StandardConstructor {
        &self.object
    }

    /// Returns the `Proxy` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-proxy-constructor
    #[inline]
    #[must_use]
    pub const fn proxy(&self) -> &StandardConstructor {
        &self.proxy
    }

    /// Returns the `Date` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date-constructor
    #[inline]
    #[must_use]
    pub const fn date(&self) -> &StandardConstructor {
        &self.date
    }

    /// Returns the `Function` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-constructor
    #[inline]
    #[must_use]
    pub const fn function(&self) -> &StandardConstructor {
        &self.function
    }

    /// Returns the `AsyncFunction` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-async-function-constructor
    #[inline]
    #[must_use]
    pub const fn async_function(&self) -> &StandardConstructor {
        &self.async_function
    }

    /// Returns the `GeneratorFunction` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generatorfunction-constructor
    #[inline]
    #[must_use]
    pub const fn generator_function(&self) -> &StandardConstructor {
        &self.generator_function
    }

    /// Returns the `Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array-constructor
    #[inline]
    #[must_use]
    pub const fn array(&self) -> &StandardConstructor {
        &self.array
    }

    /// Returns the `BigInt` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-bigint-constructor
    #[inline]
    #[must_use]
    pub const fn bigint(&self) -> &StandardConstructor {
        &self.bigint
    }

    /// Returns the `Number` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-number-constructor
    #[inline]
    #[must_use]
    pub const fn number(&self) -> &StandardConstructor {
        &self.number
    }

    /// Returns the `Boolean` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-boolean-constructor
    #[inline]
    #[must_use]
    pub const fn boolean(&self) -> &StandardConstructor {
        &self.boolean
    }

    /// Returns the `String` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string-constructor
    #[inline]
    #[must_use]
    pub const fn string(&self) -> &StandardConstructor {
        &self.string
    }

    /// Returns the `RegExp` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-regexp-constructor
    #[inline]
    #[must_use]
    pub const fn regexp(&self) -> &StandardConstructor {
        &self.regexp
    }

    /// Returns the `Symbol` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-symbol-constructor
    #[inline]
    #[must_use]
    pub const fn symbol(&self) -> &StandardConstructor {
        &self.symbol
    }

    /// Returns the `Error` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-error-constructor
    #[inline]
    #[must_use]
    pub const fn error(&self) -> &StandardConstructor {
        &self.error
    }

    /// Returns the `ReferenceError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-referenceerror
    #[inline]
    #[must_use]
    pub const fn reference_error(&self) -> &StandardConstructor {
        &self.reference_error
    }

    /// Returns the `TypeError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-typeerror
    #[inline]
    #[must_use]
    pub const fn type_error(&self) -> &StandardConstructor {
        &self.type_error
    }

    /// Returns the `RangeError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-rangeerror
    #[inline]
    #[must_use]
    pub const fn range_error(&self) -> &StandardConstructor {
        &self.range_error
    }

    /// Returns the `SyntaxError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-syntaxerror
    #[inline]
    #[must_use]
    pub const fn syntax_error(&self) -> &StandardConstructor {
        &self.syntax_error
    }

    /// Returns the `EvalError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-evalerror
    #[inline]
    #[must_use]
    pub const fn eval_error(&self) -> &StandardConstructor {
        &self.eval_error
    }

    /// Returns the `URIError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-urierror
    #[inline]
    #[must_use]
    pub const fn uri_error(&self) -> &StandardConstructor {
        &self.uri_error
    }

    /// Returns the `AggregateError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-aggregate-error-constructor
    #[inline]
    #[must_use]
    pub const fn aggregate_error(&self) -> &StandardConstructor {
        &self.aggregate_error
    }

    /// Returns the `Map` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map-constructor
    #[inline]
    #[must_use]
    pub const fn map(&self) -> &StandardConstructor {
        &self.map
    }

    /// Returns the `Set` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set-constructor
    #[inline]
    #[must_use]
    pub const fn set(&self) -> &StandardConstructor {
        &self.set
    }

    /// Returns the `TypedArray` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_array(&self) -> &StandardConstructor {
        &self.typed_array
    }

    /// Returns the `Int8Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_int8_array(&self) -> &StandardConstructor {
        &self.typed_int8_array
    }

    /// Returns the `Uint8Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_uint8_array(&self) -> &StandardConstructor {
        &self.typed_uint8_array
    }

    /// Returns the `Uint8ClampedArray` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_uint8clamped_array(&self) -> &StandardConstructor {
        &self.typed_uint8clamped_array
    }

    /// Returns the `Int16Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_int16_array(&self) -> &StandardConstructor {
        &self.typed_int16_array
    }

    /// Returns the `Uint16Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_uint16_array(&self) -> &StandardConstructor {
        &self.typed_uint16_array
    }

    /// Returns the `Uint32Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_uint32_array(&self) -> &StandardConstructor {
        &self.typed_uint32_array
    }

    /// Returns the `Int32Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_int32_array(&self) -> &StandardConstructor {
        &self.typed_int32_array
    }

    /// Returns the `BigInt64Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_bigint64_array(&self) -> &StandardConstructor {
        &self.typed_bigint64_array
    }

    /// Returns the `BigUint64Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_biguint64_array(&self) -> &StandardConstructor {
        &self.typed_biguint64_array
    }

    /// Returns the `Float16Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[cfg(feature = "float16")]
    #[inline]
    #[must_use]
    pub const fn typed_float16_array(&self) -> &StandardConstructor {
        &self.typed_float16_array
    }

    /// Returns the `Float32Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_float32_array(&self) -> &StandardConstructor {
        &self.typed_float32_array
    }

    /// Returns the `Float64Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_float64_array(&self) -> &StandardConstructor {
        &self.typed_float64_array
    }

    /// Returns the `ArrayBuffer` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-arraybuffer-constructor
    #[inline]
    #[must_use]
    pub const fn array_buffer(&self) -> &StandardConstructor {
        &self.array_buffer
    }

    /// Returns the `SharedArrayBuffer` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-sharedarraybuffer-constructor
    #[inline]
    #[must_use]
    pub const fn shared_array_buffer(&self) -> &StandardConstructor {
        &self.shared_array_buffer
    }

    /// Returns the `DataView` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview-constructor
    #[inline]
    #[must_use]
    pub const fn data_view(&self) -> &StandardConstructor {
        &self.data_view
    }

    /// Returns the `Intl.DateTimeFormat` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl-datetimeformat-constructor
    #[inline]
    #[must_use]
    pub const fn date_time_format(&self) -> &StandardConstructor {
        &self.date_time_format
    }

    /// Returns the `Promise` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise-constructor
    #[inline]
    #[must_use]
    pub const fn promise(&self) -> &StandardConstructor {
        &self.promise
    }

    /// Returns the `ReadableStream` constructor.
    ///
    /// More information:
    ///  - [WHATWG Streams reference][spec]
    ///
    /// [spec]: https://streams.spec.whatwg.org/
    #[inline]
    #[must_use]
    pub const fn readable_stream(&self) -> &StandardConstructor {
        &self.readable_stream
    }

    /// Returns the `Console` constructor.
    ///
    /// More information:
    ///  - [Console API reference][spec]
    ///
    /// [spec]: https://console.spec.whatwg.org/
    #[inline]
    #[must_use]
    pub const fn console(&self) -> &StandardConstructor {
        &self.console
    }

    /// Returns the `Blob` constructor.
    ///
    /// More information:
    ///  - [File API reference][spec]
    ///
    /// [spec]: https://w3c.github.io/FileAPI/#blob-section
    #[inline]
    #[must_use]
    pub const fn blob(&self) -> &StandardConstructor {
        &self.blob
    }

    /// Returns the `File` constructor.
    ///
    /// More information:
    ///  - [File API reference][spec]
    ///
    /// [spec]: https://w3c.github.io/FileAPI/#file-section
    #[inline]
    #[must_use]
    pub const fn file(&self) -> &StandardConstructor {
        &self.file
    }

    /// Returns the `FileReader` constructor.
    ///
    /// More information:
    ///  - [File API reference][spec]
    ///
    /// [spec]: https://w3c.github.io/FileAPI/#filereader-section
    #[inline]
    #[must_use]
    pub const fn file_reader(&self) -> &StandardConstructor {
        &self.file_reader
    }

    /// Returns the `EventSource` constructor.
    ///
    /// More information:
    ///  - [EventSource API reference][spec]
    ///
    /// [spec]: https://html.spec.whatwg.org/multipage/server-sent-events.html
    #[inline]
    #[must_use]
    pub const fn event_source(&self) -> &StandardConstructor {
        &self.event_source
    }

    /// Returns the `RTCPeerConnection` constructor.
    ///
    /// More information:
    ///  - [RTCPeerConnection API reference][spec]
    ///
    /// [spec]: https://w3c.github.io/webrtc-pc/
    #[inline]
    #[must_use]
    pub const fn rtc_peer_connection(&self) -> &StandardConstructor {
        &self.rtc_peer_connection
    }

    /// Returns the `RTCDataChannel` constructor.
    ///
    /// More information:
    ///  - [RTCDataChannel API reference][spec]
    ///
    /// [spec]: https://w3c.github.io/webrtc-pc/#rtcdatachannel
    #[inline]
    #[must_use]
    pub const fn rtc_data_channel(&self) -> &StandardConstructor {
        &self.rtc_data_channel
    }

    /// Returns the `RTCIceCandidate` constructor.
    ///
    /// More information:
    ///  - [RTCIceCandidate API reference][spec]
    ///
    /// [spec]: https://w3c.github.io/webrtc-pc/#rtcicecandidate-interface
    #[inline]
    #[must_use]
    pub const fn rtc_ice_candidate(&self) -> &StandardConstructor {
        &self.rtc_ice_candidate
    }

    /// Returns the `RTCSessionDescription` constructor.
    ///
    /// More information:
    ///  - [RTCSessionDescription API reference][spec]
    ///
    /// [spec]: https://w3c.github.io/webrtc-pc/#rtcsessiondescription-class
    #[inline]
    #[must_use]
    pub const fn rtc_session_description(&self) -> &StandardConstructor {
        &self.rtc_session_description
    }

    /// Returns the `WebSocket` constructor.
    ///
    /// More information:
    ///  - [WebSocket API reference][spec]
    ///
    /// [spec]: https://websockets.spec.whatwg.org/
    #[inline]
    #[must_use]
    pub const fn websocket(&self) -> &StandardConstructor {
        &self.websocket
    }

    /// Returns the `WebSocketStream` constructor.
    ///
    /// More information:
    ///  - [WebSocketStream API reference][spec]
    ///
    /// [spec]: https://websockets.spec.whatwg.org/#websocketstream
    #[inline]
    #[must_use]
    pub const fn websocket_stream(&self) -> &StandardConstructor {
        &self.websocket_stream
    }

    /// Returns the `WebAssembly.Module` constructor.
    ///
    /// More information:
    ///  - [WebAssembly.Module spec][spec]
    ///
    /// [spec]: https://webassembly.github.io/spec/js-api/#modules
    #[inline]
    #[must_use]
    pub const fn webassembly_module(&self) -> &StandardConstructor {
        &self.webassembly_module
    }

    /// Returns the `WebAssembly.Instance` constructor.
    ///
    /// More information:
    ///  - [WebAssembly.Instance spec][spec]
    ///
    /// [spec]: https://webassembly.github.io/spec/js-api/#instances
    #[inline]
    #[must_use]
    pub const fn webassembly_instance(&self) -> &StandardConstructor {
        &self.webassembly_instance
    }

    /// Returns the `WebAssembly.Memory` constructor.
    ///
    /// More information:
    ///  - [WebAssembly.Memory spec][spec]
    ///
    /// [spec]: https://webassembly.github.io/spec/js-api/#memories
    #[inline]
    #[must_use]
    pub const fn webassembly_memory(&self) -> &StandardConstructor {
        &self.webassembly_memory
    }

    /// Returns the `WebAssembly.Table` constructor.
    ///
    /// More information:
    ///  - [WebAssembly.Table spec][spec]
    ///
    /// [spec]: https://webassembly.github.io/spec/js-api/#tables
    #[inline]
    #[must_use]
    pub const fn webassembly_table(&self) -> &StandardConstructor {
        &self.webassembly_table
    }

    /// Returns the `WebAssembly.Global` constructor.
    ///
    /// More information:
    ///  - [WebAssembly.Global spec][spec]
    ///
    /// [spec]: https://webassembly.github.io/spec/js-api/#globals
    #[inline]
    #[must_use]
    pub const fn webassembly_global(&self) -> &StandardConstructor {
        &self.webassembly_global
    }

    /// Returns the `Worker` constructor.
    ///
    /// More information:
    ///  - [Worker spec][spec]
    ///
    /// [spec]: https://html.spec.whatwg.org/multipage/workers.html
    #[inline]
    #[must_use]
    pub const fn worker(&self) -> &StandardConstructor {
        &self.worker
    }

    /// Returns the `SharedWorker` constructor.
    ///
    /// More information:
    ///  - [SharedWorker spec][spec]
    ///
    /// [spec]: https://html.spec.whatwg.org/multipage/workers.html#shared-workers
    #[inline]
    #[must_use]
    pub const fn shared_worker(&self) -> &StandardConstructor {
        &self.shared_worker
    }

    /// Returns the `ServiceWorker` constructor.
    ///
    /// More information:
    ///  - [ServiceWorker spec][spec]
    ///
    /// [spec]: https://w3c.github.io/ServiceWorker/
    #[inline]
    #[must_use]
    pub const fn service_worker(&self) -> &StandardConstructor {
        &self.service_worker
    }

    /// Returns the `ServiceWorkerContainer` constructor.
    ///
    /// More information:
    ///  - [ServiceWorker spec][spec]
    ///
    /// [spec]: https://w3c.github.io/ServiceWorker/#serviceworkercontainer-interface
    #[inline]
    #[must_use]
    pub const fn service_worker_container(&self) -> &StandardConstructor {
        &self.service_worker_container
    }

    /// Returns the `Worklet` constructor.
    ///
    /// More information:
    ///  - [Worklet spec][spec]
    ///
    /// [spec]: https://html.spec.whatwg.org/multipage/worklets.html
    #[inline]
    #[must_use]
    pub const fn worklet(&self) -> &StandardConstructor {
        &self.worklet
    }

    /// Returns the `MessageChannel` constructor.
    ///
    /// More information:
    ///  - [MessageChannel spec][spec]
    ///
    /// [spec]: https://html.spec.whatwg.org/multipage/web-messaging.html#message-channels
    #[inline]
    #[must_use]
    pub const fn message_channel(&self) -> &StandardConstructor {
        &self.message_channel
    }

    /// Returns the `MessagePort` constructor.
    ///
    /// More information:
    ///  - [MessagePort spec][spec]
    ///
    /// [spec]: https://html.spec.whatwg.org/multipage/web-messaging.html#message-ports
    #[inline]
    #[must_use]
    pub const fn message_port(&self) -> &StandardConstructor {
        &self.message_port
    }

    /// Returns the `BroadcastChannel` constructor.
    ///
    /// More information:
    ///  - [BroadcastChannel spec][spec]
    ///
    /// [spec]: https://html.spec.whatwg.org/multipage/web-messaging.html#broadcasting-to-other-browsing-contexts
    #[inline]
    #[must_use]
    pub const fn broadcast_channel(&self) -> &StandardConstructor {
        &self.broadcast_channel
    }

    /// Returns the `Crypto` constructor.
    ///
    /// More information:
    ///  - [Web Crypto API spec][spec]
    ///
    /// [spec]: https://w3c.github.io/webcrypto/
    #[inline]
    #[must_use]
    pub const fn crypto(&self) -> &StandardConstructor {
        &self.crypto
    }

    /// Returns the `Request` constructor.
    ///
    /// More information:
    ///  - [Fetch API reference][spec]
    ///
    /// [spec]: https://fetch.spec.whatwg.org/#request-class
    #[inline]
    #[must_use]
    pub const fn request(&self) -> &StandardConstructor {
        &self.request
    }

    /// Returns the `Response` constructor.
    ///
    /// More information:
    ///  - [Fetch API reference][spec]
    ///
    /// [spec]: https://fetch.spec.whatwg.org/#response-class
    #[inline]
    #[must_use]
    pub const fn response(&self) -> &StandardConstructor {
        &self.response
    }

    /// Returns the `Headers` constructor.
    ///
    /// More information:
    ///  - [Fetch API reference][spec]
    ///
    /// [spec]: https://fetch.spec.whatwg.org/#headers-class
    #[inline]
    #[must_use]
    pub const fn headers(&self) -> &StandardConstructor {
        &self.headers
    }

    /// Returns the `AbortController` constructor.
    ///
    /// More information:
    ///  - [AbortController API reference][spec]
    ///
    /// [spec]: https://dom.spec.whatwg.org/#interface-abortcontroller
    #[inline]
    #[must_use]
    pub const fn abort_controller(&self) -> &StandardConstructor {
        &self.abort_controller
    }

    /// Returns the `XMLHttpRequest` constructor.
    ///
    /// More information:
    ///  - [XMLHttpRequest API reference][spec]
    ///
    /// [spec]: https://xhr.spec.whatwg.org/
    #[inline]
    #[must_use]
    pub const fn xmlhttprequest(&self) -> &StandardConstructor {
        &self.xmlhttprequest
    }

    /// Returns the `MutationObserver` constructor.
    ///
    /// More information:
    ///  - [MutationObserver API reference][spec]
    ///
    /// [spec]: https://dom.spec.whatwg.org/#interface-mutationobserver
    #[inline]
    #[must_use]
    pub const fn mutation_observer(&self) -> &StandardConstructor {
        &self.mutation_observer
    }

    /// Returns the `IntersectionObserver` constructor.
    ///
    /// More information:
    ///  - [IntersectionObserver API reference][spec]
    ///
    /// [spec]: https://w3c.github.io/IntersectionObserver/
    #[inline]
    #[must_use]
    pub const fn intersection_observer(&self) -> &StandardConstructor {
        &self.intersection_observer
    }

    /// Returns the `ResizeObserver` constructor.
    ///
    /// More information:
    ///  - [ResizeObserver API reference][spec]
    ///
    /// [spec]: https://wicg.github.io/ResizeObserver/
    #[inline]
    #[must_use]
    pub const fn resize_observer(&self) -> &StandardConstructor {
        &self.resize_observer
    }

    /// Returns the `Document` constructor.
    ///
    /// More information:
    ///  - [Document API reference][spec]
    ///
    /// [spec]: https://dom.spec.whatwg.org/#interface-document
    #[inline]
    #[must_use]
    pub const fn document(&self) -> &StandardConstructor {
        &self.document
    }

    /// Returns the `Window` constructor.
    ///
    /// More information:
    ///  - [Window API reference][spec]
    ///
    /// [spec]: https://html.spec.whatwg.org/#the-window-object
    #[inline]
    #[must_use]
    pub const fn window(&self) -> &StandardConstructor {
        &self.window
    }

    /// Returns the `History` constructor.
    ///
    /// More information:
    ///  - [History API reference][spec]
    ///
    /// [spec]: https://html.spec.whatwg.org/#the-history-interface
    #[inline]
    #[must_use]
    pub const fn history(&self) -> &StandardConstructor {
        &self.history
    }

    /// Returns the `PageSwapEvent` constructor.
    ///
    /// More information:
    ///  - [PageSwapEvent API reference][spec]
    ///
    /// [spec]: https://wicg.github.io/navigation-api/#pageswapevent
    #[inline]
    #[must_use]
    pub const fn pageswap_event(&self) -> &StandardConstructor {
        &self.pageswap_event
    }

    /// Returns the `Node` constructor.
    ///
    /// More information:
    ///  - [Node API reference][spec]
    ///
    /// [spec]: https://dom.spec.whatwg.org/#interface-node
    #[inline]
    #[must_use]
    pub const fn node(&self) -> &StandardConstructor {
        &self.node
    }

    /// Returns the `CharacterData` constructor.
    ///
    /// More information:
    ///  - [CharacterData API reference][spec]
    ///
    /// [spec]: https://dom.spec.whatwg.org/#interface-characterdata
    pub const fn character_data(&self) -> &StandardConstructor {
        &self.character_data
    }

    /// Returns the `Text` constructor.
    ///
    /// More information:
    ///  - [Text API reference][spec]
    ///
    /// [spec]: https://dom.spec.whatwg.org/#interface-text
    pub const fn text(&self) -> &StandardConstructor {
        &self.text
    }

    /// Returns the `DocumentFragment` constructor.
    ///
    /// More information:
    ///  - [DocumentFragment API reference][spec]
    ///
    /// [spec]: https://dom.spec.whatwg.org/#interface-documentfragment
    #[inline]
    #[must_use]
    pub const fn document_fragment(&self) -> &StandardConstructor {
        &self.document_fragment
    }

    /// Returns the `ShadowRoot` constructor.
    ///
    /// More information:
    ///  - [ShadowRoot API reference][spec]
    ///
    /// [spec]: https://dom.spec.whatwg.org/#interface-shadowroot
    #[inline]
    #[must_use]
    pub const fn shadow_root(&self) -> &StandardConstructor {
        &self.shadow_root
    }

    /// Returns the `HTMLSlotElement` constructor.
    ///
    /// More information:
    ///  - [HTMLSlotElement API reference][spec]
    ///
    /// [spec]: https://dom.spec.whatwg.org/#interface-htmlslotelement
    #[inline]
    #[must_use]
    pub const fn html_slot_element(&self) -> &StandardConstructor {
        &self.html_slot_element
    }

    /// Returns the `NodeList` constructor.
    ///
    /// More information:
    ///  - [NodeList API reference][spec]
    ///
    /// [spec]: https://dom.spec.whatwg.org/#interface-nodelist
    #[inline]
    #[must_use]
    pub const fn nodelist(&self) -> &StandardConstructor {
        &self.nodelist
    }

    /// Returns the `DOMTokenList` constructor.
    #[inline]
    #[must_use]
    pub const fn domtokenlist(&self) -> &StandardConstructor {
        &self.domtokenlist
    }

    /// Returns the `Element` constructor.
    ///
    /// More information:
    ///  - [Element API reference][spec]
    ///
    /// [spec]: https://dom.spec.whatwg.org/#interface-element
    #[inline]
    #[must_use]
    pub const fn element(&self) -> &StandardConstructor {
        &self.element
    }

    /// Returns the `Attr` constructor.
    #[inline]
    #[must_use]
    pub const fn attr(&self) -> &StandardConstructor {
        &self.attr
    }

    /// Returns the `Comment` constructor.
    #[inline]
    #[must_use]
    pub const fn comment(&self) -> &StandardConstructor {
        &self.comment
    }

    /// Returns the `CustomEvent` constructor.
    #[inline]
    #[must_use]
    pub const fn custom_event(&self) -> &StandardConstructor {
        &self.custom_event
    }

    /// Returns the `MessageEvent` constructor.
    #[inline]
    #[must_use]
    pub const fn message_event(&self) -> &StandardConstructor {
        &self.message_event
    }

    /// Returns the `ProcessingInstruction` constructor.
    #[inline]
    #[must_use]
    pub const fn processing_instruction(&self) -> &StandardConstructor {
        &self.processing_instruction
    }

    /// Returns the `CDATASection` constructor.
    #[inline]
    #[must_use]
    pub const fn cdata_section(&self) -> &StandardConstructor {
        &self.cdata_section
    }

    /// Returns the `HTMLFormElement` constructor.
    #[inline]
    #[must_use]
    pub const fn html_form_element(&self) -> &StandardConstructor {
        &self.html_form_element
    }

    /// Returns the `HTMLFormControlsCollection` constructor.
    #[inline]
    #[must_use]
    pub const fn html_form_controls_collection(&self) -> &StandardConstructor {
        &self.html_form_controls_collection
    }

    /// Returns the `HTMLInputElement` constructor.
    #[inline]
    #[must_use]
    pub const fn html_input_element(&self) -> &StandardConstructor {
        &self.html_input_element
    }

    pub const fn selection(&self) -> &StandardConstructor {
        &self.selection
    }
    pub const fn range(&self) -> &StandardConstructor {
        &self.range
    }
    pub const fn event(&self) -> &StandardConstructor {
        &self.event
    }

    /// Returns the `EventTarget` constructor.
    ///
    /// More information:
    ///  - [EventTarget API reference][spec]
    ///
    /// [spec]: https://dom.spec.whatwg.org/#interface-eventtarget
    #[inline]
    #[must_use]
    pub const fn event_target(&self) -> &StandardConstructor {
        &self.event_target
    }

    /// Returns the `WeakRef` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weak-ref-constructor
    #[inline]
    #[must_use]
    pub const fn weak_ref(&self) -> &StandardConstructor {
        &self.weak_ref
    }

    /// Returns the `WeakMap` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weakmap-constructor
    #[inline]
    #[must_use]
    pub const fn weak_map(&self) -> &StandardConstructor {
        &self.weak_map
    }

    /// Returns the `WeakSet` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weakset-constructor
    #[inline]
    #[must_use]
    pub const fn weak_set(&self) -> &StandardConstructor {
        &self.weak_set
    }

    /// Returns the `Storage` constructor.
    ///
    /// More information:
    ///  - [WHATWG Specification](https://html.spec.whatwg.org/multipage/webstorage.html#the-storage-interface)
    #[inline]
    #[must_use]
    pub const fn storage(&self) -> &StandardConstructor {
        &self.storage
    }

    /// Returns the `StorageEvent` constructor.
    ///
    /// More information:
    ///  - [WHATWG Specification](https://html.spec.whatwg.org/multipage/webstorage.html#the-storageevent-interface)
    #[inline]
    #[must_use]
    pub const fn storage_event(&self) -> &StandardConstructor {
        &self.storage_event
    }

    /// Returns the `StorageManager` constructor.
    ///
    /// More information:
    ///  - [WHATWG Storage Standard](https://storage.spec.whatwg.org/)
    #[inline]
    #[must_use]
    pub const fn storage_manager(&self) -> &StandardConstructor {
        &self.storage_manager
    }

    /// Returns the `Cache` constructor.
    ///
    /// More information:
    ///  - [WHATWG Service Worker Specification](https://w3c.github.io/ServiceWorker/#cache-interface)
    #[inline]
    #[must_use]
    pub const fn cache(&self) -> &StandardConstructor {
        &self.cache
    }

    /// Returns the `CacheStorage` constructor.
    ///
    /// More information:
    ///  - [WHATWG Service Worker Specification](https://w3c.github.io/ServiceWorker/#cachestorage-interface)
    #[inline]
    #[must_use]
    pub const fn cache_storage(&self) -> &StandardConstructor {
        &self.cache_storage
    }

    /// Returns the `CookieStore` constructor.
    ///
    /// More information:
    ///  - [WHATWG Cookie Store API Specification](https://wicg.github.io/cookie-store/)
    #[inline]
    #[must_use]
    pub const fn cookie_store(&self) -> &StandardConstructor {
        &self.cookie_store
    }

    /// Returns the `FileSystemHandle` constructor.
    ///
    /// More information:
    ///  - [WHATWG File System API Specification](https://fs.spec.whatwg.org/)
    #[inline]
    #[must_use]
    pub const fn file_system_handle(&self) -> &StandardConstructor {
        &self.file_system_handle
    }

    /// Returns the `FileSystemFileHandle` constructor.
    ///
    /// More information:
    ///  - [WHATWG File System API Specification](https://fs.spec.whatwg.org/)
    #[inline]
    #[must_use]
    pub const fn file_system_file_handle(&self) -> &StandardConstructor {
        &self.file_system_file_handle
    }

    /// Returns the `FileSystemDirectoryHandle` constructor.
    ///
    /// More information:
    ///  - [WHATWG File System API Specification](https://fs.spec.whatwg.org/)
    #[inline]
    #[must_use]
    pub const fn file_system_directory_handle(&self) -> &StandardConstructor {
        &self.file_system_directory_handle
    }

    /// Returns the `LockManager` constructor.
    ///
    /// More information:
    ///  - [W3C Web Locks API Specification](https://w3c.github.io/web-locks/)
    #[inline]
    #[must_use]
    pub const fn lock_manager(&self) -> &StandardConstructor {
        &self.lock_manager
    }

    /// Returns the `IDBFactory` constructor.
    ///
    /// More information:
    ///  - [W3C IndexedDB 3.0 Specification](https://w3c.github.io/IndexedDB/)
    #[inline]
    #[must_use]
    pub const fn idb_factory(&self) -> &StandardConstructor {
        &self.idb_factory
    }

    /// Returns the `Navigator` constructor.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Navigator
    #[inline]
    #[must_use]
    pub const fn navigator(&self) -> &StandardConstructor {
        &self.navigator
    }

    /// Sets the `Navigator` constructor.
    ///
    /// This is needed for external crates to register Navigator after extraction from Boa.
    #[inline]
    pub fn set_navigator(&mut self, constructor: StandardConstructor) {
        self.navigator = constructor;
    }

    /// Returns the `Performance` constructor.
    ///
    /// More information:
    ///  - [W3C High Resolution Time Specification][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://w3c.github.io/hr-time/
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Performance
    #[inline]
    #[must_use]
    pub const fn performance(&self) -> &StandardConstructor {
        &self.performance
    }

    /// Returns the `Intl.Collator` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.collator
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn collator(&self) -> &StandardConstructor {
        &self.collator
    }

    /// Returns the `Intl.ListFormat` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-Intl.ListFormat
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn list_format(&self) -> &StandardConstructor {
        &self.list_format
    }

    /// Returns the `Intl.Locale` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-Intl.Locale
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn locale(&self) -> &StandardConstructor {
        &self.locale
    }

    /// Returns the `Intl.Segmenter` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.segmenter
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn segmenter(&self) -> &StandardConstructor {
        &self.segmenter
    }

    /// Returns the `Intl.PluralRules` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.pluralrules
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn plural_rules(&self) -> &StandardConstructor {
        &self.plural_rules
    }

    /// Returns the `Intl.NumberFormat` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.numberformat
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn number_format(&self) -> &StandardConstructor {
        &self.number_format
    }

    /// Returns the `Temporal.Instant` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-instant-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn instant(&self) -> &StandardConstructor {
        &self.instant
    }

    /// Returns the `Temporal.PlainDateTime` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-plaindatetime-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn plain_date_time(&self) -> &StandardConstructor {
        &self.plain_date_time
    }

    /// Returns the `Temporal.PlainDate` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-plaindate-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn plain_date(&self) -> &StandardConstructor {
        &self.plain_date
    }

    /// Returns the `Temporal.PlainTime` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-plaintime-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn plain_time(&self) -> &StandardConstructor {
        &self.plain_time
    }

    /// Returns the `Temporal.PlainYearMonth` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-plainyearmonth-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn plain_year_month(&self) -> &StandardConstructor {
        &self.plain_year_month
    }

    /// Returns the `Temporal.PlainMonthDay` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-plainmonthday-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn plain_month_day(&self) -> &StandardConstructor {
        &self.plain_month_day
    }

    /// Returns the `Temporal.TimeZone` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-timezone-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn time_zone(&self) -> &StandardConstructor {
        &self.time_zone
    }

    /// Returns the `Temporal.Duration` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-duration-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn duration(&self) -> &StandardConstructor {
        &self.duration
    }

    /// Returns the `Temporal.ZonedDateTime` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-zoneddatetime-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn zoned_date_time(&self) -> &StandardConstructor {
        &self.zoned_date_time
    }

    /// Returns the `Temporal.Calendar` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-calendar-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn calendar(&self) -> &StandardConstructor {
        &self.calendar
    }


    /// Returns the `WritableStream` constructor.
    ///
    /// More information:
    ///  - [WHATWG Streams spec][spec]
    ///
    /// [spec]: https://streams.spec.whatwg.org/#writablestream
    #[inline]
    #[must_use]
    pub const fn writable_stream(&self) -> &StandardConstructor {
        &self.writable_stream
    }

    /// Returns the `TransformStream` constructor.
    ///
    /// More information:
    ///  - [WHATWG Streams spec][spec]
    ///
    /// [spec]: https://streams.spec.whatwg.org/#transformstream
    #[inline]
    #[must_use]
    pub const fn transform_stream(&self) -> &StandardConstructor {
        &self.transform_stream
    }

    /// Returns the `CountQueuingStrategy` constructor.
    ///
    /// More information:
    ///  - [WHATWG Streams spec][spec]
    ///
    /// [spec]: https://streams.spec.whatwg.org/#countqueuingstrategy
    #[inline]
    #[must_use]
    pub const fn count_queuing_strategy(&self) -> &StandardConstructor {
        &self.count_queuing_strategy
    }

    /// Returns the `ByteLengthQueuingStrategy` constructor.
    ///
    /// More information:
    ///  - [WHATWG Streams spec][spec]
    ///
    /// [spec]: https://streams.spec.whatwg.org/#bytelengthqueuingstrategy
    #[inline]
    #[must_use]
    pub const fn byte_length_queuing_strategy(&self) -> &StandardConstructor {
        &self.byte_length_queuing_strategy
    }
}

/// Cached intrinsic objects
#[derive(Debug, Trace, Finalize)]
pub struct IntrinsicObjects {
    /// [`%Reflect%`](https://tc39.es/ecma262/#sec-reflect)
    reflect: JsObject,

    /// [`%Math%`](https://tc39.es/ecma262/#sec-math)
    math: JsObject,

    /// [`%JSON%`](https://tc39.es/ecma262/#sec-json)
    json: JsObject,

    /// [`%ThrowTypeError%`](https://tc39.es/ecma262/#sec-%throwtypeerror%)
    throw_type_error: JsFunction,

    /// [`%Array.prototype.values%`](https://tc39.es/ecma262/#sec-array.prototype.values)
    array_prototype_values: JsFunction,

    /// [`%Array.prototype.toString%`](https://tc39.es/ecma262/#sec-array.prototype.tostring)
    array_prototype_to_string: JsFunction,

    /// Cached iterator prototypes.
    iterator_prototypes: IteratorPrototypes,

    /// [`%GeneratorFunction.prototype.prototype%`](https://tc39.es/ecma262/#sec-properties-of-generator-prototype)
    generator: JsObject,

    /// [`%AsyncGeneratorFunction.prototype.prototype%`](https://tc39.es/ecma262/#sec-properties-of-asyncgenerator-prototype)
    async_generator: JsObject,

    /// [`%Atomics%`](https://tc39.es/ecma262/#sec-atomics)
    atomics: JsObject,

    /// [`%eval%`](https://tc39.es/ecma262/#sec-eval-x)
    eval: JsFunction,

    /// [`%fetch%`](https://fetch.spec.whatwg.org/)
    fetch: JsFunction,

    /// URI related functions
    uri_functions: UriFunctions,

    /// [`%isFinite%`](https://tc39.es/ecma262/#sec-isfinite-number)
    is_finite: JsFunction,

    /// [`%isNaN%`](https://tc39.es/ecma262/#sec-isnan-number)
    is_nan: JsFunction,

    /// [`%parseFloat%`](https://tc39.es/ecma262/#sec-parsefloat-string)
    parse_float: JsFunction,

    /// [`%parseInt%`](https://tc39.es/ecma262/#sec-parseint-string-radix)
    parse_int: JsFunction,

    /// [`%setTimeout%`](https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#dom-settimeout)
    set_timeout: JsFunction,

    /// [`%setInterval%`](https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#dom-setinterval)
    set_interval: JsFunction,

    /// [`%clearTimeout%`](https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#dom-cleartimeout)
    clear_timeout: JsFunction,

    /// [`%clearInterval%`](https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#dom-clearinterval)
    clear_interval: JsFunction,

    /// [`%escape%`](https://tc39.es/ecma262/#sec-escape-string)
    #[cfg(feature = "annex-b")]
    escape: JsFunction,

    /// [`%unescape%`](https://tc39.es/ecma262/#sec-unescape-string)
    #[cfg(feature = "annex-b")]
    unescape: JsFunction,

    /// [`%Intl%`](https://tc39.es/ecma402/#intl-object)
    #[cfg(feature = "intl")]
    intl: JsObject<Intl>,

    /// [`%SegmentsPrototype%`](https://tc39.es/ecma402/#sec-%segmentsprototype%-object)
    #[cfg(feature = "intl")]
    segments_prototype: JsObject,

    /// [`%Temporal%`](https://tc39.es/proposal-temporal/#sec-temporal-objects)
    #[cfg(feature = "temporal")]
    temporal: JsObject,

    /// [`%Temporal.Now%`](https://tc39.es/proposal-temporal/#sec-temporal-now-object)
    #[cfg(feature = "temporal")]
    now: JsObject,

    /// [`%CSS%`](https://drafts.csswg.org/css-typed-om-1/#css-namespace)
    css: JsObject,
}

impl IntrinsicObjects {
    /// Creates a new set of uninitialized intrinsic objects.
    ///
    /// Creates all the required empty objects for every intrinsic object in this realm.
    ///
    /// To initialize all the intrinsic objects with their spec properties, see [`Realm::initialize`].
    ///
    /// [`Realm::initialize`]: crate::realm::Realm::initialize
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn uninit() -> Option<Self> {
        Some(Self {
            reflect: JsObject::default(),
            math: JsObject::default(),
            json: JsObject::default(),
            throw_type_error: JsFunction::empty_intrinsic_function(false),
            array_prototype_values: JsFunction::empty_intrinsic_function(false),
            array_prototype_to_string: JsFunction::empty_intrinsic_function(false),
            iterator_prototypes: IteratorPrototypes::default(),
            generator: JsObject::default(),
            async_generator: JsObject::default(),
            atomics: JsObject::default(),
            eval: JsFunction::empty_intrinsic_function(false),
            fetch: JsFunction::empty_intrinsic_function(false),
            uri_functions: UriFunctions::default(),
            is_finite: JsFunction::empty_intrinsic_function(false),
            is_nan: JsFunction::empty_intrinsic_function(false),
            parse_float: JsFunction::empty_intrinsic_function(false),
            parse_int: JsFunction::empty_intrinsic_function(false),
            set_timeout: JsFunction::empty_intrinsic_function(false),
            set_interval: JsFunction::empty_intrinsic_function(false),
            clear_timeout: JsFunction::empty_intrinsic_function(false),
            clear_interval: JsFunction::empty_intrinsic_function(false),
            #[cfg(feature = "annex-b")]
            escape: JsFunction::empty_intrinsic_function(false),
            #[cfg(feature = "annex-b")]
            unescape: JsFunction::empty_intrinsic_function(false),
            #[cfg(feature = "intl")]
            intl: JsObject::new_unique(None, Intl::new()?),
            #[cfg(feature = "intl")]
            segments_prototype: JsObject::default(),
            #[cfg(feature = "temporal")]
            temporal: JsObject::default(),
            #[cfg(feature = "temporal")]
            now: JsObject::default(),
            css: JsObject::default(),
        })
    }

    /// Gets the [`%ThrowTypeError%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%throwtypeerror%
    #[inline]
    #[must_use]
    pub fn throw_type_error(&self) -> JsFunction {
        self.throw_type_error.clone()
    }

    /// Gets the [`%Array.prototype.values%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array.prototype.values
    #[inline]
    #[must_use]
    pub fn array_prototype_values(&self) -> JsFunction {
        self.array_prototype_values.clone()
    }

    /// Gets the [`%Array.prototype.toString%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array.prototype.tostring
    #[inline]
    #[must_use]
    pub fn array_prototype_to_string(&self) -> JsFunction {
        self.array_prototype_to_string.clone()
    }

    /// Gets the cached iterator prototypes.
    #[inline]
    #[must_use]
    pub const fn iterator_prototypes(&self) -> &IteratorPrototypes {
        &self.iterator_prototypes
    }

    /// Gets the [`%GeneratorFunction.prototype.prototype%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generator-objects
    #[inline]
    #[must_use]
    pub fn generator(&self) -> JsObject {
        self.generator.clone()
    }

    /// Gets the [`%AsyncGeneratorFunction.prototype.prototype%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgenerator-objects
    #[inline]
    #[must_use]
    pub fn async_generator(&self) -> JsObject {
        self.async_generator.clone()
    }

    /// Gets the [`%Atomics%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-atomics
    #[inline]
    #[must_use]
    pub fn atomics(&self) -> JsObject {
        self.atomics.clone()
    }

    /// Gets the [`%eval%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-eval-x
    #[inline]
    #[must_use]
    pub fn eval(&self) -> JsFunction {
        self.eval.clone()
    }

    /// Gets the [`%fetch%`][spec] intrinsic function.
    #[inline]
    #[must_use]
    pub fn fetch(&self) -> JsFunction {
        self.fetch.clone()
    }

    /// Gets the URI intrinsic functions.
    #[inline]
    #[must_use]
    pub const fn uri_functions(&self) -> &UriFunctions {
        &self.uri_functions
    }

    /// Gets the [`%Reflect%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-reflect
    #[inline]
    #[must_use]
    pub fn reflect(&self) -> JsObject {
        self.reflect.clone()
    }

    /// Gets the [`%Math%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math
    #[inline]
    #[must_use]
    pub fn math(&self) -> JsObject {
        self.math.clone()
    }

    /// Gets the [`%JSON%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-json
    #[inline]
    #[must_use]
    pub fn json(&self) -> JsObject {
        self.json.clone()
    }

    /// Gets the [`%CSS%`][spec] intrinsic object.
    ///
    /// [spec]: https://drafts.csswg.org/css-typed-om-1/#css-namespace
    #[inline]
    #[must_use]
    pub fn css(&self) -> JsObject {
        self.css.clone()
    }

    /// Gets the [`%isFinite%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isfinite-number
    #[inline]
    #[must_use]
    pub fn is_finite(&self) -> JsFunction {
        self.is_finite.clone()
    }

    /// Gets the [`%isNaN%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isnan-number
    #[inline]
    #[must_use]
    pub fn is_nan(&self) -> JsFunction {
        self.is_nan.clone()
    }

    /// Gets the [`%parseFloat%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-parsefloat-string
    #[inline]
    #[must_use]
    pub fn parse_float(&self) -> JsFunction {
        self.parse_float.clone()
    }

    /// Gets the [`%parseInt%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-parseint-string-radix
    #[inline]
    #[must_use]
    pub fn parse_int(&self) -> JsFunction {
        self.parse_int.clone()
    }

    /// Gets the [`%setTimeout%`][spec] intrinsic function.
    ///
    /// [spec]: https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#dom-settimeout
    #[inline]
    #[must_use]
    pub fn set_timeout(&self) -> JsFunction {
        self.set_timeout.clone()
    }

    /// Gets the [`%setInterval%`][spec] intrinsic function.
    ///
    /// [spec]: https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#dom-setinterval
    #[inline]
    #[must_use]
    pub fn set_interval(&self) -> JsFunction {
        self.set_interval.clone()
    }

    /// Gets the [`%clearTimeout%`][spec] intrinsic function.
    ///
    /// [spec]: https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#dom-cleartimeout
    #[inline]
    #[must_use]
    pub fn clear_timeout(&self) -> JsFunction {
        self.clear_timeout.clone()
    }

    /// Gets the [`%clearInterval%`][spec] intrinsic function.
    ///
    /// [spec]: https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#dom-clearinterval
    #[inline]
    #[must_use]
    pub fn clear_interval(&self) -> JsFunction {
        self.clear_interval.clone()
    }

    /// Gets the [`%escape%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-escape-string
    #[must_use]
    #[cfg(feature = "annex-b")]
    #[inline]
    pub fn escape(&self) -> JsFunction {
        self.escape.clone()
    }

    /// Gets the [`%unescape%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-unescape-string
    #[must_use]
    #[cfg(feature = "annex-b")]
    #[inline]
    pub fn unescape(&self) -> JsFunction {
        self.unescape.clone()
    }

    /// Gets the [`%Intl%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma402/#intl-object
    #[must_use]
    #[cfg(feature = "intl")]
    #[inline]
    pub fn intl(&self) -> JsObject<Intl> {
        self.intl.clone()
    }

    /// Gets the [`%SegmentsPrototype%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-%segmentsprototype%-object
    #[must_use]
    #[cfg(feature = "intl")]
    pub fn segments_prototype(&self) -> JsObject {
        self.segments_prototype.clone()
    }

    /// Gets the [`%Temporal%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-objects
    #[cfg(feature = "temporal")]
    #[must_use]
    #[inline]
    pub fn temporal(&self) -> JsObject {
        self.temporal.clone()
    }

    /// Gets the [`%Temporal.Now%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-now-object
    #[cfg(feature = "temporal")]
    #[must_use]
    #[inline]
    pub fn now(&self) -> JsObject {
        self.now.clone()
    }
}

/// Contains commonly used [`ObjectTemplate`]s.
#[derive(Debug, Trace, Finalize)]
pub(crate) struct ObjectTemplates {
    iterator_result: ObjectTemplate,
    ordinary_object: ObjectTemplate,
    array: ObjectTemplate,
    number: ObjectTemplate,
    string: ObjectTemplate,
    symbol: ObjectTemplate,
    bigint: ObjectTemplate,
    boolean: ObjectTemplate,

    regexp: ObjectTemplate,
    regexp_without_proto: ObjectTemplate,

    unmapped_arguments: ObjectTemplate,
    mapped_arguments: ObjectTemplate,

    function_with_prototype: ObjectTemplate,
    function_prototype: ObjectTemplate,

    function: ObjectTemplate,
    async_function: ObjectTemplate,
    generator_function: ObjectTemplate,
    async_generator_function: ObjectTemplate,

    function_without_proto: ObjectTemplate,
    function_with_prototype_without_proto: ObjectTemplate,

    namespace: ObjectTemplate,

    with_resolvers: ObjectTemplate,

    wait_async: ObjectTemplate,
}

impl ObjectTemplates {
    pub(crate) fn new(root_shape: &RootShape, constructors: &StandardConstructors) -> Self {
        let root_shape = root_shape.shape();

        // pre-initialize used shapes.
        let ordinary_object =
            ObjectTemplate::with_prototype(root_shape, constructors.object().prototype());
        let mut array = ObjectTemplate::new(root_shape);
        let length_property_key: PropertyKey = js_string!("length").into();
        array.property(
            length_property_key.clone(),
            Attribute::WRITABLE | Attribute::PERMANENT | Attribute::NON_ENUMERABLE,
        );
        array.set_prototype(constructors.array().prototype());

        let number = ObjectTemplate::with_prototype(root_shape, constructors.number().prototype());
        let symbol = ObjectTemplate::with_prototype(root_shape, constructors.symbol().prototype());
        let bigint = ObjectTemplate::with_prototype(root_shape, constructors.bigint().prototype());
        let boolean =
            ObjectTemplate::with_prototype(root_shape, constructors.boolean().prototype());
        let mut string = ObjectTemplate::new(root_shape);
        string.property(
            length_property_key.clone(),
            Attribute::READONLY | Attribute::PERMANENT | Attribute::NON_ENUMERABLE,
        );
        string.set_prototype(constructors.string().prototype());

        let mut regexp_without_proto = ObjectTemplate::new(root_shape);
        regexp_without_proto.property(js_string!("lastIndex").into(), Attribute::WRITABLE);

        let mut regexp = regexp_without_proto.clone();
        regexp.set_prototype(constructors.regexp().prototype());

        let name_property_key: PropertyKey = js_string!("name").into();
        let mut function = ObjectTemplate::new(root_shape);
        function.property(
            length_property_key.clone(),
            Attribute::READONLY | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        );
        function.property(
            name_property_key,
            Attribute::READONLY | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        );

        let function_without_proto = function.clone();
        let mut async_function = function.clone();
        let mut function_with_prototype = function.clone();

        function_with_prototype.property(
            PROTOTYPE.into(),
            Attribute::WRITABLE | Attribute::PERMANENT | Attribute::NON_ENUMERABLE,
        );
        let mut generator_function = function_with_prototype.clone();
        let mut async_generator_function = function_with_prototype.clone();

        let function_with_prototype_without_proto = function_with_prototype.clone();

        function.set_prototype(constructors.function().prototype());
        function_with_prototype.set_prototype(constructors.function().prototype());
        async_function.set_prototype(constructors.async_function().prototype());
        generator_function.set_prototype(constructors.generator_function().prototype());
        async_generator_function.set_prototype(constructors.async_generator_function().prototype());

        let mut function_prototype = ordinary_object.clone();
        function_prototype.property(
            CONSTRUCTOR.into(),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        );

        let mut unmapped_arguments = ordinary_object.clone();

        // 4. Perform DefinePropertyOrThrow(obj, "length", PropertyDescriptor { [[Value]]: 𝔽(len),
        // [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true }).
        unmapped_arguments.property(
            length_property_key,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );

        // 7. Perform ! DefinePropertyOrThrow(obj, @@iterator, PropertyDescriptor {
        // [[Value]]: %Array.prototype.values%, [[Writable]]: true, [[Enumerable]]: false,
        // [[Configurable]]: true }).
        unmapped_arguments.property(
            JsSymbol::iterator().into(),
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );

        let mut mapped_arguments = unmapped_arguments.clone();

        // 8. Perform ! DefinePropertyOrThrow(obj, "callee", PropertyDescriptor {
        // [[Get]]: %ThrowTypeError%, [[Set]]: %ThrowTypeError%, [[Enumerable]]: false,
        // [[Configurable]]: false }).
        unmapped_arguments.accessor(
            js_string!("callee").into(),
            true,
            true,
            Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        );

        // 21. Perform ! DefinePropertyOrThrow(obj, "callee", PropertyDescriptor {
        // [[Value]]: func, [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true }).
        mapped_arguments.property(
            js_string!("callee").into(),
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );

        let mut iterator_result = ordinary_object.clone();
        iterator_result.property(
            js_string!("value").into(),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        );
        iterator_result.property(
            js_string!("done").into(),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        );

        let mut namespace = ObjectTemplate::new(root_shape);
        namespace.property(JsSymbol::to_string_tag().into(), Attribute::empty());

        let with_resolvers = {
            let mut with_resolvers = ordinary_object.clone();

            with_resolvers
                // 4. Perform ! CreateDataPropertyOrThrow(obj, "promise", promiseCapability.[[Promise]]).
                .property(js_string!("promise").into(), Attribute::all())
                // 5. Perform ! CreateDataPropertyOrThrow(obj, "resolve", promiseCapability.[[Resolve]]).
                .property(js_string!("resolve").into(), Attribute::all())
                // 6. Perform ! CreateDataPropertyOrThrow(obj, "reject", promiseCapability.[[Reject]]).
                .property(js_string!("reject").into(), Attribute::all());

            with_resolvers
        };

        let wait_async = {
            let mut obj = ordinary_object.clone();

            obj.property(js_string!("async").into(), Attribute::all())
                .property(js_string!("value").into(), Attribute::all());

            obj
        };

        Self {
            iterator_result,
            ordinary_object,
            array,
            number,
            string,
            symbol,
            bigint,
            boolean,
            regexp,
            regexp_without_proto,
            unmapped_arguments,
            mapped_arguments,
            function_with_prototype,
            function_prototype,
            function,
            async_function,
            generator_function,
            async_generator_function,
            function_without_proto,
            function_with_prototype_without_proto,
            namespace,
            with_resolvers,
            wait_async,
        }
    }

    /// Cached iterator result template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    /// 2. `"done"`: (`WRITABLE`, `CONFIGURABLE`, `ENUMERABLE`)
    /// 3. `"value"`: (`WRITABLE`, `CONFIGURABLE`, `ENUMERABLE`)
    pub(crate) const fn iterator_result(&self) -> &ObjectTemplate {
        &self.iterator_result
    }

    /// Cached ordinary object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    pub(crate) const fn ordinary_object(&self) -> &ObjectTemplate {
        &self.ordinary_object
    }

    /// Cached array object template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`WRITABLE`, `PERMANENT`,`NON_ENUMERABLE`)
    /// 2. `__proto__`: `Array.prototype`
    pub(crate) const fn array(&self) -> &ObjectTemplate {
        &self.array
    }

    /// Cached number object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Number.prototype`
    pub(crate) const fn number(&self) -> &ObjectTemplate {
        &self.number
    }

    /// Cached string object template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `PERMANENT`,`NON_ENUMERABLE`)
    /// 2. `__proto__`: `String.prototype`
    pub(crate) const fn string(&self) -> &ObjectTemplate {
        &self.string
    }

    /// Cached symbol object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Symbol.prototype`
    pub(crate) const fn symbol(&self) -> &ObjectTemplate {
        &self.symbol
    }

    /// Cached bigint object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `BigInt.prototype`
    pub(crate) const fn bigint(&self) -> &ObjectTemplate {
        &self.bigint
    }

    /// Cached boolean object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Boolean.prototype`
    pub(crate) const fn boolean(&self) -> &ObjectTemplate {
        &self.boolean
    }

    /// Cached regexp object template.
    ///
    /// Transitions:
    ///
    /// 1. `"lastIndex"`: (`WRITABLE` , `PERMANENT`,`NON_ENUMERABLE`)
    pub(crate) const fn regexp(&self) -> &ObjectTemplate {
        &self.regexp
    }

    /// Cached regexp object template without `__proto__` template.
    ///
    /// Transitions:
    ///
    /// 1. `"lastIndex"`: (`WRITABLE` , `PERMANENT`,`NON_ENUMERABLE`)
    /// 2. `__proto__`: `RegExp.prototype`
    pub(crate) const fn regexp_without_proto(&self) -> &ObjectTemplate {
        &self.regexp_without_proto
    }

    /// Cached unmapped arguments object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    /// 2. `"length"`: (`WRITABLE`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `@@iterator`: (`WRITABLE`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 4. `get/set` `"callee"`: (`NON_ENUMERABLE`, `PERMANENT`)
    pub(crate) const fn unmapped_arguments(&self) -> &ObjectTemplate {
        &self.unmapped_arguments
    }

    /// Cached mapped arguments object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    /// 2. `"length"`: (`WRITABLE`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `@@iterator`: (`WRITABLE`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 4. `"callee"`: (`WRITABLE`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    pub(crate) const fn mapped_arguments(&self) -> &ObjectTemplate {
        &self.mapped_arguments
    }

    /// Cached function object with `"prototype"` property template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `"prototype"`: (`WRITABLE`, `PERMANENT`, `NON_ENUMERABLE`)
    /// 4. `__proto__`: `Function.prototype`
    pub(crate) const fn function_with_prototype(&self) -> &ObjectTemplate {
        &self.function_with_prototype
    }

    /// Cached constructor function object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    /// 2. `"contructor"`: (`WRITABLE`, `CONFIGURABLE`, `NON_ENUMERABLE`)
    pub(crate) const fn function_prototype(&self) -> &ObjectTemplate {
        &self.function_prototype
    }

    /// Cached function object property template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `__proto__`: `Function.prototype`
    pub(crate) const fn function(&self) -> &ObjectTemplate {
        &self.function
    }

    /// Cached function object property template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `__proto__`: `AsyncFunction.prototype`
    pub(crate) const fn async_function(&self) -> &ObjectTemplate {
        &self.async_function
    }

    /// Cached function object property template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `"prototype"`: (`WRITABLE`, `PERMANENT`, `NON_ENUMERABLE`)
    /// 4. `__proto__`: `GeneratorFunction.prototype`
    pub(crate) const fn generator_function(&self) -> &ObjectTemplate {
        &self.generator_function
    }

    /// Cached function object property template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `"prototype"`: (`WRITABLE`, `PERMANENT`, `NON_ENUMERABLE`)
    /// 4. `__proto__`: `AsyncGeneratorFunction.prototype`
    pub(crate) const fn async_generator_function(&self) -> &ObjectTemplate {
        &self.async_generator_function
    }

    /// Cached function object without `__proto__` template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    pub(crate) const fn function_without_proto(&self) -> &ObjectTemplate {
        &self.function_without_proto
    }

    /// Cached function object with `"prototype"` and without `__proto__` template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `"prototype"`: (`WRITABLE`, `PERMANENT`, `NON_ENUMERABLE`)
    pub(crate) const fn function_with_prototype_without_proto(&self) -> &ObjectTemplate {
        &self.function_with_prototype_without_proto
    }

    /// Cached namespace object template.
    ///
    /// Transitions:
    ///
    /// 1. `@@toStringTag`: (`READONLY`, `NON_ENUMERABLE`, `PERMANENT`)
    pub(crate) const fn namespace(&self) -> &ObjectTemplate {
        &self.namespace
    }

    /// Cached object from the `Promise.withResolvers` method.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    /// 2. `"promise"`: (`WRITABLE`, `ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `"resolve"`: (`WRITABLE`, `ENUMERABLE`, `CONFIGURABLE`)
    /// 4. `"reject"`: (`WRITABLE`, `ENUMERABLE`, `CONFIGURABLE`)
    pub(crate) const fn with_resolvers(&self) -> &ObjectTemplate {
        &self.with_resolvers
    }

    /// Cached object from the `Atomics.waitAsync` method.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    /// 2. `"async"`: (`WRITABLE`, `ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `"value"`: (`WRITABLE`, `ENUMERABLE`, `CONFIGURABLE`)
    pub(crate) const fn wait_async(&self) -> &ObjectTemplate {
        &self.wait_async
    }
}
