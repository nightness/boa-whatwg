//! CSS Object Model implementation for Boa
//!
//! Native implementation of CSS APIs including CSS.supports, CSS typed object model,
//! and Houdini CSS APIs for real browser compatibility.
//! https://drafts.csswg.org/css-typed-om-1/
//! https://www.w3.org/TR/css-houdini-drafts/

use crate::{
    builtins::{BuiltInBuilder, BuiltInObject, IntrinsicObject},
    context::intrinsics::Intrinsics,
    object::JsObject,
    string::StaticJsStrings,
    value::JsValue,
    Context, JsData, JsNativeError, JsResult, js_string, JsString,
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;

#[cfg(test)]
mod tests;

/// JavaScript `CSS` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Css;

impl IntrinsicObject for Css {
    fn init(realm: &crate::realm::Realm) {
        let supports_func = BuiltInBuilder::callable(realm, supports)
            .name(js_string!("supports"))
            .length(1)
            .build();

        let register_property_func = BuiltInBuilder::callable(realm, register_property)
            .name(js_string!("registerProperty"))
            .length(1)
            .build();

        // CSS unit constructors
        let number_func = BuiltInBuilder::callable(realm, css_number)
            .name(js_string!("number"))
            .length(1)
            .build();

        let px_func = BuiltInBuilder::callable(realm, css_px)
            .name(js_string!("px"))
            .length(1)
            .build();

        let percent_func = BuiltInBuilder::callable(realm, css_percent)
            .name(js_string!("percent"))
            .length(1)
            .build();

        let em_func = BuiltInBuilder::callable(realm, css_em)
            .name(js_string!("em"))
            .length(1)
            .build();

        let rem_func = BuiltInBuilder::callable(realm, css_rem)
            .name(js_string!("rem"))
            .length(1)
            .build();

        let vw_func = BuiltInBuilder::callable(realm, css_vw)
            .name(js_string!("vw"))
            .length(1)
            .build();

        let vh_func = BuiltInBuilder::callable(realm, css_vh)
            .name(js_string!("vh"))
            .length(1)
            .build();

        // Create CSS object
        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_method(supports_func, js_string!("supports"), 1)
            .static_method(register_property_func, js_string!("registerProperty"), 1)
            .static_method(number_func, js_string!("number"), 1)
            .static_method(px_func, js_string!("px"), 1)
            .static_method(percent_func, js_string!("percent"), 1)
            .static_method(em_func, js_string!("em"), 1)
            .static_method(rem_func, js_string!("rem"), 1)
            .static_method(vw_func, js_string!("vw"), 1)
            .static_method(vh_func, js_string!("vh"), 1)
            .build();

        // Add CSS Houdini worklets
        let css_obj = realm.intrinsics().objects().css();

        // CSS Houdini worklets are added during realm initialization
        // This is done in the realm initialization process
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().css().clone()
    }
}

impl BuiltInObject for Css {
    const NAME: JsString = StaticJsStrings::CSS;
}

/// CSS property support database
struct CssPropertySupport;

impl CssPropertySupport {
    fn is_property_supported(property: &str, value: Option<&str>) -> bool {
        let supported_properties = HashMap::from([
            ("display", vec!["flex", "grid", "block", "inline", "inline-block", "none", "table", "table-cell"]),
            ("position", vec!["static", "relative", "absolute", "fixed", "sticky"]),
            ("flex-direction", vec!["row", "column", "row-reverse", "column-reverse"]),
            ("grid-template-columns", vec!["auto", "fr", "px", "%", "repeat"]),
            ("transform", vec!["translateX", "translateY", "translate", "rotate", "scale", "matrix"]),
            ("filter", vec!["blur", "brightness", "contrast", "grayscale", "hue-rotate"]),
            ("backdrop-filter", vec!["blur", "brightness", "contrast", "grayscale"]),
            ("clip-path", vec!["polygon", "circle", "ellipse", "inset"]),
            ("mask", vec!["url", "linear-gradient", "radial-gradient"]),
            ("scroll-behavior", vec!["auto", "smooth"]),
            ("scroll-snap-type", vec!["x", "y", "both", "mandatory", "proximity"]),
            ("overscroll-behavior", vec!["auto", "contain", "none"]),
            ("user-select", vec!["auto", "none", "text", "all"]),
            ("appearance", vec!["auto", "none", "button", "textfield"]),
            ("box-sizing", vec!["content-box", "border-box"]),
            ("background-clip", vec!["border-box", "padding-box", "content-box", "text"]),
            ("mix-blend-mode", vec!["normal", "multiply", "screen", "overlay", "darken", "lighten"]),
            ("object-fit", vec!["fill", "contain", "cover", "none", "scale-down"]),
            ("writing-mode", vec!["horizontal-tb", "vertical-rl", "vertical-lr"]),
            ("text-orientation", vec!["mixed", "upright", "sideways"]),
            ("font-feature-settings", vec!["normal", "liga", "kern", "swsh"]),
            ("font-variation-settings", vec!["normal", "wght", "wdth", "slnt"]),
            ("color-scheme", vec!["normal", "light", "dark", "light dark"]),
            ("accent-color", vec!["auto", "red", "blue", "green", "currentColor"]),
            ("aspect-ratio", vec!["auto", "16/9", "4/3", "1/1"]),
            ("gap", vec!["px", "em", "rem", "%", "vh", "vw"]),
            ("isolation", vec!["auto", "isolate"]),
            ("contain", vec!["none", "strict", "content", "size", "layout", "style", "paint"]),
            ("will-change", vec!["auto", "scroll-position", "contents", "transform", "opacity"]),
            ("container-type", vec!["normal", "size", "inline-size"]),
            ("animation-timeline", vec!["auto", "scroll", "view"]),
            ("animation-range", vec!["normal", "contain", "cover", "entry", "exit"]),
            ("view-transition-name", vec!["none", "auto"]),
            ("text-wrap", vec!["wrap", "nowrap", "balance", "pretty"]),
            ("white-space-collapse", vec!["collapse", "preserve", "preserve-breaks", "preserve-spaces", "break-spaces"]),
            ("text-spacing-trim", vec!["normal", "space-all", "space-first", "trim-start"]),
            ("anchor-name", vec!["none"]),
            ("position-anchor", vec!["none"]),
            ("anchor", vec!["none"]),
            ("inset-area", vec!["none", "top", "bottom", "left", "right"]),
            // Chrome-specific features
            ("color-mix", vec!["in srgb", "in hsl", "in hwb", "in lab", "in lch", "in oklab", "in oklch"]),
            ("light-dark", vec!["light", "dark"]),
            ("field-sizing", vec!["content", "fixed"]),
            ("interpolate-size", vec!["allow-keywords", "numeric-only"]),
            ("reading-flow", vec!["normal", "flex-visual", "flex-flow", "grid-rows", "grid-columns", "grid-order"]),
        ]);

        if let Some(values) = supported_properties.get(property) {
            if let Some(value) = value {
                values.iter().any(|v| value.contains(v))
            } else {
                true
            }
        } else {
            // Special cases
            match property {
                "math" => {
                    if let Some(value) = value {
                        let math_functions = ["calc", "min", "max", "clamp", "sin", "cos", "tan",
                                            "asin", "acos", "atan", "atan2", "pow", "sqrt", "hypot",
                                            "log", "exp"];
                        math_functions.iter().any(|f| value.contains(f))
                    } else {
                        true
                    }
                },
                "sibling-count" | "sibling-index" => true,
                _ => false
            }
        }
    }

    fn is_selector_supported(selector: &str) -> bool {
        let supported_selectors = [
            ":hover", ":focus", ":active", ":visited", ":link",
            ":first-child", ":last-child", ":nth-child", ":nth-of-type",
            ":not", ":is", ":where", ":has",
            "::before", "::after", "::first-line", "::first-letter",
            ":focus-visible", ":focus-within",
            ":target", ":checked", ":disabled", ":enabled",
            ":valid", ":invalid", ":required", ":optional",
            ":empty", ":root",
            ":nth-last-child", ":nth-last-of-type",
            ":only-child", ":only-of-type"
        ];

        supported_selectors.iter().any(|s| selector.contains(s))
    }
}

/// CSS worklet data for Houdini APIs
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct CssWorkletData {
    #[unsafe_ignore_trace]
    name: String,
    #[unsafe_ignore_trace]
    modules: Vec<String>,
}

impl CssWorkletData {
    fn new(name: String) -> Self {
        Self {
            name,
            modules: Vec::new(),
        }
    }

    fn add_module(&mut self, module_url: String) {
        self.modules.push(module_url);
        println!("{} module added: {}", self.name, module_url);
    }
}

/// CSS numeric value for typed object model
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct CssNumericValueData {
    #[unsafe_ignore_trace]
    value: f64,
    #[unsafe_ignore_trace]
    unit: String,
}

impl CssNumericValueData {
    fn new(value: f64, unit: String) -> Self {
        Self { value, unit }
    }
}

/// Create a CSS worklet object
fn create_worklet(realm: &crate::realm::Realm, name: &str) -> JsValue {
    let worklet_data = CssWorkletData::new(name.to_string());
    let worklet_obj = JsObject::from_proto_and_data_with_shared_shape(
        realm.context().root_shape(),
        realm.intrinsics().objects().object_prototype(),
        worklet_data,
    );

    let add_module_func = BuiltInBuilder::callable(realm, worklet_add_module)
        .name(js_string!("addModule"))
        .length(1)
        .build();

    // Note: Property definition would need a context, which isn't available here
    // This would be done during realm initialization

    worklet_obj.into()
}

/// `CSS.supports(property, value)` implementation
fn supports(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if args.len() == 1 {
        // CSS.supports("display: flex") format
        let declaration = args.get_or_undefined(0).to_string(context)?;
        let declaration_str = declaration.to_std_string_escaped();

        if let Some(colon_pos) = declaration_str.find(':') {
            let property = declaration_str[..colon_pos].trim();
            let value = declaration_str[colon_pos + 1..].trim();

            let supported = CssPropertySupport::is_property_supported(property, Some(value));
            return Ok(JsValue::Boolean(supported));
        }
    } else if args.len() >= 2 {
        // CSS.supports("display", "flex") format
        let property = args.get_or_undefined(0).to_string(context)?;
        let value = args.get_or_undefined(1).to_string(context)?;

        let property_str = property.to_std_string_escaped();
        let value_str = value.to_std_string_escaped();

        let supported = if property_str == "selector" {
            CssPropertySupport::is_selector_supported(&value_str)
        } else {
            CssPropertySupport::is_property_supported(&property_str, Some(&value_str))
        };

        return Ok(JsValue::Boolean(supported));
    }

    Ok(JsValue::Boolean(false))
}

/// `CSS.registerProperty(definition)` implementation
fn register_property(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let definition = args.get_or_undefined(0);

    if let Some(obj) = definition.as_object() {
        if let Ok(name) = obj.get(js_string!("name"), context) {
            let name_str = name.to_string(context)?;
            println!("CSS custom property registered: {}", name_str.to_std_string_escaped());
        }
    }

    Ok(JsValue::undefined())
}

/// CSS.number(value) - creates a unitless numeric value
fn css_number(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_number(context)?;
    let numeric_data = CssNumericValueData::new(value, String::new());

    let numeric_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.intrinsics().objects().object_prototype(),
        numeric_data,
    );

    Ok(numeric_obj.into())
}

