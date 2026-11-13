// Demo colour helpers. The engine still decides the pixels.
//
// Process:
// 1. `isDemo` decides whether the title, face, and hex boxes are on the page.
// 2. `hexWhileTyping` accepts a 6-digit hex while the box is still being typed.
// 3. `readableAccent` nudges the tint until the page type still reads on it.
//
// Config:
// - `DEFAULT_TINT` and `DEFAULT_BG` must match `LIT` and `BG` in
//   engine/src/palette.rs. Typing either one clears the override.
// - No `?demo` flag follows the dev server (boxes on). `?demo=0` or
//   `?demo=false` is the public page. Any other `?demo` turns the boxes on.

export const DEFAULT_TINT = "#d6ee6a";
export const DEFAULT_BG = "#12140f";

export function isDemo(search: string, dev: boolean): boolean {
  const flag = new URLSearchParams(search).get("demo");
  if (flag === null) return dev;
  if (flag === "0" || flag === "false") return false;
  return true;
}

export function normalizeHex(input: string): string | null {
  const body = input.trim().replace(/^#/, "");
  if (!/^[0-9a-fA-F]{3}$|^[0-9a-fA-F]{6}$/.test(body)) return null;
  const full = body.length === 3 ? body.replace(/./g, (digit) => digit + digit) : body;
  return `#${full.toLowerCase()}`;
}

/** A colour the field can apply while the box is still being typed. */
export function hexWhileTyping(input: string): string | null {
  const body = input.trim().replace(/^#/, "");
  if (/^[0-9a-fA-F]{6}$/.test(body)) return `#${body.toLowerCase()}`;
  if (/^[0-9a-fA-F]+$/.test(body) && body.length > 6) {
    return `#${body.slice(-6).toLowerCase()}`;
  }
  return null;
}

export function hexLooksOpen(input: string): boolean {
  const body = input.trim().replace(/^#/, "");
  return body.length === 0 || /^[0-9a-fA-F]{1,6}$/.test(body);
}

export function luma(hex: string): number {
  const [r, g, b] = channels(hex);
  return (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255;
}

function channels(hex: string): [number, number, number] {
  const normal = normalizeHex(hex);
  if (!normal) return [0, 0, 0];
  return [
    Number.parseInt(normal.slice(1, 3), 16),
    Number.parseInt(normal.slice(3, 5), 16),
    Number.parseInt(normal.slice(5, 7), 16),
  ];
}

export function mixHex(a: string, b: string, t: number): string {
  const pa = channels(a);
  const pb = channels(b);
  const channel = (i: number) => Math.round(pa[i] + (pb[i] - pa[i]) * t);
  const byte = (n: number) => n.toString(16).padStart(2, "0");
  return `#${byte(channel(0))}${byte(channel(1))}${byte(channel(2))}`;
}

/** Tint, nudged toward black or white until it still reads on `against`. */
export function readableAccent(tint: string, against: string): string {
  let color = normalizeHex(tint) ?? DEFAULT_TINT;
  const toward = luma(against) > 0.55 ? "#1a1c14" : "#f7ffd4";
  for (let step = 0; step < 8; step += 1) {
    if (contrast(color, against) >= 3) return color;
    color = mixHex(color, toward, 0.28);
  }
  return color;
}

function contrast(a: string, b: string): number {
  const light = (hex: string) => {
    const [r, g, b] = channels(hex).map((channel) => {
      const s = channel / 255;
      return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
    });
    return 0.2126 * r + 0.7152 * g + 0.0722 * b;
  };
  const hi = Math.max(light(a), light(b));
  const lo = Math.min(light(a), light(b));
  return (hi + 0.05) / (lo + 0.05);
}
