use std::{
    cmp::Ordering,
    fmt::{self, Debug},
    num::NonZeroU32,
};

/// A position in the ECMAScript source code.
///
/// Stores both the column number and the line number.
///
/// ## Similar Implementations
/// [V8: Location](https://cs.chromium.org/chromium/src/v8/src/parsing/scanner.h?type=cs&q=isValid+Location&g=0&l=216)
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    /// Line number.
    line_number: NonZeroU32,
    /// Column number.
    column_number: NonZeroU32,
}

impl Default for Position {
    /// Creates a new [`Position`] with line and column set to `1`.
    #[inline]
    fn default() -> Self {
        Self::new(1, 1)
    }
}

impl Position {
    /// Creates a new `Position` from Non-Zero values.
    ///
    /// # Panics
    ///
    /// Will panic if the line number or column number is zero.
    #[inline]
    #[track_caller]
    #[must_use]
    pub const fn new(line_number: u32, column_number: u32) -> Self {
        Self {
            line_number: NonZeroU32::new(line_number).expect("line number cannot be 0"),
            column_number: NonZeroU32::new(column_number).expect("column number cannot be 0"),
        }
    }

    /// Gets the line number of the position.
    #[inline]
    #[must_use]
    pub const fn line_number(self) -> u32 {
        self.line_number.get()
    }

    /// Gets the column number of the position.
    #[inline]
    #[must_use]
    pub const fn column_number(self) -> u32 {
        self.column_number.get()
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line_number, self.column_number)
    }
}

impl From<PositionGroup> for Position {
    #[inline]
    fn from(value: PositionGroup) -> Self {
        value.pos
    }
}

impl From<(NonZeroU32, NonZeroU32)> for Position {
    #[inline]
    fn from(value: (NonZeroU32, NonZeroU32)) -> Self {
        Position {
            line_number: value.0,
            column_number: value.1,
        }
    }
}

impl From<(u32, u32)> for Position {
    #[inline]
    #[track_caller]
    fn from(value: (u32, u32)) -> Self {
        Position::new(value.0, value.1)
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for Span {
    fn arbitrary(_u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Span::EMPTY)
    }
}

/// Linear position in the ECMAScript source code.
///
/// Stores linear position in the source code.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LinearPosition {
    pos: usize,
}

impl LinearPosition {
    /// Creates a new `LinearPosition`.
    #[inline]
    #[must_use]
    pub const fn new(pos: usize) -> Self {
        Self { pos }
    }
    /// Gets the linear position.
    #[inline]
    #[must_use]
    pub const fn pos(self) -> usize {
        self.pos
    }
}
impl fmt::Display for LinearPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pos())
    }
}

/// A span in the ECMAScript source code.
///
/// Stores a start position and an end position.
///
/// Note that spans are of the form [start, end) i.e. that the start position is inclusive
/// and the end position is exclusive.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    start: Position,
    end: Position,
}

impl Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Span(({}, {}), ({}, {}))",
            self.start.line_number,
            self.start.column_number,
            self.end.line_number,
            self.end.column_number,
        )
    }
}

impl Span {
    pub(crate) const EMPTY: Span = Span {
        start: Position::new(1, 1),
        end: Position::new(1, 1),
    };

    /// Creates a new `Span`.
    ///
    /// # Panics
    ///
    /// Panics if the start position is bigger than the end position.
    #[inline]
    #[track_caller]
    #[must_use]
    pub fn new<T, U>(start: T, end: U) -> Self
    where
        T: Into<Position>,
        U: Into<Position>,
    {
        let start = start.into();
        let end = end.into();

        assert!(start <= end, "a span cannot start after its end");

        Self { start, end }
    }

    /// Gets the starting position of the span.
    #[inline]
    #[must_use]
    pub const fn start(self) -> Position {
        self.start
    }

    /// Gets the final position of the span.
    #[inline]
    #[must_use]
    pub const fn end(self) -> Position {
        self.end
    }

    /// Checks if this span inclusively contains another span or position.
    pub fn contains<S>(self, other: S) -> bool
    where
        S: Into<Self>,
    {
        let other = other.into();
        self.start <= other.start && self.end >= other.end
    }
}

impl From<Position> for Span {
    fn from(pos: Position) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }
}

