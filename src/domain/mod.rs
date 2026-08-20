//! Scribe's own Scripture domain model.
//!
//! The CLI and application layers depend on these types and on the
//! [`source::ScriptureSource`] trait — never on a concrete data backend.
//! A data source (SWORD adapter, native store, future witnesses) is
//! interchangeable behind that trait.

pub mod book;
pub mod passage;
pub mod reference;
pub mod search;
pub mod source;
pub mod witness;
pub mod word;
