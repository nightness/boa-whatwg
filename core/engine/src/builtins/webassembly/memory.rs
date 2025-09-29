//! WebAssembly Memory implementation for Boa
//!
//! Implementation of the WebAssembly.Memory interface according to
//! the W3C WebAssembly JavaScript API specification
//! https://webassembly.github.io/spec/js-api/#memories

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
use super::runtime::WebAssemblyRuntime;

/// JavaScript `WebAssembly.Memory` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct WebAssemblyMemory;

impl IntrinsicObject for WebAssemblyMemory {
    fn init(realm: &Realm) {
        let buffer_getter = BuiltInBuilder::callable(realm, Self::buffer)
            .name(js_string!("get buffer"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("buffer"),
                Some(buffer_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(Self::grow, js_string!("grow"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for WebAssemblyMemory {
    const NAME: JsString = StaticJsStrings::WEBASSEMBLY_MEMORY;
}

impl BuiltInConstructor for WebAssemblyMemory {
    const LENGTH: usize = 1;
    const P: usize = 2; // buffer property, grow method
    const SP: usize = 0; // no static properties

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::webassembly_memory;

    /// `WebAssembly.Memory(descriptor)`
    ///
    /// The WebAssembly.Memory constructor creates a new Memory object
    /// which is a resizable ArrayBuffer or SharedArrayBuffer whose
    /// contents are the raw bytes of memory instances.
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("WebAssembly.Memory constructor requires 'new'")
                .into());
        }

        let descriptor = args.get_or_undefined(0);

        // Parse the memory descriptor
        let memory_descriptor = Self::parse_memory_descriptor(descriptor, context)?;

        // Create the memory using wasmtime
        Self::create_memory(memory_descriptor, context)
    }
}

impl WebAssemblyMemory {
    /// Parse a WebAssembly memory descriptor object
    fn parse_memory_descriptor(
        descriptor: &JsValue,
        context: &mut Context,
    ) -> JsResult<MemoryDescriptor> {
        let desc_obj = descriptor.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Memory descriptor must be an object")
        })?;

        // Get initial pages (required)
        let initial = desc_obj.get(js_string!("initial"), context)?
            .to_u32(context)? as u64;

        // Get maximum pages (optional)
        let maximum = if let Ok(max_val) = desc_obj.get(js_string!("maximum"), context) {
            if !max_val.is_undefined() {
                Some(max_val.to_u32(context)? as u64)
            } else {
                None
            }
        } else {
            None
        };

        // Get shared flag (optional, defaults to false)
        let shared = if let Ok(shared_val) = desc_obj.get(js_string!("shared"), context) {
            shared_val.to_boolean()
        } else {
            false
        };

        // Get index type (optional, defaults to "i32")
        let index = if let Ok(index_val) = desc_obj.get(js_string!("index"), context) {
            let index_str = index_val.to_string(context)?;
            match index_str.to_std_string_escaped().as_str() {
                "i32" => IndexType::I32,
                "i64" => IndexType::I64,
                _ => return Err(JsNativeError::typ()
                    .with_message("WebAssembly.Memory index must be 'i32' or 'i64'")
                    .into())
            }
        } else {
            IndexType::I32
        };

        // Validate page limits
        if let Some(max) = maximum {
            if initial > max {
                return Err(JsNativeError::range()
                    .with_message("WebAssembly.Memory initial size exceeds maximum")
                    .into());
            }
        }

        // WebAssembly page size is 65,536 bytes (64 KiB)
        const WASM_PAGE_SIZE: u64 = 65536;

        // For i32 memories: maximum is 2^16 pages (4 GiB)
        // For i64 memories: maximum is 2^48 pages (theoretical, but practically limited)
        let max_pages = match index {
            IndexType::I32 => 65536, // 2^16 pages = 4 GiB
            IndexType::I64 => 1u64 << 48, // Extremely large but still finite
        };

        if initial > max_pages {
            return Err(JsNativeError::range()
                .with_message("WebAssembly.Memory initial size exceeds maximum allowed")
                .into());
        }

        if let Some(max) = maximum {
            if max > max_pages {
                return Err(JsNativeError::range()
                    .with_message("WebAssembly.Memory maximum size exceeds maximum allowed")
                    .into());
            }
        }

        Ok(MemoryDescriptor {
            initial,
            maximum,
            shared,
            index,
        })
    }

    /// Create a WebAssembly.Memory object
    fn create_memory(
        descriptor: MemoryDescriptor,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Get the WebAssembly runtime
        let runtime = WebAssemblyRuntime::get_or_create(context)?;

        // Convert to wasmtime MemoryType (wasmtime uses u32 for memory sizes)
        let memory_type = wasmtime::MemoryType::new(
            descriptor.initial as u32,
            descriptor.maximum.map(|m| m as u32),
        );

        // Create the memory in wasmtime
        let memory_id = runtime.create_memory(memory_type).map_err(|err| {
            JsNativeError::typ()
                .with_message(format!("WebAssembly.Memory creation failed: {}", err))
        })?;

        // Create the JavaScript Memory object
        let proto = get_prototype_from_constructor(
            &context.intrinsics().constructors().webassembly_memory().constructor().into(),
            StandardConstructors::webassembly_memory,
            context,
        )?;

        let memory_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            WebAssemblyMemoryData::new(memory_id, descriptor),
        );

        Ok(memory_obj.into())
    }

    /// `get WebAssembly.Memory.prototype.buffer`
    ///
    /// Returns an ArrayBuffer whose contents are the memory.
    fn buffer(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let memory_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Memory.buffer called on non-object")
        })?;