impl Spanned for Span {
    #[inline]
    fn span(&self) -> Span {
        *self
    }
}

impl<T: Spanned> Spanned for &T {
    #[inline]
    fn span(&self) -> Span {
        T::span(*self)
    }
}

impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.end < other.start {
            Some(Ordering::Less)
        } else if self.start > other.end {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}..{}]", self.start, self.end)
    }
}

/// An element of the AST or any type that can be located in the source with a Span.
pub trait Spanned {
    /// Returns a span from the current type.
    #[must_use]
    fn span(&self) -> Span;
}

/// A linear span in the ECMAScript source code.
///
/// Stores a linear start position and a linear end position.
///
/// Note that linear spans are of the form [start, end) i.e. that the
/// start position is inclusive and the end position is exclusive.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LinearSpan {
    start: LinearPosition,
    end: LinearPosition,
}
impl LinearSpan {
    /// Creates a new `LinearPosition`.
    ///
    /// # Panics
    ///
    /// Panics if the start position is bigger than the end position.
    #[inline]
    #[track_caller]
    #[must_use]
    pub const fn new(start: LinearPosition, end: LinearPosition) -> Self {
        assert!(
            start.pos <= end.pos,
            "a linear span cannot start after its end"
        );

        Self { start, end }
    }

    /// Test if the span is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// Gets the starting position of the span.
    #[inline]
    #[must_use]
    pub const fn start(self) -> LinearPosition {
        self.start
    }

    /// Gets the final position of the span.
    #[inline]
    #[must_use]
    pub const fn end(self) -> LinearPosition {
        self.end
    }

    /// Checks if this span inclusively contains another span or position.
    pub fn contains<S>(self, other: S) -> bool
    where
        S: Into<Self>,
    {
        let other = other.into();
        self.start <= other.start && self.end >= other.end
    }

    /// Gets the starting position of the span.
    #[inline]
    #[must_use]
    pub fn union(self, other: impl Into<Self>) -> Self {
        let other: Self = other.into();
        Self {
            start: LinearPosition::new(self.start.pos.min(other.start.pos)),
            end: LinearPosition::new(self.end.pos.max(other.end.pos)),
        }
    }
}
#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for LinearSpan {
    fn arbitrary(_: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let zero_pos = LinearPosition::new(0);
        Ok(Self::new(zero_pos, zero_pos))
    }
}

impl From<LinearPosition> for LinearSpan {
    fn from(pos: LinearPosition) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }
}

impl PartialOrd for LinearSpan {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.end < other.start {
            Some(Ordering::Less)
        } else if self.start > other.end {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}

/// Stores a `LinearSpan` but `PartialEq`, `Eq` always return true.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct LinearSpanIgnoreEq(pub LinearSpan);
impl PartialEq for LinearSpanIgnoreEq {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl From<LinearSpan> for LinearSpanIgnoreEq {
    fn from(value: LinearSpan) -> Self {
        Self(value)
    }
}
#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for LinearSpanIgnoreEq {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(LinearSpan::arbitrary(u)?))
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A position group of `LinearPosition` and `Position` related to the same position in the ECMAScript source code.
pub struct PositionGroup {
    pos: Position,
    linear_pos: LinearPosition,
}
impl PositionGroup {
    /// Creates a new `PositionGroup`.
    #[inline]
    #[must_use]
    pub const fn new(pos: Position, linear_pos: LinearPosition) -> Self {
        Self { pos, linear_pos }
    }
    /// Get the `Position`.
    #[inline]
    #[must_use]
    pub fn position(&self) -> Position {
        self.pos
    }
    /// Get the `LinearPosition`.
    #[inline]
    #[must_use]
    pub fn linear_position(&self) -> LinearPosition {
        self.linear_pos
    }

    /// Gets the line number of the position.
    #[inline]
    #[must_use]
    pub const fn line_number(&self) -> u32 {
        self.pos.line_number()
    }

    /// Gets the column number of the position.
    #[inline]
    #[must_use]
    pub const fn column_number(&self) -> u32 {
        self.pos.column_number()
    }
}

#[cfg(test)]
mod tests;

// TODO: union Span & LinearSpan into `SpanBase<T>` and then:
//       * Span = SpanBase<Position>;
//       * LinearSpan = SpanBase<LinearPosition>;
//       ?
