#![allow(clippy::similar_names)]
#![allow(unused_must_use)]
use super::{LinearPosition, LinearSpan, Position, Span};

/// Checks that we cannot create a position with 0 as the column.
#[test]
#[should_panic(expected = "column number cannot be 0")]
fn invalid_position_column() {
    Position::new(10, 0);
}

/// Checks that we cannot create a position with 0 as the line.
#[test]
#[should_panic(expected = "line number cannot be 0")]
fn invalid_position_line() {
    Position::new(0, 10);
}

/// Checks that the `PartialEq` implementation of `Position` is consistent.
#[test]
fn position_equality() {
    assert_eq!(Position::new(10, 50), Position::new(10, 50));
    assert_ne!(Position::new(10, 50), Position::new(10, 51));
    assert_ne!(Position::new(10, 50), Position::new(11, 50));
    assert_ne!(Position::new(10, 50), Position::new(11, 51));
}

/// Checks that the `PartialEq` implementation of `LinearPosition` is consistent.
#[test]
fn linear_position_equality() {
    assert_eq!(LinearPosition::new(1050), LinearPosition::new(1050));
    assert_ne!(LinearPosition::new(1050), LinearPosition::new(1051));
}

/// Checks that the `PartialOrd` implementation of `Position` is consistent.
#[test]
fn position_order() {
    assert!(Position::new(10, 50) < Position::new(10, 51));
    assert!(Position::new(9, 50) < Position::new(10, 50));
    assert!(Position::new(10, 50) < Position::new(11, 51));
    assert!(Position::new(10, 50) < Position::new(11, 49));

    assert!(Position::new(10, 51) > Position::new(10, 50));
    assert!(Position::new(10, 50) > Position::new(9, 50));
    assert!(Position::new(11, 51) > Position::new(10, 50));
    assert!(Position::new(11, 49) > Position::new(10, 50));
}

/// Checks that the `PartialOrd` implementation of `LinearPosition` is consistent.
#[test]
fn linear_position_order() {
    assert!(LinearPosition::new(1050) < LinearPosition::new(1051));
    assert!(LinearPosition::new(1149) > LinearPosition::new(1050));
}

/// Checks that the position getters actually retrieve correct values.
#[test]
fn position_getters() {
    let pos = Position::new(10, 50);
    assert_eq!(pos.line_number(), 10);
    assert_eq!(pos.column_number(), 50);
}

/// Checks that the string representation of a position is correct.
#[test]
fn position_to_string() {
    let pos = Position::new(10, 50);

    assert_eq!("10:50", pos.to_string());
    assert_eq!("10:50", pos.to_string());
}

/// Checks that we cannot create an invalid span.
#[test]
#[should_panic(expected = "a span cannot start after its end")]
fn invalid_span() {
    let a = Position::new(10, 30);
    let b = Position::new(10, 50);
    Span::new(b, a);
}

/// Checks that we cannot create an invalid linear span.
#[test]
#[should_panic(expected = "a linear span cannot start after its end")]
fn invalid_linear_span() {
    let a = LinearPosition::new(1030);
    let b = LinearPosition::new(1050);
    LinearSpan::new(b, a);
}

/// Checks that we can create valid spans.
#[test]
fn span_creation() {
    let a = Position::new(10, 30);
    let b = Position::new(10, 50);

    Span::new(a, b);
    Span::new(a, a);
    Span::from(a);
}

/// Checks that we can create valid linear spans.
#[test]
fn linear_span_creation() {
    let a = LinearPosition::new(1030);
    let b = LinearPosition::new(1050);

    LinearSpan::new(a, b);
    let span_aa = LinearSpan::new(a, a);
    assert_eq!(LinearSpan::from(a), span_aa);
}

/// Checks that the `PartialEq` implementation of `Span` is consistent.
#[test]
fn span_equality() {
    let a = Position::new(10, 50);
    let b = Position::new(10, 52);
    let c = Position::new(11, 20);

    let span_ab = Span::new(a, b);
    let span_ab_2 = Span::new(a, b);
    let span_ac = Span::new(a, c);
    let span_bc = Span::new(b, c);

    assert_eq!(span_ab, span_ab_2);
    assert_ne!(span_ab, span_ac);
    assert_ne!(span_ab, span_bc);
    assert_ne!(span_bc, span_ac);

    let span_a = Span::from(a);
    let span_aa = Span::new(a, a);

    assert_eq!(span_a, span_aa);
}

/// Checks that the `PartialEq` implementation of `LinearSpan` is consistent.
#[test]
fn linear_span_equality() {
    let a = LinearPosition::new(1030);
    let b = LinearPosition::new(1050);
    let c = LinearPosition::new(1150);

    let span_ab = LinearSpan::new(a, b);
    let span_ab_2 = LinearSpan::new(a, b);
    let span_ac = LinearSpan::new(a, c);
    let span_bc = LinearSpan::new(b, c);

    assert_eq!(span_ab, span_ab_2);
    assert_ne!(span_ab, span_ac);
    assert_ne!(span_ab, span_bc);
    assert_ne!(span_bc, span_ac);
}

