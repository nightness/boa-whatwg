use boa_macros::js_str;
use indoc::indoc;
use serde_json::json;

use crate::{
    Context, JsObject, JsValue, TestAction, js_string, object::JsArray, run_test_actions,
};

#[test]
fn json_conversions() {
    const DATA: &str = indoc! {r#"
        {
            "name": "John Doe",
            "age": 43,
            "minor": false,
            "adult": true,
            "extra": {
                "address": null
            },
            "phones": [
                "+44 1234567",
                -45,
                {},
                true
            ],
            "7.3": "random text",
            "100": 1000,
            "24": 42
        }
    "#};

    run_test_actions([TestAction::inspect_context(|ctx| {
        let json: serde_json::Value = serde_json::from_str(DATA).unwrap();
        assert!(json.is_object());

        let value = JsValue::from_json(&json, ctx).unwrap();
        let obj = value.as_object().unwrap();
        assert_eq!(
            obj.get(js_str!("name"), ctx).unwrap(),
            js_str!("John Doe").into()
        );
        assert_eq!(obj.get(js_str!("age"), ctx).unwrap(), 43_i32.into());
        assert_eq!(obj.get(js_str!("minor"), ctx).unwrap(), false.into());
        assert_eq!(obj.get(js_str!("adult"), ctx).unwrap(), true.into());

        assert_eq!(
            obj.get(js_str!("7.3"), ctx).unwrap(),
            js_string!("random text").into()
        );
        assert_eq!(obj.get(js_str!("100"), ctx).unwrap(), 1000.into());
        assert_eq!(obj.get(js_str!("24"), ctx).unwrap(), 42.into());

        {
            let extra = obj.get(js_str!("extra"), ctx).unwrap();
            let extra = extra.as_object().unwrap();
            assert!(extra.get(js_str!("address"), ctx).unwrap().is_null());
        }
        {
            let phones = obj.get(js_str!("phones"), ctx).unwrap();
            let phones = phones.as_object().unwrap();

            let arr = JsArray::from_object(phones.clone()).unwrap();
            assert_eq!(arr.at(0, ctx).unwrap(), js_str!("+44 1234567").into());
            assert_eq!(arr.at(1, ctx).unwrap(), JsValue::from(-45_i32));
            assert!(arr.at(2, ctx).unwrap().is_object());
            assert_eq!(arr.at(3, ctx).unwrap(), true.into());
        }

        assert_eq!(Some(json), value.to_json(ctx).unwrap());
    })]);
}

#[test]
fn integer_ops_to_json() {
    run_test_actions([
        TestAction::assert_with_op("1000000 + 500", |v, ctx| {
            v.to_json(ctx).unwrap() == Some(json!(1_000_500))
        }),
        TestAction::assert_with_op("1000000 - 500", |v, ctx| {
            v.to_json(ctx).unwrap() == Some(json!(999_500))
        }),
        TestAction::assert_with_op("1000000 * 500", |v, ctx| {
            v.to_json(ctx).unwrap() == Some(json!(500_000_000))
        }),
        TestAction::assert_with_op("1000000 / 500", |v, ctx| {
            v.to_json(ctx).unwrap() == Some(json!(2_000))
        }),
        TestAction::assert_with_op("233894 % 500", |v, ctx| {
            v.to_json(ctx).unwrap() == Some(json!(394))
        }),
        TestAction::assert_with_op("36 ** 5", |v, ctx| {
            v.to_json(ctx).unwrap() == Some(json!(60_466_176))
        }),
    ]);
}

#[test]
fn to_json_cyclic() {
    let mut context = Context::default();
    let obj = JsObject::with_null_proto();
    obj.create_data_property(js_string!("a"), obj.clone(), &mut context)
        .expect("should create data property");

    assert!(
        JsValue::from(obj)
            .to_json(&mut context)
            .unwrap_err()
            .to_string()
            .starts_with("TypeError: cyclic object value"),
    );
}

#[test]
fn to_json_undefined() {
    let mut context = Context::default();
    let undefined_value = JsValue::undefined();
    assert!(undefined_value.to_json(&mut context).unwrap().is_none());
}

#[test]
fn to_json_undefined_in_structure() {
    let mut context = Context::default();
    let object_with_undefined = {
        // Defining the following structure:
        // {
        //     "outer_a": 1,
        //     "outer_b": undefined,
        //     "outer_c": [2, undefined, 3, { "inner_a": undefined }]
        // }

        let inner = JsObject::with_null_proto();
        inner
            .create_data_property(js_string!("inner_a"), JsValue::undefined(), &mut context)
            .expect("should add property");

        let array = JsArray::new(&mut context);
        array.push(2, &mut context).expect("should push");
        array
            .push(JsValue::undefined(), &mut context)
            .expect("should push");
        array.push(3, &mut context).expect("should push");
        array.push(inner, &mut context).expect("should push");

        let outer = JsObject::with_null_proto();
        outer
            .create_data_property(js_string!("outer_a"), JsValue::new(1), &mut context)
            .expect("should add property");
        outer
            .create_data_property(js_string!("outer_b"), JsValue::undefined(), &mut context)
            .expect("should add property");
        outer
            .create_data_property(js_string!("outer_c"), array, &mut context)
            .expect("should add property");

        JsValue::from(outer)
    };

    assert_eq!(
        Some(json!({
            "outer_a": 1,
            "outer_c": [2, null, 3, { }]
        })),
        object_with_undefined.to_json(&mut context).unwrap()
    );
}
