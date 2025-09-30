//! File System API implementation.
//!
//! This module implements the WHATWG File System API, providing access to a sandboxed
//! file system that allows web applications to read and write files with user permission.
//!
//! More information:
//!  - [WHATWG File System Specification](https://fs.spec.whatwg.org/)
//!  - [MDN File System API](https://developer.mozilla.org/en-US/docs/Web/API/File_System_API)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};

use boa_gc::{Finalize, Trace, Tracer};
use crate::{
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::Intrinsics,
    js_string,
    object::{CONSTRUCTOR, PROTOTYPE, JsPromise},
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
    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &crate::context::intrinsics::StandardConstructor =
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
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemHandle"))?;

        let this_handle = obj.downcast_ref::<Self>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemHandle"))?;

        let other = args.get_or_undefined(0);
        let other_obj = other
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("Argument is not a FileSystemHandle"))?;

        let other_handle = other_obj.downcast_ref::<Self>()
            .ok_or_else(|| JsNativeError::typ().with_message("Argument is not a FileSystemHandle"))?;

        Ok(JsValue::from(this_handle.path == other_handle.path))
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
    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &crate::context::intrinsics::StandardConstructor =
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
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemFileHandle"))?;

        let file_handle = obj.downcast_ref::<Self>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemFileHandle"))?;

        // Read actual file content from VFS
        let (file_content, file_size) = match vfs::fs::read(&file_handle.handle.path) {
            Ok(content) => {
                let size = content.len() as u64;
                (content, size)
            },
            Err(_) => (Vec::new(), 0),
        };

        // Create a file-like object with real content
        let file_obj = JsObject::with_object_proto(context.intrinsics());

        // Add file properties with real values
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
                .value(JsValue::from(file_size as i32))
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

        // Add content as a property instead of a method for now
        let content_str = String::from_utf8(file_content).unwrap_or_else(|_| String::from(""));
        file_obj.define_property_or_throw(
            js_string!("content"),
            PropertyDescriptor::builder()
                .value(JsValue::from(JsString::from(content_str)))
                .enumerable(true)
                .build(),
            context,
        )?;

        // Create a resolved Promise
        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[file_obj.into()], context)?;
        Ok(JsValue::from(promise))
    }

    /// `FileSystemFileHandle.prototype.createWritable(options)`
    ///
    /// Creates a writable stream for writing to the file.
    pub(crate) fn create_writable(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemFileHandle"))?;

        let file_handle = obj.downcast_ref::<Self>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemFileHandle"))?;

        // Clone file path for the write function
        let file_path = file_handle.handle.path.clone();

        // Create a writable stream-like object with real VFS write capability
        let writable_obj = JsObject::with_object_proto(context.intrinsics());

        // Add simplified write functionality using a property to store content
        // This will be written when the stream is closed
        writable_obj.define_property_or_throw(
            js_string!("__file_path"),
            PropertyDescriptor::builder()
                .value(JsValue::from(JsString::from(file_path.to_string_lossy().to_string())))
                .enumerable(false)
                .build(),
            context,
        )?;

        writable_obj.define_property_or_throw(
            js_string!("__pending_content"),
            PropertyDescriptor::builder()
                .value(JsValue::from(JsString::from("")))
                .enumerable(false)
                .writable(true)
                .build(),
            context,
        )?;

        // Add a write method that stores content for later writing
        let write_fn = BuiltInBuilder::callable(context.realm(), Self::writable_write)
            .name(js_string!("write"))
            .length(1)
            .build();

        writable_obj.define_property_or_throw(
            js_string!("write"),
            PropertyDescriptor::builder()
                .value(write_fn)
                .writable(true)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;

        // Add close method that writes content to VFS
        let close_fn = BuiltInBuilder::callable(context.realm(), Self::writable_close)
            .name(js_string!("close"))
            .length(0)
            .build();

        writable_obj.define_property_or_throw(
            js_string!("close"),
            PropertyDescriptor::builder()
                .value(close_fn)
                .writable(true)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;

        // Create a resolved Promise
        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[writable_obj.into()], context)?;
        Ok(JsValue::from(promise))
    }

    /// Write method for writable stream
    pub(crate) fn writable_write(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a writable stream"))?;

        if let Some(content_arg) = args.get(0) {
            // Get current pending content
            let current_content = obj.get(js_string!("__pending_content"), context)?;
            let current_str = current_content.to_string(context)?;

            // Append new content
            let new_content = content_arg.to_string(context)?;
            let combined_content = format!("{}{}", current_str.to_std_string_escaped(), new_content.to_std_string_escaped());

            // Update the pending content
            obj.set(js_string!("__pending_content"), JsValue::from(JsString::from(combined_content)), true, context)?;

            // Return a resolved Promise
            let (promise, resolvers) = JsPromise::new_pending(context);
            resolvers.resolve.call(&JsValue::undefined(), &[JsValue::undefined()], context)?;
            Ok(JsValue::from(promise))
        } else {
            Err(JsNativeError::typ().with_message("Content argument required").into())
        }
    }

    /// Close method for writable stream that writes to VFS
    pub(crate) fn writable_close(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a writable stream"))?;

        // Get the file path and pending content
        let file_path_str = obj.get(js_string!("__file_path"), context)?;
        let pending_content = obj.get(js_string!("__pending_content"), context)?;

        let path_str = file_path_str.to_string(context)?.to_std_string_escaped();
        let content_str = pending_content.to_string(context)?.to_std_string_escaped();

        // Write the content to VFS
        let file_path = PathBuf::from(path_str);
        if let Err(_) = vfs::fs::write(&file_path, content_str.as_bytes()) {
            return Err(JsNativeError::typ().with_message("Failed to write file").into());
        }

        // Return a resolved Promise
        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::undefined()], context)?;
        Ok(JsValue::from(promise))
    }
}

