//! Node interface implementation for DOM Level 4
//!
//! The Node interface is the primary datatype for the entire Document Object Model.
//! It represents a single node in the document tree.
//! https://dom.spec.whatwg.org/#interface-node

mod node;

#[cfg(test)]
mod tests;

pub use node::*;