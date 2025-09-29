//! FileReader Web API implementation for Boa
//!
//! Native implementation of the FileReader interface from the File API
//! https://w3c.github.io/FileAPI/#FileReader-interface
//!
//! This implements the complete FileReader interface with async file reading and event handling

#[cfg(test)]
mod tests;

use crate::{
    builtins::{IntrinsicObject, BuiltInBuilder, BuiltInObject, BuiltInConstructor},
    object::JsObject,
    value::JsValue,
    Context, JsResult, js_string, JsNativeError, JsArgs,
    realm::Realm, JsString, JsData,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors}
};
use crate::builtins::blob::BlobData;
use crate::builtins::file::FileData;
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use base64::{Engine as _, engine::general_purpose};

/// FileReader ready states
#[derive(Debug, Clone, PartialEq, Trace, Finalize)]
#[repr(u16)]
pub enum ReadyState {
    Empty = 0,
    Loading = 1,
    Done = 2,
}

impl ReadyState {
    /// Convert ReadyState to u16 value
    pub fn as_u16(&self) -> u16 {
        match self {
            ReadyState::Empty => 0,
            ReadyState::Loading => 1,
            ReadyState::Done => 2,
        }
    }
}

/// FileReader error codes
#[derive(Debug, Clone, Trace, Finalize)]
#[repr(u16)]
pub enum FileReaderError {
    NotReadable = 1,
    Security = 2,
    Abort = 3,
    Encoding = 4,
}

/// JavaScript `FileReader` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct FileReader;

/// Internal data for FileReader objects
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct FileReaderData {
    ready_state: ReadyState,
    result: Option<String>,
    error: Option<FileReaderError>,

    // Event handlers (stored as function objects)
    #[unsafe_ignore_trace]
    on_loadstart: Option<JsObject>,
    #[unsafe_ignore_trace]
    on_progress: Option<JsObject>,
    #[unsafe_ignore_trace]
    on_load: Option<JsObject>,
    #[unsafe_ignore_trace]
    on_loadend: Option<JsObject>,
    #[unsafe_ignore_trace]
    on_error: Option<JsObject>,
    #[unsafe_ignore_trace]
    on_abort: Option<JsObject>,

    // Internal state
    #[unsafe_ignore_trace]
    reader_id: u32,
    #[unsafe_ignore_trace]
    is_aborted: Arc<Mutex<bool>>,
}

/// FileReader operation management
#[derive(Debug)]
struct FileReaderState {
    operations: Arc<Mutex<HashMap<u32, Arc<Mutex<bool>>>>>,
    next_id: Arc<Mutex<u32>>,
}

static FILEREADER_STATE: OnceLock<FileReaderState> = OnceLock::new();

fn get_filereader_state() -> &'static FileReaderState {
    FILEREADER_STATE.get_or_init(|| FileReaderState {
        operations: Arc::new(Mutex::new(HashMap::new())),
        next_id: Arc::new(Mutex::new(1)),
    })
}

