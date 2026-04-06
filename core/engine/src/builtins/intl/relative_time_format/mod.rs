//! This module implements the global `Intl.RelativeTimeFormat` object.
//!
//! `Intl.RelativeTimeFormat` is a built-in object that enables language-sensitive
//! relative time formatting.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma402/#relativetimeformat-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/RelativeTimeFormat

use boa_gc::{Finalize, Trace};
use icu_locale::Locale;

use crate::{
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{ObjectInitializer, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
};

use super::locale::canonicalize_locale_list;
use super::options::coerce_options_to_object;

/// The style of relative time format.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Style {
    /// Long style (e.g., "in 1 month")
    #[default]
    Long,
    /// Short style (e.g., "in 1 mo.")
    Short,
    /// Narrow style (e.g., "in 1mo")
    Narrow,
}

/// The numeric option for relative time format.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Numeric {
    /// Always use numeric value (e.g., "1 day ago")
    #[default]
    Always,
    /// Use words when possible (e.g., "yesterday")
    Auto,
}

/// JavaScript `Intl.RelativeTimeFormat` object.
#[derive(Debug, Trace, Finalize, JsData)]
#[boa_gc(unsafe_empty_trace)]
pub(crate) struct RelativeTimeFormat {
    locale: Locale,
    style: Style,
    numeric: Numeric,
}

impl IntrinsicObject for RelativeTimeFormat {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(
                Self::supported_locales_of,
                js_string!("supportedLocalesOf"),
                1,
            )
            .property(
                JsSymbol::to_string_tag(),
                js_string!("Intl.RelativeTimeFormat"),
                Attribute::CONFIGURABLE,
            )
            .method(Self::format, js_string!("format"), 2)
            .method(Self::format_to_parts, js_string!("formatToParts"), 2)
            .method(Self::resolved_options, js_string!("resolvedOptions"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for RelativeTimeFormat {
    const NAME: JsString = StaticJsStrings::RELATIVE_TIME_FORMAT;
}

impl BuiltInConstructor for RelativeTimeFormat {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 4;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::relative_time_format;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("cannot call `Intl.RelativeTimeFormat` constructor without `new`")
                .into());
        }

        let proto = get_prototype_from_constructor(
            new_target,
            StandardConstructors::relative_time_format,
            context,
        )?;

        let locales = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // Resolve locale
        let requested_locales = canonicalize_locale_list(locales, context)?;
        let options_obj = coerce_options_to_object(options, context)?;

        // Get the locale - use "en-US" as default if none requested
        let locale = if requested_locales.is_empty() {
            Locale::try_from_str("en-US")
                .unwrap_or_else(|_| Locale::try_from_str("en").expect("\"en\" is a valid locale"))
        } else {
            requested_locales.into_iter().next().unwrap_or_else(|| {
                Locale::try_from_str("en-US").unwrap_or_else(|_| {
                    Locale::try_from_str("en").expect("\"en\" is a valid locale")
                })
            })
        };

        // Get style option
        let style_val = options_obj.get(js_string!("style"), context)?;
        let style = if style_val.is_undefined() {
            Style::Long
        } else {
            match style_val
                .to_string(context)?
                .to_std_string_escaped()
                .as_str()
            {
                "short" => Style::Short,
                "narrow" => Style::Narrow,
                _ => Style::Long,
            }
        };

        // Get numeric option
        let numeric_val = options_obj.get(js_string!("numeric"), context)?;
        let numeric = if numeric_val.is_undefined() {
            Numeric::Always
        } else {
            match numeric_val
                .to_string(context)?
                .to_std_string_escaped()
                .as_str()
            {
                "auto" => Numeric::Auto,
                _ => Numeric::Always,
            }
        };

        Ok(JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            Self {
                locale,
                style,
                numeric,
            },
        )
        .into())
    }
}

impl RelativeTimeFormat {
    /// `Intl.RelativeTimeFormat.prototype.format ( value, unit )`
    ///
    /// Formats a value and unit according to the locale and formatting options.
    fn format(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let object = this.as_object();
        let rtf = object
            .as_ref()
            .and_then(|o| o.downcast_ref::<Self>())
            .ok_or_else(|| {
                JsNativeError::typ().with_message(
                    "`format` can only be called on an `Intl.RelativeTimeFormat` object",
                )
            })?;

        let value = args.get_or_undefined(0).to_number(context)?;
        let unit = args.get_or_undefined(1).to_string(context)?;

        // Format based on unit
        let unit_str = unit.to_std_string_escaped();
        let formatted = format_relative_time(value, &unit_str, &rtf.locale, rtf.style, rtf.numeric);

        Ok(js_string!(formatted).into())
    }

    /// `Intl.RelativeTimeFormat.prototype.formatToParts ( value, unit )`
    ///
    /// Returns an array of objects representing the relative time format in parts.
    fn format_to_parts(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let object = this.as_object();
        let rtf = object
            .as_ref()
            .and_then(|o| o.downcast_ref::<Self>())
            .ok_or_else(|| {
                JsNativeError::typ().with_message(
                    "`formatToParts` can only be called on an `Intl.RelativeTimeFormat` object",
                )
            })?;

        let value = args.get_or_undefined(0).to_number(context)?;
        let unit = args.get_or_undefined(1).to_string(context)?;

        // For now, return an array with a single object containing the formatted value
        let unit_str = unit.to_std_string_escaped();
        let formatted = format_relative_time(value, &unit_str, &rtf.locale, rtf.style, rtf.numeric);

        // Create parts array
        let parts = crate::builtins::Array::create_array_from_list(
            vec![
                ObjectInitializer::new(context)
                    .property(js_string!("type"), js_string!("literal"), Attribute::all())
                    .property(js_string!("value"), js_string!(formatted), Attribute::all())
                    .build()
                    .into(),
            ],
            context,
        );

        Ok(parts.into())
    }

