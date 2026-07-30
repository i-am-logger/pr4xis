#!/usr/bin/env python3
"""Detect colliding and out-of-bounds text in a built PDF.

The `Caregiver PDF page limit` gate counts PAGES, and overlapping text costs no
extra page — so it stayed green while the last six rows of a submission table
were being drawn on top of each other, illegibly, over the page number. Worse,
the collision made the document APPEAR to meet ACL's 10-page appendix cap by
hiding content that would otherwise have pushed it over.

This checks the geometry instead of the page count:

  COLLISION   two words whose boxes overlap by more than `--overlap` of the
              smaller box's area. Normal typesetting never does this; colliding
              glyph runs are exactly what an overflowing non-breakable block
              produces.
  OUT-OF-PAGE any word extending beyond the page box, i.e. content that is
              simply not on the paper.

Reads `pdftotext -bbox-layout` XML, so it needs poppler-utils, not a PDF library.

  scripts/pdf-overlap-check.py FILE.pdf [FILE.pdf ...]

Exits non-zero if any document has a defect, naming page and text.
"""

import sys
import subprocess
import xml.etree.ElementTree as ET
from collections import defaultdict

NS = {"x": "http://www.w3.org/1999/xhtml"}
# Fraction of the smaller box's area that must be covered before two words count
# as colliding. Kerned/italic glyph runs can touch by a hair; a real collision
# buries one run inside another.
DEFAULT_OVERLAP = 0.55
# Words shorter than this are skipped: single glyphs legitimately overlap in
# ligatures, accents and maths.
MIN_LEN = 2


def words_per_page(pdf):
    """[(page_no, [(text, x0, y0, x1, y1), ...]), ...] via pdftotext -bbox."""
    xml = subprocess.run(
        ["pdftotext", "-bbox", pdf, "-"],
        capture_output=True, text=True, check=True,
    ).stdout
    root = ET.fromstring(xml)
    out = []
    for i, page in enumerate(root.iter(f"{{{NS['x']}}}page"), start=1):
        w = float(page.get("width"))
        h = float(page.get("height"))
        ws = []
        for word in page.iter(f"{{{NS['x']}}}word"):
            t = (word.text or "").strip()
            if not t:
                continue
            ws.append((
                t,
                float(word.get("xMin")), float(word.get("yMin")),
                float(word.get("xMax")), float(word.get("yMax")),
            ))
        out.append((i, w, h, ws))
    return out


def area(b):
    return max(0.0, b[3] - b[1]) * max(0.0, b[4] - b[2])


def intersect_area(a, b):
    dx = min(a[3], b[3]) - max(a[1], b[1])
    dy = min(a[4], b[4]) - max(a[2], b[2])
    return dx * dy if dx > 0 and dy > 0 else 0.0


def check(pdf, threshold):
    problems = []
    for page, pw, ph, ws in words_per_page(pdf):
        for w in ws:
            if w[1] < -1 or w[2] < -1 or w[3] > pw + 1 or w[4] > ph + 1:
                problems.append(f"  p{page} OUT-OF-PAGE  {w[0]!r}")

        # Bucket by vertical band so this stays linear-ish rather than O(n^2)
        # over the whole page. Bands are 6pt; a colliding pair always shares one.
        bands = defaultdict(list)
        for w in ws:
            if len(w[0]) < MIN_LEN:
                continue
            for band in range(int(w[2] // 6), int(w[4] // 6) + 1):
                bands[band].append(w)

        seen = set()
        for band_words in bands.values():
            for i in range(len(band_words)):
                for j in range(i + 1, len(band_words)):
                    a, b = band_words[i], band_words[j]
                    inter = intersect_area(a, b)
                    if inter <= 0:
                        continue
                    smaller = min(area(a), area(b))
                    if smaller <= 0 or inter / smaller < threshold:
                        continue
                    key = (page, a[0], b[0], round(a[1], 1), round(a[2], 1))
                    if key in seen:
                        continue
                    seen.add(key)
                    problems.append(
                        f"  p{page} COLLISION    {a[0]!r} over {b[0]!r} "
                        f"({inter / smaller:.0%} of the smaller box)"
                    )
    return problems


def main(argv):
    threshold = DEFAULT_OVERLAP
    pdfs = []
    it = iter(argv)
    for arg in it:
        if arg == "--overlap":
            threshold = float(next(it))
        else:
            pdfs.append(arg)
    if not pdfs:
        print(__doc__)
        return 2

    bad = 0
    for pdf in pdfs:
        problems = check(pdf, threshold)
        name = pdf.rsplit("/", 1)[-1]
        if problems:
            bad = 1
            print(f"{name}: {len(problems)} problem(s)")
            for p in problems[:25]:
                print(p)
            if len(problems) > 25:
                print(f"  … and {len(problems) - 25} more")
        else:
            print(f"{name}: clean")
    return bad


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
