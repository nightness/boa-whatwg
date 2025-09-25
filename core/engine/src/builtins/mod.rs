//! Boa's ECMAScript built-in object implementations, e.g. Object, String, Math, Array, etc.

pub mod abort_controller;
pub mod array;
pub mod array_buffer;
pub mod async_function;
pub mod readable_stream;
pub mod websocket;
pub mod websocket_stream;
pub mod document;
pub mod document_parse;
pub mod form;
pub mod window;
pub mod history;
pub mod pageswap_event;
pub mod node;
pub mod character_data;
pub mod text;
pub mod document_fragment;
pub mod nodelist;
pub mod element;
pub mod attr;
pub mod comment;
pub mod processing_instruction;
pub mod cdata_section;
pub mod selection;
pub mod frame_selection;
pub mod range;
pub mod event;
pub mod event_target;
pub mod custom_event;
pub mod domtokenlist;
pub mod css;
pub mod fetch;
pub mod xmlhttprequest;
pub mod mutation_observer;
pub mod intersection_observer;
pub mod resize_observer;
pub mod console;
pub mod timers;
pub mod blob;
pub mod async_generator;
pub mod async_generator_function;
pub mod atomics;
pub mod bigint;
pub mod boolean;
pub mod dataview;
pub mod date;
pub mod error;
pub mod eval;
pub mod function;
pub mod generator;
pub mod generator_function;
pub mod iterable;
pub mod json;
pub mod map;
pub mod math;
pub mod number;
pub mod object;
pub mod promise;
pub mod proxy;
pub mod reflect;
pub mod regexp;
pub mod set;
pub mod string;
pub mod symbol;
pub mod typed_array;
pub mod uri;
pub mod weak;
pub mod weak_map;
pub mod weak_set;
pub mod storage;
pub mod web_locks;
pub mod indexed_db;
pub mod navigator;

mod builder;

use builder::BuiltInBuilder;
use error::Error;

#[cfg(feature = "annex-b")]
pub mod escape;

#[cfg(feature = "intl")]
pub mod intl;

// TODO: remove `cfg` when `Temporal` gets to stage 4.
#[cfg(any(feature = "intl", feature = "temporal"))]
pub(crate) mod options;

#[cfg(feature = "temporal")]
pub mod temporal;

pub(crate) use self::{
    abort_controller::AbortController,
    array::Array,
    async_function::AsyncFunction,
    bigint::BigInt,
    boolean::Boolean,
    dataview::DataView,
    date::Date,
    error::{
        AggregateError, EvalError, RangeError, ReferenceError, SyntaxError, TypeError, UriError,
    },
    eval::Eval,
    function::BuiltInFunctionObject,
    json::Json,
    map::Map,
    math::Math,
    number::{IsFinite, IsNaN, Number, ParseFloat, ParseInt},
    object::OrdinaryObject,
    promise::Promise,
    proxy::Proxy,
    readable_stream::ReadableStream,
    websocket::WebSocket,
    websocket_stream::WebSocketStream,
    document::Document,
    form::{HTMLFormElement, HTMLFormControlsCollection, HTMLInputElement},
    window::Window,
    history::History,
    pageswap_event::PageSwapEvent,
    node::Node,
    character_data::CharacterData,
    text::Text,
    document_fragment::DocumentFragment,
    nodelist::NodeList,
    element::Element,
    attr::Attr,
    comment::Comment,
    domtokenlist::DOMTokenList,
    selection::Selection,
    range::Range,
    event::Event,
    event_target::EventTarget,
    console::Console,
    blob::Blob,
    xmlhttprequest::XmlHttpRequest,
    mutation_observer::MutationObserver,
    intersection_observer::IntersectionObserver,
    resize_observer::ResizeObserver,
    reflect::Reflect,
    regexp::RegExp,
    set::Set,
    string::String,
    symbol::Symbol,
    typed_array::{
        BigInt64Array, BigUint64Array, Float32Array, Float64Array, Int8Array, Int16Array,
        Int32Array, Uint8Array, Uint8ClampedArray, Uint16Array, Uint32Array,
    },
};

