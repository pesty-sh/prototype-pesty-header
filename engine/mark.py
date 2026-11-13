"""Redraw the letter files. Run this after you edit a glyph.

Process:
1. The hand-drawn letters `p e s t y . h` live in `GLYPHS` below.
   Every other printable ASCII character lives in `alphabet.py`.
   Capitals use the lowercase glyph.
2. Each glyph is 19 rows. Bowls sit on the x-height, t and h ascend,
   p and y descend. `#` is ink. `.` is empty.
3. This script adds one blank row above and below, and two empty columns
   between letters.
4. It writes `engine/src/mark.txt` from the word `pesty.sh` only, and
   `src/glyphs.json` from the whole set.

Config:
- Edit a string in `GLYPHS` or in `alphabet.py`, then run `python3 engine/mark.py`.
- Do not edit `mark.txt` or `glyphs.json` by hand. This script overwrites them.
- `logo.rs` checks 80×21 and 465 ink cells. Update that test if the count moves.
"""

from pathlib import Path

from alphabet import EXTRA

GLYPHS = {
    "p": [
        ".........",
        ".........",
        ".........",
        ".........",
        ".........",
        "..#####..",
        ".#######.",
        "###...###",
        "###...###",
        "###...###",
        "###...###",
        "###...###",
        "###...###",
        "########.",
        "#######..",
        "###......",
        "###......",
        "###......",
        ".##......",
    ],
    "e": [
        ".........",
        ".........",
        ".........",
        ".........",
        ".........",
        "..#####..",
        ".#######.",
        "###...###",
        "###......",
        "###......",
        "########.",
        "#########",
        "###......",
        "###...###",
        ".#######.",
        "..#####..",
        ".........",
        ".........",
        ".........",
    ],
    "s": [
        ".........",
        ".........",
        ".........",
        ".........",
        ".........",
        "..#####..",
        ".#######.",
        "###...###",
        "###......",
        ".#######.",
        "..######.",
        "......###",
        "......###",
        "###...###",
        ".#######.",
        "..#####..",
        ".........",
        ".........",
        ".........",
    ],
    "t": [
        "...###...",
        "...###...",
        "...###...",
        "...###...",
        "...###...",
        "#########",
        ".########",
        "...###...",
        "...###...",
        "...###...",
        "...###...",
        "...###...",
        "...###...",
        "...###...",
        "...###...",
        "..#####..",
        ".........",
        ".........",
        ".........",
    ],
    "y": [
        ".........",
        ".........",
        ".........",
        ".........",
        ".........",
        "###...###",
        "###...###",
        "###...###",
        "###...###",
        ".###.###.",
        "..#####..",
        "...###...",
        "...###...",
        "...###...",
        "..###....",
        ".###.....",
        "###......",
        "###......",
        ".##......",
    ],
    ".": [
        "...",
        "...",
        "...",
        "...",
        "...",
        "...",
        "...",
        "...",
        "...",
        "...",
        "...",
        "...",
        "...",
        "###",
        "###",
        "###",
        "...",
        "...",
        "...",
    ],
    "h": [
        "###......",
        "###......",
        "###......",
        "###......",
        "###......",
        "###..####",
        "#########",
        "###...###",
        "###...###",
        "###...###",
        "###...###",
        "###...###",
        "###...###",
        "###...###",
        "###...###",
        "###...###",
        ".........",
        ".........",
        ".........",
    ],
}

ORDER = ["p", "e", "s", "t", "y", ".", "s", "h"]
GAP = 2

_owned = set(GLYPHS) & set(EXTRA)
if _owned:
    raise SystemExit(f"alphabet.py must not replace the hand glyphs: {sorted(_owned)}")
GLYPHS.update(EXTRA)


def _ascii_covered() -> None:
    missing = []
    for code in range(32, 127):
        char = chr(code).lower()
        if char not in GLYPHS:
            missing.append(chr(code))
    if missing:
        raise SystemExit("printable ASCII still missing: " + "".join(missing))


_ascii_covered()


def rows() -> list[str]:
    composed: list[str] | None = None
    for name in ORDER:
        glyph = GLYPHS[name]
        if composed is None:
            composed = list(glyph)
        else:
            composed = [a + "." * GAP + b for a, b in zip(composed, glyph)]
    assert composed is not None
    blank = "." * len(composed[0])
    return [blank, *composed, blank]


def main() -> None:
    import json

    lines = rows()
    path = Path(__file__).resolve().parent / "src" / "mark.txt"
    path.write_text("\n".join(lines) + "\n")
    ink = sum(line.count("#") for line in lines)
    print(f"wrote {path} {len(lines[0])}x{len(lines)} ink={ink}")
    glyphs = Path(__file__).resolve().parents[1] / "src" / "glyphs.json"
    glyphs.write_text(
        json.dumps({"gap": GAP, "glyphs": GLYPHS}, indent=2) + "\n"
    )
    print(f"wrote {glyphs}")


if __name__ == "__main__":
    main()
