//! WebAssembly Global implementation for Boa
//!
//! Implementation of the WebAssembly.Global interface according to
//! the W3C WebAssembly JavaScript API specification
//! https://webassembly.github.io/spec/js-api/#globals

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

/// JavaScript `WebAssembly.Global` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct WebAssemblyGlobal;

impl IntrinsicObject for WebAssemblyGlobal {
    fn init(realm: &Realm) {
        let value_getter = BuiltInBuilder::callable(realm, Self::value)
            .name(js_string!("get value"))
            .build();
        let value_setter = BuiltInBuilder::callable(realm, Self::set_value)
            .name(js_string!("set value"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("value"),
                Some(value_getter),
                Some(value_setter),
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for WebAssemblyGlobal {
    const NAME: JsString = StaticJsStrings::WEBASSEMBLY_GLOBAL;
}

impl BuiltInConstructor for WebAssemblyGlobal {
    const LENGTH: usize = 2;
    const P: usize = 2; // value property, valueOf method
    const SP: usize = 0; // no static properties

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::webassembly_global;

    /// `WebAssembly.Global(descriptor, value?)`
    ///
    /// The WebAssembly.Global constructor creates a new Global object
    /// which is a JavaScript wrapper for a WebAssembly global instance.
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("WebAssembly.Global constructor requires 'new'")
                .into());
        }

        let descriptor = args.get_or_undefined(0);
        let value = args.get_or_undefined(1);

        // Parse the global descriptor
        let global_descriptor = Self::parse_global_descriptor(descriptor, context)?;

        // Create the global using wasmtime
        Self::create_global(global_descriptor, value, context)
    }
}

impl WebAssemblyGlobal {
    /// Parse a WebAssembly global descriptor object
    fn parse_global_descriptor(
        descriptor: &JsValue,
        context: &mut Context,
    ) -> JsResult<GlobalDescriptor> {
        let desc_obj = descriptor.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Global descriptor must be an object")
        })?;

        // Get value type (required)
        let value_type = desc_obj.get(js_string!("value"), context)?;
        let value_str = value_type.to_string(context)?;
        let wasm_type = match value_str.to_std_string_escaped().as_str() {
            "i32" => ValueType::I32,
            "i64" => ValueType::I64,
            "f32" => ValueType::F32,
            "f64" => ValueType::F64,
            "v128" => ValueType::V128,
            "externref" => ValueType::ExternRef,
            "funcref" => ValueType::FuncRef,
            _ => return Err(JsNativeError::typ()
                .with_message("WebAssembly.Global value must be a valid WebAssembly type")
                .into())
        };

        // Get mutable flag (optional, defaults to false)
        let mutable = if let Ok(mut_val) = desc_obj.get(js_string!("mutable"), context) {
            mut_val.to_boolean()
        } else {
            false
        };