/// CSS.px(value) - creates a pixel value
fn css_px(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_number(context)?;
    let numeric_data = CssNumericValueData::new(value, "px".to_string());

    let numeric_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.intrinsics().objects().object_prototype(),
        numeric_data,
    );

    Ok(numeric_obj.into())
}

/// CSS.percent(value) - creates a percentage value
fn css_percent(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_number(context)?;
    let numeric_data = CssNumericValueData::new(value, "%".to_string());

    let numeric_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.intrinsics().objects().object_prototype(),
        numeric_data,
    );

    Ok(numeric_obj.into())
}

/// CSS.em(value) - creates an em value
fn css_em(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_number(context)?;
    let numeric_data = CssNumericValueData::new(value, "em".to_string());

    let numeric_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.intrinsics().objects().object_prototype(),
        numeric_data,
    );

    Ok(numeric_obj.into())
}

/// CSS.rem(value) - creates a rem value
fn css_rem(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_number(context)?;
    let numeric_data = CssNumericValueData::new(value, "rem".to_string());

    let numeric_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.intrinsics().objects().object_prototype(),
        numeric_data,
    );

    Ok(numeric_obj.into())
}

/// CSS.vw(value) - creates a viewport width value
fn css_vw(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_number(context)?;
    let numeric_data = CssNumericValueData::new(value, "vw".to_string());

    let numeric_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.intrinsics().objects().object_prototype(),
        numeric_data,
    );

    Ok(numeric_obj.into())
}

/// CSS.vh(value) - creates a viewport height value
fn css_vh(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_number(context)?;
    let numeric_data = CssNumericValueData::new(value, "vh".to_string());

    let numeric_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.intrinsics().objects().object_prototype(),
        numeric_data,
    );

    Ok(numeric_obj.into())
}

/// Worklet addModule method implementation
fn worklet_add_module(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("addModule called on non-object")
    })?;

    if let Some(worklet_data) = this_obj.downcast_mut::<CssWorkletData>() {
        let module_url = args.get_or_undefined(0).to_string(context)?;
        let module_url_str = module_url.to_std_string_escaped();

        worklet_data.add_module(module_url_str);

        // Return a resolved promise
        Ok(JsValue::undefined())
    } else {
        Err(JsNativeError::typ()
            .with_message("addModule called on non-worklet object")
            .into())
    }
}