use super::*;

#[test]
fn test_eq() {
    let int: i64 = 42;
    let int_or_inf = IntegerOrInfinity::Integer(10);
    assert!(int != int_or_inf);
    assert!(int_or_inf != int);

    let int: i64 = 10;
    assert!(int == int_or_inf);
    assert!(int_or_inf == int);
}

#[test]
fn test_ord() {
    let int: i64 = 42;

    let int_or_inf = IntegerOrInfinity::Integer(10);
    assert!(int_or_inf < int);
    assert!(int > int_or_inf);

    let int_or_inf = IntegerOrInfinity::Integer(100);
    assert!(int_or_inf > int);
    assert!(int < int_or_inf);

    let int_or_inf = IntegerOrInfinity::PositiveInfinity;
    assert!(int_or_inf > int);
    assert!(int < int_or_inf);

    let int_or_inf = IntegerOrInfinity::NegativeInfinity;
    assert!(int_or_inf < int);
    assert!(int > int_or_inf);
}