        Ok(GlobalDescriptor {
            value_type: wasm_type,
            mutable,
        })
    }

    /// Create a WebAssembly.Global object
    fn create_global(
        descriptor: GlobalDescriptor,
        value: &JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Get the WebAssembly runtime
        let runtime = WebAssemblyRuntime::get_or_create(context)?;

        // Convert JavaScript value to WebAssembly value
        let wasm_value = Self::js_value_to_wasm_value(value, &descriptor.value_type, context)?;

        // Convert to wasmtime GlobalType
        let global_type = wasmtime::GlobalType::new(
            Self::value_type_to_wasmtime(&descriptor.value_type),
            if descriptor.mutable {
                wasmtime::Mutability::Var
            } else {
                wasmtime::Mutability::Const
            },
        );

        // Create the global in wasmtime
        let global_id = runtime.create_global(global_type, wasm_value).map_err(|err| {
            JsNativeError::typ()
                .with_message(format!("WebAssembly.Global creation failed: {}", err))
        })?;

        // Create the JavaScript Global object
        let proto = get_prototype_from_constructor(
            &context.intrinsics().constructors().webassembly_global().constructor().into(),
            StandardConstructors::webassembly_global,
            context,
        )?;

        let global_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            WebAssemblyGlobalData::new(global_id, descriptor),
        );

        Ok(global_obj.into())
    }

    /// Convert ValueType to wasmtime::ValType
    fn value_type_to_wasmtime(value_type: &ValueType) -> wasmtime::ValType {
        match value_type {
            ValueType::I32 => wasmtime::ValType::I32,
            ValueType::I64 => wasmtime::ValType::I64,
            ValueType::F32 => wasmtime::ValType::F32,
            ValueType::F64 => wasmtime::ValType::F64,
            ValueType::V128 => wasmtime::ValType::V128,
            ValueType::ExternRef => wasmtime::ValType::EXTERNREF,
            ValueType::FuncRef => wasmtime::ValType::FUNCREF,
        }
    }

    /// Convert JavaScript value to WebAssembly value
    fn js_value_to_wasm_value(
        value: &JsValue,
        value_type: &ValueType,
        context: &mut Context,
    ) -> JsResult<wasmtime::Val> {
        match value_type {
            ValueType::I32 => {
                let num = value.to_i32(context)?;
                Ok(wasmtime::Val::I32(num))
            }
            ValueType::I64 => {
                // WebAssembly i64 values are represented as BigInt in JavaScript
                if let Some(bigint) = value.as_bigint() {
                    // Convert BigInt to i64
                    let i128_val = bigint.to_i128();
                    let i64_val = i128_val as i64; // Truncate to i64
                    Ok(wasmtime::Val::I64(i64_val))
                } else {
                    // Fallback to regular number conversion
                    let num = value.to_number(context)? as i64;
                    Ok(wasmtime::Val::I64(num))
                }
            }
            ValueType::F32 => {
                let num = value.to_number(context)? as f32;
                Ok(wasmtime::Val::F32(num.to_bits()))
            }
            ValueType::F64 => {
                let num = value.to_number(context)?;
                Ok(wasmtime::Val::F64(num.to_bits()))
            }
            ValueType::V128 => {
                // V128 values are complex and require special handling
                // For now, default to zeros
                Ok(wasmtime::Val::V128(0.into()))
            }
            ValueType::ExternRef => {
                // Convert any JavaScript value to externref
                if value.is_null() {
                    Ok(wasmtime::Val::ExternRef(None))
                } else {
                    // TODO: Implement proper externref value wrapping
                    Ok(wasmtime::Val::ExternRef(None))
                }
            }
            ValueType::FuncRef => {
                // Convert JavaScript function to funcref
                if value.is_null() {
                    Ok(wasmtime::Val::FuncRef(None))
                } else {
                    // TODO: Implement proper function reference wrapping
                    Ok(wasmtime::Val::FuncRef(None))
                }
            }
        }
    }

    /// `get/set WebAssembly.Global.prototype.value`
    ///
    /// Returns or sets the value of the global.
    fn value(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let global_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Global.value called on non-object")
        })?;

        let global_data = global_obj.downcast_ref::<WebAssemblyGlobalData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Global.value called on non-Global object")
        })?;

        // If no arguments, this is a getter
        if args.is_empty() {
            // Get the runtime to access the global value
            let runtime = WebAssemblyRuntime::get_or_create(context)?;

            // TODO: Implement actual global value retrieval from wasmtime
            // For now, return default values based on type
            match global_data.descriptor().value_type {
                ValueType::I32 => Ok(JsValue::new(0)),
                ValueType::I64 => Ok(JsValue::new(0)), // Should be BigInt
                ValueType::F32 => Ok(JsValue::new(0.0)),
                ValueType::F64 => Ok(JsValue::new(0.0)),
                ValueType::V128 => Ok(JsValue::new(0)), // Should be proper V128 representation
                ValueType::ExternRef => Ok(JsValue::null()),
                ValueType::FuncRef => Ok(JsValue::null()),
            }
        } else {
            // This is a setter
            if !global_data.descriptor().mutable {
                return Err(JsNativeError::typ()
                    .with_message("Cannot set value of immutable WebAssembly.Global")
                    .into());
            }

            let new_value = args.get_or_undefined(0);

            // Get the runtime to set the global value
            let runtime = WebAssemblyRuntime::get_or_create(context)?;

            // Convert and validate the new value
            let wasm_value = Self::js_value_to_wasm_value(
                new_value,
                &global_data.descriptor().value_type,
                context,
            )?;

            // TODO: Implement actual global value setting in wasmtime

            Ok(JsValue::undefined())
        }
    }

    /// Setter for WebAssembly.Global.prototype.value
    fn set_value(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let global_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Global.value called on non-object")
        })?;

        let global_data = global_obj.downcast_ref::<WebAssemblyGlobalData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Global.value called on non-Global object")
        })?;

        if !global_data.descriptor().mutable {
            return Err(JsNativeError::typ()
                .with_message("Cannot set value of immutable WebAssembly.Global")
                .into());
        }

        let new_value = args.get_or_undefined(0);

        // Get the runtime to set the global value
        let runtime = WebAssemblyRuntime::get_or_create(context)?;

        // Convert and validate the new value
        let wasm_value = Self::js_value_to_wasm_value(
            new_value,
            &global_data.descriptor().value_type,
            context,
        )?;

        // TODO: Implement actual global value setting in wasmtime

        Ok(JsValue::undefined())
    }
}

/// Internal data for WebAssembly.Global instances
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct WebAssemblyGlobalData {
    global_id: String,
    descriptor: GlobalDescriptor,
}

impl WebAssemblyGlobalData {
    pub fn new(global_id: String, descriptor: GlobalDescriptor) -> Self {
        Self { global_id, descriptor }
    }

    pub fn global_id(&self) -> &str {
        &self.global_id
    }

    pub fn descriptor(&self) -> &GlobalDescriptor {
        &self.descriptor
    }
}

/// WebAssembly global descriptor
#[derive(Debug, Clone, Trace, Finalize)]
pub struct GlobalDescriptor {
    pub value_type: ValueType,
    pub mutable: bool,
}

/// WebAssembly value types
#[derive(Debug, Clone, Trace, Finalize)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
    V128,
    ExternRef,
    FuncRef,
}