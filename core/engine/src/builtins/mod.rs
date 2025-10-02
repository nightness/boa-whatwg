//! Boa's ECMAScript built-in object implementations, e.g. Object, String, Math, Array, etc.

pub mod array;
pub mod array_buffer;
pub mod async_function;
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
pub mod webassembly;

// Make builder public for external browser API crates
pub mod builder;

pub use builder::BuiltInBuilder;
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

// Public exports - make Array, Json, Promise public for external browser API crates
pub use self::{
    array::Array,
    json::Json,
    promise::Promise,
};

// Internal-only exports
pub(crate) use self::{
    array_buffer::ArrayBuffer,
    async_function::AsyncFunction,
    bigint::BigInt,
    boolean::Boolean,
    dataview::DataView,
    date::Date,
    eval::Eval,
    function::BuiltInFunctionObject,
    generator::Generator,
    generator_function::GeneratorFunction,
    iterable::IteratorPrototypes,
    map::Map,
    math::Math,
    number::Number,
    object::OrdinaryObject,
    promise::PromiseCapability,
    proxy::Proxy,
    regexp::RegExp,
    set::Set,
    string::String,
    symbol::Symbol,
    typed_array::{
        BigInt64Array, BigUint64Array, Float32Array, Float64Array, Int16Array, Int32Array,
        Int8Array, Uint16Array, Uint32Array, Uint8Array, Uint8ClampedArray, TypedArray,
    },
    uri::UriFunctions,
    weak::WeakRef,
    weak_map::WeakMap,
    weak_set::WeakSet,
};

use crate::{
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::JsObject,
    property::{Attribute, PropertyDescriptor},
    realm::Realm,
    Context, JsResult, JsString, JsSymbol, JsValue,
};

#[cfg(feature = "intl")]
use crate::builtins::intl::Intl;

#[cfg(feature = "temporal")]
use crate::builtins::temporal::Temporal;

#[cfg(any(feature = "intl", feature = "temporal"))]
use num_traits::ToPrimitive;

/// Trait representing a global built-in object such as `Math`, `JSON`, `Reflect`, etc.
///
/// This trait must be implemented for any global built-in that will be initialized
/// via [`BuiltInBuilder::with_intrinsic`].
pub trait IntrinsicObject {
    /// Initializes the intrinsic object.
    ///
    /// This is where the methods, properties, static methods, etc. are added to the intrinsic.
    fn init(realm: &Realm);

    /// Gets the intrinsic object.
    fn get(intrinsics: &Intrinsics) -> JsObject;
}

/// Trait representing a built-in object that has a constructor.
///
/// This trait is required for all built-in objects that can be constructed via `new BuiltIn()`.
pub trait BuiltInObject: IntrinsicObject {
    /// The binding name of the built-in object.
    const NAME: JsString;

    /// Property attribute flags of the built-in. Check [`Attribute`] for more information.
    const ATTRIBUTE: Attribute = Attribute::WRITABLE
        .union(Attribute::NON_ENUMERABLE)
        .union(Attribute::CONFIGURABLE);
}

/// Trait representing a built-in constructor.
///
/// This trait is required for all built-in objects that can be constructed via `new BuiltIn()`.
pub trait BuiltInConstructor: BuiltInObject {
    /// The amount of arguments the constructor function takes.
    ///
    /// # Note
    ///
    /// This is the value of the `length` property of the constructor function.
    const LENGTH: usize;

    /// The amount of reserved slots for private elements.
    ///
    /// # Note
    ///
    /// This is the number of private slots that will be reserved on instances of this builtin.
    const P: usize = 0;

    /// The amount of reserved slots for static private elements.
    ///
    /// # Note
    ///
    /// This is the number of private slots that will be reserved on the constructor object itself.
    const SP: usize = 0;

    /// The standard constructor getter that returns the constructor's [`StandardConstructor`].
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor;

    /// The native constructor function.
    ///
    /// This function is called when the constructor is called with `new BuiltIn()`.
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue>;
}

