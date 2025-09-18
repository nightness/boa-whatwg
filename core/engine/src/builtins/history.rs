//! History Web API implementation for Boa
//!
//! Native implementation of History standard
//! https://html.spec.whatwg.org/#the-history-interface

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
use std::sync::{Arc, Mutex};

/// JavaScript `History` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct History;

impl IntrinsicObject for History {
    fn init(realm: &Realm) {
        let length_func = BuiltInBuilder::callable(realm, get_length)
            .name(js_string!("get length"))
            .build();

        let state_func = BuiltInBuilder::callable(realm, get_state)
            .name(js_string!("get state"))
            .build();

        let scroll_restoration_func = BuiltInBuilder::callable(realm, get_scroll_restoration)
            .name(js_string!("get scrollRestoration"))
            .build();

        let scroll_restoration_setter_func = BuiltInBuilder::callable(realm, set_scroll_restoration)
            .name(js_string!("set scrollRestoration"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("length"),
                Some(length_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("state"),
                Some(state_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("scrollRestoration"),
                Some(scroll_restoration_func),
                Some(scroll_restoration_setter_func),
                Attribute::CONFIGURABLE,
            )
            .method(back, js_string!("back"), 0)
            .method(forward, js_string!("forward"), 0)
            .method(go, js_string!("go"), 1)
            .method(push_state, js_string!("pushState"), 3)
            .method(replace_state, js_string!("replaceState"), 3)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for History {
    const NAME: JsString = StaticJsStrings::HISTORY;
}

impl BuiltInConstructor for History {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::history;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::history,
            context,
        )?;

        let history_data = HistoryData::new();

        let history = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            history_data,
        );

        Ok(history.into())
    }
}

/// Internal data for History objects
#[derive(Debug, Trace, Finalize, JsData)]
pub struct HistoryData {
    #[unsafe_ignore_trace]
    entries: Arc<Mutex<Vec<HistoryEntry>>>,
    #[unsafe_ignore_trace]
    current_index: Arc<Mutex<i32>>,
    #[unsafe_ignore_trace]
    scroll_restoration: Arc<Mutex<String>>,
}

#[derive(Debug, Clone)]
struct HistoryEntry {
    url: String,
    title: String,
    state: Option<String>, // JSON-serialized state
}

impl HistoryData {
    fn new() -> Self {
        let initial_entry = HistoryEntry {
            url: "about:blank".to_string(),
            title: "".to_string(),
            state: None,
        };

        Self {
            entries: Arc::new(Mutex::new(vec![initial_entry])),
            current_index: Arc::new(Mutex::new(0)),
            scroll_restoration: Arc::new(Mutex::new("auto".to_string())),
        }
    }

    pub fn get_length(&self) -> i32 {
        self.entries.lock().unwrap().len() as i32
    }

    pub fn get_current_state(&self) -> Option<String> {
        let entries = self.entries.lock().unwrap();
        let index = *self.current_index.lock().unwrap() as usize;

        if index < entries.len() {
            entries[index].state.clone()
        } else {
            None
        }
    }

    pub fn get_scroll_restoration(&self) -> String {
        self.scroll_restoration.lock().unwrap().clone()
    }

    pub fn set_scroll_restoration(&self, value: String) {
        let normalized = if value == "manual" { "manual" } else { "auto" };
        *self.scroll_restoration.lock().unwrap() = normalized.to_string();
    }

    pub fn back(&self) -> bool {
        let mut index = self.current_index.lock().unwrap();
        if *index > 0 {
            *index -= 1;
            true
        } else {
            false
        }
    }

    pub fn forward(&self) -> bool {
        let mut index = self.current_index.lock().unwrap();
        let length = self.entries.lock().unwrap().len() as i32;
        if *index < length - 1 {
            *index += 1;
            true
        } else {
            false
        }
    }

    pub fn go(&self, delta: i32) -> bool {
        let mut index = self.current_index.lock().unwrap();
        let length = self.entries.lock().unwrap().len() as i32;
        let new_index = *index + delta;

        if new_index >= 0 && new_index < length {
            *index = new_index;
            true
        } else {
            false
        }
    }

    pub fn push_state(&self, state: Option<String>, title: String, url: Option<String>) {
        let mut entries = self.entries.lock().unwrap();
        let mut index = self.current_index.lock().unwrap();

        // Get current URL if not provided
        let current_url = if *index >= 0 && (*index as usize) < entries.len() {
            entries[*index as usize].url.clone()
        } else {
            "about:blank".to_string()
        };

        let new_url = url.unwrap_or(current_url);

        let new_entry = HistoryEntry {
            url: new_url,
            title,
            state,
        };

        // Remove any entries after current index
        entries.truncate(*index as usize + 1);

        // Add new entry
        entries.push(new_entry);
        *index += 1;
    }

    pub fn replace_state(&self, state: Option<String>, title: String, url: Option<String>) {
        let mut entries = self.entries.lock().unwrap();
        let index = *self.current_index.lock().unwrap();

        if index >= 0 && (index as usize) < entries.len() {
            let current_entry = &mut entries[index as usize];

            current_entry.state = state;
            current_entry.title = title;

            if let Some(new_url) = url {
                current_entry.url = new_url;
            }
        }
    }

    pub fn get_current_url(&self) -> String {
        let entries = self.entries.lock().unwrap();
        let index = *self.current_index.lock().unwrap() as usize;

        if index < entries.len() {
            entries[index].url.clone()
        } else {
            "about:blank".to_string()
        }
    }

    pub fn set_current_url(&self, url: String) {
        let mut entries = self.entries.lock().unwrap();
        let index = *self.current_index.lock().unwrap() as usize;

        if index < entries.len() {
            entries[index].url = url;
        }
    }
}

/// `History.prototype.length` getter
fn get_length(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("History.prototype.length called on non-object")
    })?;

