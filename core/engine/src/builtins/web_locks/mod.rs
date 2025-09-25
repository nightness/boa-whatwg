//! Implementation of the Web Locks API.
//!
//! The Web Locks API allows scripts running in one tab to asynchronously acquire a lock,
//! hold it while work is performed, then release it. While held, no other script executing
//! in the same or any other tab can acquire the same lock.
//!
//! More information:
//! - [W3C Web Locks API Specification](https://w3c.github.io/web-locks/)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/Web_Locks_API)

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use boa_gc::{Finalize, Trace};
use crate::{
    builtins::{BuiltInBuilder, Array},
    context::intrinsics::Intrinsics,
    js_string,
    object::{JsObject, JsPromise},
    property::Attribute,
    realm::Realm,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, JsValue,
    native_function::NativeFunction,
};
use crate::builtins::{BuiltInConstructor, BuiltInObject, IntrinsicObject};
use crate::context::intrinsics::StandardConstructor;

/// Lock mode enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockMode {
    Exclusive,
    Shared,
}

impl From<&str> for LockMode {
    fn from(s: &str) -> Self {
        match s {
            "shared" => LockMode::Shared,
            _ => LockMode::Exclusive,
        }
    }
}

impl std::fmt::Display for LockMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockMode::Exclusive => write!(f, "exclusive"),
            LockMode::Shared => write!(f, "shared"),
        }
    }
}

/// A single lock request
#[derive(Debug, Clone)]
struct LockRequest {
    name: String,
    mode: LockMode,
    callback: JsValue,
    options: LockOptions,
    created_at: Instant,
}

/// Lock options for configuration
#[derive(Debug, Clone)]
struct LockOptions {
    mode: LockMode,
    if_available: bool,
    steal: bool,
    signal: Option<JsValue>, // AbortSignal if provided
}

impl Default for LockOptions {
    fn default() -> Self {
        Self {
            mode: LockMode::Exclusive,
            if_available: false,
            steal: false,
            signal: None,
        }
    }
}

/// Information about a held lock
#[derive(Debug, Clone)]
struct HeldLock {
    name: String,
    mode: LockMode,
    acquired_at: Instant,
}

/// Information about a pending lock request
#[derive(Debug, Clone)]
struct PendingLock {
    name: String,
    mode: LockMode,
    requested_at: Instant,
}

/// Global lock manager state
#[derive(Debug, Clone)]
struct LockManager {
    /// Currently held locks by name
    held_locks: Arc<RwLock<HashMap<String, Vec<HeldLock>>>>,
    /// Queue of pending lock requests
    pending_requests: Arc<RwLock<VecDeque<LockRequest>>>,
}

