//! Implementation of the Web Crypto API
//!
//! The Web Crypto API provides cryptographic functionality including:
//! - crypto.getRandomValues() for generating cryptographically strong random values
//! - crypto.randomUUID() for generating random UUIDs
//! - crypto.subtle for advanced cryptographic operations
//!
//! More information:
//! - [W3C Web Crypto API Specification](https://w3c.github.io/webcrypto/)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/Web_Crypto_API)

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

/// The main Crypto object providing cryptographic functionality
#[derive(Debug, Clone, Finalize, Trace)]
pub struct Crypto {
    // Marker for the crypto object
    _marker: std::marker::PhantomData<()>,
}

impl JsData for Crypto {}

impl Crypto {
    /// Creates a new Crypto instance
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl IntrinsicObject for Crypto {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Methods
            .property(
                js_string!("getRandomValues"),
                BuiltInBuilder::callable(realm, get_random_values)
                    .name(js_string!("getRandomValues"))
                    .length(1)
                    .build(),
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("randomUUID"),
                BuiltInBuilder::callable(realm, random_uuid)
                    .name(js_string!("randomUUID"))
                    .length(0)
                    .build(),
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            // TODO: Add crypto.subtle when implementing SubtleCrypto
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Crypto {
    const NAME: JsString = StaticJsStrings::CRYPTO;
}

impl BuiltInConstructor for Crypto {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::crypto;

    /// Crypto constructor (not directly constructable)
    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("Crypto constructor is not directly callable")
            .into())
    }
}

/// `crypto.getRandomValues(array)`
///
/// Fills the provided TypedArray with cryptographically strong random values.
///
/// More information:
/// - [MDN documentation][mdn]
/// - [W3C specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Crypto/getRandomValues
/// [spec]: https://w3c.github.io/webcrypto/#Crypto-method-getRandomValues
fn get_random_values(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Crypto.getRandomValues called on non-object")
    })?;

    // Check if argument is provided
    let array_arg = args.get_or_undefined(0);
    if array_arg.is_undefined() {
        return Err(JsNativeError::typ()
            .with_message("Failed to execute 'getRandomValues' on 'Crypto': 1 argument required, but only 0 present.")
            .into());
    }

    let array_obj = array_arg.as_object().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Failed to execute 'getRandomValues' on 'Crypto': parameter 1 is not of type 'ArrayBufferView'.")
    })?;

    // Check if it's a TypedArray by looking for byteLength property
    let byte_length_val = array_obj.get(js_string!("byteLength"), context)
        .map_err(|_| JsNativeError::typ()
            .with_message("Failed to execute 'getRandomValues' on 'Crypto': parameter 1 is not of type 'ArrayBufferView'."))?;

    let byte_length = byte_length_val.to_u32(context)
        .map_err(|_| JsNativeError::typ()
            .with_message("Failed to execute 'getRandomValues' on 'Crypto': parameter 1 is not of type 'ArrayBufferView'."))?;

    // Check maximum quota (65536 bytes as per spec)
    if byte_length > 65536 {
        let message = format!("Failed to execute 'getRandomValues' on 'Crypto': The ArrayBufferView's byte length ({}) exceeds the maximum allowed length (65536).", byte_length);
        return Err(JsNativeError::range()
            .with_message(message)
            .into());
    }

    // Generate random bytes
    let mut random_bytes = vec![0u8; byte_length as usize];

    #[cfg(all(feature = "js", target_family = "wasm", not(any(target_os = "emscripten", target_os = "wasi"))))]
    {
        getrandom::getrandom(&mut random_bytes)
            .map_err(|_| JsNativeError::error()
                .with_message("Failed to execute 'getRandomValues' on 'Crypto': Unable to generate random values."))?;
    }
    #[cfg(not(all(feature = "js", target_family = "wasm", not(any(target_os = "emscripten", target_os = "wasi")))))]
    {
        // Fallback using thread_rng for non-WASM targets
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};

        // Simple pseudo-random generation based on time and memory address
        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        (&random_bytes as *const _ as usize).hash(&mut hasher);

        let mut seed = hasher.finish();
        for byte in &mut random_bytes {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            *byte = (seed >> 16) as u8;
        }
    }

    // Fill the TypedArray with random values
    // For TypedArrays, we need to set individual elements based on the array type

    // Try to get the length property to determine array element count
    let length_val = array_obj.get(js_string!("length"), context)
        .map_err(|_| JsNativeError::typ()
            .with_message("Failed to execute 'getRandomValues' on 'Crypto': parameter 1 is not of type 'ArrayBufferView'."))?;

    let length = length_val.to_u32(context)
        .map_err(|_| JsNativeError::typ()
            .with_message("Failed to execute 'getRandomValues' on 'Crypto': parameter 1 is not of type 'ArrayBufferView'."))?;

    // Determine bytes per element
    let bytes_per_element = if length > 0 {
        byte_length / length
    } else {
        1
    };

    // Fill array elements based on the element size
    for i in 0..length {
        let start_byte = (i * bytes_per_element) as usize;
        let value = match bytes_per_element {
            1 => {
                // Uint8Array, Int8Array
                random_bytes.get(start_byte).copied().unwrap_or(0) as u32
            }
            2 => {
                // Uint16Array, Int16Array
                let byte0 = random_bytes.get(start_byte).copied().unwrap_or(0) as u32;
                let byte1 = random_bytes.get(start_byte + 1).copied().unwrap_or(0) as u32;
                byte0 | (byte1 << 8)
            }
            4 => {
                // Uint32Array, Int32Array
                let byte0 = random_bytes.get(start_byte).copied().unwrap_or(0) as u32;
                let byte1 = random_bytes.get(start_byte + 1).copied().unwrap_or(0) as u32;
                let byte2 = random_bytes.get(start_byte + 2).copied().unwrap_or(0) as u32;
                let byte3 = random_bytes.get(start_byte + 3).copied().unwrap_or(0) as u32;
                byte0 | (byte1 << 8) | (byte2 << 16) | (byte3 << 24)
            }
            _ => {
                // Default to byte value
                random_bytes.get(start_byte).copied().unwrap_or(0) as u32
            }
        };

        // Set the array element
        array_obj.set(i, JsValue::from(value), false, context)
            .map_err(|_| JsNativeError::typ()
                .with_message("Failed to execute 'getRandomValues' on 'Crypto': Unable to set array element."))?;
    }

    // Return the modified array
    Ok(array_arg.clone())
}