    /// `Intl.RelativeTimeFormat.supportedLocalesOf ( locales [ , options ] )`
    fn supported_locales_of(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let locales = args.get_or_undefined(0);
        let requested_locales = canonicalize_locale_list(locales, context)?;

        // Return the requested locales as supported (simplified implementation)
        let result = crate::builtins::Array::create_array_from_list(
            requested_locales
                .into_iter()
                .map(|loc| js_string!(loc.to_string()).into()),
            context,
        );

        Ok(result.into())
    }

    /// `Intl.RelativeTimeFormat.prototype.resolvedOptions ( )`
    fn resolved_options(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let object = this.as_object();
        let rtf = object
            .as_ref()
            .and_then(|o| o.downcast_ref::<Self>())
            .ok_or_else(|| {
                JsNativeError::typ().with_message(
                    "`resolvedOptions` can only be called on an `Intl.RelativeTimeFormat` object",
                )
            })?;

        let options = ObjectInitializer::new(context)
            .property(
                js_string!("locale"),
                js_string!(rtf.locale.to_string()),
                Attribute::all(),
            )
            .property(
                js_string!("style"),
                match rtf.style {
                    Style::Long => js_string!("long"),
                    Style::Short => js_string!("short"),
                    Style::Narrow => js_string!("narrow"),
                },
                Attribute::all(),
            )
            .property(
                js_string!("numeric"),
                match rtf.numeric {
                    Numeric::Always => js_string!("always"),
                    Numeric::Auto => js_string!("auto"),
                },
                Attribute::all(),
            )
            .property(
                js_string!("numberingSystem"),
                js_string!("latn"),
                Attribute::all(),
            )
            .build();

        Ok(options.into())
    }
}

/// Helper function to format relative time
#[allow(clippy::float_cmp)]
fn format_relative_time(
    value: f64,
    unit: &str,
    _locale: &Locale,
    style: Style,
    numeric: Numeric,
) -> String {
    let abs_value = value.abs();
    let is_past = value < 0.0;

    // Handle "auto" numeric option for common cases
    if numeric == Numeric::Auto {
        if abs_value == 1.0 {
            match (unit, is_past) {
                ("day" | "days", true) => return "yesterday".to_string(),
                ("day" | "days", false) => return "tomorrow".to_string(),
                _ => {}
            }
        }
        if abs_value == 0.0 {
            match unit {
                "day" | "days" => return "today".to_string(),
                "hour" | "hours" => return "this hour".to_string(),
                "minute" | "minutes" => return "this minute".to_string(),
                "second" | "seconds" => return "now".to_string(),
                _ => {}
            }
        }
    }

    // Normalize unit name
    let unit_singular = match unit {
        "years" => "year",
        "months" => "month",
        "weeks" => "week",
        "days" => "day",
        "hours" => "hour",
        "minutes" => "minute",
        "seconds" => "second",
        "quarters" => "quarter",
        u => u,
    };

    // Format based on style
    let unit_display = match style {
        Style::Long => match unit_singular {
            "year" => {
                if abs_value == 1.0 {
                    "year"
                } else {
                    "years"
                }
            }
            "month" => {
                if abs_value == 1.0 {
                    "month"
                } else {
                    "months"
                }
            }
            "week" => {
                if abs_value == 1.0 {
                    "week"
                } else {
                    "weeks"
                }
            }
            "day" => {
                if abs_value == 1.0 {
                    "day"
                } else {
                    "days"
                }
            }
            "hour" => {
                if abs_value == 1.0 {
                    "hour"
                } else {
                    "hours"
                }
            }
            "minute" => {
                if abs_value == 1.0 {
                    "minute"
                } else {
                    "minutes"
                }
            }
            "second" => {
                if abs_value == 1.0 {
                    "second"
                } else {
                    "seconds"
                }
            }
            "quarter" => {
                if abs_value == 1.0 {
                    "quarter"
                } else {
                    "quarters"
                }
            }
            u => u,
        },
        Style::Short => match unit_singular {
            "year" => "yr.",
            "month" => "mo.",
            "week" => "wk.",
            "day" => {
                if abs_value == 1.0 {
                    "day"
                } else {
                    "days"
                }
            }
            "hour" => "hr.",
            "minute" => "min.",
            "second" => "sec.",
            "quarter" => "qtr.",
            u => u,
        },
        Style::Narrow => match unit_singular {
            "year" => "y",
            "month" => "mo",
            "week" => "w",
            "day" => "d",
            "hour" => "h",
            "minute" => "m",
            "second" => "s",
            "quarter" => "q",
            u => u,
        },
    };

    // Format the value
    let value_str = if abs_value.fract() == 0.0 {
        format!("{}", abs_value as i64)
    } else {
        format!("{abs_value}")
    };

    if is_past {
        match style {
            Style::Long | Style::Short => format!("{value_str} {unit_display} ago"),
            Style::Narrow => format!("-{value_str}{unit_display}"),
        }
    } else {
        match style {
            Style::Long | Style::Short => format!("in {value_str} {unit_display}"),
            Style::Narrow => format!("+{value_str}{unit_display}"),
        }
    }
}
