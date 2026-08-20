# Greek Apocrypha coverage

Scribe's English grid is the KJV 1769 Apocrypha. Its Greek witness is the
CCAT LXXMorph Unicode corpus, representing Rahlfs' 1935 *Septuaginta*.
The two grids are not treated as interchangeable: a CCAT file is mapped only
when its textual identity and reference conversion are known.

`Full` means Scribe can address the complete KJV work in the selected Greek
witness. `Partial` means the work is imported on matching source references,
but source/KJV verse divisions are known to diverge, so `compare` is withheld
unless a reference has an explicit verified mapping. `Unavailable` and
`Versification conflict` are deliberate, controlled states.

| KJV book | CCAT source / Scribe mapping | State | Note |
|---|---|---|---|
| 1 Esdras | `17.1Esdras.txt` / `1Esdr` | Partial | Same work, but chapter verse divisions differ (for example chapters 1, 2, 5, 6, and 8). |
| 2 Esdras | `18.2Esdras.txt` is **not mapped** | Unavailable | CCAT `2Esdr` is Greek Ezra-Nehemiah (Esdras B), not KJV 2 Esdras / 4 Ezra. |
| Tobit | `21.TobitBA.txt` / BA recension | Partial | Selected BA text; chapter verse divisions differ. The `TobitS` recension is not blended in. |
| Judith | `20.Judith.txt` / `Jdt` | Partial | Source chapter 15 has a different final verse division. |
| Rest of Esther | `19.Esther.txt` is **not mapped** | Versification conflict | The Greek additions use lettered units (for example `Esth 1:1a`, `3:13a`, `10:3a`); no KJV 10–16 crosswalk is asserted. |
| Wisdom of Solomon | `33.Wisdom.txt` / `Wis` | Partial | Source/KJV division differs in chapter 17. |
| Sirach | `34.Sirach.txt` / `Sir` | Partial | The source prologue is outside KJV; several later chapter divisions differ. Sirach 2 is explicitly compare-safe. |
| Baruch | `50.Baruch.txt` / `Bar` | Partial | Source Baruch 3 has one extra verse division. |
| Epistle of Jeremy | `51.EpJer.txt` / `EpJer` → KJV 1:1–73 | Full | Separate Scribe book; CCAT's unnumbered opener is normalized to 1:1. |
| Prayer of Azariah | `57.DanielTh.txt` / `DanTh 3:24–91` → `Prayer of Azariah 1:1–68` | Full | Theodotion is selected; Daniel OG is not merged. |
| Susanna | `59.SusTh.txt` / `SusTh 1–64` | Full | Theodotion is selected; Susanna OG is not merged. |
| Bel and the Dragon | `55.BelTh.txt` / `BelTh 1–42` | Full | Theodotion is selected; Bel OG is not merged. |
| Prayer of Manasses | `28.Odes.txt` / `Od 12:1–15` → KJV 1:1 | Full | KJV OSIS prints one long verse; Scribe combines the 15 Greek Odes units and retains all source markers. |
| 1 Maccabees | `23.1Macc.txt` / `1Mac` | Full | Matching KJV reference grid. |
| 2 Maccabees | `24.2Macc.txt` / `2Mac` | Full | Matching KJV reference grid. |

Every imported Greek row retains its original CCAT marker in the JSONL store
and binary cache (`source_reference`). The importer rejects duplicate target
references and target references outside the KJV grid for full mappings.
For partial source grids, rows that have no KJV-grid address are counted and
skipped during import rather than silently rehomed.
