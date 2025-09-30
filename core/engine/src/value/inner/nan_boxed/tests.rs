//! Tests for the parent module.

use super::*;
use crate::Context;

macro_rules! assert_type {
    (@@is $value: ident, $u: literal, $n: literal, $b: literal, $i: literal, $f: literal, $bi: literal, $s: literal, $o: literal, $sy: literal) => {
        assert_eq!($u  != 0, $value.is_undefined());
        assert_eq!($n  != 0, $value.is_null());
        assert_eq!($b  != 0, $value.is_bool());
        assert_eq!($i  != 0, $value.is_integer32());
        assert_eq!($f  != 0, $value.is_float64());
        assert_eq!($bi != 0, $value.is_bigint());
        assert_eq!($s  != 0, $value.is_string());
        assert_eq!($o  != 0, $value.is_object());
        assert_eq!($sy != 0, $value.is_symbol());
    };
    (@@as $value: ident, $u: literal, $n: literal, $b: literal, $i: literal, $f: literal, $bi: literal, $s: literal, $o: literal, $sy: literal) => {
        if $b  == 0 { assert_eq!($value.as_bool(), None); }
        if $i  == 0 { assert_eq!($value.as_integer32(), None); }
        if $f  == 0 { assert_eq!($value.as_float64(), None); }
        if $bi == 0 { assert_eq!($value.as_bigint(), None); }
        if $s  == 0 { assert_eq!($value.as_string(), None); }
        if $o  == 0 { assert_eq!($value.as_object(), None); }
        if $sy == 0 { assert_eq!($value.as_symbol(), None); }
    };
    ($value: ident is undefined) => {
        assert_type!(@@is $value, 1, 0, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!($value.as_variant(), JsVariant::Undefined);
    };
    ($value: ident is null) => {
        assert_type!(@@is $value, 0, 1, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!($value.as_variant(), JsVariant::Null);
    };
    ($value: ident is bool($scalar: ident)) => {
        assert_type!(@@is $value, 0, 0, 1, 0, 0, 0, 0, 0, 0);
        assert_type!(@@as $value, 0, 0, 1, 0, 0, 0, 0, 0, 0);
        assert_eq!(Some($scalar), $value.as_bool());
        assert_eq!($value.as_variant(), JsVariant::Boolean($scalar));
    };
    ($value: ident is integer($scalar: ident)) => {
        assert_type!(@@is $value, 0, 0, 0, 1, 0, 0, 0, 0, 0);
        assert_type!(@@as $value, 0, 0, 0, 1, 0, 0, 0, 0, 0);
        assert_eq!(Some($scalar), $value.as_integer32());
        assert_eq!($value.as_variant(), JsVariant::Integer32($scalar));
    };
    ($value: ident is float($scalar: ident)) => {
        assert_type!(@@is $value, 0, 0, 0, 0, 1, 0, 0, 0, 0);
        assert_type!(@@as $value, 0, 0, 0, 0, 1, 0, 0, 0, 0);
        assert_eq!(Some($scalar), $value.as_float64());
        // Verify parity.
        assert_eq!(Some(1.0 / $scalar), $value.as_float64().map(|f| 1.0 / f));
        assert_eq!($value.as_variant(), JsVariant::Float64($scalar));

        // Verify that the clone is still the same.
        let new_value = $value.clone();

        assert_eq!(Some($scalar), new_value.as_float64());
        assert_eq!($value.as_float64(), new_value.as_float64());
        // Verify parity.
        assert_eq!(Some(1.0 / $scalar), new_value.as_float64().map(|f| 1.0 / f));
        assert_eq!(new_value.as_variant(), JsVariant::Float64($scalar));

        let JsVariant::Float64(new_scalar) = new_value.as_variant() else {
            panic!("Expected Float64, got {:?}", new_value.as_variant());
        };
        assert_eq!(Some(new_scalar), new_value.as_float64());
        assert_eq!($value.as_float64(), new_value.as_float64());
        // Verify parity.
        assert_eq!(Some(1.0 / new_scalar), new_value.as_float64().map(|f| 1.0 / f));
        assert_eq!(new_value.as_variant(), JsVariant::Float64(new_scalar));
    };
    ($value: ident is nan) => {
        assert_type!(@@is $value, 0, 0, 0, 0, 1, 0, 0, 0, 0);
        assert_type!(@@as $value, 0, 0, 0, 0, 1, 0, 0, 0, 0);
        assert!($value.as_float64().unwrap().is_nan());
        assert!(matches!($value.as_variant(), JsVariant::Float64(f) if f.is_nan()));
    };
    ($value: ident is bigint($scalar: ident)) => {
        assert_type!(@@is $value, 0, 0, 0, 0, 0, 1, 0, 0, 0);
        assert_type!(@@as $value, 0, 0, 0, 0, 0, 1, 0, 0, 0);
        assert_eq!(Some(&$scalar), $value.as_bigint().as_ref());
        assert_eq!($value.as_variant(), JsVariant::BigInt($scalar));
    };
    ($value: ident is object($scalar: ident)) => {
        assert_type!(@@is $value, 0, 0, 0, 0, 0, 0, 0, 1, 0);
        assert_type!(@@as $value, 0, 0, 0, 0, 0, 0, 0, 1, 0);
        assert_eq!(Some(&$scalar), $value.as_object().as_ref());
        assert_eq!($value.as_variant(), JsVariant::Object($scalar));
    };
    ($value: ident is symbol($scalar: ident)) => {
        assert_type!(@@is $value, 0, 0, 0, 0, 0, 0, 0, 0, 1);
        assert_type!(@@as $value, 0, 0, 0, 0, 0, 0, 0, 0, 1);
        assert_eq!(Some(&$scalar), $value.as_symbol().as_ref());
        assert_eq!($value.as_variant(), JsVariant::Symbol($scalar));
    };
    ($value: ident is string($scalar: ident)) => {
        assert_type!(@@is $value, 0, 0, 0, 0, 0, 0, 1, 0, 0);
        assert_type!(@@as $value, 0, 0, 0, 0, 0, 0, 1, 0, 0);
        assert_eq!(Some(&$scalar), $value.as_string().as_ref());
        assert_eq!($value.as_variant(), JsVariant::String($scalar));
    };
}

#[test]
fn null() {
    let v = NanBoxedValue::null();
    assert_type!(v is null);
}

#[test]
fn undefined() {
    let v = NanBoxedValue::undefined();
    assert_type!(v is undefined);
}

#[test]
fn boolean() {
    let v = NanBoxedValue::boolean(true);
    assert_type!(v is bool(true));

    let v = NanBoxedValue::boolean(false);
    assert_type!(v is bool(false));
}

#[test]
fn integer() {
    fn assert_integer(i: i32) {
        let v = NanBoxedValue::integer32(i);
        assert_type!(v is integer(i));
    }

    assert_integer(0);
    assert_integer(1);
    assert_integer(-1);
    assert_integer(42);
    assert_integer(-42);
    assert_integer(i32::MAX);
    assert_integer(i32::MIN);
    assert_integer(i32::MAX - 1);
    assert_integer(i32::MIN + 1);
}

#[test]
#[allow(clippy::float_cmp)]
fn float() {
    fn assert_float(f: f64) {
        let v = NanBoxedValue::float64(f);
        assert_type!(v is float(f));
    }

    assert_float(0.0);
    assert_float(-0.0);
    assert_float(0.1 + 0.2);
    assert_float(-42.123);
    assert_float(f64::INFINITY);
    assert_float(f64::NEG_INFINITY);

    // Some edge cases around zeroes.
    let neg_zero = NanBoxedValue::float64(-0.0);
    assert!(neg_zero.as_float64().unwrap().is_sign_negative());
    assert_eq!(0.0f64, neg_zero.as_float64().unwrap());

    let pos_zero = NanBoxedValue::float64(0.0);
    assert!(!pos_zero.as_float64().unwrap().is_sign_negative());
    assert_eq!(0.0f64, pos_zero.as_float64().unwrap());

    assert_eq!(pos_zero.as_float64(), neg_zero.as_float64());

    let nan = NanBoxedValue::float64(f64::NAN);
    assert_type!(nan is nan);
}

#[test]
fn bigint() {
    let bigint = JsBigInt::from(42);
    let v = NanBoxedValue::bigint(bigint.clone());
    assert_type!(v is bigint(bigint));
}

#[test]
fn object() {
    let object = JsObject::with_null_proto();
    let v = NanBoxedValue::object(object.clone());
    assert_type!(v is object(object));
}

#[test]
fn string() {
    let str = crate::js_string!("Hello World");
    let v = NanBoxedValue::string(str.clone());
    assert_type!(v is string(str));
}

#[test]
fn symbol() {
    let sym = JsSymbol::new(Some(JsString::from("Hello World"))).unwrap();
    let v = NanBoxedValue::symbol(sym.clone());
    assert_type!(v is symbol(sym));

    let sym = JsSymbol::new(None).unwrap();
    let v = NanBoxedValue::symbol(sym.clone());
    assert_type!(v is symbol(sym));
}