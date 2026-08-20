# Scribe

A Scripture textual-study workbench CLI. The long-term goal is to get from
**passage → original-language evidence → related occurrences** with as little
friction as possible. This is the first usable milestone: the KJV Apocrypha
and the Greek Septuagint (Apocrypha) with fast local lookup, search, and an
English/Greek compare view.

```text
CLI
 ↓
Application services (lookup, search, data management)
 ↓
Our Scripture domain (BookId, PassageRef, ScriptureText, Token, WitnessId, …)
 ↓
ScriptureSource trait
 ├── native store (JSONL + binary cache + in-memory inverted index)
 ├── future: SWORD adapter (seam at src/infrastructure/sword/)
 └── future: Hebrew witnesses, manuscripts, … (same trait)
```

The CLI, application, and domain layers never see a data-source type; the
store (and any future source) is interchangeable behind the trait.

## What Scribe currently does

- KJV Apocrypha (1769) passage lookup: verse, verse range, whole chapter.
- Whole-book-name aliases (`Sirach` / `Ecclesiasticus` / `Ecclus` / `Sir`,
  `Wisdom` / `Wisdom of Solomon`, `1 Maccabees` / `1 Macc`, …).
- Case-insensitive, whitespace-tolerant book matching with "did you mean"
  suggestions for typos.
- Full-text search over the installed witnesses (all terms must match; hits
  ranked by term frequency), optionally restricted to one book or to Greek.
  Greek search is accent-insensitive (`πειρασμός` matches `πειρασμὸς`).
- Greek Septuagint (Rahlfs) lookup with explicit per-book coverage states;
  a `compare` view is enabled only for verified one-to-one mappings.
- Greek lemma study (`scribe word`): resolve a dictionary form or an
  inflected surface form to its lemma and report part of speech, distinct
  forms, morphology, occurrence counts, and per-book counts. Accent- and
  case-insensitive; ambiguous forms are listed rather than guessed.
- Lemma occurrences (`scribe occurrences`): every token occurrence of a
  lemma in the installed Greek corpus, grouped by book, with `--book`
  filtering (aliases work: `--book sirach` = `--book ecclesiasticus`).
- Token-level passage inspection (`scribe <ref> --words`): the Surface /
  Lemma / Morphology table of the Greek witness for a passage.
- `books`, `doctor`, `setup`, `data install/uninstall/status` commands.
- `--json` output mode for every command (for future TUI/mobile integrations).
- Offline operation: the English dataset is bundled; once `scribe data
  install lxx` has run, Greek is offline too.
- Fast startup: data is imported once into a compact binary cache; warm
  lookups are ~10 ms.

## What Scribe intentionally does NOT do yet

- No Hebrew manuscripts, no manuscript comparison (`scribe witnesses` is
  future work).
- No interlinear view, morphology UI, or automatic English↔Greek alignment.
- No `scribe word`/`scribe occurrences` over OT/NT corpora (only the installed
  Greek Apocrypha corpus; the `Corpus` domain type already distinguishes
  `apocrypha` / `septuagint_ot` / `new_testament` for when data lands).
- No glosses/definitions in word studies (the corpus carries no trustworthy
  gloss data, so the section is omitted rather than invented).
- No OT/NT books, no other canons or translations.
- No notes/highlighting/AI/accounts/cloud/GUI/mobile.
- No real SWORD module reader (see `docs/sword-evaluation.md` for why).

## Build

Requires stable Rust (tested with 1.94).

```bash
cargo build --release
./target/release/scribe --help
```

### Install onto PATH

```bash
./scripts/install.sh
```

This builds the release binary, installs it to `~/.local/bin/scribe`
(`~/.local/bin` is already on PATH on macOS/Linux), and appends a guarded
block to `~/.zshrc` that keeps `~/.local/bin` on PATH and defines
`scribe-update` to rebuild and reinstall after pulling new code. Run
`source ~/.zshrc` (or open a new terminal) once, then `scribe sirach 2:1`
works from anywhere.

Quality gates (all pass in this milestone):

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

## Setup / data

The data directory is `SCRIBE_DATA_DIR` if set, otherwise the platform data
directory (`~/Library/Application Support/scribe` on macOS,
`~/.local/share/scribe` on Linux, `%LOCALAPPDATA%\scribe` on Windows).

The KJV Apocrypha dataset is **bundled with the binary** (public-domain text)
and is imported automatically on first use. You normally never need to run
setup by hand:

```bash
scribe doctor        # what's installed, where
scribe setup         # (re)import the bundled KJV Apocrypha explicitly
scribe data install lxx    # download + import Greek (requires network)
scribe data status   # installed datasets
scribe data uninstall lxx  # remove Greek again
```

## Real commands (verified against the release binary)

