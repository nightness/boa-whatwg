//! WebAssembly Instance implementation for Boa
//!
//! Implementation of the WebAssembly.Instance interface according to
//! the W3C WebAssembly JavaScript API specification
//! https://webassembly.github.io/spec/js-api/#instances

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

/// JavaScript `WebAssembly.Instance` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct WebAssemblyInstance;

impl IntrinsicObject for WebAssemblyInstance {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for WebAssemblyInstance {
    const NAME: JsString = StaticJsStrings::WEBASSEMBLY_INSTANCE;
}

impl BuiltInConstructor for WebAssemblyInstance {
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::webassembly_instance;

    /// `WebAssembly.Instance(module, importObject)`
    ///
    /// The WebAssembly.Instance constructor creates a new Instance object
    /// which is a stateful, executable instance of a WebAssembly.Module.
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("WebAssembly.Instance constructor requires 'new'")
                .into());
        }

        let module_arg = args.get_or_undefined(0);
        let import_object = args.get_or_undefined(1);

        // Validate that module is a WebAssembly.Module object
        let module_obj = module_arg.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Instance expects a WebAssembly.Module as first argument")
        })?;

        // Verify it's actually a WebAssembly.Module
        if !module_obj.is::<super::module::WebAssemblyModuleData>() {
            return Err(JsNativeError::typ()
                .with_message("WebAssembly.Instance expects a WebAssembly.Module as first argument")
                .into());
        }

        Self::from_module(module_obj.clone(), import_object, context)
    }
}

impl WebAssemblyInstance {
    /// Create an Instance from a Module object
    pub fn from_module(
        module_obj: JsObject,
        import_object: &JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Get the module data
        let module_data = module_obj.downcast_ref::<super::module::WebAssemblyModuleData>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("Invalid WebAssembly.Module object")
            })?;

        // Get the WebAssembly runtime
        let runtime = WebAssemblyRuntime::get_or_create(context)?;

        // Get the compiled module
        let module = runtime.get_module(module_data.module_id()).ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Invalid WebAssembly.Module - module not found in runtime")
        })?;

        // Process import object to create imports for wasmtime
        let imports = Self::process_import_object(import_object, &module, context)?;

        // Create a new store for this instance
        let store_id = runtime.create_store();

        // Instantiate the module
        let instance_id = runtime.instantiate_module(module_data.module_id(), store_id.clone(), imports)
            .map_err(|err| {
                JsNativeError::typ()
                    .with_message(format!("WebAssembly instantiation failed: {}", err))
            })?;

        // Create the JavaScript Instance object
        let proto = get_prototype_from_constructor(
            &context.intrinsics().constructors().webassembly_instance().constructor().into(),
            StandardConstructors::webassembly_instance,
            context,
        )?;

        let instance_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            WebAssemblyInstanceData::new(instance_id.clone(), store_id.clone()),
        );

        // Create and populate the exports object
        let exports_obj = Self::create_exports_object(&instance_id, &store_id, context)?;
        instance_obj.set(js_string!("exports"), exports_obj, false, context)?;

        Ok(instance_obj.into())
    }

    /// Process the import object to create wasmtime imports
    fn process_import_object(
        import_object: &JsValue,
        module: &wasmtime::Module,
        context: &mut Context,
    ) -> JsResult<std::collections::HashMap<String, std::collections::HashMap<String, wasmtime::Extern>>> {
        let mut imports = std::collections::HashMap::new();

        // If import_object is undefined or null, use empty imports
        if import_object.is_undefined() || import_object.is_null() {
            return Ok(imports);
        }

        let import_obj = import_object.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Import object must be an object")
        })?;

        // For now, we'll support modules without imports
        // TODO: Implement full import processing when other WebAssembly APIs are ready
        let _import_count = module.imports().count();
        if _import_count > 0 {
            return Err(JsNativeError::typ()
                .with_message("Modules with imports are not yet fully supported")
                .into());
        }

        Ok(imports)
    }

    /// Convert a JavaScript value to a wasmtime::Extern based on the import type
    fn js_value_to_extern(
        _value: &JsValue,
        _import_type: &wasmtime::ExternType,
        _context: &mut Context,
    ) -> JsResult<wasmtime::Extern> {
        // For now, we'll skip imports validation and just create empty imports
        // TODO: Implement proper JavaScript function/memory/table/global wrapping
        Err(JsNativeError::typ()
            .with_message("Import handling not yet fully implemented")
            .into())
    }

    /// Create the exports object for an instantiated module
    fn create_exports_object(
        instance_id: &str,
        store_id: &str,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        let exports_obj = JsObject::with_object_proto(context.intrinsics());

        // Get the runtime to access the instance
        let runtime = WebAssemblyRuntime::get_or_create(context)?;

        // TODO: Get the actual exports from the wasmtime instance
        // For now, create an empty exports object
        // This will be populated when we implement proper instance management in the runtime

        Ok(exports_obj)
    }
}

/// Internal data for WebAssembly.Instance instances
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct WebAssemblyInstanceData {
    instance_id: String,
    store_id: String,
}

impl WebAssemblyInstanceData {
    pub fn new(instance_id: String, store_id: String) -> Self {
        Self { instance_id, store_id }
    }

    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn store_id(&self) -> &str {
        &self.store_id
    }
}