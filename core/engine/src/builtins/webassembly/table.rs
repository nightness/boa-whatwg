//! WebAssembly Table implementation for Boa
//!
//! Implementation of the WebAssembly.Table interface according to
//! the W3C WebAssembly JavaScript API specification
//! https://webassembly.github.io/spec/js-api/#tables

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

/// JavaScript `WebAssembly.Table` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct WebAssemblyTable;

impl IntrinsicObject for WebAssemblyTable {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(js_string!("length"), Self::length, Attribute::READONLY)
            .method(Self::get, js_string!("get"), 1)
            .method(Self::set, js_string!("set"), 2)
            .method(Self::grow, js_string!("grow"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for WebAssemblyTable {
    const NAME: JsString = StaticJsStrings::WEBASSEMBLY_TABLE;
}

impl BuiltInConstructor for WebAssemblyTable {
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::webassembly_table;

    /// `WebAssembly.Table(descriptor, value?)`
    ///
    /// The WebAssembly.Table constructor creates a new Table object
    /// which is a JavaScript wrapper for a WebAssembly table instance.
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("WebAssembly.Table constructor requires 'new'")
                .into());
        }

        let descriptor = args.get_or_undefined(0);
        let value = args.get_or_undefined(1);

        // Parse the table descriptor
        let table_descriptor = Self::parse_table_descriptor(descriptor, context)?;

        // Create the table using wasmtime
        Self::create_table(table_descriptor, value, context)
    }
}

