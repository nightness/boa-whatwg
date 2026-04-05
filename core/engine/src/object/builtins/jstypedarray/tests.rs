//! Tests for the parent module.

use super::*;
use crate::{Context, JsValue};

#[test]
fn typed_iterators_uint8() {
    let context = &mut Context::default();
    let vec = vec![1u8, 2, 3, 4, 5, 6, 7, 8];

    let array = JsUint8Array::from_iter(vec.clone(), context).unwrap();
    let vec2 = array.iter(context).collect::<Vec<_>>();
    assert_eq!(vec, vec2);
}

#[test]
fn typed_iterators_uint32() {
    let context = &mut Context::default();
    let vec = vec![1u32, 2, 0xFFFF, 4, 0xFF12_3456, 6, 7, 8];

    let array = JsUint32Array::from_iter(vec.clone(), context).unwrap();
    let vec2 = array.iter(context).collect::<Vec<_>>();
    assert_eq!(vec, vec2);
}

#[test]
fn typed_iterators_f32() {
    let context = &mut Context::default();
    let vec = vec![0.1f32, 0.2, 0.3, 0.4, 1.1, 9.99999];

    let array = JsFloat32Array::from_iter(vec.clone(), context).unwrap();
    let vec2 = array.iter(context).collect::<Vec<_>>();
    assert_eq!(vec, vec2);
}

#[test]
fn uint8_array_to_vec_roundtrip() {
    let context = &mut Context::default();
    let data: Vec<u8> = (0..=255).collect();
    let array = JsUint8Array::from_iter(data.clone(), context).unwrap();
    let bytes = array.to_vec(context).unwrap();
    assert_eq!(bytes, data);
}

#[test]
fn typed_array_to_string() {
    let context = &mut Context::default();
    let vec = vec![1u8, 2, 3];
    let array = JsUint8Array::from_iter(vec, context).unwrap();
    assert_eq!(
        array.to_string(context).unwrap(),
        crate::js_string!("1,2,3")
    );
}

#[test]
fn typed_array_entries() {
    let context = &mut Context::default();
    let vec = vec![1u8, 2];
    let array = JsUint8Array::from_iter(vec, context).unwrap();
    let entries = array.entries(context).unwrap();
    let mut entries_vec = Vec::new();
    let next_str = crate::js_string!("next");
    loop {
        let next_fn = entries
            .as_object()
            .unwrap()
            .get(next_str.clone(), context)
            .unwrap();
        let result = next_fn
            .as_object()
            .unwrap()
            .call(&entries, &[], context)
            .unwrap();
        if result
            .as_object()
            .unwrap()
            .get(crate::js_string!("done"), context)
            .unwrap()
            .to_boolean()
        {
            break;
        }
        entries_vec.push(
            result
                .as_object()
                .unwrap()
                .get(crate::js_string!("value"), context)
                .unwrap(),
        );
    }
    assert_eq!(entries_vec.len(), 2);
}

#[test]
fn typed_array_keys() {
    let context = &mut Context::default();
    let vec = vec![1u8, 2];
    let array = JsUint8Array::from_iter(vec, context).unwrap();
    let keys = array.keys(context).unwrap();
    let mut keys_vec = Vec::new();
    let next_str = crate::js_string!("next");
    loop {
        let next_fn = keys
            .as_object()
            .unwrap()
            .get(next_str.clone(), context)
            .unwrap();
        let result = next_fn
            .as_object()
            .unwrap()
            .call(&keys, &[], context)
            .unwrap();
        if result
            .as_object()
            .unwrap()
            .get(crate::js_string!("done"), context)
            .unwrap()
            .to_boolean()
        {
            break;
        }
        keys_vec.push(
            result
                .as_object()
                .unwrap()
                .get(crate::js_string!("value"), context)
                .unwrap(),
        );
    }
    assert_eq!(keys_vec, vec![JsValue::new(0), JsValue::new(1)]);
}

