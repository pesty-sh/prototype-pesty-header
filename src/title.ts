// Turns the title box into cells the engine can draw.
//
// Process:
// 1. Keep at most 24 characters. An empty string draws nothing.
// 2. Face `pesty` stamps glyphs from src/glyphs.json. `engine/mark.py` writes
//    that file. Capitals use the lowercase glyph. A missing character is a small box.
// 3. `plex`, `georgia`, and `impact` are drawn by the browser, then read back
//    as ink or empty. If that draw has no ink, the Pesty face is used instead.
//
// Config:
// - `FACES` is the menu. `id` is the `?face=` value. `family` is the CSS font.
// - `MAX_COLS` is 160, the same cap as engine/src/lib.rs.
// - `ROWS` is 21, the same height as the baked word.

import glyphFile from "./glyphs.json";

export const FACES = [
  { id: "pesty", label: "Pesty", family: '"IBM Plex Mono", ui-monospace, monospace' },
  { id: "plex", label: "Plex Mono", family: '"IBM Plex Mono", ui-monospace, monospace' },
  { id: "georgia", label: "Georgia", family: 'Georgia, "Times New Roman", serif' },
  { id: "impact", label: "Impact", family: 'Impact, "Arial Black", sans-serif' },
] as const;

export type FaceId = (typeof FACES)[number]["id"];

const GAP = glyphFile.gap;
const GLYPHS = glyphFile.glyphs;

export type Cells = {
  cols: number;
  rows: number;
  cells: Uint8Array;
};

const MAX_COLS = 160;
const ROWS = 21;

export function faceById(id: string): (typeof FACES)[number] {
  return FACES.find((face) => face.id === id) ?? FACES[0];
}

export function cellsFor(text: string, face: string): Cells | null {
  const title = text.trim().slice(0, 24);
  if (!title) return null;
  if (face === "pesty") return composePesty(title);
  const drawn = raster(title, faceById(face).family);
  if (ink(drawn) === 0) return composePesty(title);
  return drawn;
}

function ink(mark: Cells): number {
  let n = 0;
  for (const bit of mark.cells) n += bit;
  return n;
}

function composePesty(text: string): Cells | null {
  const chars = [...text.toLowerCase()];
  if (chars.length === 0) return null;
  let rows: string[] | null = null;
  for (const char of chars) {
    const glyph = glyphFor(char);
    if (!rows) {
      rows = [...glyph];
      continue;
    }
    rows = rows.map((line, i) => `${line}${".".repeat(GAP)}${glyph[i] ?? ""}`);
  }
  if (!rows) return null;
  if (rows[0].length > MAX_COLS) {
    rows = rows.map((line) => line.slice(0, MAX_COLS));
  }
  const blank = ".".repeat(rows[0].length);
  const lines = [blank, ...rows, blank];
  const cols = lines[0].length;
  const cells = new Uint8Array(cols * lines.length);
  lines.forEach((line, row) => {
    for (let col = 0; col < cols; col += 1) {
      if (line[col] === "#") cells[row * cols + col] = 1;
    }
  });
  return { cols, rows: lines.length, cells };
}

function glyphFor(char: string): string[] {
  const known = GLYPHS[char as keyof typeof GLYPHS];
  if (known) return known;
  if (char === " ") return Array.from({ length: 19 }, () => "....");
  return [
    ".........",
    ".........",
    ".........",
    ".........",
    ".........",
    "..#####..",
    "..#...#..",
    "..#...#..",
    "..#...#..",
    "..#...#..",
    "..#...#..",
    "..#...#..",
    "..#...#..",
    "..#...#..",
    "..#####..",
    ".........",
    ".........",
    ".........",
    ".........",
  ];
}

function raster(text: string, family: string): Cells {
  const canvas = document.createElement("canvas");
  const ctx = canvas.getContext("2d", { willReadFrequently: true });
  if (!ctx) return { cols: 8, rows: ROWS, cells: new Uint8Array(8 * ROWS) };
  let size = 20;
  const widthAt = (px: number) => {
    ctx.font = `600 ${px}px ${family}`;
    return Math.ceil(ctx.measureText(text).width);
  };
  while (size > 8 && widthAt(size) > MAX_COLS) size -= 1;
  const cols = Math.max(8, Math.min(MAX_COLS, widthAt(size)));
  canvas.width = cols;
  canvas.height = ROWS;
  ctx.clearRect(0, 0, cols, ROWS);
  ctx.font = `600 ${size}px ${family}`;
  ctx.fillStyle = "#ffffff";
  ctx.textBaseline = "middle";
  ctx.textAlign = "left";
  ctx.fillText(text, 0, ROWS / 2 + 0.5);
  const image = ctx.getImageData(0, 0, cols, ROWS);
  const cells = new Uint8Array(cols * ROWS);
  for (let i = 0; i < cells.length; i += 1) {
    const alpha = image.data[i * 4 + 3];
    const white = image.data[i * 4];
    cells[i] = alpha > 96 && white > 48 ? 1 : 0;
  }
  return trim(cols, ROWS, cells);
}

function trim(cols: number, rows: number, cells: Uint8Array): Cells {
  let left = cols;
  let right = -1;
  for (let col = 0; col < cols; col += 1) {
    for (let row = 0; row < rows; row += 1) {
      if (cells[row * cols + col] === 0) continue;
      left = Math.min(left, col);
      right = Math.max(right, col);
    }
  }
  if (right < left) return { cols, rows, cells };
  const nextCols = right - left + 1;
  const next = new Uint8Array(nextCols * rows);
  for (let row = 0; row < rows; row += 1) {
    next.set(cells.subarray(row * cols + left, row * cols + right + 1), row * nextCols);
  }
  return { cols: nextCols, rows, cells: next };
}
