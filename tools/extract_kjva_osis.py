#!/usr/bin/env python3
"""One-time provenance tool: extract complete KJV + Apocrypha plain text from the
CrossWire Bible Society's `kjva.osis.xml` OSIS source.

Source   : https://gitlab.com/crosswire-bible-society/kjv (file kjva.osis.xml)
           (the OSIS source used to build the SWORD "KJVA" module)
Text     : King James Version (Authorized Version), 1769 edition, Apocrypha.
License  : The KJV 1769 text is public domain in the USA. CrossWire states in
           kjva.conf: "CrossWire Bible Society hereby grants a general public
           license to use this text for any purpose." (module packaging is
           distributed under the GPL; the base text is public domain).
Output   : data/kjva.tsv  (one verse per line: BOOK<TAB>CHAPTER<TAB>VERSE<TAB>TEXT)

Usage    : python3 tools/extract_kjva_osis.py <path-to-kjva.osis.xml> data/kjva.tsv
"""

import sys
import xml.etree.ElementTree as ET

# OSIS book id -> Scribe's stable canonical display name, in library order.
BOOKS = {
    "Gen": "Genesis", "Exod": "Exodus", "Lev": "Leviticus", "Num": "Numbers",
    "Deut": "Deuteronomy", "Josh": "Joshua", "Judg": "Judges", "Ruth": "Ruth",
    "1Sam": "1 Samuel", "2Sam": "2 Samuel", "1Kgs": "1 Kings", "2Kgs": "2 Kings",
    "1Chr": "1 Chronicles", "2Chr": "2 Chronicles", "Ezra": "Ezra", "Neh": "Nehemiah",
    "Esth": "Esther", "Job": "Job", "Ps": "Psalms", "Prov": "Proverbs",
    "Eccl": "Ecclesiastes", "Song": "Song of Solomon", "Isa": "Isaiah", "Jer": "Jeremiah",
    "Lam": "Lamentations", "Ezek": "Ezekiel", "Dan": "Daniel", "Hos": "Hosea",
    "Joel": "Joel", "Amos": "Amos", "Obad": "Obadiah", "Jonah": "Jonah", "Mic": "Micah",
    "Nah": "Nahum", "Hab": "Habakkuk", "Zeph": "Zephaniah", "Hag": "Haggai",
    "Zech": "Zechariah", "Mal": "Malachi",
    "1Esd": "1 Esdras",
    "2Esd": "2 Esdras",
    "Tob": "Tobit",
    "Jdt": "Judith",
    "AddEsth": "Rest of Esther",
    "Wis": "Wisdom of Solomon",
    "Sir": "Sirach",
    "Bar": "Baruch",
    "PrAzar": "Prayer of Azariah",
    "Sus": "Susanna",
    "Bel": "Bel and the Dragon",
    "PrMan": "Prayer of Manasses",
    "1Macc": "1 Maccabees",
    "2Macc": "2 Maccabees",
    "Matt": "Matthew", "Mark": "Mark", "Luke": "Luke", "John": "John", "Acts": "Acts",
    "Rom": "Romans", "1Cor": "1 Corinthians", "2Cor": "2 Corinthians", "Gal": "Galatians",
    "Eph": "Ephesians", "Phil": "Philippians", "Col": "Colossians",
    "1Thess": "1 Thessalonians", "2Thess": "2 Thessalonians", "1Tim": "1 Timothy",
    "2Tim": "2 Timothy", "Titus": "Titus", "Phlm": "Philemon", "Heb": "Hebrews",
    "Jas": "James", "1Pet": "1 Peter", "2Pet": "2 Peter", "1John": "1 John",
    "2John": "2 John", "3John": "3 John", "Jude": "Jude", "Rev": "Revelation",
}

NS = "{http://www.bibletechnologies.net/2003/OSIS/namespace}"


def local(tag):
    return tag.rsplit("}", 1)[-1]


def inner_text(el, drop=("note", "title")):
    """All text content of `el`, excluding subtrees whose local tag is in `drop`."""
    parts = []
    def visit(node):
        if local(node.tag) in drop:
            return
        parts.append(node.text or "")
        for child in node:
            visit(child)
            parts.append(child.tail or "")
    visit(el)
    return "".join(parts)


def extract_verses(root):
    """Yield all mapped KJVA OSIS verses as source book/chapter/verse/text."""
    verses = []
    chapter_tag = f"{NS}chapter"
    for chapter in root.iter(chapter_tag):
        osis_id = chapter.get("osisID") or ""
        book_id, _, chap_str = osis_id.partition(".")
        if book_id not in BOOKS:
            continue
        chapter_no = int(chap_str)
        current = None  # [book_id, chapter_no, verse_no, parts]
        def walk(el):
            """Traverse a chapter child in document order around verse milestones."""
            nonlocal current
            tag = local(el.tag)
            if tag in ("note", "title"):
                return
            if tag == "verse":
                sid = el.get("sID")
                eid = el.get("eID")
                if sid:
                    _, _, verse_str = sid.rpartition(".")
                    try:
                        verse_no = int(verse_str)
                    except ValueError:
                        verse_no = None
                    # The parent's traversal adds `el.tail` in document order.
                    # Keeping it here as well would duplicate every milestone
                    # verse (the KJVA source uses sID/eID markers).
                    current = [book_id, chapter_no, verse_no, []]
                elif eid:
                    if current is not None:
                        text = " ".join("".join(current[3]).split())
                        verses.append((current[0], current[1], current[2], text))
                        current = None
                elif el.get("osisID"):
                    # container-form verse: <verse osisID="B.C.V">text</verse>
                    _, _, verse_str = el.get("osisID").rpartition(".")
                    try:
                        verse_no = int(verse_str)
                    except ValueError:
                        verse_no = None
                    text = " ".join(inner_text(el).split())
                    verses.append((book_id, chapter_no, verse_no, text))
                return
            if current is not None:
                current[3].append(el.text or "")
            for child in el:
                walk(child)
                if current is not None:
                    current[3].append(child.tail or "")

        # Verse markers can occur inside formatting containers, so recurse
        # instead of assuming they are direct chapter children.
        for el in chapter:
            walk(el)
            if current is not None:
                current[3].append(el.tail or "")
    return verses


def main():
    src, dst = sys.argv[1], sys.argv[2]
    tree = ET.parse(src)
    root = tree.getroot()
    verses = extract_verses(root)

    # Remap: in the CrossWire KJVA OSIS the "Epistle of Jeremy" is Baruch 6
    # (Bar.6.N).  KJV printings treat it as a separate one-chapter book, so we
    # renumber it to its own book with 73 verses.
    out = []
    for book_id, ch, v, text in verses:
        if book_id == "Bar" and ch == 6:
            book = "Epistle of Jeremy"
            ch, v = 1, v
        else:
            book = BOOKS[book_id]
        out.append((book, ch, v, text))

    out.sort(key=lambda r: (list(BOOKS.values()).index(r[0]) if r[0] in BOOKS.values() else 99, r[1], r[2]))

    with open(dst, "w", encoding="utf-8") as f:
        f.write("# book\tchapter\tverse\ttext\n")
        written = 0
        for book, ch, v, text in out:
            # CrossWire emits twelve `…` placeholder verses for the omitted
            # KJV Rest-of-Esther slots. They are not Scripture text and were
            # intentionally absent from the prior Apocrypha dataset.
            if v is None or not text.strip("…").strip():
                continue
            f.write(f"{book}\t{ch}\t{v}\t{text}\n")
            written += 1
    print(f"wrote {written} verses -> {dst}")


if __name__ == "__main__":
    main()