#[test]
fn typed_array_values() {
    let context = &mut Context::default();
    let vec = vec![1u8, 2];
    let array = JsUint8Array::from_iter(vec, context).unwrap();
    let values = array.values(context).unwrap();
    let mut values_vec = Vec::new();
    let next_str = crate::js_string!("next");
    loop {
        let next_fn = values
            .as_object()
            .unwrap()
            .get(next_str.clone(), context)
            .unwrap();
        let result = next_fn
            .as_object()
            .unwrap()
            .call(&values, &[], context)
            .unwrap();
        if result
            .as_object()
            .unwrap()
            .get(crate::js_string!("done"), context)
            .unwrap()
            .to_boolean()
        {
            break;
        }
        values_vec.push(
            result
                .as_object()
                .unwrap()
                .get(crate::js_string!("value"), context)
                .unwrap(),
        );
    }
    assert_eq!(values_vec, vec![JsValue::new(1), JsValue::new(2)]);
}

#[test]
fn typed_array_iterator() {
    let context = &mut Context::default();
    let array = JsUint8Array::from_iter(vec![1u8, 2], context).unwrap();
    let values = array.iterator(context).unwrap();
    let mut values_vec = Vec::new();
    let next_str = crate::js_string!("next");
    loop {
        let next_fn = values
            .as_object()
            .unwrap()
            .get(next_str.clone(), context)
            .unwrap();
        let result = next_fn
            .as_object()
            .unwrap()
            .call(&values, &[], context)
            .unwrap();
        if result
            .as_object()
            .unwrap()
            .get(crate::js_string!("done"), context)
            .unwrap()
            .to_boolean()
        {
            break;
        }
        values_vec.push(
            result
                .as_object()
                .unwrap()
                .get(crate::js_string!("value"), context)
                .unwrap(),
        );
    }
    assert_eq!(values_vec, vec![JsValue::new(1), JsValue::new(2)]);
}

#[test]
fn typed_array_to_reversed() {
    let context = &mut Context::default();
    let array = JsUint8Array::from_iter(vec![3u8, 1, 2], context).unwrap();

    let reversed = array.to_reversed(context).unwrap();

    // New array has reversed order
    assert_eq!(reversed.at(0i64, context).unwrap(), JsValue::new(2));
    assert_eq!(reversed.at(1i64, context).unwrap(), JsValue::new(1));
    assert_eq!(reversed.at(2i64, context).unwrap(), JsValue::new(3));

    // Original is unchanged
    assert_eq!(array.at(0i64, context).unwrap(), JsValue::new(3));
    assert_eq!(array.at(1i64, context).unwrap(), JsValue::new(1));
    assert_eq!(array.at(2i64, context).unwrap(), JsValue::new(2));
}

#[test]
fn typed_array_to_sorted() {
    let context = &mut Context::default();
    let array = JsUint8Array::from_iter(vec![3u8, 1, 2], context).unwrap();

    let sorted = array.to_sorted(None, context).unwrap();

    // New array is sorted
    assert_eq!(sorted.at(0i64, context).unwrap(), JsValue::new(1));
    assert_eq!(sorted.at(1i64, context).unwrap(), JsValue::new(2));
    assert_eq!(sorted.at(2i64, context).unwrap(), JsValue::new(3));

    // Original is unchanged
    assert_eq!(array.at(0i64, context).unwrap(), JsValue::new(3));
    assert_eq!(array.at(1i64, context).unwrap(), JsValue::new(1));
    assert_eq!(array.at(2i64, context).unwrap(), JsValue::new(2));
}

#[test]
fn typed_array_to_locale_string() {
    let context = &mut Context::default();
    let array = JsUint8Array::from_iter(vec![1u8, 2, 3], context).unwrap();

    let result = array.to_locale_string(None, None, context).unwrap();

    let result_str = result
        .as_string()
        .expect("toLocaleString should return a string");
    assert!(
        result_str.to_std_string_escaped().contains('1'),
        "result should contain element values"
    );
}
