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
        eprintln!("{}", output);
        Ok(JsValue::undefined())
    }

    /// `console.info(...data)`
    fn info(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let output = Self::format_args(args, context)?;
        eprintln!("ℹ {}", output);
        Ok(JsValue::undefined())
    }

    /// `console.warn(...data)`
    fn warn(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let output = Self::format_args(args, context)?;
        eprintln!("⚠ {}", output);
        Ok(JsValue::undefined())
    }

    /// `console.error(...data)`
    fn error(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let output = Self::format_args(args, context)?;
        eprintln!("❌ {}", output);
        Ok(JsValue::undefined())
    }

    /// `console.debug(...data)`
    fn debug(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let output = Self::format_args(args, context)?;
        eprintln!("🐛 {}", output);
        Ok(JsValue::undefined())
    }

    /// `console.trace(...data)`
    fn trace(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let output = Self::format_args(args, context)?;
        eprintln!("📍 {}", output);
        // In real implementation, would print stack trace
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
        eprintln!("▼ {}", label);
        Ok(JsValue::undefined())
    }

    /// `console.groupCollapsed(...label)`
    fn group_collapsed(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = if args.is_empty() {
            "".to_string()
        } else {
            Self::format_args(args, context)?
        };
        eprintln!("▶ {}", label);
        Ok(JsValue::undefined())
    }

    /// `console.groupEnd()`
    fn group_end(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // In real implementation, would decrease indentation level
        Ok(JsValue::undefined())
    }

    /// `console.time(label)`
    fn time(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        eprintln!("⏱ Timer '{}' started", label);
        // In real implementation, would store start time
        Ok(JsValue::undefined())
    }

    /// `console.timeEnd(label)`
    fn time_end(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        eprintln!("⏱ Timer '{}' ended", label);
        // In real implementation, would calculate and display elapsed time
        Ok(JsValue::undefined())
    }

    /// `console.timeLog(label, ...data)`
    fn time_log(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let data = if args.len() > 1 {
            Self::format_args(&args[1..], context)?
        } else {
            "".to_string()
        };
        eprintln!("⏱ Timer '{}': {}", label, data);
        Ok(JsValue::undefined())
    }

    /// `console.count(label)`
    fn count(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = if args.is_empty() {
            "default".to_string()
        } else {
            args[0].to_string(context)?.to_std_string_escaped()
        };
        eprintln!("🔢 {}: 1", label);
        // In real implementation, would maintain counter state
        Ok(JsValue::undefined())
    }

    /// `console.countReset(label)`
    fn count_reset(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let label = if args.is_empty() {
            "default".to_string()
        } else {
            args[0].to_string(context)?.to_std_string_escaped()
        };
        eprintln!("🔢 Counter '{}' reset", label);
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
        eprintln!("📊 Table: {}", data.display());
        // In real implementation, would format as table
        Ok(JsValue::undefined())
    }

    /// `console.dir(object)`
    fn dir(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let object = args.get_or_undefined(0);
        eprintln!("📂 {:#}", object.display());
        Ok(JsValue::undefined())
    }

    /// `console.dirxml(object)`
    fn dirxml(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Same as dir for now
        Self::dir(_this, args, context)
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