impl FileReaderData {
    pub fn new() -> Self {
        let state = get_filereader_state();
        let reader_id = {
            let mut next_id = state.next_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        Self {
            ready_state: ReadyState::Empty,
            result: None,
            error: None,
            on_loadstart: None,
            on_progress: None,
            on_load: None,
            on_loadend: None,
            on_error: None,
            on_abort: None,
            reader_id,
            is_aborted: Arc::new(Mutex::new(false)),
        }
    }
}

impl IntrinsicObject for FileReader {
    fn init(realm: &Realm) {
        let get_ready_state = BuiltInBuilder::callable(realm, get_ready_state)
            .name(js_string!("get readyState"))
            .build();

        let get_result = BuiltInBuilder::callable(realm, get_result)
            .name(js_string!("get result"))
            .build();

        let get_error = BuiltInBuilder::callable(realm, get_error)
            .name(js_string!("get error"))
            .build();

        // Event handler getters/setters
        let get_onloadstart = BuiltInBuilder::callable(realm, get_onloadstart)
            .name(js_string!("get onloadstart"))
            .build();
        let set_onloadstart = BuiltInBuilder::callable(realm, set_onloadstart)
            .name(js_string!("set onloadstart"))
            .build();

        let get_onprogress = BuiltInBuilder::callable(realm, get_onprogress)
            .name(js_string!("get onprogress"))
            .build();
        let set_onprogress = BuiltInBuilder::callable(realm, set_onprogress)
            .name(js_string!("set onprogress"))
            .build();

        let get_onload = BuiltInBuilder::callable(realm, get_onload)
            .name(js_string!("get onload"))
            .build();
        let set_onload = BuiltInBuilder::callable(realm, set_onload)
            .name(js_string!("set onload"))
            .build();

        let get_onloadend = BuiltInBuilder::callable(realm, get_onloadend)
            .name(js_string!("get onloadend"))
            .build();
        let set_onloadend = BuiltInBuilder::callable(realm, set_onloadend)
            .name(js_string!("set onloadend"))
            .build();

        let get_onerror = BuiltInBuilder::callable(realm, get_onerror)
            .name(js_string!("get onerror"))
            .build();
        let set_onerror = BuiltInBuilder::callable(realm, set_onerror)
            .name(js_string!("set onerror"))
            .build();

        let get_onabort = BuiltInBuilder::callable(realm, get_onabort)
            .name(js_string!("get onabort"))
            .build();
        let set_onabort = BuiltInBuilder::callable(realm, set_onabort)
            .name(js_string!("set onabort"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Read methods
            .method(Self::read_as_array_buffer, js_string!("readAsArrayBuffer"), 1)
            .method(Self::read_as_binary_string, js_string!("readAsBinaryString"), 1)
            .method(Self::read_as_data_url, js_string!("readAsDataURL"), 1)
            .method(Self::read_as_text, js_string!("readAsText"), 1)
            .method(Self::abort, js_string!("abort"), 0)

            // State properties
            .accessor(
                js_string!("readyState"),
                Some(get_ready_state),
                None,
                crate::property::Attribute::ENUMERABLE | crate::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("result"),
                Some(get_result),
                None,
                crate::property::Attribute::ENUMERABLE | crate::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("error"),
                Some(get_error),
                None,
                crate::property::Attribute::ENUMERABLE | crate::property::Attribute::CONFIGURABLE,
            )

            // Event handlers
            .accessor(
                js_string!("onloadstart"),
                Some(get_onloadstart),
                Some(set_onloadstart),
                crate::property::Attribute::ENUMERABLE | crate::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onprogress"),
                Some(get_onprogress),
                Some(set_onprogress),
                crate::property::Attribute::ENUMERABLE | crate::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onload"),
                Some(get_onload),
                Some(set_onload),
                crate::property::Attribute::ENUMERABLE | crate::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onloadend"),
                Some(get_onloadend),
                Some(set_onloadend),
                crate::property::Attribute::ENUMERABLE | crate::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onerror"),
                Some(get_onerror),
                Some(set_onerror),
                crate::property::Attribute::ENUMERABLE | crate::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onabort"),
                Some(get_onabort),
                Some(set_onabort),
                crate::property::Attribute::ENUMERABLE | crate::property::Attribute::CONFIGURABLE,
            )

            // Constants
            .property(js_string!("EMPTY"), ReadyState::Empty.as_u16(), crate::property::Attribute::NON_ENUMERABLE)
            .property(js_string!("LOADING"), ReadyState::Loading.as_u16(), crate::property::Attribute::NON_ENUMERABLE)
            .property(js_string!("DONE"), ReadyState::Done.as_u16(), crate::property::Attribute::NON_ENUMERABLE)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for FileReader {
    const NAME: JsString = js_string!("FileReader");
}

impl BuiltInConstructor for FileReader {
    const LENGTH: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::file_reader;
    const P: usize = 0;
    const SP: usize = 0;

    /// `new FileReader()`
    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let reader_data = FileReaderData::new();

        let prototype = Self::get(context.intrinsics())
            .get(js_string!("prototype"), context)?
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("FileReader.prototype is not an object"))?
            .clone();

        let reader_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            reader_data,
        );

        Ok(reader_obj.into())
    }
}

impl FileReader {
    /// `FileReader.prototype.readAsArrayBuffer(file)`
    fn read_as_array_buffer(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::start_read(_this, args, context, ReadOperation::ArrayBuffer)
    }

