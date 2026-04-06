//! This module implements the global `Intl.DisplayNames` object.
//!
//! `Intl.DisplayNames` is a built-in object that enables the consistent translation
//! of language, region, and script display names.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma402/#displaynames-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/DisplayNames

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

/// The type of display names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DisplayNamesType {
    Language,
    Region,
    Script,
    Currency,
    Calendar,
    DateTimeField,
}

/// The style of display names.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum DisplayNamesStyle {
    /// Long style (e.g., "United States")
    #[default]
    Long,
    /// Short style (e.g., "US")
    Short,
    /// Narrow style
    Narrow,
}

/// The fallback behavior for display names.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum DisplayNamesFallback {
    /// Return the input code if not found
    #[default]
    Code,
    /// Return undefined if not found
    None,
}

/// JavaScript `Intl.DisplayNames` object.
#[derive(Debug, Trace, Finalize, JsData)]
#[boa_gc(unsafe_empty_trace)]
pub(crate) struct DisplayNames {
    locale: Locale,
    display_type: DisplayNamesType,
    style: DisplayNamesStyle,
    fallback: DisplayNamesFallback,
    language_display: Option<String>,
}

impl IntrinsicObject for DisplayNames {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(
                Self::supported_locales_of,
                js_string!("supportedLocalesOf"),
                1,
            )
            .property(
                JsSymbol::to_string_tag(),
                js_string!("Intl.DisplayNames"),
                Attribute::CONFIGURABLE,
            )
            .method(Self::of, js_string!("of"), 1)
            .method(Self::resolved_options, js_string!("resolvedOptions"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for DisplayNames {
    const NAME: JsString = StaticJsStrings::DISPLAY_NAMES;
}

impl BuiltInConstructor for DisplayNames {
    const CONSTRUCTOR_ARGUMENTS: usize = 2;
    const PROTOTYPE_STORAGE_SLOTS: usize = 3;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::display_names;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("cannot call `Intl.DisplayNames` constructor without `new`")
                .into());
        }

        let proto = get_prototype_from_constructor(
            new_target,
            StandardConstructors::display_names,
            context,
        )?;

        let locales = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // Options is required for DisplayNames
        if options.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("options argument is required for Intl.DisplayNames")
                .into());
        }

        // Resolve locale
        let requested_locales = canonicalize_locale_list(locales, context)?;
        let options_obj = coerce_options_to_object(options, context)?;

        // Get the locale - use "en-US" as default if none requested
        let locale = if requested_locales.is_empty() {
            Locale::try_from_str("en-US").unwrap_or_else(|_| Locale::try_from_str("en").unwrap())
        } else {
            requested_locales.into_iter().next().unwrap_or_else(|| {
                Locale::try_from_str("en-US")
                    .unwrap_or_else(|_| Locale::try_from_str("en").unwrap())
            })
        };

        // Get type option (required)
        let type_val = options_obj.get(js_string!("type"), context)?;
        if type_val.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("type option is required for Intl.DisplayNames")
                .into());
        }
        let type_str = type_val.to_string(context)?.to_std_string_escaped();
        let display_type = match type_str.as_str() {
            "language" => DisplayNamesType::Language,
            "region" => DisplayNamesType::Region,
            "script" => DisplayNamesType::Script,
            "currency" => DisplayNamesType::Currency,
            "calendar" => DisplayNamesType::Calendar,
            "dateTimeField" => DisplayNamesType::DateTimeField,
            t => {
                return Err(JsNativeError::range()
                    .with_message(format!("invalid type option: {}", t))
                    .into());
            }
        };

        // Get style option
        let style_val = options_obj.get(js_string!("style"), context)?;
        let style = if style_val.is_undefined() {
            DisplayNamesStyle::Long
        } else {
            match style_val
                .to_string(context)?
                .to_std_string_escaped()
                .as_str()
            {
                "long" => DisplayNamesStyle::Long,
                "short" => DisplayNamesStyle::Short,
                "narrow" => DisplayNamesStyle::Narrow,
                _ => DisplayNamesStyle::Long,
            }
        };

        // Get fallback option
        let fallback_val = options_obj.get(js_string!("fallback"), context)?;
        let fallback = if fallback_val.is_undefined() {
            DisplayNamesFallback::Code
        } else {
            match fallback_val
                .to_string(context)?
                .to_std_string_escaped()
                .as_str()
            {
                "code" => DisplayNamesFallback::Code,
                "none" => DisplayNamesFallback::None,
                _ => DisplayNamesFallback::Code,
            }
        };

        // Get languageDisplay option (only valid for type "language")
        let language_display = if display_type == DisplayNamesType::Language {
            let ld_val = options_obj.get(js_string!("languageDisplay"), context)?;
            if ld_val.is_undefined() {
                None
            } else {
                Some(ld_val.to_string(context)?.to_std_string_escaped())
            }
        } else {
            None
        };

        Ok(JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            Self {
                locale,
                display_type,
                style,
                fallback,
                language_display,
            },
        )
        .into())
    }
}

