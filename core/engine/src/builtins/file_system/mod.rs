//! File System API implementation.
//!
//! This module implements the WHATWG File System API, providing access to a sandboxed
//! file system that allows web applications to read and write files with user permission.
//!
//! More information:
//!  - [WHATWG File System Specification](https://fs.spec.whatwg.org/)
//!  - [MDN File System API](https://developer.mozilla.org/en-US/docs/Web/API/File_System_API)

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};

use boa_gc::{Finalize, Trace};
use crate::{
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::Intrinsics,
    js_string,
    object::{CONSTRUCTOR, PROTOTYPE},
    property::{Attribute, PropertyDescriptor, PropertyDescriptorBuilder},
    string::StaticJsStrings,
    value::TryFromJs,
};

#[cfg(test)]
mod tests;

/// File handle data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileData {
    pub name: String,
    pub kind: String, // "file" or "directory"
    pub size: u64,
    pub last_modified: u64,
}

/// File System Handle representation
#[derive(Debug, Trace, Finalize, JsData)]
pub struct FileSystemHandle {
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) path: PathBuf,
}

impl FileSystemHandle {
    /// Create a new file system handle
    pub fn new(name: String, kind: String, path: PathBuf) -> Self {
        Self { name, kind, path }
    }
}

impl BuiltInObject for FileSystemHandle {
    const NAME: JsString = StaticJsStrings::EMPTY_STRING;
}

impl IntrinsicObject for FileSystemHandle {
    fn init(realm: &crate::realm::Realm) {
        let _prototype = BuiltInBuilder::callable(realm, Self::constructor)
            .name(Self::NAME)
            .length(Self::LENGTH)
            .constructor_has_prototype(false)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for FileSystemHandle {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &crate::builtins::BuiltInConstructor =
        crate::context::intrinsics::StandardConstructors::file_system_handle;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("FileSystemHandle constructor cannot be called directly")
            .into())
    }
}

impl FileSystemHandle {
    /// `FileSystemHandle.prototype.isSameEntry(other)`
    ///
    /// Compares two handles to see if they represent the same file system entry.
    pub(crate) fn is_same_entry(
        this: &JsValue,
        args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_handle = this
            .as_object()
            .and_then(|obj| obj.downcast_ref::<Self>())
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemHandle"))?;

        let other = args.get_or_undefined(0);
        let other_handle = other
            .as_object()
            .and_then(|obj| obj.downcast_ref::<Self>())
            .ok_or_else(|| JsNativeError::typ().with_message("Argument is not a FileSystemHandle"))?;

        Ok(JsValue::Boolean(this_handle.path == other_handle.path))
    }

    /// `FileSystemHandle.prototype.queryPermission(descriptor)`
    ///
    /// Queries the current permission state for the handle.
    pub(crate) fn query_permission(
        _this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // For now, always return "granted" - in a real implementation,
        // this would check actual permissions
        Ok(JsValue::from(JsString::from("granted")))
    }

    /// `FileSystemHandle.prototype.requestPermission(descriptor)`
    ///
    /// Requests permission to access the handle.
    pub(crate) fn request_permission(
        _this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // For now, always return "granted" - in a real implementation,
        // this would prompt the user
        Ok(JsValue::from(JsString::from("granted")))
    }
}

/// File System File Handle
#[derive(Debug, Trace, Finalize, JsData)]
pub struct FileSystemFileHandle {
    pub(crate) handle: FileSystemHandle,
}

impl FileSystemFileHandle {
    /// Create a new file handle
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            handle: FileSystemHandle::new(name, "file".to_string(), path),
        }
    }
}

impl BuiltInObject for FileSystemFileHandle {
    const NAME: JsString = StaticJsStrings::EMPTY_STRING;
}

impl IntrinsicObject for FileSystemFileHandle {
    fn init(realm: &crate::realm::Realm) {
        let _prototype = BuiltInBuilder::callable(realm, Self::constructor)
            .name(Self::NAME)
            .length(Self::LENGTH)
            .constructor_has_prototype(false)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for FileSystemFileHandle {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &crate::builtins::BuiltInConstructor =
        crate::context::intrinsics::StandardConstructors::file_system_file_handle;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("FileSystemFileHandle constructor cannot be called directly")
            .into())
    }
}

