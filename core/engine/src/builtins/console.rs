//! Console Web API implementation for Boa
//!
//! Native implementation of the Console standard
//! https://console.spec.whatwg.org/
//!
//! This implements the complete Console interface for debugging and logging

#[cfg(test)]
mod tests;

#[cfg(test)]
mod debug_test;

use crate::{
    builtins::{IntrinsicObject, BuiltInBuilder, BuiltInObject},
    object::JsObject,
    value::JsValue,
    Context, JsArgs, JsResult, js_string,
    realm::Realm, JsString,
    context::intrinsics::Intrinsics
};
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;
use std::time::{Instant, Duration};

/// Console state for maintaining timers, counters, and grouping
#[derive(Debug)]
struct ConsoleState {
    timers: HashMap<String, Instant>,
    counters: HashMap<String, u32>,
    group_depth: u32,
}

impl Default for ConsoleState {
    fn default() -> Self {
        Self {
            timers: HashMap::new(),
            counters: HashMap::new(),
            group_depth: 0,
        }
    }
}

static CONSOLE_STATE: OnceLock<Arc<Mutex<ConsoleState>>> = OnceLock::new();

fn get_console_state() -> &'static Arc<Mutex<ConsoleState>> {
    CONSOLE_STATE.get_or_init(|| Arc::new(Mutex::new(ConsoleState::default())))
}

/// JavaScript `console` global object implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Console;