impl DisplayNames {
    /// `Intl.DisplayNames.prototype.of ( code )`
    ///
    /// Returns a string based on the given code.
    fn of(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let object = this.as_object();
        let dn = object
            .as_ref()
            .and_then(|o| o.downcast_ref::<Self>())
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("`of` can only be called on an `Intl.DisplayNames` object")
            })?;

        let code = args.get_or_undefined(0);
        if code.is_undefined() {
            return Err(JsNativeError::range()
                .with_message("code argument is required")
                .into());
        }

        let code_str = code.to_string(context)?.to_std_string_escaped();

        // Get display name based on type
        let result = get_display_name(&code_str, dn.display_type, dn.style, &dn.locale);

        match result {
            Some(name) => Ok(js_string!(name).into()),
            None => match dn.fallback {
                DisplayNamesFallback::Code => Ok(js_string!(code_str).into()),
                DisplayNamesFallback::None => Ok(JsValue::undefined()),
            },
        }
    }

    /// `Intl.DisplayNames.supportedLocalesOf ( locales [ , options ] )`
    fn supported_locales_of(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let locales = args.get_or_undefined(0);
        let requested_locales = canonicalize_locale_list(locales, context)?;

        let result = crate::builtins::Array::create_array_from_list(
            requested_locales
                .into_iter()
                .map(|loc| js_string!(loc.to_string()).into()),
            context,
        );

        Ok(result.into())
    }

    /// `Intl.DisplayNames.prototype.resolvedOptions ( )`
    fn resolved_options(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let object = this.as_object();
        let dn = object
            .as_ref()
            .and_then(|o| o.downcast_ref::<Self>())
            .ok_or_else(|| {
                JsNativeError::typ().with_message(
                    "`resolvedOptions` can only be called on an `Intl.DisplayNames` object",
                )
            })?;

        let mut options = ObjectInitializer::new(context);

        options
            .property(
                js_string!("locale"),
                js_string!(dn.locale.to_string()),
                Attribute::all(),
            )
            .property(
                js_string!("style"),
                match dn.style {
                    DisplayNamesStyle::Long => js_string!("long"),
                    DisplayNamesStyle::Short => js_string!("short"),
                    DisplayNamesStyle::Narrow => js_string!("narrow"),
                },
                Attribute::all(),
            )
            .property(
                js_string!("type"),
                match dn.display_type {
                    DisplayNamesType::Language => js_string!("language"),
                    DisplayNamesType::Region => js_string!("region"),
                    DisplayNamesType::Script => js_string!("script"),
                    DisplayNamesType::Currency => js_string!("currency"),
                    DisplayNamesType::Calendar => js_string!("calendar"),
                    DisplayNamesType::DateTimeField => js_string!("dateTimeField"),
                },
                Attribute::all(),
            )
            .property(
                js_string!("fallback"),
                match dn.fallback {
                    DisplayNamesFallback::Code => js_string!("code"),
                    DisplayNamesFallback::None => js_string!("none"),
                },
                Attribute::all(),
            );

        if let Some(ref ld) = dn.language_display {
            options.property(
                js_string!("languageDisplay"),
                js_string!(ld.clone()),
                Attribute::all(),
            );
        }

        Ok(options.build().into())
    }
}