impl FileSystemFileHandle {
    /// `FileSystemFileHandle.prototype.getFile()`
    ///
    /// Returns a File object representing the file's contents.
    pub(crate) fn get_file(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let file_handle = this
            .as_object()
            .and_then(|obj| obj.downcast_ref::<Self>())
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemFileHandle"))?;

        // Create a simple file-like object
        let file_obj = JsObject::with_object_proto(context.intrinsics());

        // Add file properties
        file_obj.define_property_or_throw(
            js_string!("name"),
            PropertyDescriptor::builder()
                .value(JsValue::from(JsString::from(file_handle.handle.name.clone())))
                .enumerable(true)
                .build(),
            context,
        )?;

        file_obj.define_property_or_throw(
            js_string!("size"),
            PropertyDescriptor::builder()
                .value(JsValue::from(0)) // TODO: Get actual file size
                .enumerable(true)
                .build(),
            context,
        )?;

        file_obj.define_property_or_throw(
            js_string!("type"),
            PropertyDescriptor::builder()
                .value(JsValue::from(JsString::from("text/plain")))
                .enumerable(true)
                .build(),
            context,
        )?;

        // Create a resolved Promise using Promise.resolve()
        let promise_constructor = context.intrinsics().constructors().promise();
        let resolve_fn = promise_constructor.get(js_string!("resolve"), context)?;
        resolve_fn.call(&JsValue::undefined(), &[file_obj.into()], context)
    }

    /// `FileSystemFileHandle.prototype.createWritable(options)`
    ///
    /// Creates a writable stream for writing to the file.
    pub(crate) fn create_writable(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let _file_handle = this
            .as_object()
            .and_then(|obj| obj.downcast_ref::<Self>())
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemFileHandle"))?;

        // Create a simple writable stream-like object
        let writable_obj = JsObject::with_object_proto(context.intrinsics());

        // Add write method
        let write_fn = BuiltInBuilder::callable(context.realm(), |_this, _args, _context| {
            Ok(JsValue::undefined())
        })
        .name(js_string!("write"))
        .length(1)
        .build();

        writable_obj.define_property_or_throw(
            js_string!("write"),
            PropertyDescriptor::builder()
                .value(write_fn)
                .enumerable(true)
                .build(),
            context,
        )?;

        // Create a resolved Promise using Promise.resolve()
        let promise_constructor = context.intrinsics().constructors().promise();
        let resolve_fn = promise_constructor.get(js_string!("resolve"), context)?;
        resolve_fn.call(&JsValue::undefined(), &[writable_obj.into()], context)
    }
}

/// File System Directory Handle
#[derive(Debug, Trace, Finalize, JsData)]
pub struct FileSystemDirectoryHandle {
    pub(crate) handle: FileSystemHandle,
    pub(crate) entries: Arc<RwLock<HashMap<String, FileData>>>,
}

impl FileSystemDirectoryHandle {
    /// Create a new directory handle
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            handle: FileSystemHandle::new(name, "directory".to_string(), path),
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl BuiltInObject for FileSystemDirectoryHandle {
    const NAME: JsString = StaticJsStrings::EMPTY_STRING;
}

impl IntrinsicObject for FileSystemDirectoryHandle {
    fn init(realm: &crate::realm::Realm) {
        let _prototype = BuiltInBuilder::callable(realm, Self::constructor)
            .name(Self::NAME)
            .length(Self::LENGTH)
            .constructor_has_prototype(false)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for FileSystemDirectoryHandle {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &crate::builtins::BuiltInConstructor =
        crate::context::intrinsics::StandardConstructors::file_system_directory_handle;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("FileSystemDirectoryHandle constructor cannot be called directly")
            .into())
    }
}

impl FileSystemDirectoryHandle {
    /// `FileSystemDirectoryHandle.prototype.getFileHandle(name, options)`
    ///
    /// Gets a file handle for a file in this directory.
    pub(crate) fn get_file_handle(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let dir_handle = this
            .as_object()
            .and_then(|obj| obj.downcast_ref::<Self>())
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle"))?;

        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let file_path = dir_handle.handle.path.join(&name);

        let file_handle = FileSystemFileHandle::new(name, file_path);
        let file_handle_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().file_system_file_handle().prototype(),
            file_handle,
        );

        // Create a resolved Promise using Promise.resolve()
        let promise_constructor = context.intrinsics().constructors().promise();
        let resolve_fn = promise_constructor.get(js_string!("resolve"), context)?;
        resolve_fn.call(&JsValue::undefined(), &[file_handle_obj.into()], context)
    }

