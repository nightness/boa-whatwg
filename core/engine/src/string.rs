//! This module contains the [`js_string`][crate::js_string] macro and the
//! [`js_str`][crate::js_str] macro.
//!
//! The [`js_string`][crate::js_string] macro is used when you need to create a new [`JsString`],
//! and the [`js_str`][crate::js_str] macro is used for const conversions of string literals to [`JsStr`].

#[doc(inline)]
pub use boa_string::*;

/// Utility macro to create a [`JsString`].
///
/// # Examples
///
/// You can call the macro without arguments to create an empty `JsString`:
///
/// ```
/// use boa_engine::js_string;
///
/// let empty_str = js_string!();
/// assert!(empty_str.is_empty());
/// ```
///
///
/// You can create a `JsString` from a string literal, which completely skips the runtime
/// conversion from [`&str`] to <code>[&\[u16\]][slice]</code>:
///
/// ```
/// # use boa_engine::js_string;
/// let hw = js_string!("Hello, world!");
/// assert_eq!(&hw, "Hello, world!");
/// ```
///
/// Any `&[u16]` slice is a valid `JsString`, including unpaired surrogates:
///
/// ```
/// # use boa_engine::js_string;
/// let array = js_string!(&[0xD8AFu16, 0x00A0, 0xD8FF, 0x00F0]);
/// ```
///
/// You can also pass it any number of `&[u16]` as arguments to create a new `JsString` with
/// the concatenation of every slice:
///
/// ```
/// # use boa_engine::{js_string, js_str, JsStr};
/// const NAME: JsStr<'_> = js_str!("human! ");
/// let greeting = js_string!("Hello, ");
/// let msg = js_string!(&greeting, NAME, js_str!("Nice to meet you!"));
///
/// assert_eq!(&msg, "Hello, human! Nice to meet you!");
/// ```
#[macro_export]
#[allow(clippy::module_name_repetitions)]
macro_rules! js_string {
    () => {
        $crate::string::JsString::default()
    };
    ($s:literal) => {{
        const LITERAL: &$crate::string::JsStr<'static> = &$crate::js_str!($s);

        $crate::string::JsString::from_static_js_str(LITERAL)
    }};
    ($s:expr) => {
        $crate::string::JsString::from($s)
    };
    ( $x:expr, $y:expr ) => {
        $crate::string::JsString::concat($crate::string::JsStr::from($x), $crate::string::JsStr::from($y))
    };
    ( $( $s:expr ),+ ) => {
        $crate::string::JsString::concat_array(&[ $( $crate::string::JsStr::from($s) ),+ ])
    };
}

#[allow(clippy::redundant_clone)]
#[cfg(test)]
mod tests;