/// Checks that the getters retrieve the correct value.
#[test]
fn span_getters() {
    let a = Position::new(10, 50);
    let b = Position::new(10, 52);

    let span = Span::new(a, b);

    assert_eq!(span.start(), a);
    assert_eq!(span.end(), b);
}

/// Checks that the `Span::contains()` method works properly.
#[test]
fn span_contains() {
    let a = Position::new(10, 50);
    let b = Position::new(10, 52);
    let c = Position::new(11, 20);
    let d = Position::new(12, 5);

    let span_ac = Span::new(a, c);
    assert!(span_ac.contains(b));

    let span_ab = Span::new(a, b);
    let span_cd = Span::new(c, d);

    assert!(!span_ab.contains(span_cd));
    assert!(span_ab.contains(b));

    let span_ad = Span::new(a, d);
    let span_bc = Span::new(b, c);

    assert!(span_ad.contains(span_bc));
    assert!(!span_bc.contains(span_ad));

    let span_ac = Span::new(a, c);
    let span_bd = Span::new(b, d);

    assert!(!span_ac.contains(span_bd));
    assert!(!span_bd.contains(span_ac));
}

/// Checks that the `LinearSpan::contains()` method works properly.
#[test]
fn linear_span_contains() {
    let a = LinearPosition::new(1050);
    let b = LinearPosition::new(1080);
    let c = LinearPosition::new(1120);
    let d = LinearPosition::new(1125);

    let span_ac = LinearSpan::new(a, c);
    assert!(span_ac.contains(b));

    let span_ab = LinearSpan::new(a, b);
    let span_cd = LinearSpan::new(c, d);

    assert!(!span_ab.contains(span_cd));
    assert!(span_ab.contains(b));

    let span_ad = LinearSpan::new(a, d);
    let span_bc = LinearSpan::new(b, c);

    assert!(span_ad.contains(span_bc));
    assert!(!span_bc.contains(span_ad));

    let span_ac = LinearSpan::new(a, c);
    let span_bd = LinearSpan::new(b, d);

    assert!(!span_ac.contains(span_bd));
    assert!(!span_bd.contains(span_ac));
}

/// Checks that the string representation of a span is correct.
#[test]
fn span_to_string() {
    let a = Position::new(10, 50);
    let b = Position::new(11, 20);
    let span = Span::new(a, b);

    assert_eq!("[10:50..11:20]", span.to_string());
    assert_eq!("[10:50..11:20]", span.to_string());
}

/// Checks that the ordering of spans is correct.
#[test]
fn span_ordering() {
    let a = Position::new(10, 50);
    let b = Position::new(10, 52);
    let c = Position::new(11, 20);
    let d = Position::new(12, 5);

    let span_ab = Span::new(a, b);
    let span_cd = Span::new(c, d);

    assert!(span_ab < span_cd);
    assert!(span_cd > span_ab);
}

/// Checks that the ordering of linear spans is correct.
#[test]
fn linear_span_ordering() {
    let a = LinearPosition::new(1050);
    let b = LinearPosition::new(1052);
    let c = LinearPosition::new(1120);
    let d = LinearPosition::new(1125);

    let span_ab = LinearSpan::new(a, b);
    let span_cd = LinearSpan::new(c, d);

    let span_ac = LinearSpan::new(a, c);
    let span_bd = LinearSpan::new(b, d);

    assert!(span_ab < span_cd);
    assert!(span_cd > span_ab);
    assert_eq!(span_bd.partial_cmp(&span_ac), None);
    assert_eq!(span_ac.partial_cmp(&span_bd), None);
}

/// Checks that the ordering of linear spans is correct.
#[test]
fn linear_union() {
    let a = LinearPosition::new(1050);
    let b = LinearPosition::new(1052);
    let c = LinearPosition::new(1120);
    let d = LinearPosition::new(1125);

    let span_ab = LinearSpan::new(a, b);
    let span_ad = LinearSpan::new(a, d);
    let span_bc = LinearSpan::new(b, c);
    let span_cd = LinearSpan::new(c, d);
    let span_ac = LinearSpan::new(a, c);
    let span_bd = LinearSpan::new(b, d);

    assert_eq!(span_bd.union(a), span_ad);
    assert_eq!(span_ab.union(a), span_ab);
    assert_eq!(span_bd.union(span_ac), span_ad);
    assert_eq!(span_ac.union(span_bd), span_ad);
    assert_eq!(span_ac.union(span_bd), span_ad);
    assert_eq!(span_ac.union(b), span_ac);
    assert_eq!(span_bc.union(span_ab), span_ac);
    assert_eq!(span_ab.union(span_bc), span_ac);
    assert_eq!(span_ac.union(span_ab), span_ac);
    assert_eq!(span_cd.union(a), span_ad);
    assert_eq!(span_cd.union(span_bc), span_bd);
}
