// Asks the built wasm a few questions. `npm test` runs this after the Rust tests.
//
// Process:
// 1. The logo must be 80×21.
// 2. After 2 seconds the word has more crest pixels than the first frame.
// 3. A pointer near the word brightens the buffer.
// 4. A press and release on the field leaves one icon, then it is gone later.
//
// A failure here usually means paint.rs, appears.rs, or the wasm export in lib.rs changed.

import { readFile } from "node:fs/promises";

const bytes = await readFile(new URL("../public/engine.wasm", import.meta.url));
const { instance } = await WebAssembly.instantiate(bytes, {});
const api = instance.exports;

const cols = api.logo_cols();
const rows = api.logo_rows();
if (cols !== 80 || rows !== 21) {
  throw new Error(`unexpected logo size ${cols}x${rows}`);
}

const ptr = api.engine_new();
api.engine_resize(ptr, 480, 260);
const markH = 400 * (rows / cols);
api.engine_frame(ptr, 0, 24, 28, 400, markH, 0);
const early = count(api, ptr, [0xf7, 0xff, 0xd4]);
api.engine_frame(ptr, 2000, 24, 28, 400, markH, 0);
const full = count(api, ptr, [0xf7, 0xff, 0xd4]);
if (full <= early + 80) {
  throw new Error(`print did not land: early ${early} full ${full}`);
}

const before = brightness(api, ptr);
api.engine_pointer(ptr, 24 + 200, 28 + markH / 2, 1);
for (let i = 0; i < 12; i += 1) {
  api.engine_frame(ptr, 2020 + i * 16, 24, 28, 400, markH, 0);
}
const after = brightness(api, ptr);
if (after <= before + 2000) {
  throw new Error(`pointer did not lift the field: ${before} -> ${after}`);
}

api.engine_pointer_down(ptr, 4, 4, 3000);
if (api.engine_pointer_up(ptr, 4, 4, 3200) !== 0) {
  throw new Error("a field press was treated as the wordmark");
}
if (api.engine_ripples(ptr) !== 1) {
  throw new Error(`expected one sprite, saw ${api.engine_ripples(ptr)}`);
}
if (api.engine_appear(ptr) !== 0) {
  throw new Error(`first appear should be print, saw ${api.engine_appear(ptr)}`);
}
api.engine_replay(ptr, 4000);
if (api.engine_appear(ptr) !== 1) {
  throw new Error(`click should advance the appear, saw ${api.engine_appear(ptr)}`);
}

api.engine_free(ptr);
console.log(`pesty engine ok · logo ${cols}×${rows} · crest ${full} · pointer +${after - before}`);

function view(api, ptr) {
  const address = api.engine_pixels(ptr);
  const length = api.engine_byte_len(ptr);
  return new Uint8Array(api.memory.buffer, address, length);
}

function count(api, ptr, [r, g, b]) {
  const pixels = view(api, ptr);
  let n = 0;
  for (let i = 0; i < pixels.length; i += 4) {
    if (pixels[i] === r && pixels[i + 1] === g && pixels[i + 2] === b) n += 1;
  }
  return n;
}

function brightness(api, ptr) {
  const pixels = view(api, ptr);
  let n = 0;
  for (let i = 0; i < pixels.length; i += 1) n += pixels[i];
  return n;
}
