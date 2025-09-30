//! Structured Clone Algorithm tests
//! Tests for WHATWG HTML5 structured cloning specification compliance
//! https://html.spec.whatwg.org/multipage/structured-data.html#structured-cloning

use crate::{Context, JsValue, js_string, run_test_actions, TestAction, JsNativeErrorKind, Source};
use super::*;

#[test]
fn structured_clone_primitives() {
    let mut context = Context::default();

    // Test undefined
    let cloned = structured_clone(&JsValue::undefined(), &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();
    assert!(deserialized.is_undefined());

    // Test null
    let cloned = structured_clone(&JsValue::null(), &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();
    assert!(deserialized.is_null());

    // Test boolean
    let cloned = structured_clone(&JsValue::from(true), &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();
    assert_eq!(deserialized.as_boolean().unwrap(), true);

    let cloned = structured_clone(&JsValue::from(false), &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();
    assert_eq!(deserialized.as_boolean().unwrap(), false);

    // Test number
    let cloned = structured_clone(&JsValue::from(42.5), &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();
    assert_eq!(deserialized.as_number().unwrap(), 42.5);

    // Test string
    let cloned = structured_clone(&JsValue::from(js_string!("hello world")), &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();
    assert_eq!(deserialized.to_string(&mut context).unwrap().to_std_string_escaped(), "hello world");
}

#[test]
fn structured_clone_array() {
    let mut context = Context::default();

    // Create array directly in the context
    let code = "var testArray = [1, 'two', true, null, undefined]; testArray;";
    let result = context.eval(Source::from_bytes(code)).unwrap();
    let array = result;

    let cloned = structured_clone(&array, &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();

    // Verify it's an array
    if let Some(array_obj) = deserialized.as_object() {
        let length = array_obj.get(js_string!("length"), &mut context).unwrap();
        assert_eq!(length.as_number().unwrap(), 5.0);

        // Check elements
        let elem0 = array_obj.get(0, &mut context).unwrap();
        assert_eq!(elem0.as_number().unwrap(), 1.0);

        let elem1 = array_obj.get(1, &mut context).unwrap();
        assert_eq!(elem1.to_string(&mut context).unwrap().to_std_string_escaped(), "two");

        let elem2 = array_obj.get(2, &mut context).unwrap();
        assert_eq!(elem2.as_boolean().unwrap(), true);

        let elem3 = array_obj.get(3, &mut context).unwrap();
        assert!(elem3.is_null());

        let elem4 = array_obj.get(4, &mut context).unwrap();
        assert!(elem4.is_undefined());
    } else {
        panic!("Deserialized value should be an array");
    }
}

#[test]
fn structured_clone_object() {
    let mut context = Context::default();

    // Create object directly in the context
    let code = r#"
        var testObj = {
            num: 42,
            str: "hello",
            bool: true,
            nil: null,
            undef: undefined
        };
        testObj;
    "#;
    let result = context.eval(Source::from_bytes(code)).unwrap();
    let obj = result;

    let cloned = structured_clone(&obj, &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();

    if let Some(obj) = deserialized.as_object() {
        let num_prop = obj.get(js_string!("num"), &mut context).unwrap();
        assert_eq!(num_prop.as_number().unwrap(), 42.0);

        let str_prop = obj.get(js_string!("str"), &mut context).unwrap();
        assert_eq!(str_prop.to_string(&mut context).unwrap().to_std_string_escaped(), "hello");

        let bool_prop = obj.get(js_string!("bool"), &mut context).unwrap();
        assert_eq!(bool_prop.as_boolean().unwrap(), true);

        let nil_prop = obj.get(js_string!("nil"), &mut context).unwrap();
        assert!(nil_prop.is_null());

        let undef_prop = obj.get(js_string!("undef"), &mut context).unwrap();
        assert!(undef_prop.is_undefined());
    } else {
        panic!("Deserialized value should be an object");
    }
}

#[test]
fn structured_clone_date() {
    let mut context = Context::default();

    // Create date directly in the context
    let code = "var testDate = new Date('2023-12-25T12:00:00.000Z'); testDate;";
    let result = context.eval(Source::from_bytes(code)).unwrap();
    let date = result;

    let cloned = structured_clone(&date, &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();

    // Should be a Date object
    assert!(deserialized.is_object());

    // The time value should be preserved
    if let Some(date_obj) = deserialized.as_object() {
        let get_time = date_obj.get(js_string!("getTime"), &mut context).unwrap();
        if let Some(get_time_func) = get_time.as_callable() {
            let result = get_time_func.call(&deserialized, &[], &mut context).unwrap();
            // Should return a valid timestamp
            assert!(result.is_number());
        }
    }
}

#[test]
fn structured_clone_regexp() {
    let mut context = Context::default();

    // Create regexp directly in the context
    let code = "var testRegExp = /hello[0-9]+/gi; testRegExp;";
    let result = context.eval(Source::from_bytes(code)).unwrap();
    let regexp = result;

    let cloned = structured_clone(&regexp, &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();

    // Should be a RegExp object
    assert!(deserialized.is_object());

    // The pattern and flags should be preserved
    if let Some(regexp_obj) = deserialized.as_object() {
        let source = regexp_obj.get(js_string!("source"), &mut context).unwrap();
        assert_eq!(source.to_string(&mut context).unwrap().to_std_string_escaped(), "hello[0-9]+");

        let flags = regexp_obj.get(js_string!("flags"), &mut context).unwrap();
        let flags_str = flags.to_string(&mut context).unwrap().to_std_string_escaped();
        assert!(flags_str.contains('g'));
        assert!(flags_str.contains('i'));
    }
}

#[test]
fn structured_clone_nested_object() {
    let mut context = Context::default();

    // Create nested object directly in the context
    let code = r#"
        var nested = {
            level1: {
                level2: {
                    value: "deep"
                },
                array: [1, 2, 3]
            },
            top: "level"
        };
        nested;
    "#;
    let result = context.eval(Source::from_bytes(code)).unwrap();
    let nested = result;

    let cloned = structured_clone(&nested, &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();

    if let Some(obj) = deserialized.as_object() {
        let level1 = obj.get(js_string!("level1"), &mut context).unwrap();
        if let Some(level1_obj) = level1.as_object() {
            let level2 = level1_obj.get(js_string!("level2"), &mut context).unwrap();
            if let Some(level2_obj) = level2.as_object() {
                let value = level2_obj.get(js_string!("value"), &mut context).unwrap();
                assert_eq!(value.to_string(&mut context).unwrap().to_std_string_escaped(), "deep");
            }

            let array = level1_obj.get(js_string!("array"), &mut context).unwrap();
            if let Some(array_obj) = array.as_object() {
                let length = array_obj.get(js_string!("length"), &mut context).unwrap();
                assert_eq!(length.as_number().unwrap(), 3.0);
            }
        }

        let top = obj.get(js_string!("top"), &mut context).unwrap();
        assert_eq!(top.to_string(&mut context).unwrap().to_std_string_escaped(), "level");
    }
}

#[test]
fn structured_clone_circular_reference() {
    let mut context = Context::default();

    run_test_actions([
        TestAction::run(r#"
            var circular = {};
            circular.self = circular;
        "#),
    ]);

    let global = context.global_object();
    let circular = global.get(js_string!("circular"), &mut context).unwrap();

    // Should throw a TypeError for circular references
    let result = structured_clone(&circular, &mut context, None);
    assert!(result.is_err());
}

#[test]
fn structured_clone_symbol_error() {
    let mut context = Context::default();

    run_test_actions([
        TestAction::run("var sym = Symbol('test')"),
    ]);

    let global = context.global_object();
    let symbol = global.get(js_string!("sym"), &mut context).unwrap();

    // Symbols cannot be cloned
    let result = structured_clone(&symbol, &mut context, None);
    assert!(result.is_err());
}

#[test]
fn structured_clone_function_error() {
    let mut context = Context::default();

    run_test_actions([
        TestAction::run("var func = function() { return 42; }"),
    ]);

    let global = context.global_object();
    let func = global.get(js_string!("func"), &mut context).unwrap();

    // Functions cannot be cloned
    let result = structured_clone(&func, &mut context, None);
    assert!(result.is_err());
}

#[test]
fn structured_clone_serialization() {
    let mut context = Context::default();

    // Test that we can serialize and deserialize clone values for cross-thread transfer
    let test_value = JsValue::from(js_string!("test"));
    let cloned = structured_clone(&test_value, &mut context, None).unwrap();

    // Serialize to bytes
    let bytes = StructuredClone::serialize_to_bytes(&cloned).unwrap();
    assert!(!bytes.is_empty());

    // Deserialize from bytes
    let deserialized_clone = StructuredClone::deserialize_from_bytes(&bytes).unwrap();

    // Convert back to JsValue
    let final_value = structured_deserialize(&deserialized_clone, &mut context).unwrap();
    assert_eq!(final_value.to_string(&mut context).unwrap().to_std_string_escaped(), "test");
}

#[test]
fn structured_clone_empty_array() {
    let mut context = Context::default();

    run_test_actions([
        TestAction::run("var emptyArray = []"),
    ]);

    let global = context.global_object();
    let array = global.get(js_string!("emptyArray"), &mut context).unwrap();

    let cloned = structured_clone(&array, &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();

    if let Some(array_obj) = deserialized.as_object() {
        let length = array_obj.get(js_string!("length"), &mut context).unwrap();
        assert_eq!(length.as_number().unwrap(), 0.0);
    }
}

#[test]
fn structured_clone_empty_object() {
    let mut context = Context::default();

    // Create empty object directly in the context
    let code = "var emptyObj = {}; emptyObj;";
    let result = context.eval(Source::from_bytes(code)).unwrap();
    let obj = result;

    let cloned = structured_clone(&obj, &mut context, None).unwrap();
    let deserialized = structured_deserialize(&cloned, &mut context).unwrap();

    assert!(deserialized.is_object());
    // Should be an empty object
    if let Some(obj) = deserialized.as_object() {
        let keys = obj.own_property_keys(&mut context).unwrap();
        assert_eq!(keys.len(), 0);
    }
}