impl LockManager {
    fn new() -> Self {
        Self {
            held_locks: Arc::new(RwLock::new(HashMap::new())),
            pending_requests: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Check if a lock can be acquired immediately
    fn can_acquire_lock(&self, name: &str, mode: &LockMode) -> bool {
        let held = self.held_locks.read().unwrap();

        match held.get(name) {
            None => true, // No locks held for this name
            Some(locks) => {
                // For shared locks, can acquire if all held locks are also shared
                if *mode == LockMode::Shared {
                    locks.iter().all(|lock| lock.mode == LockMode::Shared)
                } else {
                    // For exclusive locks, cannot acquire if any lock is held
                    false
                }
            }
        }
    }

    /// Acquire a lock
    fn acquire_lock(&self, name: String, mode: LockMode) -> bool {
        let mut held = self.held_locks.write().unwrap();

        // Check if lock can be acquired while holding the write lock
        let can_acquire = match held.get(&name) {
            None => true, // No locks held for this name
            Some(locks) => {
                // For shared locks, can acquire if all held locks are also shared
                if mode == LockMode::Shared {
                    locks.iter().all(|lock| lock.mode == LockMode::Shared)
                } else {
                    // For exclusive locks, cannot acquire if any lock is held
                    false
                }
            }
        };

        if can_acquire {
            let lock = HeldLock {
                name: name.clone(),
                mode,
                acquired_at: Instant::now(),
            };

            held.entry(name).or_insert_with(Vec::new).push(lock);
            true
        } else {
            false
        }
    }

    /// Release a lock
    fn release_lock(&self, name: &str, mode: &LockMode) {
        let mut held = self.held_locks.write().unwrap();

        if let Some(locks) = held.get_mut(name) {
            locks.retain(|lock| !(lock.name == name && lock.mode == *mode));
            if locks.is_empty() {
                held.remove(name);
            }
        }
    }

    /// Get current lock state for query()
    fn get_state(&self) -> (Vec<HeldLock>, Vec<PendingLock>) {
        let held = self.held_locks.read().unwrap();
        let pending = self.pending_requests.read().unwrap();

        let held_locks: Vec<HeldLock> = held.values().flatten().cloned().collect();
        let pending_locks: Vec<PendingLock> = pending.iter().map(|req| PendingLock {
            name: req.name.clone(),
            mode: req.mode.clone(),
            requested_at: req.created_at,
        }).collect();

        (held_locks, pending_locks)
    }
}

/// `Lock` object that represents a held lock
#[derive(Debug, Clone, Finalize)]
pub struct Lock {
    name: String,
    mode: LockMode,
}

unsafe impl Trace for Lock {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // No GC'd objects in Lock, nothing to trace
    }

    unsafe fn trace_non_roots(&self) {
        // No GC'd objects in Lock, nothing to trace
    }

    fn run_finalizer(&self) {
        // No cleanup needed for Lock
    }
}

impl JsData for Lock {}

impl Lock {
    pub(crate) fn new(name: String, mode: LockMode) -> Self {
        Self { name, mode }
    }
}

#[cfg(test)]
mod tests;

/// `LockManager` object that provides the Web Locks API
#[derive(Debug, Clone, Finalize)]
pub struct LockManagerObject {
    manager: LockManager,
}

unsafe impl Trace for LockManagerObject {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // No GC'd objects in LockManagerObject, nothing to trace
    }

    unsafe fn trace_non_roots(&self) {
        // No GC'd objects in LockManagerObject, nothing to trace
    }

    fn run_finalizer(&self) {
        // No cleanup needed for LockManagerObject
    }
}

impl JsData for LockManagerObject {}

impl LockManagerObject {
    pub(crate) fn new() -> Self {
        Self {
            manager: LockManager::new(),
        }
    }
}