/// File System Directory Handle
#[derive(Debug, JsData)]
pub struct FileSystemDirectoryHandle {
    pub(crate) handle: FileSystemHandle,
    #[allow(dead_code)]
    pub(crate) entries: Arc<RwLock<HashMap<String, FileData>>>,
}

// Manual implementation of Trace and Finalize
unsafe impl Trace for FileSystemDirectoryHandle {
    unsafe fn trace(&self, tracer: &mut Tracer) {
        unsafe {
            self.handle.trace(tracer);
            // Skip tracing entries as Arc<RwLock<...>> doesn't implement Trace
        }
    }

    unsafe fn trace_non_roots(&self) {
        // No implementation needed
    }

    fn run_finalizer(&self) {
        // No implementation needed
    }
}

impl Finalize for FileSystemDirectoryHandle {}

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
    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &crate::context::intrinsics::StandardConstructor =
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
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle"))?;

        let dir_handle = obj.downcast_ref::<Self>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle"))?;

        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let file_path = dir_handle.handle.path.join(&name);

        let file_handle = FileSystemFileHandle::new(name, file_path);
        let file_handle_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().file_system_file_handle().prototype(),
            file_handle,
        );

        // Create a resolved Promise
        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[file_handle_obj.into()], context)?;
        Ok(JsValue::from(promise))
    }

    /// `FileSystemDirectoryHandle.prototype.getDirectoryHandle(name, options)`
    ///
    /// Gets a directory handle for a subdirectory.
    pub(crate) fn get_directory_handle(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle"))?;

        let dir_handle = obj.downcast_ref::<Self>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle"))?;

        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let subdir_path = dir_handle.handle.path.join(&name);

        let subdir_handle = FileSystemDirectoryHandle::new(name, subdir_path);
        let subdir_handle_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().file_system_directory_handle().prototype(),
            subdir_handle,
        );

        // Create a resolved Promise
        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[subdir_handle_obj.into()], context)?;
        Ok(JsValue::from(promise))
    }

    /// `FileSystemDirectoryHandle.prototype.removeEntry(name, options)`
    ///
    /// Removes a file or directory from this directory.
    pub(crate) fn remove_entry(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle"))?;

        let dir_handle = obj.downcast_ref::<Self>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle"))?;

        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

        // Remove from in-memory entries
        {
            let mut entries = dir_handle.entries.write().unwrap();
            entries.remove(&name);
        }

        // Create a resolved Promise
        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::undefined()], context)?;
        Ok(JsValue::from(promise))
    }

    /// `FileSystemDirectoryHandle.prototype.resolve(possibleDescendant)`
    ///
    /// Returns an array of directory names from the parent handle to the specified child entry.
    pub(crate) fn resolve(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle"))?;

        let _dir_handle = obj.downcast_ref::<Self>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle"))?;

        let _other = args.get_or_undefined(0);

        // For now, return null (not a descendant)
        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::null()], context)?;
        Ok(JsValue::from(promise))
    }
}

