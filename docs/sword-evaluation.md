# SWORD / `rsword_chirho` evaluation (spike results)

Date: 2026-08. Crate: `rsword_chirho` 0.3.0 (pure-Rust port of the SWORD
library). Modules tested: official CrossWire **KJVA** (KJV 1769 + Apocrypha,
zText, v3.1) and **LXX** (Rahlfs Septuagint, zText, v3.2), downloaded from
`https://www.crosswire.org/ftpmirror/pub/sword/packages/rawzip/`.

The spike project is kept at `tools/rsword-spike/` for reproducibility
(`cargo run -- spike/sword`, after placing the two modules under
`spike/sword/modules/...` and `spike/sword/mods.d/...`).

## Procedure

The spike loaded each module with `SwMgrChirho` + `load_module_chirho` and
read probe references through `read_entry_filtered_chirho` with
`OutputFormatChirho::PlainChirho`.

## Results

### KJVA module

The module conf declares `Versification=KJVA`; `rsword_chirho` does not
register a KJVA versification (available: KJV, Catholic, LXX, Synodal, Luther,
Vulgate, NRSV, Leningrad, Ethiopian) and silently falls back to KJV.

```
[KJVA] v11n in conf: "KJVA" (registered: false)
[KJVA] Sirach 2:1: Err(InvalidVerseReferenceChirho { reference_chirho: "Sirach 2:1" })
[KJVA] Tobit 1:1: Err(InvalidVerseReferenceChirho { reference_chirho: "Tobit 1:1" })
[KJVA] Genesis 1:1: Ok("In the beginning God created the heaven and the earth.")
[KJVA] 1 Maccabees 3:1: Err(InvalidVerseReferenceChirho { reference_chirho: "1 Maccabees 3:1" })
```

Every Apocrypha reference fails; only KJV-canon books resolve. **Blocker.**

### LXX module

The module loads with the LXX versification, but its *internal* versification
differs from `rsword_chirho`'s built-in LXX canon (the module's own conf
warns: "The versification differs slightly from what is defined in
canon_lxx.h in the Sword sources"). Lookups therefore return **wrong verses**:

```
[LXX] Sirach 2:1:     Ok("καὶ γὰρ εἰ μηδὲν αὐτοὺς ταραχῶδες ἐφόβει …")   // = Wisdom 17 text
[LXX] Tobit 1:1:      Ok("σὺ δὲ Αχιωρ μισθωτὲ τοῦ Αμμων …")              // = Judith 14:5
[LXX] Genesis 1:1:    Ok("ἐν ἀρχῇ ἐποίησεν ὁ θεὸς τὸν οὐρανὸν καὶ τὴν γῆν")  // correct (no drift yet)
[LXX] 1 Maccabees 3:1: Ok("ὅτε δὲ συνετέλεσαν δειπνοῦντες εἰσήγαγον Τωβιαν …") // = Tobit 8:1
```

Book offsets drift as verse counts diverge from the module's actual
versification. **Blocker.**

## Decision

Per the milestone rules ("If `rsword_chirho` fails specifically for KJVA or
otherwise creates a real blocker: do not burn the whole task repairing
somebody else's library"), Scribe does **not** use `rsword_chirho` in the
product and does not re-implement SWORD storage. Instead:

* English KJV Apocrypha comes from the same CrossWire **OSIS source**
  (`kjva.osis.xml`, public-domain KJV 1769 text) that builds the KJVA module,
  extracted by `tools/extract_kjva_osis.py` into `data/kjva.tsv`.
* Greek comes from the same CCAT **LXXMorph** corpus (Rahlfs) that builds the
  CrossWire LXX module, downloaded on demand as plain text.

Both feed the same native store behind the `ScriptureSource` trait. The
adapter seam is `src/infrastructure/sword/mod.rs`: a working SWORD importer
(any library) can be plugged in there later without touching the CLI,
application, or domain layers.
