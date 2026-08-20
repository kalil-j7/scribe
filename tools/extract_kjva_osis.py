#!/usr/bin/env python3
"""One-time provenance tool: extract KJV Apocrypha (1769) plain text from the
CrossWire Bible Society's `kjva.osis.xml` OSIS source.

Source   : https://gitlab.com/crosswire-bible-society/kjv (file kjva.osis.xml)
           (the OSIS source used to build the SWORD "KJVA" module)
Text     : King James Version (Authorized Version), 1769 edition, Apocrypha.
License  : The KJV 1769 text is public domain in the USA. CrossWire states in
           kjva.conf: "CrossWire Bible Society hereby grants a general public
           license to use this text for any purpose." (module packaging is
           distributed under the GPL; the base text is public domain).
Output   : data/kjva.tsv  (one verse per line:  BOOK<TAB>CHAPTER<TAB>VERSE<TAB>TEXT)

Usage    : python3 tools/extract_kjva_osis.py <path-to-kjva.osis.xml> data/kjva.tsv
"""

import sys
import xml.etree.ElementTree as ET

# OSIS book id -> our canonical book display name
BOOKS = {
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
}

NS = "{http://www.bibletechnologies.net/2003/OSIS/namespace}"


def local(tag):
    return tag.rsplit("}", 1)[-1]


def inner_text(el, drop=("note", "title")):
    """All text content of `el`, excluding subtrees whose local tag is in `drop`."""
    parts = [el.text or ""]
    for child in el.iter():
        if child is el:
            continue
        if local(child.tag) in drop:
            continue
        parts.append(child.text or "")
        parts.append(child.tail or "")
    return "".join(parts)


def extract_verses(root):
    """Yield (osis_book, chapter_no, verse_no, text) for apocrypha verses."""
    verses = []
    chapter_tag = f"{NS}chapter"
    for chapter in root.iter(chapter_tag):
        osis_id = chapter.get("osisID") or ""
        book_id, _, chap_str = osis_id.partition(".")
        if book_id not in BOOKS:
            continue
        chapter_no = int(chap_str)
        current = None  # [book_id, chapter_no, verse_no, parts]
        for el in chapter.iter():
            tag = local(el.tag)
            if tag == "verse":
                sid = el.get("sID")
                eid = el.get("eID")
                if sid:
                    _, _, verse_str = sid.rpartition(".")
                    try:
                        verse_no = int(verse_str)
                    except ValueError:
                        verse_no = None
                    current = [book_id, chapter_no, verse_no, [el.tail or ""]]
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
                continue
            if current is None:
                continue
            if tag in ("note", "title"):
                continue
            current[3].append(el.text or "")
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
        for book, ch, v, text in out:
            if v is None or text == "":
                continue
            f.write(f"{book}\t{ch}\t{v}\t{text}\n")
    print(f"wrote {len(out)} verses -> {dst}")


if __name__ == "__main__":
    main()
