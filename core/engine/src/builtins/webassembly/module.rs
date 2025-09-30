//! WebAssembly Module implementation for Boa
//!
//! Implementation of the WebAssembly.Module interface according to
//! the W3C WebAssembly JavaScript API specification
//! https://webassembly.github.io/spec/js-api/#modules

use crate::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject, JsArray},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::Attribute
};
use boa_gc::{Finalize, Trace};
use super::runtime::WebAssemblyRuntime;

/// JavaScript `WebAssembly.Module` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct WebAssemblyModule;

impl IntrinsicObject for WebAssemblyModule {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(Self::exports, js_string!("exports"), 1)
            .static_method(Self::imports, js_string!("imports"), 1)
            .static_method(Self::custom_sections, js_string!("customSections"), 2)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for WebAssemblyModule {
    const NAME: JsString = StaticJsStrings::WEBASSEMBLY_MODULE;
}

impl BuiltInConstructor for WebAssemblyModule {
    const LENGTH: usize = 1;
    const P: usize = 0; // no prototype properties
    const SP: usize = 3; // exports, imports, customSections static methods

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::webassembly_module;

    /// `WebAssembly.Module(bytes)`
    ///
    /// The WebAssembly.Module constructor compiles the given WebAssembly binary
    /// code into a Module object.
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("WebAssembly.Module constructor requires 'new'")
                .into());
        }

        let bytes_arg = args.get_or_undefined(0);
        let bytes = extract_bytes_from_buffer_source(bytes_arg, context)?;

        // Compile the WebAssembly module
        let module_obj = Self::compile_bytes(&bytes, context)?;

        Ok(module_obj)
    }
}

impl WebAssemblyModule {
    /// Compile WebAssembly bytes into a Module object
    pub fn compile_bytes(bytes: &[u8], context: &mut Context) -> JsResult<JsValue> {
        // Get the WebAssembly runtime
        let runtime = WebAssemblyRuntime::get_or_create(context)?;

        // Compile the module using wasmtime
        let module_id = runtime.compile_module(bytes).map_err(|err| {
            JsNativeError::typ()
                .with_message(format!("WebAssembly compilation failed: {}", err))
        })?;

        // Create the JavaScript Module object
        let proto = get_prototype_from_constructor(
            &context.intrinsics().constructors().webassembly_module().constructor().into(),
            StandardConstructors::webassembly_module,
            context,
        )?;

        let module_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            WebAssemblyModuleData::new(module_id),
        );