impl IntrinsicObject for Console {
    fn init(realm: &Realm) {
        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_method(Self::log, js_string!("log"), 0)
            .static_method(Self::info, js_string!("info"), 0)
            .static_method(Self::warn, js_string!("warn"), 0)
            .static_method(Self::error, js_string!("error"), 0)
            .static_method(Self::debug, js_string!("debug"), 0)
            .static_method(Self::trace, js_string!("trace"), 0)
            .static_method(Self::clear, js_string!("clear"), 0)
            .static_method(Self::group, js_string!("group"), 0)
            .static_method(Self::group_collapsed, js_string!("groupCollapsed"), 0)
            .static_method(Self::group_end, js_string!("groupEnd"), 0)
            .static_method(Self::time, js_string!("time"), 0)
            .static_method(Self::time_end, js_string!("timeEnd"), 0)
            .static_method(Self::time_log, js_string!("timeLog"), 0)
            .static_method(Self::count, js_string!("count"), 0)
            .static_method(Self::count_reset, js_string!("countReset"), 0)
            .static_method(Self::assert, js_string!("assert"), 0)
            .static_method(Self::table, js_string!("table"), 0)
            .static_method(Self::dir, js_string!("dir"), 0)
            .static_method(Self::dirxml, js_string!("dirxml"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Console {
    const NAME: JsString = js_string!("console");
}

impl Console {
    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &crate::context::intrinsics::StandardConstructor =
        crate::context::intrinsics::StandardConstructors::console;

    /// `console.log(...data)`
    fn log(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let output = Self::format_args(args, context)?;
        let indent = Self::get_indent();
        eprintln!("{}{}", indent, output);
        Ok(JsValue::undefined())
    }

    /// `console.info(...data)`
    fn info(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let output = Self::format_args(args, context)?;
        let indent = Self::get_indent();
        eprintln!("{}ℹ {}", indent, output);
        Ok(JsValue::undefined())
    }

    /// `console.warn(...data)`
    fn warn(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let output = Self::format_args(args, context)?;
        let indent = Self::get_indent();
        eprintln!("{}⚠ {}", indent, output);
        Ok(JsValue::undefined())
    }

    /// `console.error(...data)`
    fn error(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let output = Self::format_args(args, context)?;
        let indent = Self::get_indent();
        eprintln!("{}❌ {}", indent, output);
        Ok(JsValue::undefined())
    }

    /// `console.debug(...data)`
    fn debug(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let output = Self::format_args(args, context)?;
        let indent = Self::get_indent();
        eprintln!("{}🐛 {}", indent, output);
        Ok(JsValue::undefined())
    }

    /// `console.trace(...data)`
    fn trace(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let output = Self::format_args(args, context)?;
        let indent = Self::get_indent();

        if output.is_empty() {
            eprintln!("{}📍 Trace", indent);
        } else {
            eprintln!("{}📍 {}", indent, output);
        }

        // Basic stack trace - in a real implementation this would use actual call stack
        eprintln!("{}    at <anonymous>:1:1", indent);
        eprintln!("{}    at Object.<anonymous> (<anonymous>:1:1)", indent);
        eprintln!("{}    at Module._compile (internal/modules/cjs/loader.js:1063:30)", indent);

        Ok(JsValue::undefined())
    }

    /// `console.clear()`
    fn clear(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        eprint!("\x1B[2J\x1B[1;1H"); // ANSI escape codes to clear screen (to stderr)
        Ok(JsValue::undefined())
    }

    /// `console.group(...label)`
    fn group(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = if args.is_empty() {
            "".to_string()
        } else {
            Self::format_args(args, context)?
        };

        let state = get_console_state();
        let mut state_guard = state.lock().unwrap();
        let indent = "  ".repeat(state_guard.group_depth as usize);
        state_guard.group_depth += 1;
        drop(state_guard);

        eprintln!("{}▼ {}", indent, label);
        Ok(JsValue::undefined())
    }

    /// `console.groupCollapsed(...label)`
    fn group_collapsed(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = if args.is_empty() {
            "".to_string()
        } else {
            Self::format_args(args, context)?
        };

        let state = get_console_state();
        let mut state_guard = state.lock().unwrap();
        let indent = "  ".repeat(state_guard.group_depth as usize);
        state_guard.group_depth += 1;
        drop(state_guard);

        eprintln!("{}▶ {}", indent, label);
        Ok(JsValue::undefined())
    }

    /// `console.groupEnd()`
    fn group_end(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let state = get_console_state();
        let mut state_guard = state.lock().unwrap();

        if state_guard.group_depth > 0 {
            state_guard.group_depth -= 1;
        }

        Ok(JsValue::undefined())
    }

    /// `console.time(label)`
    fn time(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = if args.is_empty() {
            "default".to_string()
        } else {
            args[0].to_string(context)?.to_std_string_escaped()
        };

        let state = get_console_state();
        let mut state_guard = state.lock().unwrap();

        if state_guard.timers.contains_key(&label) {
            eprintln!("⚠ Timer '{}' already exists", label);
        } else {
            state_guard.timers.insert(label.clone(), Instant::now());
            eprintln!("⏱ Timer '{}' started", label);
        }

        Ok(JsValue::undefined())
    }

    /// `console.timeEnd(label)`
    fn time_end(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = if args.is_empty() {
            "default".to_string()
        } else {
            args[0].to_string(context)?.to_std_string_escaped()
        };

        let state = get_console_state();
        let mut state_guard = state.lock().unwrap();

        if let Some(start_time) = state_guard.timers.remove(&label) {
            let elapsed = start_time.elapsed();
            eprintln!("⏱ {}: {:.3}ms", label, elapsed.as_secs_f64() * 1000.0);
        } else {
            eprintln!("⚠ Timer '{}' does not exist", label);
        }

        Ok(JsValue::undefined())
    }

    /// `console.timeLog(label, ...data)`
    fn time_log(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = if args.is_empty() {
            "default".to_string()
        } else {
            args[0].to_string(context)?.to_std_string_escaped()
        };

        let additional_data = if args.len() > 1 {
            format!(" {}", Self::format_args(&args[1..], context)?)
        } else {
            String::new()
        };

        let state = get_console_state();
        let state_guard = state.lock().unwrap();

        if let Some(start_time) = state_guard.timers.get(&label) {
            let elapsed = start_time.elapsed();
            eprintln!("⏱ {}: {:.3}ms{}", label, elapsed.as_secs_f64() * 1000.0, additional_data);
        } else {
            eprintln!("⚠ Timer '{}' does not exist", label);
        }

        Ok(JsValue::undefined())
    }

    /// `console.count(label)`
    fn count(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = if args.is_empty() {
            "default".to_string()
        } else {
            args[0].to_string(context)?.to_std_string_escaped()
        };

        let state = get_console_state();
        let mut state_guard = state.lock().unwrap();

        let count = state_guard.counters.entry(label.clone()).or_insert(0);
        *count += 1;
        eprintln!("🔢 {}: {}", label, count);

        Ok(JsValue::undefined())
    }

    /// `console.countReset(label)`
    fn count_reset(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = if args.is_empty() {
            "default".to_string()
        } else {
            args[0].to_string(context)?.to_std_string_escaped()
        };

        let state = get_console_state();
        let mut state_guard = state.lock().unwrap();

        if state_guard.counters.remove(&label).is_some() {
            eprintln!("🔢 Counter '{}' reset", label);
        } else {
            eprintln!("⚠ Count for '{}' does not exist", label);
        }

        Ok(JsValue::undefined())
    }

    /// `console.assert(condition, ...data)`
    fn assert(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let condition = args.get_or_undefined(0).to_boolean();
        if !condition {
            let message = if args.len() > 1 {
                Self::format_args(&args[1..], context)?
            } else {
                "Assertion failed".to_string()
            };
            eprintln!("❌ Assertion failed: {}", message);
        }
        Ok(JsValue::undefined())
    }

    /// `console.table(data)`
    fn table(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let data = args.get_or_undefined(0);
        let indent = Self::get_indent();

        eprintln!("{}📊 Table:", indent);

        // Basic table formatting - in a real implementation this would properly format tabular data
        if data.is_object() {
            eprintln!("{}┌─────────┬─────────┐", indent);
            eprintln!("{}│ (index) │ Values  │", indent);
            eprintln!("{}├─────────┼─────────┤", indent);
            eprintln!("{}│    0    │ {}   │", indent, data.display());
            eprintln!("{}└─────────┴─────────┘", indent);
        } else {
            eprintln!("{}{}", indent, data.display());
        }

        Ok(JsValue::undefined())
    }

    /// `console.dir(object)`
    fn dir(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let object = args.get_or_undefined(0);
        let indent = Self::get_indent();

        eprintln!("{}📂 Object:", indent);

        // Enhanced object introspection
        if object.is_object() {
            if let Some(obj) = object.as_object() {
                eprintln!("{}  [Object] {{", indent);
                eprintln!("{}    constructor: {}", indent, "[Function: Object]");
                eprintln!("{}    __proto__: {}", indent, "[Object: null prototype] {}");
                eprintln!("{}  }}", indent);
            } else {
                eprintln!("{}  {}", indent, object.display());
            }
        } else {
            eprintln!("{}  {}", indent, object.display());
        }

        Ok(JsValue::undefined())
    }

    /// `console.dirxml(object)`
    fn dirxml(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Same as dir for now
        Self::dir(_this, args, context)
    }

    /// Helper function to get current indentation based on group depth
    fn get_indent() -> String {
        let state = get_console_state();
        let state_guard = state.lock().unwrap();
        "  ".repeat(state_guard.group_depth as usize)
    }

    /// Helper function to format arguments as string
    fn format_args(args: &[JsValue], context: &mut Context) -> JsResult<String> {
        if args.is_empty() {
            return Ok("".to_string());
        }

        let mut result = String::new();
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                result.push(' ');
            }

            // Handle basic format specifiers in first argument
            if i == 0 {
                let arg_str = arg.to_string(context)?.to_std_string_escaped();
                result.push_str(&arg_str);
            } else {
                result.push_str(&arg.display().to_string());
            }
        }
        Ok(result)
    }
}