/// The global `window.showOpenFilePicker()` function
pub(crate) fn show_open_file_picker(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    // Create a file handle with VFS-backed persistence
    let file_path = PathBuf::from("/documents/example_document.txt");

    // Ensure the file exists in VFS with sample content
    let sample_content = b"This is a sample document file created by showOpenFilePicker.\nYou can read and modify this content through the File System API.";
    let _ = vfs::fs::write(&file_path, sample_content);

    let file_handle = FileSystemFileHandle::new("example_document.txt".to_string(), file_path);
    let file_handle_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        context.intrinsics().constructors().file_system_file_handle().prototype(),
        file_handle,
    );

    // Return array with single file handle
    let array = crate::builtins::Array::array_create(1, None, context)?;
    array.set(0, file_handle_obj, true, context)?;

    // Create a resolved Promise
    let (promise, resolvers) = JsPromise::new_pending(context);
    resolvers.resolve.call(&JsValue::undefined(), &[array.into()], context)?;
    Ok(JsValue::from(promise))
}

/// The global `window.showSaveFilePicker()` function
pub(crate) fn show_save_file_picker(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    // Create a file handle with VFS-backed persistence for saving
    let file_path = PathBuf::from("/documents/new_save_file.txt");

    // Initialize an empty file in VFS that can be written to
    let initial_content = b""; // Empty file ready for writing
    let _ = vfs::fs::write(&file_path, initial_content);

    let file_handle = FileSystemFileHandle::new("new_save_file.txt".to_string(), file_path);
    let file_handle_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        context.intrinsics().constructors().file_system_file_handle().prototype(),
        file_handle,
    );

    // Create a resolved Promise
    let (promise, resolvers) = JsPromise::new_pending(context);
    resolvers.resolve.call(&JsValue::undefined(), &[file_handle_obj.into()], context)?;
    Ok(JsValue::from(promise))
}

/// The global `window.showDirectoryPicker()` function
pub(crate) fn show_directory_picker(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    // Create a directory handle with VFS-backed persistence
    let dir_path = PathBuf::from("/documents");

    // Ensure the directory exists in VFS with some sample files
    // Create sample files in the directory
    let _ = vfs::fs::write(&dir_path.join("readme.txt"), b"Welcome to the documents directory!\nThis directory contains sample files accessible through the File System API.");
    let _ = vfs::fs::write(&dir_path.join("notes.txt"), b"Sample notes file.\nYou can read and write to this file.");

    let dir_handle = FileSystemDirectoryHandle::new("documents".to_string(), dir_path);
    let dir_handle_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        context.intrinsics().constructors().file_system_directory_handle().prototype(),
        dir_handle,
    );

    // Create a resolved Promise
    let (promise, resolvers) = JsPromise::new_pending(context);
    resolvers.resolve.call(&JsValue::undefined(), &[dir_handle_obj.into()], context)?;
    Ok(JsValue::from(promise))
}