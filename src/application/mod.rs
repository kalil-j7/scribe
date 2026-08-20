//! Application services: wire the CLI to the domain and the store.

pub mod commands;
pub mod data;
pub mod paths;

pub use commands::{
    run_books, run_chapter, run_compare, run_occurrences, run_passage, run_search, run_word,
};
pub use data::{run_data, run_doctor, run_setup};