use crate::{
    Context, JsResult, JsString, JsValue, JsNativeError,
    builtins::{
        array::ArrayIterator,
        array_buffer::{ArrayBuffer, SharedArrayBuffer},
        async_generator::AsyncGenerator,
        async_generator_function::AsyncGeneratorFunction,
        atomics::Atomics,
        error::r#type::ThrowTypeError,
        fetch::Fetch,
        timers::{SetTimeout, SetInterval, ClearTimeout, ClearInterval},
        generator::Generator,
        generator_function::GeneratorFunction,
        iterable::{AsyncFromSyncIterator, AsyncIterator, Iterator},
        map::MapIterator,
        object::for_in_iterator::ForInIterator,
        regexp::RegExpStringIterator,
        set::SetIterator,
        string::StringIterator,
        typed_array::BuiltinTypedArray,
        uri::{DecodeUri, DecodeUriComponent, EncodeUri, EncodeUriComponent},
        weak::WeakRef,
        weak_map::WeakMap,
        weak_set::WeakSet,
        storage::Storage,
        web_locks::{LockManagerObject, Lock},
        indexed_db::IdbFactory,
        navigator::Navigator,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    property::{Attribute, PropertyDescriptor},
    realm::Realm,
};

/// A [Well-Known Intrinsic Object].
///
/// Well-known intrinsics are built-in objects that are explicitly referenced by the algorithms of
/// the specification and which usually have realm-specific identities.
///
/// [Well-Known Intrinsic Object]: https://tc39.es/ecma262/#sec-well-known-intrinsic-objects
pub(crate) trait IntrinsicObject {
    /// Initializes the intrinsic object.
    ///
    /// This is where the methods, properties, static methods and the constructor of a built-in must
    /// be initialized to be accessible from ECMAScript.
    fn init(realm: &Realm);

    /// Gets the intrinsic object.
    fn get(intrinsics: &Intrinsics) -> JsObject;
}

/// A [built-in object].
///
/// This trait must be implemented for any global built-in that lives in the global context of a script.
///
/// [built-in object]: https://tc39.es/ecma262/#sec-built-in-object
pub(crate) trait BuiltInObject: IntrinsicObject {
    /// Binding name of the builtin inside the global object.
    ///
    /// E.g. If you want access the properties of a `Complex` built-in with the name `Cplx` you must
    /// assign `"Cplx"` to this constant, making any property inside it accessible from ECMAScript
    /// as `Cplx.prop`
    // `JsString` can only be const-constructed for static strings.
    #[allow(clippy::declare_interior_mutable_const)]
    const NAME: JsString;

    /// Property attribute flags of the built-in. Check [`Attribute`] for more information.
    const ATTRIBUTE: Attribute = Attribute::WRITABLE
        .union(Attribute::NON_ENUMERABLE)
        .union(Attribute::CONFIGURABLE);
}

/// A [built-in object] that is also a constructor.
///
/// This trait must be implemented for any global built-in that can also be called with `new` to
/// construct an object instance e.g. `Array`, `Map` or `Object`.
///
/// [built-in object]: https://tc39.es/ecma262/#sec-built-in-object
pub(crate) trait BuiltInConstructor: BuiltInObject {
    /// Const Generic `P` is the minimum storage capacity for the prototype's Property table.
    const P: usize;
    /// Const Generic `SP` is the minimum storage capacity for the object's Static Property table.
    const SP: usize;
    /// The amount of arguments this function object takes.
    const LENGTH: usize;

    /// The corresponding standard constructor of this constructor.
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor;

    /// The native constructor function.
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue>;
}

fn global_binding<B: BuiltInObject>(context: &mut Context) -> JsResult<()> {
    let name = B::NAME;
    let attr = B::ATTRIBUTE;
    let intrinsic = B::get(context.intrinsics());
    let global_object = context.global_object();

    global_object.define_property_or_throw(
        name,
        PropertyDescriptor::builder()
            .value(intrinsic)
            .writable(attr.writable())
            .enumerable(attr.enumerable())
            .configurable(attr.configurable())
            .build(),
        context,
    )?;
    Ok(())
}

impl Realm {
    /// Abstract operation [`CreateIntrinsics ( realmRec )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createintrinsics
    pub(crate) fn initialize(&self) {
        BuiltInFunctionObject::init(self);
        OrdinaryObject::init(self);
        Iterator::init(self);
        AsyncIterator::init(self);
        AsyncFromSyncIterator::init(self);
        ForInIterator::init(self);
        Math::init(self);
        Json::init(self);
        css::Css::init(self);
        Array::init(self);
        ArrayIterator::init(self);
        Proxy::init(self);
        ArrayBuffer::init(self);
        SharedArrayBuffer::init(self);
        BigInt::init(self);
        Boolean::init(self);
        Date::init(self);
        DataView::init(self);
        Map::init(self);
        MapIterator::init(self);
        IsFinite::init(self);
        IsNaN::init(self);
        ParseInt::init(self);
        ParseFloat::init(self);
        Number::init(self);
        Eval::init(self);
        Set::init(self);
        SetIterator::init(self);
        String::init(self);
        StringIterator::init(self);
        RegExp::init(self);
        RegExpStringIterator::init(self);
        BuiltinTypedArray::init(self);
        Int8Array::init(self);
        Uint8Array::init(self);
        Uint8ClampedArray::init(self);
        Int16Array::init(self);
        Uint16Array::init(self);
        Int32Array::init(self);
        Uint32Array::init(self);
        BigInt64Array::init(self);
        BigUint64Array::init(self);
        #[cfg(feature = "float16")]
        typed_array::Float16Array::init(self);
        Float32Array::init(self);
        Float64Array::init(self);
        Symbol::init(self);
        Error::init(self);
        RangeError::init(self);
        ReferenceError::init(self);
        TypeError::init(self);
        ThrowTypeError::init(self);
        SyntaxError::init(self);
        EvalError::init(self);
        UriError::init(self);
        AggregateError::init(self);
        Reflect::init(self);
        Generator::init(self);
        GeneratorFunction::init(self);
        Promise::init(self);
        ReadableStream::init(self);
        WebSocket::init(self);
        WebSocketStream::init(self);
        AbortController::init(self);
        Document::init(self);
        document_parse::setup_parse_html_unsafe(self);
        Window::init(self);
        History::init(self);
        PageSwapEvent::init(self);
        Node::init(self);
        CharacterData::init(self);
        Text::init(self);
        DocumentFragment::init(self);
        NodeList::init(self);
        Element::init(self);
        Attr::init(self);
        Comment::init(self);
            DOMTokenList::init(self);
        processing_instruction::ProcessingInstruction::init(self);
        cdata_section::CDATASection::init(self);
        HTMLFormElement::init(self);
        HTMLFormControlsCollection::init(self);
        HTMLInputElement::init(self);
        Selection::init(self);
        Range::init(self);
        Event::init(self);
        EventTarget::init(self);
        custom_event::CustomEvent::init(self);
        Fetch::init(self);
        XmlHttpRequest::init(self);
        MutationObserver::init(self);
        IntersectionObserver::init(self);
        ResizeObserver::init(self);
        Console::init(self);
        Blob::init(self);
        SetTimeout::init(self);
        SetInterval::init(self);
        ClearTimeout::init(self);
        ClearInterval::init(self);
        AsyncFunction::init(self);
        AsyncGenerator::init(self);
        AsyncGeneratorFunction::init(self);
        EncodeUri::init(self);
        EncodeUriComponent::init(self);
        DecodeUri::init(self);
        DecodeUriComponent::init(self);
        WeakRef::init(self);
        WeakMap::init(self);
        WeakSet::init(self);
        Storage::init(self);
        web_locks::LockManagerObject::init(self);
        Navigator::init(self);
        Atomics::init(self);

        #[cfg(feature = "annex-b")]
        {
            escape::Escape::init(self);
            escape::Unescape::init(self);
        }

        #[cfg(feature = "intl")]
        {
            intl::Intl::init(self);
            intl::Collator::init(self);
            intl::ListFormat::init(self);
            intl::Locale::init(self);
            intl::DateTimeFormat::init(self);
            intl::Segmenter::init(self);
            intl::segmenter::Segments::init(self);
            intl::segmenter::SegmentIterator::init(self);
            intl::PluralRules::init(self);
            intl::NumberFormat::init(self);
        }

        #[cfg(feature = "temporal")]
        {
            temporal::Temporal::init(self);
            temporal::Now::init(self);
            temporal::Instant::init(self);
            temporal::Duration::init(self);
            temporal::PlainDate::init(self);
            temporal::PlainTime::init(self);
            temporal::PlainDateTime::init(self);
            temporal::PlainMonthDay::init(self);
            temporal::PlainYearMonth::init(self);
            temporal::ZonedDateTime::init(self);
        }
    }
}

/// Abstract operation [`SetDefaultGlobalBindings ( realmRec )`][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-setdefaultglobalbindings
pub(crate) fn set_default_global_bindings(context: &mut Context) -> JsResult<()> {
    let global_object = context.global_object();

    global_object.define_property_or_throw(
        js_string!("globalThis"),
        PropertyDescriptor::builder()
            .value(context.realm().global_this().clone())
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;
    // Create an actual Window object instead of just pointing to globalThis
    let window_constructor = context.intrinsics().constructors().window().constructor();
    let window_obj = Window::constructor(&window_constructor.clone().into(), &[], context)?;
    global_object.define_property_or_throw(
        js_string!("window"),
        PropertyDescriptor::builder()
            .value(window_obj.clone())
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Also expose localStorage and sessionStorage as global properties for convenience
    if let Some(window_object) = window_obj.as_object() {
        if let Ok(local_storage) = window_object.get(js_string!("localStorage"), context) {
            global_object.define_property_or_throw(
                js_string!("localStorage"),
                PropertyDescriptor::builder()
                    .value(local_storage)
                    .writable(false)
                    .enumerable(false)
                    .configurable(true),
                context,
            )?;
        }
        if let Ok(session_storage) = window_object.get(js_string!("sessionStorage"), context) {
            global_object.define_property_or_throw(
                js_string!("sessionStorage"),
                PropertyDescriptor::builder()
                    .value(session_storage)
                    .writable(false)
                    .enumerable(false)
                    .configurable(true),
                context,
            )?;
        }

        // Also expose navigator as global for convenience
        if let Ok(navigator) = window_object.get(js_string!("navigator"), context) {
            global_object.define_property_or_throw(
                js_string!("navigator"),
                PropertyDescriptor::builder()
                    .value(navigator)
                    .writable(false)
                    .enumerable(false)
                    .configurable(true),
                context,
            )?;
        }

        // Also expose indexedDB as global for convenience
        if let Ok(indexed_db) = window_object.get(js_string!("indexedDB"), context) {
            global_object.define_property_or_throw(
                js_string!("indexedDB"),
                PropertyDescriptor::builder()
                    .value(indexed_db)
                    .writable(false)
                    .enumerable(false)
                    .configurable(true),
                context,
            )?;
        }
    }
    let restricted = PropertyDescriptor::builder()
        .writable(false)
        .enumerable(false)
        .configurable(false);
    global_object.define_property_or_throw(
        js_string!("Infinity"),
        restricted.clone().value(f64::INFINITY),
        context,
    )?;
    global_object.define_property_or_throw(
        js_string!("NaN"),
        restricted.clone().value(f64::NAN),
        context,
    )?;
    global_object.define_property_or_throw(
        js_string!("undefined"),
        restricted.value(JsValue::undefined()),
        context,
    )?;

    global_binding::<BuiltInFunctionObject>(context)?;
    global_binding::<OrdinaryObject>(context)?;
    global_binding::<Math>(context)?;
    global_binding::<Json>(context)?;
    global_binding::<css::Css>(context)?;
    css::setup_css_worklets(context)?;
    global_binding::<Array>(context)?;
    global_binding::<Proxy>(context)?;
    global_binding::<ArrayBuffer>(context)?;
    global_binding::<SharedArrayBuffer>(context)?;
    global_binding::<BigInt>(context)?;
    global_binding::<Boolean>(context)?;
    global_binding::<Date>(context)?;
    global_binding::<DataView>(context)?;
    global_binding::<Map>(context)?;
    global_binding::<IsFinite>(context)?;
    global_binding::<IsNaN>(context)?;
    global_binding::<ParseInt>(context)?;
    global_binding::<ParseFloat>(context)?;
    global_binding::<Number>(context)?;
    global_binding::<Eval>(context)?;
    global_binding::<Set>(context)?;
    global_binding::<String>(context)?;
    global_binding::<RegExp>(context)?;
    global_binding::<BuiltinTypedArray>(context)?;
    global_binding::<Int8Array>(context)?;
    global_binding::<Uint8Array>(context)?;
    global_binding::<Uint8ClampedArray>(context)?;
    global_binding::<Int16Array>(context)?;
    global_binding::<Uint16Array>(context)?;
    global_binding::<Int32Array>(context)?;
    global_binding::<Uint32Array>(context)?;
    global_binding::<BigInt64Array>(context)?;
    global_binding::<BigUint64Array>(context)?;
    #[cfg(feature = "float16")]
    global_binding::<typed_array::Float16Array>(context)?;
    global_binding::<Float32Array>(context)?;
    global_binding::<Float64Array>(context)?;
    global_binding::<Symbol>(context)?;
    global_binding::<Error>(context)?;
    global_binding::<RangeError>(context)?;
    global_binding::<ReferenceError>(context)?;
    global_binding::<TypeError>(context)?;
    global_binding::<SyntaxError>(context)?;
    global_binding::<EvalError>(context)?;
    global_binding::<UriError>(context)?;
    global_binding::<AggregateError>(context)?;
    global_binding::<Reflect>(context)?;
    global_binding::<Promise>(context)?;
    global_binding::<ReadableStream>(context)?;
    global_binding::<WebSocket>(context)?;
    global_binding::<AbortController>(context)?;
    global_binding::<XmlHttpRequest>(context)?;
    global_binding::<MutationObserver>(context)?;
    global_binding::<IntersectionObserver>(context)?;
    global_binding::<ResizeObserver>(context)?;
    global_binding::<Console>(context)?;
    global_binding::<Blob>(context)?;
    global_binding::<Range>(context)?;
    global_binding::<Event>(context)?;
    global_binding::<EventTarget>(context)?;
    global_binding::<custom_event::CustomEvent>(context)?;
    global_binding::<Node>(context)?;
    global_binding::<Element>(context)?;
    global_binding::<Attr>(context)?;
    global_binding::<Comment>(context)?;
    global_binding::<DOMTokenList>(context)?;
    global_binding::<processing_instruction::ProcessingInstruction>(context)?;
    global_binding::<cdata_section::CDATASection>(context)?;
    global_binding::<CharacterData>(context)?;
    global_binding::<Text>(context)?;
    global_binding::<DocumentFragment>(context)?;
    global_binding::<NodeList>(context)?;
    global_binding::<Document>(context)?;
    global_binding::<SetTimeout>(context)?;
    global_binding::<SetInterval>(context)?;
    global_binding::<ClearTimeout>(context)?;
    global_binding::<ClearInterval>(context)?;
    global_binding::<EncodeUri>(context)?;
    global_binding::<EncodeUriComponent>(context)?;
    global_binding::<DecodeUri>(context)?;
    global_binding::<DecodeUriComponent>(context)?;
    global_binding::<WeakRef>(context)?;
    global_binding::<WeakMap>(context)?;
    global_binding::<WeakSet>(context)?;
    global_binding::<Storage>(context)?;
    global_binding::<WebSocketStream>(context)?;
    global_binding::<Atomics>(context)?;

    // Add getSelection method to global object (window.getSelection)
    let get_selection_func = BuiltInBuilder::callable(context.realm(), window_get_selection)
        .name(js_string!("getSelection"))
        .length(0)
        .build();

    global_object.define_property_or_throw(
        js_string!("getSelection"),
        PropertyDescriptor::builder()
            .value(get_selection_func)
            .writable(true)
            .enumerable(true)
            .configurable(true),
        context,
    )?;

    #[cfg(feature = "annex-b")]
    {
        global_binding::<escape::Escape>(context)?;
        global_binding::<escape::Unescape>(context)?;
    }

    #[cfg(feature = "intl")]
    global_binding::<intl::Intl>(context)?;

    #[cfg(feature = "temporal")]
    {
        global_binding::<temporal::Temporal>(context)?;
    }

    Ok(())
}

/// `window.getSelection()` global function
fn window_get_selection(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    // Create a new Selection object
    let selection_constructor = context.intrinsics().constructors().selection().constructor();
    let selection_obj = selection_constructor.construct(&[], None, context)?;
    Ok(selection_obj.into())
}