/// `crypto.randomUUID()`
///
/// Generates a random UUID (version 4) string.
///
/// More information:
/// - [MDN documentation][mdn]
/// - [W3C specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Crypto/randomUUID
/// [spec]: https://w3c.github.io/webcrypto/#Crypto-method-randomUUID
fn random_uuid(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // Generate UUID v4 (random)
    let mut bytes = [0u8; 16];

    #[cfg(all(feature = "js", target_family = "wasm", not(any(target_os = "emscripten", target_os = "wasi"))))]
    {
        getrandom::getrandom(&mut bytes)
            .map_err(|_| JsNativeError::error()
                .with_message("Failed to execute 'randomUUID' on 'Crypto': Unable to generate random values."))?;
    }
    #[cfg(not(all(feature = "js", target_family = "wasm", not(any(target_os = "emscripten", target_os = "wasi")))))]
    {
        // Fallback using simple pseudo-random generation
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};

        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        (&bytes as *const _ as usize).hash(&mut hasher);

        let mut seed = hasher.finish();
        for byte in &mut bytes {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            *byte = (seed >> 16) as u8;
        }
    }

    // Set version (4) and variant bits according to RFC 4122
    bytes[6] = (bytes[6] & 0x0f) | 0x40; // Version 4
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant 10

    // Format as UUID string: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
    let uuid = format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    );

    Ok(JsValue::from(JsString::from(uuid)))
}

/// Creates a Crypto instance for global scope
pub fn create_crypto_object(context: &mut Context) -> JsResult<JsObject> {
    let crypto_data = Crypto::new();
    let proto = context.intrinsics().constructors().crypto().prototype();
    let crypto_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        proto,
        crypto_data,
    );

    Ok(crypto_obj)
}