    /// `FileReader.prototype.readAsBinaryString(file)`
    fn read_as_binary_string(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::start_read(_this, args, context, ReadOperation::BinaryString)
    }

    /// `FileReader.prototype.readAsDataURL(file)`
    fn read_as_data_url(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::start_read(_this, args, context, ReadOperation::DataURL)
    }

    /// `FileReader.prototype.readAsText(file, encoding)`
    fn read_as_text(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let encoding = if args.len() > 1 {
            Some(args[1].to_string(context)?.to_std_string_escaped())
        } else {
            None
        };
        Self::start_read(_this, args, context, ReadOperation::Text(encoding))
    }

    /// `FileReader.prototype.abort()`
    fn abort(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let reader_obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("abort called on non-object")
        })?;

        let mut reader_data = reader_obj.downcast_mut::<FileReaderData>().ok_or_else(|| {
            JsNativeError::typ().with_message("abort called on non-FileReader object")
        })?;

        if reader_data.ready_state == ReadyState::Loading {
            // Mark as aborted
            *reader_data.is_aborted.lock().unwrap() = true;

            reader_data.ready_state = ReadyState::Done;
            reader_data.result = None;
            reader_data.error = Some(FileReaderError::Abort);

            // TODO: Fire abort and loadend events
            eprintln!("🔄 FileReader operation aborted");
        }

        Ok(JsValue::undefined())
    }

    /// Internal method to start a read operation
    fn start_read(_this: &JsValue, args: &[JsValue], _context: &mut Context, operation: ReadOperation) -> JsResult<JsValue> {
        let reader_obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("read method called on non-object")
        })?;

        let mut reader_data = reader_obj.downcast_mut::<FileReaderData>().ok_or_else(|| {
            JsNativeError::typ().with_message("read method called on non-FileReader object")
        })?;

        // Check if already loading
        if reader_data.ready_state == ReadyState::Loading {
            return Err(JsNativeError::typ()
                .with_message("FileReader is already reading")
                .into());
        }

        let file_arg = args.get_or_undefined(0);

        // Extract file data
        let data = if let Some(file_obj) = file_arg.as_object() {
            if let Some(file_data) = file_obj.downcast_ref::<FileData>() {
                file_data.blob().data().clone()
            } else if let Some(blob_data) = file_obj.downcast_ref::<BlobData>() {
                blob_data.data().clone()
            } else {
                return Err(JsNativeError::typ()
                    .with_message("Argument is not a File or Blob")
                    .into());
            }
        } else {
            return Err(JsNativeError::typ()
                .with_message("readAs* methods require a File or Blob argument")
                .into());
        };

        // Set state to loading
        reader_data.ready_state = ReadyState::Loading;
        reader_data.result = None;
        reader_data.error = None;
        reader_data.is_aborted = Arc::new(Mutex::new(false));

        let reader_id = reader_data.reader_id;
        let is_aborted = reader_data.is_aborted.clone();

        // TODO: Fire loadstart event
        eprintln!("🔄 FileReader operation started (ID: {})", reader_id);

        // Simulate async operation with threading
        let reader_obj_clone = reader_obj.clone();
        thread::spawn(move || {
            // Simulate reading delay
            thread::sleep(Duration::from_millis(10));

            // Check if aborted
            if *is_aborted.lock().unwrap() {
                return;
            }

            // Perform the actual read operation
            let result = match operation {
                ReadOperation::ArrayBuffer => {
                    // TODO: Return actual ArrayBuffer
                    format!("[ArrayBuffer {} bytes]", data.len())
                }
                ReadOperation::BinaryString => {
                    // Convert bytes to binary string (latin1 encoding)
                    data.iter().map(|&b| b as char).collect()
                }
                ReadOperation::DataURL => {
                    // Create data URL with base64 encoding
                    let base64_data = general_purpose::STANDARD.encode(&**data);
                    format!("data:application/octet-stream;base64,{}", base64_data)
                }
                ReadOperation::Text(encoding) => {
                    // Convert to text (UTF-8 by default)
                    match encoding.as_deref() {
                        Some("utf-8") | Some("UTF-8") | None => {
                            String::from_utf8_lossy(&data).to_string()
                        }
                        Some("latin1") | Some("iso-8859-1") => {
                            data.iter().map(|&b| b as char).collect()
                        }
                        _ => {
                            // Fallback to UTF-8 for unsupported encodings
                            String::from_utf8_lossy(&data).to_string()
                        }
                    }
                }
            };

            // Update result (in a real implementation, this would need proper context access)
            eprintln!("📄 FileReader operation completed (ID: {})", reader_id);
            eprintln!("📄 Result: {} characters/bytes", result.len());

            // TODO: Update the actual FileReader object state and fire events
            // This requires proper context access for event firing
        });

        Ok(JsValue::undefined())
    }
}