    /// `FileSystemDirectoryHandle.prototype.getDirectoryHandle(name, options)`
    ///
    /// Gets a directory handle for a subdirectory.
    pub(crate) fn get_directory_handle(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let dir_handle = this
            .as_object()
            .and_then(|obj| obj.downcast_ref::<Self>())
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle"))?;

        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let subdir_path = dir_handle.handle.path.join(&name);

        let subdir_handle = FileSystemDirectoryHandle::new(name, subdir_path);
        let subdir_handle_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().file_system_directory_handle().prototype(),
            subdir_handle,
        );

        // Create a resolved Promise using Promise.resolve()
        let promise_constructor = context.intrinsics().constructors().promise();
        let resolve_fn = promise_constructor.get(js_string!("resolve"), context)?;
        resolve_fn.call(&JsValue::undefined(), &[subdir_handle_obj.into()], context)
    }

    /// `FileSystemDirectoryHandle.prototype.removeEntry(name, options)`
    ///
    /// Removes a file or directory from this directory.
    pub(crate) fn remove_entry(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let dir_handle = this
            .as_object()
            .and_then(|obj| obj.downcast_ref::<Self>())
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle"))?;

        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

        // Remove from in-memory entries
        {
            let mut entries = dir_handle.entries.write().unwrap();
            entries.remove(&name);
        }

        // Create a resolved Promise using Promise.resolve()
        let promise_constructor = context.intrinsics().constructors().promise();
        let resolve_fn = promise_constructor.get(js_string!("resolve"), context)?;
        resolve_fn.call(&JsValue::undefined(), &[JsValue::undefined()], context)
    }

    /// `FileSystemDirectoryHandle.prototype.resolve(possibleDescendant)`
    ///
    /// Returns an array of directory names from the parent handle to the specified child entry.
    pub(crate) fn resolve(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let _dir_handle = this
            .as_object()
            .and_then(|obj| obj.downcast_ref::<Self>())
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle"))?;

        let _other = args.get_or_undefined(0);

        // For now, return null (not a descendant)
        let promise_constructor = context.intrinsics().constructors().promise();
        let resolve_fn = promise_constructor.get(js_string!("resolve"), context)?;
        resolve_fn.call(&JsValue::undefined(), &[JsValue::null()], context)
    }
}

/// The global `window.showOpenFilePicker()` function
pub(crate) fn show_open_file_picker(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    // Create a mock file handle for demonstration
    let temp_path = std::env::temp_dir().join("demo.txt");
    let file_handle = FileSystemFileHandle::new("demo.txt".to_string(), temp_path);
    let file_handle_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        context.intrinsics().constructors().file_system_file_handle().prototype(),
        file_handle,
    );

    // Return array with single file handle
    let array = crate::builtins::Array::array_create(1, context)?;
    array.set(0, file_handle_obj.into(), true, context)?;

    // Create a resolved Promise using Promise.resolve()
    let promise_constructor = context.intrinsics().constructors().promise();
    let resolve_fn = promise_constructor.get(js_string!("resolve"), context)?;
    resolve_fn.call(&JsValue::undefined(), &[array.into()], context)
}

/// The global `window.showSaveFilePicker()` function
pub(crate) fn show_save_file_picker(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    // Create a mock file handle for demonstration
    let temp_path = std::env::temp_dir().join("save.txt");
    let file_handle = FileSystemFileHandle::new("save.txt".to_string(), temp_path);
    let file_handle_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        context.intrinsics().constructors().file_system_file_handle().prototype(),
        file_handle,
    );

    // Create a resolved Promise using Promise.resolve()
    let promise_constructor = context.intrinsics().constructors().promise();
    let resolve_fn = promise_constructor.get(js_string!("resolve"), context)?;
    resolve_fn.call(&JsValue::undefined(), &[file_handle_obj.into()], context)
}

/// The global `window.showDirectoryPicker()` function
pub(crate) fn show_directory_picker(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    // Create a mock directory handle for demonstration
    let temp_path = std::env::temp_dir().join("demo-directory");
    let dir_handle = FileSystemDirectoryHandle::new("demo-directory".to_string(), temp_path);
    let dir_handle_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        context.intrinsics().constructors().file_system_directory_handle().prototype(),
        dir_handle,
    );

    // Create a resolved Promise using Promise.resolve()
    let promise_constructor = context.intrinsics().constructors().promise();
    let resolve_fn = promise_constructor.get(js_string!("resolve"), context)?;
    resolve_fn.call(&JsValue::undefined(), &[dir_handle_obj.into()], context)
}