/// Helper function to get display name for a code
fn get_display_name(
    code: &str,
    display_type: DisplayNamesType,
    _style: DisplayNamesStyle,
    _locale: &Locale,
) -> Option<String> {
    match display_type {
        DisplayNamesType::Language => get_language_display_name(code),
        DisplayNamesType::Region => get_region_display_name(code),
        DisplayNamesType::Script => get_script_display_name(code),
        DisplayNamesType::Currency => get_currency_display_name(code),
        DisplayNamesType::Calendar => get_calendar_display_name(code),
        DisplayNamesType::DateTimeField => get_date_time_field_display_name(code),
    }
}

fn get_language_display_name(code: &str) -> Option<String> {
    // Common language codes
    match code.to_lowercase().as_str() {
        "en" | "en-us" | "en-gb" => Some("English".to_string()),
        "es" | "es-es" => Some("Spanish".to_string()),
        "fr" | "fr-fr" => Some("French".to_string()),
        "de" | "de-de" => Some("German".to_string()),
        "it" | "it-it" => Some("Italian".to_string()),
        "pt" | "pt-br" | "pt-pt" => Some("Portuguese".to_string()),
        "ru" | "ru-ru" => Some("Russian".to_string()),
        "zh" | "zh-cn" | "zh-hans" => Some("Chinese".to_string()),
        "zh-tw" | "zh-hant" => Some("Traditional Chinese".to_string()),
        "ja" | "ja-jp" => Some("Japanese".to_string()),
        "ko" | "ko-kr" => Some("Korean".to_string()),
        "ar" | "ar-sa" => Some("Arabic".to_string()),
        "hi" | "hi-in" => Some("Hindi".to_string()),
        "nl" | "nl-nl" => Some("Dutch".to_string()),
        "sv" | "sv-se" => Some("Swedish".to_string()),
        "pl" | "pl-pl" => Some("Polish".to_string()),
        "tr" | "tr-tr" => Some("Turkish".to_string()),
        "he" | "he-il" => Some("Hebrew".to_string()),
        "th" | "th-th" => Some("Thai".to_string()),
        "vi" | "vi-vn" => Some("Vietnamese".to_string()),
        "uk" | "uk-ua" => Some("Ukrainian".to_string()),
        "cs" | "cs-cz" => Some("Czech".to_string()),
        "el" | "el-gr" => Some("Greek".to_string()),
        "ro" | "ro-ro" => Some("Romanian".to_string()),
        "hu" | "hu-hu" => Some("Hungarian".to_string()),
        "fi" | "fi-fi" => Some("Finnish".to_string()),
        "da" | "da-dk" => Some("Danish".to_string()),
        "no" | "nb" | "nn" => Some("Norwegian".to_string()),
        _ => None,
    }
}

fn get_region_display_name(code: &str) -> Option<String> {
    match code.to_uppercase().as_str() {
        "US" => Some("United States".to_string()),
        "GB" | "UK" => Some("United Kingdom".to_string()),
        "CA" => Some("Canada".to_string()),
        "AU" => Some("Australia".to_string()),
        "DE" => Some("Germany".to_string()),
        "FR" => Some("France".to_string()),
        "ES" => Some("Spain".to_string()),
        "IT" => Some("Italy".to_string()),
        "JP" => Some("Japan".to_string()),
        "CN" => Some("China".to_string()),
        "KR" => Some("South Korea".to_string()),
        "BR" => Some("Brazil".to_string()),
        "MX" => Some("Mexico".to_string()),
        "IN" => Some("India".to_string()),
        "RU" => Some("Russia".to_string()),
        "NL" => Some("Netherlands".to_string()),
        "SE" => Some("Sweden".to_string()),
        "CH" => Some("Switzerland".to_string()),
        "AT" => Some("Austria".to_string()),
        "BE" => Some("Belgium".to_string()),
        "PL" => Some("Poland".to_string()),
        "PT" => Some("Portugal".to_string()),
        "GR" => Some("Greece".to_string()),
        "TR" => Some("Turkey".to_string()),
        "IL" => Some("Israel".to_string()),
        "SA" => Some("Saudi Arabia".to_string()),
        "AE" => Some("United Arab Emirates".to_string()),
        "EG" => Some("Egypt".to_string()),
        "ZA" => Some("South Africa".to_string()),
        "NG" => Some("Nigeria".to_string()),
        "KE" => Some("Kenya".to_string()),
        "NZ" => Some("New Zealand".to_string()),
        "SG" => Some("Singapore".to_string()),
        "HK" => Some("Hong Kong".to_string()),
        "TW" => Some("Taiwan".to_string()),
        "TH" => Some("Thailand".to_string()),
        "VN" => Some("Vietnam".to_string()),
        "ID" => Some("Indonesia".to_string()),
        "MY" => Some("Malaysia".to_string()),
        "PH" => Some("Philippines".to_string()),
        _ => None,
    }
}