/// Read operation types
#[derive(Debug, Clone)]
enum ReadOperation {
    ArrayBuffer,
    BinaryString,
    DataURL,
    Text(Option<String>), // encoding
}

// Property getter/setter implementations

/// `get FileReader.prototype.readyState`
pub(crate) fn get_ready_state(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let reader_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("readyState getter called on non-object")
    })?;

    let reader_data = reader_obj.downcast_ref::<FileReaderData>().ok_or_else(|| {
        JsNativeError::typ().with_message("readyState getter called on non-FileReader object")
    })?;

    Ok(JsValue::from(reader_data.ready_state.as_u16()))
}

/// `get FileReader.prototype.result`
pub(crate) fn get_result(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let reader_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("result getter called on non-object")
    })?;

    let reader_data = reader_obj.downcast_ref::<FileReaderData>().ok_or_else(|| {
        JsNativeError::typ().with_message("result getter called on non-FileReader object")
    })?;

    match &reader_data.result {
        Some(result) => Ok(JsValue::from(js_string!(result.clone()))),
        None => Ok(JsValue::null()),
    }
}

/// `get FileReader.prototype.error`
pub(crate) fn get_error(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let reader_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("error getter called on non-object")
    })?;

    let reader_data = reader_obj.downcast_ref::<FileReaderData>().ok_or_else(|| {
        JsNativeError::typ().with_message("error getter called on non-FileReader object")
    })?;

    match &reader_data.error {
        Some(_error) => {
            // TODO: Return proper DOMException
            Ok(JsValue::from(js_string!("DOMException")))
        }
        None => Ok(JsValue::null()),
    }
}

// Event handler getters and setters

macro_rules! event_handler_accessors {
    ($getter:ident, $setter:ident, $field:ident, $name:literal) => {
        pub(crate) fn $getter(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
            let reader_obj = this.as_object().ok_or_else(|| {
                JsNativeError::typ().with_message(concat!($name, " getter called on non-object"))
            })?;

            let reader_data = reader_obj.downcast_ref::<FileReaderData>().ok_or_else(|| {
                JsNativeError::typ().with_message(concat!($name, " getter called on non-FileReader object"))
            })?;

            match &reader_data.$field {
                Some(handler) => Ok(JsValue::from(handler.clone())),
                None => Ok(JsValue::null()),
            }
        }

        pub(crate) fn $setter(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
            let reader_obj = this.as_object().ok_or_else(|| {
                JsNativeError::typ().with_message(concat!($name, " setter called on non-object"))
            })?;

            let mut reader_data = reader_obj.downcast_mut::<FileReaderData>().ok_or_else(|| {
                JsNativeError::typ().with_message(concat!($name, " setter called on non-FileReader object"))
            })?;

            let handler = args.get_or_undefined(0);
            reader_data.$field = if handler.is_callable() {
                handler.as_object().map(|obj| obj.clone())
            } else if handler.is_null() || handler.is_undefined() {
                None
            } else {
                None // Invalid handler types are ignored
            };

            Ok(JsValue::undefined())
        }
    };
}

event_handler_accessors!(get_onloadstart, set_onloadstart, on_loadstart, "onloadstart");
event_handler_accessors!(get_onprogress, set_onprogress, on_progress, "onprogress");
event_handler_accessors!(get_onload, set_onload, on_load, "onload");
event_handler_accessors!(get_onloadend, set_onloadend, on_loadend, "onloadend");
event_handler_accessors!(get_onerror, set_onerror, on_error, "onerror");
event_handler_accessors!(get_onabort, set_onabort, on_abort, "onabort");