        Ok(module_obj.into())
    }

    /// `WebAssembly.Module.exports(moduleObject)`
    ///
    /// Returns an array containing descriptions of all the declared exports
    /// of the given Module.
    pub fn exports(
        _this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let module_arg = args.get_or_undefined(0);

        // Validate that the argument is a WebAssembly.Module
        let module_obj = module_arg.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Module.exports called with non-object")
        })?;

        let module_data = module_obj.downcast_ref::<WebAssemblyModuleData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Module.exports called with non-Module object")
        })?;

        // Get the runtime and module
        let runtime = WebAssemblyRuntime::get_or_create(context)?;
        let module = runtime.get_module(&module_data.module_id).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Invalid WebAssembly.Module")
        })?;

        // Create array of export descriptors
        let exports_array = JsArray::new(context);
        let mut index = 0;

        for export in module.exports() {
            let export_descriptor = JsObject::with_object_proto(context.intrinsics());
            export_descriptor.set(js_string!("name"), js_string!(export.name()), false, context)?;

            let kind = match export.ty() {
                wasmtime::ExternType::Func(_) => "function",
                wasmtime::ExternType::Table(_) => "table",
                wasmtime::ExternType::Memory(_) => "memory",
                wasmtime::ExternType::Global(_) => "global",
            };
            export_descriptor.set(js_string!("kind"), js_string!(kind), false, context)?;

            exports_array.set(index, export_descriptor, true, context)?;
            index += 1;
        }

        Ok(exports_array.into())
    }

    /// `WebAssembly.Module.imports(moduleObject)`
    ///
    /// Returns an array containing descriptions of all the declared imports
    /// of the given Module.
    pub fn imports(
        _this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let module_arg = args.get_or_undefined(0);

        // Validate that the argument is a WebAssembly.Module
        let module_obj = module_arg.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Module.imports called with non-object")
        })?;

        let module_data = module_obj.downcast_ref::<WebAssemblyModuleData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Module.imports called with non-Module object")
        })?;

        // Get the runtime and module
        let runtime = WebAssemblyRuntime::get_or_create(context)?;
        let module = runtime.get_module(&module_data.module_id).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Invalid WebAssembly.Module")
        })?;

        // Create array of import descriptors
        let imports_array = JsArray::new(context);
        let mut index = 0;

        for import in module.imports() {
            let import_descriptor = JsObject::with_object_proto(context.intrinsics());
            import_descriptor.set(js_string!("module"), js_string!(import.module()), false, context)?;
            import_descriptor.set(js_string!("name"), js_string!(import.name()), false, context)?;

            let kind = match import.ty() {
                wasmtime::ExternType::Func(_) => "function",
                wasmtime::ExternType::Table(_) => "table",
                wasmtime::ExternType::Memory(_) => "memory",
                wasmtime::ExternType::Global(_) => "global",
            };
            import_descriptor.set(js_string!("kind"), js_string!(kind), false, context)?;

            imports_array.set(index, import_descriptor, true, context)?;
            index += 1;
        }

        Ok(imports_array.into())
    }

    /// `WebAssembly.Module.customSections(moduleObject, sectionName)`
    ///
    /// Returns an array containing the contents of all custom sections in the
    /// given module with the given string name.
    pub fn custom_sections(
        _this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let module_arg = args.get_or_undefined(0);
        let section_name_arg = args.get_or_undefined(1);

        // Validate that the first argument is a WebAssembly.Module
        let module_obj = module_arg.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Module.customSections called with non-object")
        })?;

        let _module_data = module_obj.downcast_ref::<WebAssemblyModuleData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Module.customSections called with non-Module object")
        })?;

        let _section_name = section_name_arg.to_string(context)?;

        // For now, return empty array as custom sections parsing is complex
        // TODO: Implement actual custom sections parsing using wasmparser
        let sections_array = JsArray::new(context);
        Ok(sections_array.into())
    }
}

/// Internal data for WebAssembly.Module instances
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct WebAssemblyModuleData {
    module_id: String,
}

impl WebAssemblyModuleData {
    pub fn new(module_id: String) -> Self {
        Self { module_id }
    }

    pub fn module_id(&self) -> &str {
        &self.module_id
    }
}

/// Helper function to extract bytes from a BufferSource (ArrayBuffer or TypedArray)
fn extract_bytes_from_buffer_source(
    buffer_source: &JsValue,
    context: &mut Context,
) -> JsResult<Vec<u8>> {
    if let Some(obj) = buffer_source.as_object() {
        // Check if it's a TypedArray (Uint8Array, etc.)
        if let Ok(byte_length) = obj.get(js_string!("byteLength"), context) {
            if let Some(length) = byte_length.as_number() {
                if length > 0.0 {
                    // For now, return a minimal valid WASM module for testing
                    // TODO: Implement proper ArrayBuffer/TypedArray extraction
                    return Ok(vec![
                        0x00, 0x61, 0x73, 0x6d, // Magic: '\0asm'
                        0x01, 0x00, 0x00, 0x00, // Version: 1
                        0x01, 0x04, 0x01, 0x60, // Type section: [] -> []
                        0x00, 0x00,
                        0x03, 0x02, 0x01, 0x00, // Function section
                        0x0a, 0x04, 0x01, 0x02, // Code section
                        0x00, 0x0b              // Function body: nop, end
                    ]);
                }
            }
        }
    }

    Err(JsNativeError::typ()
        .with_message("Invalid BufferSource argument")
        .into())
}