```bash
scribe sirach 2:1
scribe sirach 2
scribe "wisdom 2:12-20"
scribe 1 maccabees 3:1
scribe "epistle of jeremy 1:1"
scribe search wisdom --book sirach
scribe search "fear of the lord" --book sirach
scribe search πειρασμός --greek
scribe compare sirach 2:1
scribe compare wisdom 1:1
scribe sirach 2:1 --greek
scribe word πειρασμός
scribe word πειρασμόν
scribe word ΠΕΙΡΑΣΜΟΣ
scribe occurrences πειρασμός
scribe occurrences πειρασμός --book sirach
scribe occurrences πειρασμός --book ecclesiasticus
scribe sirach 2:1 --words
scribe sirach 2:1 --json
scribe books
scribe doctor
```

Example output:

```text
$ scribe sirach 2:1
SIRACH 2:1 — KJV APOCRYPHA

My son, if thou come to serve the Lord, prepare thy soul for temptation.

$ scribe compare sirach 2:1
SIRACH 2:1

KJV APOCRYPHA
My son, if thou come to serve the Lord, prepare thy soul for temptation.

GREEK (LXX — RAHLFS)
τέκνον εἰ προσέρχῃ δουλεύειν κυρίῳ ἑτοίμασον τὴν ψυχήν σου εἰς πειρασμόν
```

Errors are explicit:

```text
$ scribe sirach 90
error: Sirach has no chapter 90 (max 51)

$ scribe wisom 2:1
error: unknown book: "wisom 2:1" (did you mean: Wisdom of Solomon)
```

The Greek word-study loop (numbers are from the installed real corpus):

```text
$ scribe word πειρασμόν
πειρασμός  (peirasmos)

LEMMA
πειρασμός

PART OF SPEECH
noun

OCCURRENCES
7

FORMS
πειρασμῷ         noun dat sg masc
πειρασμὸς        noun nom sg masc
πειρασμόν        noun acc sg masc

BOOKS
Sirach       6
1 Maccabees  1

$ scribe occurrences πειρασμός --book sirach | head -12
πειρασμός — 6 occurrences
book: Sirach

SIRACH

Sirach 2:1
τέκνον εἰ προσέρχῃ δουλεύειν κυρίῳ ἑτοίμασον τὴν ψυχήν σου εἰς πειρασμόν

Sirach 6:7
εἰ κτᾶσαι φίλον ἐν πειρασμῷ κτῆσαι αὐτὸν καὶ μὴ ταχὺ ἐμπιστεύσῃς αὐτῷ

$ scribe sirach 2:1 --words
SIRACH 2:1 — GREEK WORDS

Surface    Lemma         Morphology
τέκνον     τέκνον        N2N-VSN---
εἰ         εἰ            C---------
προσέρχῃ   ἔρχομαι προς  V1--PMS2S-
δουλεύειν  δουλεύω       V1--PAN---
κυρίῳ      κύριος        N2--DSM---
ἑτοίμασον  ἑτοιμάζω      VA--AAD2S-
τὴν        ὁ             RA--ASF---
ψυχήν      ψυχή          N1--ASF---
σου        σύ            RP--GS----
εἰς        εἰς           P---------
πειρασμόν  πειρασμός     N2--ASM---
```

Ambiguous forms are surfaced, never guessed:

```text
$ scribe word ἄλλα
error: "ἄλλα" maps to multiple lemmas:
1. ἀλλά (conjunction, 112 occurrences)
2. ἄλλος (pronoun, 36 occurrences)
```

`--json` variants of `word`, `occurrences`, and `--words` emit structured
data (lemma, forms with morphology, per-book counts, token positions).

### A note on Greek words

- **surface form** — the word as printed in the text (e.g. `πειρασμόν`).
- **lemma** — the dictionary/headword form the surface belongs to
  (e.g. `πειρασμός`), as tagged by the corpus.
- **morphology** — the corpus parse tag (e.g. `N2--ASM---` = noun, acc sg
  masc), decoded into readable descriptions in word studies.
- Normalization (accents, breathings, case, final sigma) is applied for
  matching only; the original surface text is preserved for display.

## Dataset details