        let memory_data = memory_obj.downcast_ref::<WebAssemblyMemoryData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Memory.buffer called on non-Memory object")
        })?;

        // Get the runtime to access the memory
        let runtime = WebAssemblyRuntime::get_or_create(context)?;

        // For now, create a placeholder ArrayBuffer
        // TODO: Implement proper memory buffer access from wasmtime
        let array_buffer = crate::builtins::array_buffer::ArrayBuffer::constructor(
            &JsValue::undefined(),
            &[JsValue::new(0)], // Start with empty buffer
            context,
        )?;

        Ok(array_buffer)
    }

    /// `WebAssembly.Memory.prototype.grow(delta)`
    ///
    /// Increases the size of the memory instance by delta pages.
    fn grow(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let memory_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Memory.grow called on non-object")
        })?;

        let memory_data = memory_obj.downcast_ref::<WebAssemblyMemoryData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Memory.grow called on non-Memory object")
        })?;

        let delta = args.get_or_undefined(0).to_u32(context)? as u64;

        // Get the runtime to grow the memory
        let runtime = WebAssemblyRuntime::get_or_create(context)?;

        // TODO: Implement actual memory growth via wasmtime
        // For now, return the previous size (0)
        Ok(JsValue::new(0))
    }
}

/// Internal data for WebAssembly.Memory instances
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct WebAssemblyMemoryData {
    memory_id: String,
    descriptor: MemoryDescriptor,
}

impl WebAssemblyMemoryData {
    pub fn new(memory_id: String, descriptor: MemoryDescriptor) -> Self {
        Self { memory_id, descriptor }
    }

    pub fn memory_id(&self) -> &str {
        &self.memory_id
    }

    pub fn descriptor(&self) -> &MemoryDescriptor {
        &self.descriptor
    }
}

/// WebAssembly memory descriptor
#[derive(Debug, Clone, Trace, Finalize)]
pub struct MemoryDescriptor {
    pub initial: u64,
    pub maximum: Option<u64>,
    pub shared: bool,
    pub index: IndexType,
}

/// WebAssembly memory index type (i32 or i64)
#[derive(Debug, Clone, Trace, Finalize)]
pub enum IndexType {
    I32,
    I64,
}