impl WebAssemblyTable {
    /// Parse a WebAssembly table descriptor object
    fn parse_table_descriptor(
        descriptor: &JsValue,
        context: &mut Context,
    ) -> JsResult<TableDescriptor> {
        let desc_obj = descriptor.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Table descriptor must be an object")
        })?;

        // Get element type (required)
        let element = desc_obj.get(js_string!("element"), context)?;
        let element_str = element.to_string(context)?;
        let element_type = match element_str.to_std_string_escaped().as_str() {
            "anyfunc" | "funcref" => ElementType::FuncRef,
            "externref" => ElementType::ExternRef,
            _ => return Err(JsNativeError::typ()
                .with_message("WebAssembly.Table element must be 'funcref' or 'externref'")
                .into())
        };

        // Get initial size (required)
        let initial = desc_obj.get(js_string!("initial"), context)?
            .to_u32(context)?;

        // Get maximum size (optional)
        let maximum = if let Ok(max_val) = desc_obj.get(js_string!("maximum"), context) {
            if !max_val.is_undefined() {
                Some(max_val.to_u32(context)?)
            } else {
                None
            }
        } else {
            None
        };

        // Validate size limits
        if let Some(max) = maximum {
            if initial > max {
                return Err(JsNativeError::range()
                    .with_message("WebAssembly.Table initial size exceeds maximum")
                    .into());
            }
        }

        // Theoretical maximum table size (implementation-defined)
        const MAX_TABLE_SIZE: u32 = 1_000_000; // 1M elements

        if initial > MAX_TABLE_SIZE {
            return Err(JsNativeError::range()
                .with_message("WebAssembly.Table initial size exceeds implementation limit")
                .into());
        }

        if let Some(max) = maximum {
            if max > MAX_TABLE_SIZE {
                return Err(JsNativeError::range()
                    .with_message("WebAssembly.Table maximum size exceeds implementation limit")
                    .into());
            }
        }

        Ok(TableDescriptor {
            element: element_type,
            initial,
            maximum,
        })
    }

    /// Create a WebAssembly.Table object
    fn create_table(
        descriptor: TableDescriptor,
        _value: &JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Get the WebAssembly runtime
        let runtime = WebAssemblyRuntime::get_or_create(context)?;

        // Convert to wasmtime TableType
        let wasm_type = match descriptor.element {
            ElementType::FuncRef => wasmtime::ValType::FuncRef,
            ElementType::ExternRef => wasmtime::ValType::ExternRef,
        };

        let table_type = wasmtime::TableType::new(
            wasm_type,
            descriptor.initial,
            descriptor.maximum,
        );

        // Create initial value for the table
        let init_val = match descriptor.element {
            ElementType::FuncRef => wasmtime::Val::FuncRef(None),
            ElementType::ExternRef => wasmtime::Val::ExternRef(None),
        };

        // Create the table in wasmtime
        let table_id = runtime.create_table(table_type, init_val).map_err(|err| {
            JsNativeError::typ()
                .with_message(format!("WebAssembly.Table creation failed: {}", err))
        })?;

        // Create the JavaScript Table object
        let proto = get_prototype_from_constructor(
            &context.intrinsics().constructors().webassembly_table().constructor().into(),
            StandardConstructors::webassembly_table,
            context,
        )?;

        let table_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            WebAssemblyTableData::new(table_id, descriptor),
        );

        Ok(table_obj.into())
    }

    /// `get WebAssembly.Table.prototype.length`
    ///
    /// Returns the current size of the table.
    fn length(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let table_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Table.length called on non-object")
        })?;

        let table_data = table_obj.downcast_ref::<WebAssemblyTableData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Table.length called on non-Table object")
        })?;

        // For now, return the initial size
        // TODO: Implement actual table size tracking
        Ok(JsValue::new(table_data.descriptor().initial))
    }

    /// `WebAssembly.Table.prototype.get(index)`
    ///
    /// Returns the element stored at the given index.
    fn get(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let table_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Table.get called on non-object")
        })?;

        let table_data = table_obj.downcast_ref::<WebAssemblyTableData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Table.get called on non-Table object")
        })?;

        let index = args.get_or_undefined(0).to_u32(context)?;

        // Validate index bounds
        if index >= table_data.descriptor().initial {
            return Err(JsNativeError::range()
                .with_message("WebAssembly.Table.get index out of bounds")
                .into());
        }

        // Get the runtime to access the table
        let runtime = WebAssemblyRuntime::get_or_create(context)?;

        // TODO: Implement actual table element retrieval
        // For now, return null for all slots
        Ok(JsValue::null())
    }

    /// `WebAssembly.Table.prototype.set(index, value)`
    ///
    /// Sets the element at the given index to the given value.
    fn set(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let table_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Table.set called on non-object")
        })?;

        let table_data = table_obj.downcast_ref::<WebAssemblyTableData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Table.set called on non-Table object")
        })?;

        let index = args.get_or_undefined(0).to_u32(context)?;
        let value = args.get_or_undefined(1);

        // Validate index bounds
        if index >= table_data.descriptor().initial {
            return Err(JsNativeError::range()
                .with_message("WebAssembly.Table.set index out of bounds")
                .into());
        }

        // Validate value type
        match table_data.descriptor().element {
            ElementType::FuncRef => {
                if !value.is_null() && !value.is_undefined() {
                    // TODO: Validate that value is a WebAssembly function
                }
            }
            ElementType::ExternRef => {
                // Any JavaScript value is valid for externref
            }
        }

        // Get the runtime to modify the table
        let runtime = WebAssemblyRuntime::get_or_create(context)?;

        // TODO: Implement actual table element setting
        Ok(JsValue::undefined())
    }

    /// `WebAssembly.Table.prototype.grow(delta, value?)`
    ///
    /// Increases the size of the table by delta elements.
    fn grow(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let table_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Table.grow called on non-object")
        })?;

        let table_data = table_obj.downcast_ref::<WebAssemblyTableData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("WebAssembly.Table.grow called on non-Table object")
        })?;

        let delta = args.get_or_undefined(0).to_u32(context)?;
        let _value = args.get_or_undefined(1);

        // Get the runtime to grow the table
        let runtime = WebAssemblyRuntime::get_or_create(context)?;

        // TODO: Implement actual table growth
        // For now, return the previous size
        Ok(JsValue::new(table_data.descriptor().initial))
    }
}

/// Internal data for WebAssembly.Table instances
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct WebAssemblyTableData {
    table_id: String,
    descriptor: TableDescriptor,
}

impl WebAssemblyTableData {
    pub fn new(table_id: String, descriptor: TableDescriptor) -> Self {
        Self { table_id, descriptor }
    }

    pub fn table_id(&self) -> &str {
        &self.table_id
    }

    pub fn descriptor(&self) -> &TableDescriptor {
        &self.descriptor
    }
}

/// WebAssembly table descriptor
#[derive(Debug, Clone, Trace, Finalize)]
pub struct TableDescriptor {
    pub element: ElementType,
    pub initial: u32,
    pub maximum: Option<u32>,
}

/// WebAssembly table element type
#[derive(Debug, Clone, Copy, Trace, Finalize)]
pub enum ElementType {
    FuncRef,
    ExternRef,
}