impl IntrinsicObject for LockManagerObject {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::request, js_string!("request"), 2)
            .method(Self::query, js_string!("query"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for LockManagerObject {
    const NAME: JsString = js_string!("LockManager");
}

impl BuiltInConstructor for LockManagerObject {
    const LENGTH: usize = 0;
    const P: usize = 0;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &StandardConstructor =
        |constructors| constructors.lock_manager();

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // LockManager constructor is not meant to be called directly
        Err(JsNativeError::typ()
            .with_message("LockManager constructor cannot be called directly")
            .into())
    }
}

// LockManager prototype methods
impl LockManagerObject {
    /// `navigator.locks.request(name, options, callback)`
    fn request(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a LockManager object")
            })?;

        let lock_manager = obj.downcast_ref::<LockManagerObject>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a LockManager object")
            })?;

        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

        // Parse arguments - can be (name, callback) or (name, options, callback)
        let (options, callback) = if args.len() >= 3 {
            // (name, options, callback)
            let options = args.get_or_undefined(1);
            let callback = args.get_or_undefined(2);
            (Some(options), callback)
        } else if args.len() >= 2 {
            // (name, callback) - check if second arg is function or options
            let second_arg = args.get_or_undefined(1);
            if second_arg.is_callable() {
                (None, second_arg)
            } else {
                // Assume it's options, callback is missing
                return Err(JsNativeError::typ()
                    .with_message("callback function is required")
                    .into());
            }
        } else {
            return Err(JsNativeError::typ()
                .with_message("name and callback are required")
                .into());
        };

        // Parse options if provided
        let lock_options = if let Some(opts) = options {
            if opts.is_object() {
                let opts_obj = opts.as_object().unwrap();

                let mode_val = opts_obj.get(js_string!("mode"), context)?;
                let mode = if mode_val.is_string() {
                    LockMode::from(mode_val.to_string(context)?.to_std_string_escaped().as_str())
                } else {
                    LockMode::Exclusive
                };

                let if_available = opts_obj.get(js_string!("ifAvailable"), context)?.to_boolean();
                let steal = opts_obj.get(js_string!("steal"), context)?.to_boolean();
                let signal = opts_obj.get(js_string!("signal"), context)?;

                LockOptions {
                    mode,
                    if_available,
                    steal,
                    signal: if signal.is_undefined() { None } else { Some(signal) },
                }
            } else {
                LockOptions::default()
            }
        } else {
            LockOptions::default()
        };

        // Create and return a Promise
        let (promise, resolvers) = JsPromise::new_pending(context);

        // Try to acquire lock immediately if possible or ifAvailable is true
        if lock_manager.manager.can_acquire_lock(&name, &lock_options.mode) || lock_options.if_available {
            if lock_manager.manager.acquire_lock(name.clone(), lock_options.mode.clone()) {
                // Lock acquired, create Lock object and call callback
                let lock_obj = JsObject::from_proto_and_data(
                    Some(context.intrinsics().constructors().object().prototype()),
                    Lock::new(name.clone(), lock_options.mode.clone())
                );

                // Set lock properties
                lock_obj.set(js_string!("name"), JsValue::from(JsString::from(name.clone())), false, context).ok();
                lock_obj.set(js_string!("mode"), JsValue::from(JsString::from(lock_options.mode.to_string())), false, context).ok();

                // Call the callback with the lock object
                let result = callback.call(&JsValue::undefined(), &[JsValue::from(lock_obj.clone())], context);

                // Release the lock after callback execution
                lock_manager.manager.release_lock(&name, &lock_options.mode);

                // Resolve the promise with the callback result
                match result {
                    Ok(callback_result) => {
                        resolvers.resolve.call(&JsValue::undefined(), &[callback_result], context).ok();
                    },
                    Err(callback_error) => {
                        let js_error = callback_error.to_opaque(context);
                        resolvers.reject.call(&JsValue::undefined(), &[js_error], context).ok();
                    }
                }
            } else if lock_options.if_available {
                // Lock not available and ifAvailable is true, resolve with null
                resolvers.resolve.call(&JsValue::undefined(), &[JsValue::null()], context).ok();
            }
        } else {
            // Would need to implement proper queuing for non-ifAvailable requests
            // For now, reject with not available error
            let error: crate::JsError = JsNativeError::typ()
                .with_message("Lock not available and queuing not yet implemented")
                .into();
            let js_error = error.to_opaque(context);
            resolvers.reject.call(&JsValue::undefined(), &[js_error], context).ok();
        }

        Ok(JsValue::from(promise))
    }

    /// `navigator.locks.query()`
    fn query(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a LockManager object")
            })?;

        let lock_manager = obj.downcast_ref::<LockManagerObject>()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a LockManager object")
            })?;

        let (held_locks, pending_locks) = lock_manager.manager.get_state();

        // Create arrays for held and pending locks
        let held_array = Array::create_array_from_list([], context);
        let pending_array = Array::create_array_from_list([], context);

        // Populate held locks array
        for (i, lock) in held_locks.iter().enumerate() {
            let lock_info = JsObject::with_object_proto(context.intrinsics());
            lock_info.set(js_string!("name"), JsValue::from(JsString::from(lock.name.clone())), false, context)?;
            lock_info.set(js_string!("mode"), JsValue::from(JsString::from(lock.mode.to_string())), false, context)?;
            held_array.set(i, JsValue::from(lock_info), false, context)?;
        }

        // Populate pending locks array
        for (i, lock) in pending_locks.iter().enumerate() {
            let lock_info = JsObject::with_object_proto(context.intrinsics());
            lock_info.set(js_string!("name"), JsValue::from(JsString::from(lock.name.clone())), false, context)?;
            lock_info.set(js_string!("mode"), JsValue::from(JsString::from(lock.mode.to_string())), false, context)?;
            pending_array.set(i, JsValue::from(lock_info), false, context)?;
        }

        // Create result object
        let result = JsObject::with_object_proto(context.intrinsics());
        result.set(js_string!("held"), JsValue::from(held_array), false, context)?;
        result.set(js_string!("pending"), JsValue::from(pending_array), false, context)?;

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(&JsValue::undefined(), &[JsValue::from(result)], context).ok();

        Ok(JsValue::from(promise))
    }

    /// Create a LockManager instance for navigator.locks
    pub fn create_lock_manager() -> JsObject {
        let lock_manager = LockManagerObject::new();
        JsObject::from_proto_and_data(None, lock_manager)
    }
}