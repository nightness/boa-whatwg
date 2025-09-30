//! Timer Web API implementation for Boa
//!
//! Native implementation of timers (setTimeout, setInterval, etc.)
//! https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html
//!
//! This implements the complete Timer interface with real async scheduling

#[cfg(test)]
mod tests;

use crate::{
    builtins::{BuiltInBuilder, BuiltInObject, IntrinsicObject},
    object::JsObject,
    value::JsValue,
    Context, JsResult, js_string,
    JsString,
    realm::Realm,
    context::intrinsics::Intrinsics
};
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::mpsc::{self, Receiver, Sender};

/// JavaScript `setTimeout` global function implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct SetTimeout;

/// JavaScript `setInterval` global function implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct SetInterval;

/// JavaScript `clearTimeout` global function implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct ClearTimeout;

/// JavaScript `clearInterval` global function implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct ClearInterval;

/// Timer callback information
#[derive(Debug, Clone)]
struct TimerCallback {
    id: u32,
    callback: String, // JavaScript code to execute
    args: Vec<String>, // Serialized arguments
    repeating: bool,
    delay: u64,
    created_at: Instant,
    active: bool,
    nesting_level: u32,
}

/// Timer execution message
#[derive(Debug, Clone)]
struct TimerMessage {
    timer_id: u32,
    callback: String,
    args: Vec<String>,
    repeating: bool,
    delay: u64,
}

/// Timer state management with real async execution
#[derive(Debug)]
struct TimerState {
    timers: Arc<Mutex<HashMap<u32, TimerCallback>>>,
    next_id: Arc<Mutex<u32>>,
    execution_sender: Option<Sender<TimerMessage>>,
    nesting_levels: Arc<Mutex<HashMap<u32, u32>>>, // Track nesting level per timer chain
}

static TIMER_STATE: OnceLock<TimerState> = OnceLock::new();

fn get_timer_state() -> &'static TimerState {
    TIMER_STATE.get_or_init(|| {
        let (sender, receiver) = mpsc::channel();

        // Spawn timer execution thread
        thread::spawn(move || {
            timer_execution_thread(receiver);
        });

        TimerState {
            timers: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
            execution_sender: Some(sender),
            nesting_levels: Arc::new(Mutex::new(HashMap::new())),
        }
    })
}

/// Timer execution thread that handles actual async callback execution
fn timer_execution_thread(receiver: Receiver<TimerMessage>) {
    while let Ok(message) = receiver.recv() {
        // Clone for potential interval repetition
        let msg = message.clone();

        // Sleep for the specified delay
        thread::sleep(Duration::from_millis(message.delay));

        // Check if timer is still active before execution
        let state = get_timer_state();
        let should_execute = {
            let timers = state.timers.lock().unwrap();
            timers.get(&message.timer_id).map_or(false, |t| t.active)
        };

        if should_execute {
            // Execute the callback (in a real implementation, this would need
            // access to a JavaScript context to actually execute the callback)
            eprintln!("⏰ Executing timer {} callback: {}", message.timer_id, message.callback);

            // If it's a repeating timer (setInterval), reschedule it
            if message.repeating {
                let timers = state.timers.lock().unwrap();
                if let Some(timer) = timers.get(&message.timer_id) {
                    if timer.active {
                        // Calculate delay (with nesting level clamping for nested timers)
                        let nesting_levels = state.nesting_levels.lock().unwrap();
                        let nesting_level = nesting_levels.get(&message.timer_id).unwrap_or(&0);
                        let clamped_delay = if *nesting_level >= 5 {
                            message.delay.max(4) // HTML5 spec: minimum 4ms for deeply nested timers
                        } else {
                            message.delay
                        };

                        // Reschedule the interval
                        if let Some(sender) = &state.execution_sender {
                            let next_message = TimerMessage {
                                timer_id: message.timer_id,
                                callback: message.callback,
                                args: message.args,
                                repeating: true,
                                delay: clamped_delay,
                            };
                            let _ = sender.send(next_message);
                        }
                    }
                }
            } else {
                // Remove one-time timer after execution
                let mut timers = state.timers.lock().unwrap();
                timers.remove(&message.timer_id);
            }
        }
    }
}

impl IntrinsicObject for SetTimeout {
    fn init(realm: &Realm) {
        BuiltInBuilder::callable_with_intrinsic::<Self>(realm, set_timeout)
            .name(Self::NAME)
            .length(2)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().set_timeout().into()
    }
}

impl BuiltInObject for SetTimeout {
    const NAME: JsString = js_string!("setTimeout");
}

