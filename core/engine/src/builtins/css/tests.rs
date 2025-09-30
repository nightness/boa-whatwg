//! Tests for the CSS Object Model implementation

use crate::{run_test_actions, TestAction};

#[test]
fn css_global_object_available() {
    run_test_actions([
        TestAction::assert("typeof CSS === 'object'"),
        TestAction::assert("CSS !== null"),
    ]);
}

#[test]
fn css_supports_method() {
    run_test_actions([
        TestAction::assert("typeof CSS.supports === 'function'"),

        // Test single parameter format
        TestAction::run("var result1 = CSS.supports('display: flex')"),
        TestAction::assert("typeof result1 === 'boolean'"),
        TestAction::assert("result1 === true"),

        // Test two parameter format
        TestAction::run("var result2 = CSS.supports('display', 'flex')"),
        TestAction::assert("typeof result2 === 'boolean'"),
        TestAction::assert("result2 === true"),

        // Test unsupported property
        TestAction::run("var result3 = CSS.supports('invalid-property', 'value')"),
        TestAction::assert("result3 === false"),
    ]);
}

#[test]
fn css_supports_modern_features() {
    run_test_actions([
        // Grid support
        TestAction::run("var gridSupport = CSS.supports('display', 'grid')"),
        TestAction::assert("gridSupport === true"),

        // Flexbox support
        TestAction::run("var flexSupport = CSS.supports('display: flex')"),
        TestAction::assert("flexSupport === true"),

        // Custom properties
        TestAction::run("var customPropSupport = CSS.supports('color', 'var(--main-color)')"),
        TestAction::assert("typeof customPropSupport === 'boolean'"),

        // Transform support
        TestAction::run("var transformSupport = CSS.supports('transform', 'translateX(10px)')"),
        TestAction::assert("transformSupport === true"),
    ]);
}

#[test]
fn css_supports_chrome_features() {
    run_test_actions([
        // Container queries
        TestAction::run("var containerSupport = CSS.supports('container-type', 'inline-size')"),
        TestAction::assert("containerSupport === true"),

        // Scroll timeline
        TestAction::run("var scrollTimelineSupport = CSS.supports('animation-timeline', 'scroll')"),
        TestAction::assert("scrollTimelineSupport === true"),

        // View transitions
        TestAction::run("var viewTransitionSupport = CSS.supports('view-transition-name', 'none')"),
        TestAction::assert("viewTransitionSupport === true"),

        // Color mixing
        TestAction::run("var colorMixSupport = CSS.supports('color-mix', 'in srgb')"),
        TestAction::assert("colorMixSupport === true"),
    ]);
}

#[test]
fn css_register_property() {
    run_test_actions([
        TestAction::assert("typeof CSS.registerProperty === 'function'"),

        // Test property registration
        TestAction::run(r#"
            CSS.registerProperty({
                name: '--my-custom-property',
                syntax: '<color>',
                inherits: false,
                initialValue: 'blue'
            })
        "#),

        // Should not throw errors
        TestAction::assert("true"),
    ]);
}

#[test]
fn css_typed_object_model() {
    run_test_actions([
        // CSS unit constructors
        TestAction::assert("typeof CSS.number === 'function'"),
        TestAction::assert("typeof CSS.px === 'function'"),
        TestAction::assert("typeof CSS.percent === 'function'"),
        TestAction::assert("typeof CSS.em === 'function'"),
        TestAction::assert("typeof CSS.rem === 'function'"),
        TestAction::assert("typeof CSS.vw === 'function'"),
        TestAction::assert("typeof CSS.vh === 'function'"),

        // Test CSS.px
        TestAction::run("var pxValue = CSS.px(100)"),
        TestAction::assert("typeof pxValue === 'object'"),
        TestAction::assert("pxValue !== null"),

        // Test CSS.percent
        TestAction::run("var percentValue = CSS.percent(50)"),
        TestAction::assert("typeof percentValue === 'object'"),
        TestAction::assert("percentValue !== null"),
    ]);
}

#[test]
fn css_selector_support() {
    run_test_actions([
        // Test selector support
        TestAction::run("var result1 = CSS.supports('selector', ':hover')"),
        TestAction::assert("result1 === true"),

        TestAction::run("var result2 = CSS.supports('selector', ':focus-visible')"),
        TestAction::assert("result2 === true"),

        TestAction::run("var result3 = CSS.supports('selector', ':has(.child)')"),
        TestAction::assert("result3 === true"),

        TestAction::run("var result4 = CSS.supports('selector', '::before')"),
        TestAction::assert("result4 === true"),

        // Test invalid selector
        TestAction::run("var result5 = CSS.supports('selector', ':::invalid')"),
        TestAction::assert("result5 === false"),
    ]);
}

#[test]
fn css_math_functions() {
    run_test_actions([
        // Test math function support
        TestAction::run("var calcSupport = CSS.supports('math', 'calc(10px + 5px)')"),
        TestAction::assert("calcSupport === true"),

        TestAction::run("var minSupport = CSS.supports('math', 'min(10px, 5px)')"),
        TestAction::assert("minSupport === true"),

        TestAction::run("var maxSupport = CSS.supports('math', 'max(10px, 5px)')"),
        TestAction::assert("maxSupport === true"),

        TestAction::run("var clampSupport = CSS.supports('math', 'clamp(5px, 10px, 15px)')"),
        TestAction::assert("clampSupport === true"),

        // Test advanced math functions
        TestAction::run("var sinSupport = CSS.supports('math', 'sin(45deg)')"),
        TestAction::assert("sinSupport === true"),

        TestAction::run("var cosSupport = CSS.supports('math', 'cos(45deg)')"),
        TestAction::assert("cosSupport === true"),
    ]);
}

#[test]
fn css_houdini_worklets() {
    run_test_actions([
        // CSS Houdini worklets should be available
        TestAction::assert("typeof CSS.paintWorklet === 'object'"),
        TestAction::assert("CSS.paintWorklet !== null"),
        TestAction::assert("typeof CSS.paintWorklet.addModule === 'function'"),

        TestAction::assert("typeof CSS.layoutWorklet === 'object'"),
        TestAction::assert("CSS.layoutWorklet !== null"),
        TestAction::assert("typeof CSS.layoutWorklet.addModule === 'function'"),

        TestAction::assert("typeof CSS.animationWorklet === 'object'"),
        TestAction::assert("CSS.animationWorklet !== null"),
        TestAction::assert("typeof CSS.animationWorklet.addModule === 'function'"),
    ]);
}

#[test]
fn css_worklet_add_module() {
    run_test_actions([
        // Test CSS Paint Worklet module addition
        TestAction::run("CSS.paintWorklet.addModule('test-module.js')"),
        TestAction::assert("true"), // Should not throw

        // Test CSS Layout Worklet module addition
        TestAction::run("CSS.layoutWorklet.addModule('layout-module.js')"),
        TestAction::assert("true"), // Should not throw

        // Test CSS Animation Worklet module addition
        TestAction::run("CSS.animationWorklet.addModule('animation-module.js')"),
        TestAction::assert("true"), // Should not throw
    ]);
}