/// Initializes the ECMAScript built-in objects and functions.
#[inline]
pub(crate) fn init(realm: &Realm) {
    macro_rules! create_intrinsics {
        (
            $($(#[$attr:meta])* $name:ident $( {
                $($(#[$constructor_attr:meta])* $constructor:ident)?
            })? ,)*
        ) => {
            $({
                $(#[$attr])*
                {
                    <$name as IntrinsicObject>::init(realm);
                }
            })*
        }
    }

    create_intrinsics! {
        OrdinaryObject { Object },
        Math,
        Json,
        Array { Array },
        Proxy,
        BuiltInFunctionObject { Function },
        Generator {GeneratorFunction},
        AsyncFunction,
        RegExp { RegExp },
        String { String },
        Number { Number },
        BigInt { BigInt },
        Boolean { Boolean },
        Error { Error },
        Symbol { Symbol },
        Map { Map },
        Set { Set },
        Int8Array { Int8Array },
        Uint8Array { Uint8Array },
        Uint8ClampedArray { Uint8ClampedArray },
        Int16Array { Int16Array },
        Uint16Array { Uint16Array },
        Int32Array { Int32Array },
        Uint32Array { Uint32Array },
        Float32Array { Float32Array },
        Float64Array { Float64Array },
        BigInt64Array { BigInt64Array },
        BigUint64Array { BigUint64Array },
        ArrayBuffer,
        DataView { DataView },
        Date { Date },
        Promise { Promise },
        WeakRef { WeakRef },
        WeakMap { WeakMap },
        WeakSet { WeakSet },
        #[cfg(feature = "intl")]
        Intl,
        #[cfg(feature = "temporal")]
        Temporal,
    }
}

/// Helper function to register a global builtin object in the context.
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

/// Initialize the default global bindings (intrinsic objects) for a new context.
///
/// This function is called automatically when a context is created.
pub(crate) fn set_default_global_bindings(context: &mut Context) -> JsResult<()> {
    use crate::js_string;

    // First, initialize all intrinsic objects with their methods and properties
    let realm = context.realm().clone();
    realm.intrinsics().initialize(&realm);

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
    global_binding::<Array>(context)?;
    global_binding::<Proxy>(context)?;
    global_binding::<ArrayBuffer>(context)?;
    global_binding::<array_buffer::SharedArrayBuffer>(context)?;
    global_binding::<BigInt>(context)?;
    global_binding::<Boolean>(context)?;
    global_binding::<Date>(context)?;
    global_binding::<DataView>(context)?;
    global_binding::<Map>(context)?;
    global_binding::<number::IsFinite>(context)?;
    global_binding::<number::IsNaN>(context)?;
    global_binding::<number::ParseInt>(context)?;
    global_binding::<number::ParseFloat>(context)?;
    global_binding::<Number>(context)?;
    global_binding::<Eval>(context)?;
    global_binding::<Set>(context)?;
    global_binding::<String>(context)?;
    global_binding::<RegExp>(context)?;
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
    global_binding::<error::RangeError>(context)?;
    global_binding::<error::ReferenceError>(context)?;
    global_binding::<error::TypeError>(context)?;
    global_binding::<error::SyntaxError>(context)?;
    global_binding::<error::EvalError>(context)?;
    global_binding::<error::UriError>(context)?;
    global_binding::<error::AggregateError>(context)?;
    global_binding::<reflect::Reflect>(context)?;
    global_binding::<Promise>(context)?;
    global_binding::<uri::EncodeUri>(context)?;
    global_binding::<uri::EncodeUriComponent>(context)?;
    global_binding::<uri::DecodeUri>(context)?;
    global_binding::<uri::DecodeUriComponent>(context)?;
    global_binding::<WeakRef>(context)?;
    global_binding::<WeakMap>(context)?;
    global_binding::<WeakSet>(context)?;
    global_binding::<atomics::Atomics>(context)?;

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

/// Public function to allow external crates (like thalora-browser-apis) to register global bindings.
///
/// This should be called after creating a context to set up browser/DOM APIs.
///
/// # Example
/// ```ignore
/// use boa_engine::{Context, js_string};
///
/// let mut context = Context::default();
/// // Register browser APIs here
/// thalora_browser_apis::register_globals(&mut context)?;
/// ```
#[inline]
pub fn register_global_binding(
    context: &mut Context,
    name: JsString,
    value: JsValue,
) -> JsResult<()> {
    let global = context.global_object();
    global.create_data_property_or_throw(name, value, context)?;
    Ok(())
}
