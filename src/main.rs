//! Scribe — Scripture textual-study workbench.
//!
//! First usable milestone: KJV Apocrypha + Greek Septuagint (Apocrypha)
//! passage/chapter lookup, corpus search, and an English/Greek compare view.

mod application;
mod cli;
mod domain;
mod error;
mod greek;
mod infrastructure;
mod output;
mod text;

use std::process::ExitCode;

use clap::Parser;
use cli::{Cli, Command};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::ExternalSubcommand(ref parts) => run_external(cli.json, parts),
        Command::Passage(args) => {
            application::run_passage(&join_parts(&args.parts), args.greek, args.words, args.json)
        }
        Command::Chapter(args) => application::run_chapter(&join_parts(&args.parts), args.json),
        Command::Compare(args) => application::run_compare(&join_parts(&args.parts), args.json),
        Command::Word(args) => application::run_word(&args.word, args.json),
        Command::Occurrences(args) => {
            application::run_occurrences(&args.word, args.book.as_deref(), args.json)
        }
        Command::Search(args) => application::run_search(
            &args.query,
            args.book.as_deref(),
            args.greek,
            args.limit,
            args.json,
        ),
        Command::Books { json } => application::run_books(json),
        Command::Doctor { json } => application::run_doctor(json),
        Command::Setup { force } => application::run_setup(force),
        Command::Data(args) => application::run_data(args.action),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// `scribe sirach 2:1` — a free-form reference given as trailing arguments.
fn run_external(global_json: bool, parts: &[std::ffi::OsString]) -> Result<(), error::ScribeError> {
    let mut greek = false;
    let mut words = false;
    let mut json = global_json;
    let mut ref_parts: Vec<String> = Vec::new();
    for p in parts {
        if let Some(s) = p.to_str() {
            match s {
                "--json" => {
                    json = true;
                    continue;
                }
                "--greek" => {
                    greek = true;
                    continue;
                }
                "--words" => {
                    words = true;
                    continue;
                }
                _ => {}
            }
        }
        ref_parts.push(
            p.to_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| error::ScribeError::Other("non-UTF-8 argument".into()))?,
        );
    }
    application::run_passage(&join_parts(&ref_parts), greek, words, json)
}

fn join_parts(parts: &[String]) -> String {
    parts.join(" ")
}
