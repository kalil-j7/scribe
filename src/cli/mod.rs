//! Command-line surface for Scribe.

use std::ffi::OsString;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "scribe",
    version,
    about = "Scripture textual-study workbench: KJV Apocrypha + Greek Septuagint",
    long_about = "Scribe gets you from passage -> original-language evidence -> related \
occurrences with as little friction as possible.\n\n\
USAGE\n\
  scribe sirach 2:1\n\
  scribe sirach 2\n\
  scribe \"wisdom 2:12-20\"\n\
  scribe 1 maccabees 3:1\n\
  scribe search wisdom --book sirach\n\
  scribe compare sirach 2:1\n\
  scribe word πειρασμός\n\
  scribe word πειρασμόν\n\
  scribe occurrences πειρασμός\n\
  scribe occurrences πειρασμός --book sirach\n\
  scribe sirach 2:1 --words\n\
  scribe books\n\
  scribe doctor\n\
  scribe setup\n\
  scribe data install lxx",
    allow_external_subcommands = true,
    subcommand_negates_reqs = true
)]
pub struct Cli {
    /// Emit JSON (machine-readable output) instead of plain text.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Free-form reference: `scribe sirach 2:1`, `scribe 1 maccabees 3:1`.
    #[command(external_subcommand)]
    ExternalSubcommand(Vec<OsString>),

    /// Show a passage (book, chapter, verse or verse range).
    Passage(PassageArgs),

    /// Show a whole chapter.
    Chapter(ChapterArgs),

    /// Show the English passage together with the Greek witness.
    Compare(PassageArgs),

    /// Greek lemma study: resolve a word (dictionary or inflected form) to
    /// its lemma and report forms, morphology, and occurrences.
    Word(WordArgs),

    /// List every token occurrence of a Greek lemma in the installed corpus.
    Occurrences(OccurrencesArgs),

    /// Full-text search over the installed witnesses.
    Search(SearchArgs),

    /// List the books of the Apocrypha canon understood by Scribe.
    Books {
        /// Emit JSON.
        #[arg(long, global = true)]
        json: bool,
    },

    /// Report installation status of data and indexes.
    Doctor {
        /// Emit JSON.
        #[arg(long, global = true)]
        json: bool,
    },

    /// Import the bundled KJV Apocrypha data into the data directory.
    Setup {
        /// Re-import even if already installed.
        #[arg(long)]
        force: bool,
    },

    /// Manage data sets (install / uninstall / status).
    Data(DataArgs),
}

#[derive(Args, Debug)]
pub struct PassageArgs {
    /// Reference parts, e.g. `sirach 2:1` or `1 maccabees 3:1` or `wisdom 2:12-20`.
    #[arg(required = true, num_args = 1..)]
    pub parts: Vec<String>,

    /// Show the Greek witness for the passage.
    #[arg(long)]
    pub greek: bool,

    /// Show the token-level Greek word view (Surface / Lemma / Morphology).
    #[arg(long)]
    pub words: bool,

    /// Emit JSON.
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct WordArgs {
    /// The Greek word to study: a dictionary form (`πειρασμός`) or an
    /// inflected surface form (`πειρασμόν`), accent/case-insensitive.
    #[arg(required = true)]
    pub word: String,

    /// Emit JSON.
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct OccurrencesArgs {
    /// The Greek word whose lemma occurrences to list.
    #[arg(required = true)]
    pub word: String,

    /// Restrict occurrences to one book (aliases supported, e.g. `sirach`,
    /// `ecclesiasticus`, `ecclus`).
    #[arg(long)]
    pub book: Option<String>,

    /// Emit JSON.
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct ChapterArgs {
    /// Reference parts, e.g. `sirach 2`.
    #[arg(required = true, num_args = 1..)]
    pub parts: Vec<String>,

    /// Emit JSON.
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// The phrase to search for, e.g. `fear of the lord`.
    #[arg(required = true)]
    pub query: String,

    /// Restrict the search to one book.
    #[arg(long)]
    pub book: Option<String>,

    /// Search the Greek (Septuagint) witness.
    #[arg(long)]
    pub greek: bool,

    /// Maximum number of hits to print.
    #[arg(long, default_value_t = 20)]
    pub limit: usize,

    /// Emit JSON.
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct DataArgs {
    #[command(subcommand)]
    pub action: DataAction,
}

#[derive(Subcommand, Debug)]
pub enum DataAction {
    /// Install a dataset. `kjva` is bundled; `lxx` is downloaded on demand.
    Install {
        /// Which dataset: kjva | lxx | all.
        #[arg(value_name = "DATASET")]
        dataset: String,
    },
    /// Remove an installed dataset.
    Uninstall {
        /// Which dataset: kjva | lxx.
        #[arg(value_name = "DATASET")]
        dataset: String,
    },
    /// Report which datasets are installed.
    Status {
        #[arg(long, global = true)]
        json: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn word_is_an_explicit_subcommand_not_a_passage() {
        let cli = Cli::try_parse_from(["scribe", "word", "πειρασμός"]).unwrap();
        match cli.command {
            Command::Word(args) => assert_eq!(args.word, "πειρασμός"),
            other => panic!("expected Command::Word, got {other:?}"),
        }
    }

    #[test]
    fn occurrences_is_an_explicit_subcommand_not_a_passage() {
        let cli = Cli::try_parse_from(["scribe", "occurrences", "πειρασμός", "--book", "sirach"])
            .unwrap();
        match cli.command {
            Command::Occurrences(args) => {
                assert_eq!(args.word, "πειρασμός");
                assert_eq!(args.book.as_deref(), Some("sirach"));
            }
            other => panic!("expected Command::Occurrences, got {other:?}"),
        }
    }

    #[test]
    fn passage_shorthand_still_goes_to_external_subcommand() {
        let cli = Cli::try_parse_from(["scribe", "sirach", "2:1"]).unwrap();
        match cli.command {
            Command::ExternalSubcommand(parts) => {
                let parts: Vec<String> = parts
                    .iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect();
                assert_eq!(parts, vec!["sirach", "2:1"]);
            }
            other => panic!("expected Command::ExternalSubcommand, got {other:?}"),
        }
    }

    #[test]
    fn search_still_dispatches_to_search() {
        let cli = Cli::try_parse_from(["scribe", "search", "wisdom", "--book", "sirach"]).unwrap();
        match cli.command {
            Command::Search(args) => assert_eq!(args.query, "wisdom"),
            other => panic!("expected Command::Search, got {other:?}"),
        }
    }

    #[test]
    fn words_flag_on_passage_subcommand() {
        let cli = Cli::try_parse_from(["scribe", "passage", "sirach", "2:1", "--words"]).unwrap();
        match cli.command {
            Command::Passage(args) => assert!(args.words),
            other => panic!("expected Command::Passage, got {other:?}"),
        }
    }

    #[test]
    fn passage_shorthand_words_flag_collected_externally() {
        // `--words` after the shorthand reference is collected by the
        // external-subcommand path in main (tested at the CLI integration
        // level); here we only ensure the parser keeps it in the external
        // argument list rather than rejecting it.
        let cli = Cli::try_parse_from(["scribe", "sirach", "2:1", "--words"]).unwrap();
        match cli.command {
            Command::ExternalSubcommand(parts) => {
                let joined: Vec<String> = parts
                    .iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect();
                assert!(joined.iter().any(|p| p == "--words"));
            }
            other => panic!("expected Command::ExternalSubcommand, got {other:?}"),
        }
    }
}
