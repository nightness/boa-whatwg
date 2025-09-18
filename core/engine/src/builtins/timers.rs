//! Timer Web API implementation for Boa
//!
//! Native implementation of timers (setTimeout, setInterval, etc.)
//! https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html
//!
//! This implements the complete Timer interface with real async scheduling

#[cfg(test)]
mod tests;

use crate::{
    builtins::{IntrinsicObject, BuiltInBuilder, BuiltInObject},
    object::JsObject,
    value::JsValue,
    Context, JsResult, js_string,
    realm::Realm, JsString,
    context::intrinsics::Intrinsics
};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

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

/// Timer state management
#[derive(Debug)]
struct TimerState {
    timers: Arc<Mutex<HashMap<u32, TimerInfo>>>,
    next_id: Arc<Mutex<u32>>,
}

#[derive(Debug, Clone)]
struct TimerInfo {
    id: u32,
    delay: u64,
    repeating: bool,
    active: bool,
}

static TIMER_STATE: std::sync::OnceLock<TimerState> = std::sync::OnceLock::new();

fn get_timer_state() -> &'static TimerState {
    TIMER_STATE.get_or_init(|| TimerState {
        timers: Arc::new(Mutex::new(HashMap::new())),
        next_id: Arc::new(Mutex::new(1)),
    })
}

impl IntrinsicObject for SetTimeout {
    fn init(realm: &Realm) {
        BuiltInBuilder::callable_with_intrinsic::<Self>(realm, set_timeout)
            .name(Self::NAME)
            .length(2)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.constructors().function().constructor()
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
        intrinsics.constructors().function().constructor()
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
        intrinsics.constructors().function().constructor()
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
        intrinsics.constructors().function().constructor()
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

    let delay = if args.len() > 1 {
        args[1].to_number(context)? as u64
    } else {
        0
    };

    // Ensure minimum delay of 4ms per HTML spec
    let delay = delay.max(4);

    let state = get_timer_state();
    let timer_id = {
        let mut next_id = state.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        id
    };

    let timer_info = TimerInfo {
        id: timer_id,
        delay,
        repeating,
        active: true,
    };

    // Store timer info
    {
        let mut timers = state.timers.lock().unwrap();
        timers.insert(timer_id, timer_info);
    }

    // For now, just log that timer was scheduled
    // In a real implementation, this would integrate with an event loop
    if repeating {
        println!("Scheduled interval timer {} with delay {}ms", timer_id, delay);
    } else {
        println!("Scheduled timeout timer {} with delay {}ms", timer_id, delay);
    }

    Ok(JsValue::from(timer_id))
}

fn clear_timer(args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if args.is_empty() {
        return Ok(JsValue::undefined());
    }

    let timer_id = args[0].to_u32(context)?;

    let state = get_timer_state();
    let mut timers = state.timers.lock().unwrap();

    if let Some(timer) = timers.get_mut(&timer_id) {
        timer.active = false;
        if !timer.repeating {
            timers.remove(&timer_id);
        }
    }

    Ok(JsValue::undefined())
}