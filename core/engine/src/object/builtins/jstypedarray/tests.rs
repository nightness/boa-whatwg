//! Tests for the parent module.

use super::*;
use crate::Context;

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