    if let Some(history) = this_obj.downcast_ref::<HistoryData>() {
        Ok(JsValue::from(history.get_length()))
    } else {
        Err(JsNativeError::typ()
            .with_message("History.prototype.length called on non-History object")
            .into())
    }
}

/// `History.prototype.state` getter
fn get_state(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("History.prototype.state called on non-object")
    })?;

    if let Some(history) = this_obj.downcast_ref::<HistoryData>() {
        if let Some(state_json) = history.get_current_state() {
            // Parse JSON state
            let parse_result = context.eval(boa_engine::Source::from_bytes(&format!("JSON.parse('{}')", state_json)));
            match parse_result {
                Ok(value) => Ok(value),
                Err(_) => Ok(JsValue::null()),
            }
        } else {
            Ok(JsValue::null())
        }
    } else {
        Err(JsNativeError::typ()
            .with_message("History.prototype.state called on non-History object")
            .into())
    }
}

/// `History.prototype.scrollRestoration` getter
fn get_scroll_restoration(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("History.prototype.scrollRestoration called on non-object")
    })?;

    if let Some(history) = this_obj.downcast_ref::<HistoryData>() {
        Ok(JsString::from(history.get_scroll_restoration()).into())
    } else {
        Err(JsNativeError::typ()
            .with_message("History.prototype.scrollRestoration called on non-History object")
            .into())
    }
}

/// `History.prototype.scrollRestoration` setter
fn set_scroll_restoration(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("History.prototype.scrollRestoration setter called on non-object")
    })?;

    if let Some(history) = this_obj.downcast_ref::<HistoryData>() {
        let value = args.get_or_undefined(0).to_string(context)?;
        history.set_scroll_restoration(value.to_std_string_escaped());
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("History.prototype.scrollRestoration setter called on non-History object")
            .into())
    }
}

/// `History.prototype.back()`
fn back(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("History.prototype.back called on non-object")
    })?;

    if let Some(history) = this_obj.downcast_ref::<HistoryData>() {
        history.back();
        // In a real implementation, this would trigger pageswap event and navigation
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("History.prototype.back called on non-History object")
            .into())
    }
}

/// `History.prototype.forward()`
fn forward(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("History.prototype.forward called on non-object")
    })?;

    if let Some(history) = this_obj.downcast_ref::<HistoryData>() {
        history.forward();
        // In a real implementation, this would trigger pageswap event and navigation
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("History.prototype.forward called on non-History object")
            .into())
    }
}

/// `History.prototype.go(delta)`
fn go(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("History.prototype.go called on non-object")
    })?;

    if let Some(history) = this_obj.downcast_ref::<HistoryData>() {
        let delta = args.get_or_undefined(0).to_i32(context)?;
        history.go(delta);
        // In a real implementation, this would trigger pageswap event and navigation
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("History.prototype.go called on non-History object")
            .into())
    }
}

/// `History.prototype.pushState(state, title, url)`
fn push_state(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("History.prototype.pushState called on non-object")
    })?;

    if let Some(history) = this_obj.downcast_ref::<HistoryData>() {
        let state = args.get_or_undefined(0);
        let title = args.get_or_undefined(1).to_string(context)?;
        let url = args.get(2);

        // Serialize state to JSON if not null/undefined
        let state_json = if state.is_null() || state.is_undefined() {
            None
        } else {
            // Use JSON.stringify to serialize state
            let stringify_result = context.eval(boa_engine::Source::from_bytes(&format!("JSON.stringify({})", state.display())));
            match stringify_result {
                Ok(json_val) => Some(json_val.to_string(context)?.to_std_string_escaped()),
                Err(_) => None,
            }
        };

        let url_string = if let Some(url_val) = url {
            if !url_val.is_null() && !url_val.is_undefined() {
                Some(url_val.to_string(context)?.to_std_string_escaped())
            } else {
                None
            }
        } else {
            None
        };

        history.push_state(state_json, title.to_std_string_escaped(), url_string);
        // In a real implementation, this would trigger pageswap event and update URL
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("History.prototype.pushState called on non-History object")
            .into())
    }
}

/// `History.prototype.replaceState(state, title, url)`
fn replace_state(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("History.prototype.replaceState called on non-object")
    })?;

    if let Some(history) = this_obj.downcast_ref::<HistoryData>() {
        let state = args.get_or_undefined(0);
        let title = args.get_or_undefined(1).to_string(context)?;
        let url = args.get(2);

        // Serialize state to JSON if not null/undefined
        let state_json = if state.is_null() || state.is_undefined() {
            None
        } else {
            // Use JSON.stringify to serialize state
            let stringify_result = context.eval(boa_engine::Source::from_bytes(&format!("JSON.stringify({})", state.display())));
            match stringify_result {
                Ok(json_val) => Some(json_val.to_string(context)?.to_std_string_escaped()),
                Err(_) => None,
            }
        };

        let url_string = if let Some(url_val) = url {
            if !url_val.is_null() && !url_val.is_undefined() {
                Some(url_val.to_string(context)?.to_std_string_escaped())
            } else {
                None
            }
        } else {
            None
        };

        history.replace_state(state_json, title.to_std_string_escaped(), url_string);
        // In a real implementation, this would trigger pageswap event and update URL
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("History.prototype.replaceState called on non-History object")
            .into())
    }
}