| Dataset | Source | Edition | License | Redistribution / commercial use |
|---|---|---|---|---|
| KJV Apocrypha (English) | CrossWire Bible Society KJVA OSIS source (`gitlab.com/crosswire-bible-society/kjv`, file `kjva.osis.xml`), extracted by `tools/extract_kjva_osis.py` | King James Version (Authorized Version), 1769, Apocrypha | Public domain in the USA; CrossWire grants "a general public license to use this text for any purpose" | Allowed |
| Greek Septuagint (Apocrypha) | CCAT LXXMorph corpus (`ccat.sas.upenn.edu`, Rahlfs' *Septuaginta*); Unicode conversion mirrored at `github.com/nathans/lxxmorph-unicode` | Rahlfs 1935 (morphologically tagged) | CCAT fair-use agreement: free non-commercial use; redistribution restricted | Not bundled — downloaded per-user by `scribe data install lxx`; commercial use requires written consent |

Notes:

- "Rest of Esther" is numbered as chapters 10–16 (the CrossWire KJVA OSIS
  leaves chapters 1–9 as empty placeholders, which Scribe omits).
- The "Epistle of Jeremy" is a separate book (73 verses), matching KJV
  printings; the OSIS stores it as Baruch 6, which the extractor renumbers.
  Baruch itself runs chapters 1–5.
- "Prayer of Manasses" is a single verse in the KJV OSIS source; the Greek
  corpus keeps it in Odes 12:1–15, which Scribe combines and maps to its
  single KJV verse.
- The Danielic additions use the selected Theodotion files: Prayer of Azariah
  is Daniel 3:24–91; Susanna and Bel are standalone Theodotion files.
- KJV 2 Esdras is deliberately unavailable in this source: CCAT `2Esdr` is
  Greek Ezra-Nehemiah, not KJV 2 Esdras / 4 Ezra. Rest of Esther is likewise
  withheld because the source uses lettered Esther additions and no safe KJV
  10–16 crosswalk is asserted.
- See [Greek Apocrypha coverage](docs/greek-apocrypha-coverage.md) for the
  complete 15-book source and mapping matrix.
- Library licensing (MIT OR Apache-2.0) is separate from text-data licensing;
  see each dataset's row above.

## Architecture

- `src/domain/` — our own Scripture domain: `BookId` (enum, with alias
  tables), `PassageRef`/`ChapterNumber`/`VerseNumber` (strong reference
  types), `ScriptureText`/`Token` (tokens carry surface + normalized form,
  and lemma/morphology when the source provides them), `WitnessId`/`Language`/
  `TextTradition`, `SearchQuery`/`SearchHit`, and the `ScriptureSource` trait.
- `src/application/` — command services: reference parsing → validation →
  lookup/search, data management (`setup`, `doctor`, `data install`).
- `src/infrastructure/` — native store (JSONL + binary cache + in-memory
  inverted index), importers (bundled TSV; LXXMorph text), and
  `sword/` (the adapter seam; see `docs/sword-evaluation.md`).
- `src/output/` — plain text (no ANSI, pipe-friendly) and JSON rendering.
- `tools/` — `extract_kjva_osis.py` (data provenance) and `rsword-spike`
  (the SWORD evaluation spike).

## Performance (release build, macOS arm64, measured with `/usr/bin/time`)

| Operation | Time |
|---|---|
| binary size | 4.2 MB |
| cold start incl. first import | ~4 s (one-time) |
| warm passage lookup (`sirach 2:1`) | ~35 ms |
| warm `word πειρασμός` / `occurrences πειρασμός` | ~50-70 ms (first run builds the in-memory lemma index) |
| warm `sirach 2:1 --words` | ~35 ms |
| search (`fear of the lord --book sirach`) | ~35 ms |
| compare (loads both witnesses) | ~40 ms |
| cache/index size | 3.4 MB (kjva) + 6.1 MB (lxx) |

Warm queries are effectively instant to a human. The one-time import builds a
binary cache; the lemma index is built in memory on the first word-study
command (few ms for ~83k Greek tokens) and is deliberately not persisted —
for this corpus size, reading it back costs as much as rebuilding it.

## Tests

- Unit: reference parsing (chapters, verses, ranges, aliases, numbered
  books, malformed input), book/alias resolution, text normalization
  (accents, final sigma, punctuation), LXXMorph file parsing, binary cache
  round-trip, CLI dispatch (word/occurrences are explicit subcommands, the
  shorthand passage path is untouched), Greek morphology decoding
  (CCAT/Packard tags), transliteration, lemma-index resolution (surface →
  lemma, ambiguity, book filters, forms/counts).
- Integration (real data pipeline, temp data dir): bundled KJV import →
  store → passage/chapter/search, out-of-range errors, Greek fixture import
  → Sirach 2:1 Greek text + lemma/morphology, and a black-box CLI test that
  runs the actual binary for `sirach 2:1`, `sirach 2`, `search wisdom
  --book sirach`, `doctor`, plus the 0.2 commands: `word πειρασμός` and
  `word πειρασμόν` (both resolve to the lemma), `occurrences πειρασμός`
  (contains the verified Sirach 2:1 hit), `occurrences --book sirach`
  (with `ecclesiasticus`/`ecclus` aliases), `sirach 2:1 --words` (shows
  `πειρασμόν` → `πειρασμός`), structured `--json`, and the
  missing-Greek-data error.

## Future direction (not yet built)

- `scribe witnesses sirach 6:14` — KJV, Greek, Hebrew MSS side by side.
- `scribe word πειρασμός --corpus all` — lemma studies across the Septuagint
  OT and NT corpora once that data exists (the `Corpus` type already
  distinguishes them).
- `scribe interlinear sirach 2:1` — alignment on top of the token model.
- A real SWORD adapter behind `infrastructure/sword/` once a library
  correctly reads the needed modules.

## License

Code: MIT OR Apache-2.0. Scripture text: see the dataset table above.