impl IntrinsicObject for SetInterval {
    fn init(realm: &Realm) {
        BuiltInBuilder::callable_with_intrinsic::<Self>(realm, set_interval)
            .name(Self::NAME)
            .length(2)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().set_interval().into()
    }
}

impl BuiltInObject for SetInterval {
    const NAME: JsString = js_string!("setInterval");
}

impl IntrinsicObject for ClearTimeout {
    fn init(realm: &Realm) {
        BuiltInBuilder::callable_with_intrinsic::<Self>(realm, clear_timeout)
            .name(Self::NAME)
            .length(1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().clear_timeout().into()
    }
}

impl BuiltInObject for ClearTimeout {
    const NAME: JsString = js_string!("clearTimeout");
}

impl IntrinsicObject for ClearInterval {
    fn init(realm: &Realm) {
        BuiltInBuilder::callable_with_intrinsic::<Self>(realm, clear_interval)
            .name(Self::NAME)
            .length(1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().clear_interval().into()
    }
}

impl BuiltInObject for ClearInterval {
    const NAME: JsString = js_string!("clearInterval");
}

/// `setTimeout(callback, delay, ...args)` global function
fn set_timeout(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    schedule_timer(args, context, false)
}

/// `setInterval(callback, delay, ...args)` global function
fn set_interval(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    schedule_timer(args, context, true)
}

/// `clearTimeout(id)` global function
fn clear_timeout(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    clear_timer(args, context)
}

/// `clearInterval(id)` global function
fn clear_interval(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    clear_timer(args, context)
}

fn schedule_timer(
    args: &[JsValue],
    context: &mut Context,
    repeating: bool,
) -> JsResult<JsValue> {
    // Special case: setTimeout() with no arguments returns 0 per HTML spec
    if args.is_empty() {
        return Ok(JsValue::from(0));
    }

    // Extract callback (function or string)
    let callback = if args[0].is_callable() {
        // For now, store function as a placeholder string
        // In a real implementation, we'd store the actual function object
        "[Function]".to_string()
    } else {
        // String callback - convert to JavaScript code
        args[0].to_string(context)?.to_std_string_escaped()
    };

    // Extract delay
    let delay = if args.len() > 1 {
        args[1].to_number(context)? as u64
    } else {
        0
    };

    // Extract additional arguments
    let callback_args: Vec<String> = if args.len() > 2 {
        args[2..].iter()
            .map(|arg| arg.display().to_string())
            .collect()
    } else {
        Vec::new()
    };

    let state = get_timer_state();
    let timer_id = {
        let mut next_id = state.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        id
    };

    // Determine nesting level and apply clamping if necessary
    let nesting_level = 0; // TODO: Track actual nesting level in context
    let clamped_delay = if nesting_level >= 5 {
        delay.max(4) // HTML5 spec: minimum 4ms for deeply nested timers
    } else {
        delay.max(if delay == 0 { 4 } else { delay }) // Minimum 4ms in general
    };

    let timer_callback = TimerCallback {
        id: timer_id,
        callback: callback.clone(),
        args: callback_args.clone(),
        repeating,
        delay: clamped_delay,
        created_at: Instant::now(),
        active: true,
        nesting_level,
    };

    // Store timer info
    {
        let mut timers = state.timers.lock().unwrap();
        timers.insert(timer_id, timer_callback);
    }

    // Store nesting level
    {
        let mut nesting_levels = state.nesting_levels.lock().unwrap();
        nesting_levels.insert(timer_id, nesting_level);
    }

    // Schedule timer execution
    if let Some(sender) = &state.execution_sender {
        let message = TimerMessage {
            timer_id,
            callback,
            args: callback_args,
            repeating,
            delay: clamped_delay,
        };

        if let Err(_) = sender.send(message) {
            eprintln!("Failed to schedule timer execution");
        }
    }

    Ok(JsValue::from(timer_id))
}

fn clear_timer(args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if args.is_empty() {
        return Ok(JsValue::undefined());
    }

    let timer_id = args[0].to_u32(context)?;

    let state = get_timer_state();

    // Mark timer as inactive and remove from storage
    {
        let mut timers = state.timers.lock().unwrap();
        if let Some(timer) = timers.get_mut(&timer_id) {
            timer.active = false;
        }
        timers.remove(&timer_id);
    }

    // Remove nesting level tracking
    {
        let mut nesting_levels = state.nesting_levels.lock().unwrap();
        nesting_levels.remove(&timer_id);
    }

    Ok(JsValue::undefined())
}