fn get_script_display_name(code: &str) -> Option<String> {
    match code {
        "Latn" => Some("Latin".to_string()),
        "Cyrl" => Some("Cyrillic".to_string()),
        "Arab" => Some("Arabic".to_string()),
        "Hans" => Some("Simplified Han".to_string()),
        "Hant" => Some("Traditional Han".to_string()),
        "Jpan" => Some("Japanese".to_string()),
        "Kore" => Some("Korean".to_string()),
        "Grek" => Some("Greek".to_string()),
        "Hebr" => Some("Hebrew".to_string()),
        "Thai" => Some("Thai".to_string()),
        "Deva" => Some("Devanagari".to_string()),
        _ => None,
    }
}

fn get_currency_display_name(code: &str) -> Option<String> {
    match code.to_uppercase().as_str() {
        "USD" => Some("US Dollar".to_string()),
        "EUR" => Some("Euro".to_string()),
        "GBP" => Some("British Pound".to_string()),
        "JPY" => Some("Japanese Yen".to_string()),
        "CNY" => Some("Chinese Yuan".to_string()),
        "CAD" => Some("Canadian Dollar".to_string()),
        "AUD" => Some("Australian Dollar".to_string()),
        "CHF" => Some("Swiss Franc".to_string()),
        "INR" => Some("Indian Rupee".to_string()),
        "KRW" => Some("South Korean Won".to_string()),
        "BRL" => Some("Brazilian Real".to_string()),
        "MXN" => Some("Mexican Peso".to_string()),
        "RUB" => Some("Russian Ruble".to_string()),
        "SEK" => Some("Swedish Krona".to_string()),
        "NOK" => Some("Norwegian Krone".to_string()),
        "DKK" => Some("Danish Krone".to_string()),
        "NZD" => Some("New Zealand Dollar".to_string()),
        "SGD" => Some("Singapore Dollar".to_string()),
        "HKD" => Some("Hong Kong Dollar".to_string()),
        "PLN" => Some("Polish Zloty".to_string()),
        "TRY" => Some("Turkish Lira".to_string()),
        "ZAR" => Some("South African Rand".to_string()),
        "AED" => Some("UAE Dirham".to_string()),
        "SAR" => Some("Saudi Riyal".to_string()),
        _ => None,
    }
}

fn get_calendar_display_name(code: &str) -> Option<String> {
    match code {
        "gregory" | "gregorian" => Some("Gregorian Calendar".to_string()),
        "buddhist" => Some("Buddhist Calendar".to_string()),
        "chinese" => Some("Chinese Calendar".to_string()),
        "coptic" => Some("Coptic Calendar".to_string()),
        "ethiopic" => Some("Ethiopic Calendar".to_string()),
        "hebrew" => Some("Hebrew Calendar".to_string()),
        "indian" => Some("Indian National Calendar".to_string()),
        "islamic" => Some("Islamic Calendar".to_string()),
        "islamic-civil" => Some("Islamic Civil Calendar".to_string()),
        "japanese" => Some("Japanese Calendar".to_string()),
        "persian" => Some("Persian Calendar".to_string()),
        "roc" => Some("Minguo Calendar".to_string()),
        _ => None,
    }
}

fn get_date_time_field_display_name(code: &str) -> Option<String> {
    match code {
        "era" => Some("era".to_string()),
        "year" => Some("year".to_string()),
        "quarter" => Some("quarter".to_string()),
        "month" => Some("month".to_string()),
        "weekOfYear" => Some("week".to_string()),
        "weekday" => Some("day of the week".to_string()),
        "day" => Some("day".to_string()),
        "dayPeriod" => Some("AM/PM".to_string()),
        "hour" => Some("hour".to_string()),
        "minute" => Some("minute".to_string()),
        "second" => Some("second".to_string()),
        "timeZoneName" => Some("time zone".to_string()),
        _ => None,
    }
}
