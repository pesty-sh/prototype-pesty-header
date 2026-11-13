// Loads public/engine.wasm and copies each finished picture onto the canvas.
// The Rust source is engine/src. Do not edit the wasm file. Rebuild it.
//
// Process:
// 1. `Field.load` fetches `/engine.wasm` and calls `engine_new`.
// 2. The page calls `frame`, which runs the Rust `paint`.
// 3. `blit` reads the pixel pointer after `frame` returns. The wasm memory
//    can move on the next call, so do not keep that pointer.
//
// Config that crosses this file:
// - Bools are `0` or `1`. Rust has no boolean in the wasm exports.
// - `setHex` kind `0` is the tint. Kind `1` is the background.
//   The string is at most 16 bytes (`#rrggbb` fits).
// - `setMark` writes one byte per cell. `1` is ink. `0` is empty.
//   The size limits (1–160 columns, 8–48 rows) are checked in engine/src/lib.rs.

interface Exports {
  memory: WebAssembly.Memory;
  engine_new(): number;
  engine_free(ptr: number): void;
  engine_resize(ptr: number, width: number, height: number): void;
  engine_clear_quiet(ptr: number): void;
  engine_push_quiet(ptr: number, x: number, y: number, w: number, h: number): void;
  engine_pointer(ptr: number, x: number, y: number, inside: number): void;
  engine_pointer_down(ptr: number, x: number, y: number, now: number): void;
  engine_pointer_up(ptr: number, x: number, y: number, now: number): number;
  engine_pointer_cancel(ptr: number): void;
  engine_replay(ptr: number, now: number): void;
  engine_hit(ptr: number, x: number, y: number): number;
  engine_frame(
    ptr: number,
    now: number,
    markX: number,
    markY: number,
    markW: number,
    markH: number,
    reduced: number,
  ): void;
  engine_pixels(ptr: number): number;
  engine_byte_len(ptr: number): number;
  engine_width(ptr: number): number;
  engine_height(ptr: number): number;
  engine_ripples(ptr: number): number;
  engine_reveal(ptr: number): number;
  engine_cell(ptr: number): number;
  engine_appear(ptr: number): number;
  appear_name(index: number): number;
  appear_name_len(index: number): number;
  logo_cols(): number;
  logo_rows(): number;
  engine_mark_prepare(ptr: number, cols: number, rows: number): number;
  engine_mark_commit(ptr: number): void;
  engine_mark_reset(ptr: number): void;
  engine_logo_cols(ptr: number): number;
  engine_logo_rows(ptr: number): number;
  engine_hex_prepare(ptr: number): number;
  engine_hex_commit(ptr: number, kind: number, len: number): number;
}

export class Field {
  cols: number;
  rows: number;

  private constructor(
    private readonly api: Exports,
    private readonly ptr: number,
  ) {
    this.cols = api.logo_cols();
    this.rows = api.logo_rows();
  }

  static async load(url = "/engine.wasm"): Promise<Field> {
    const instance = await instantiate(url);
    const api = instance.exports as unknown as Exports;
    return new Field(api, api.engine_new());
  }

  resize(width: number, height: number): void {
    this.api.engine_resize(this.ptr, width, height);
  }

  clearQuiet(): void {
    this.api.engine_clear_quiet(this.ptr);
  }

  pushQuiet(x: number, y: number, w: number, h: number): void {
    this.api.engine_push_quiet(this.ptr, x, y, w, h);
  }

  pointer(x: number, y: number, inside: boolean): void {
    this.api.engine_pointer(this.ptr, x, y, inside ? 1 : 0);
  }

  pointerDown(x: number, y: number, now: number): void {
    this.api.engine_pointer_down(this.ptr, x, y, now);
  }

  pointerUp(x: number, y: number, now: number): boolean {
    return this.api.engine_pointer_up(this.ptr, x, y, now) === 1;
  }

  pointerCancel(): void {
    this.api.engine_pointer_cancel(this.ptr);
  }

  replay(now: number): void {
    this.api.engine_replay(this.ptr, now);
  }

  hit(x: number, y: number): boolean {
    return this.api.engine_hit(this.ptr, x, y) === 1;
  }

  frame(
    now: number,
    markX: number,
    markY: number,
    markW: number,
    markH: number,
    reduced: boolean,
  ): void {
    this.api.engine_frame(this.ptr, now, markX, markY, markW, markH, reduced ? 1 : 0);
  }

  blit(ctx: CanvasRenderingContext2D): void {
    const ptr = this.api.engine_pixels(this.ptr);
    const len = this.api.engine_byte_len(this.ptr);
    const width = this.api.engine_width(this.ptr);
    const height = this.api.engine_height(this.ptr);
    if (width === 0 || height === 0 || len !== width * height * 4) {
      return;
    }
    const bytes = new Uint8ClampedArray(this.api.memory.buffer, ptr, len);
    ctx.putImageData(new ImageData(bytes, width, height), 0, 0);
  }

  ripples(): number {
    return this.api.engine_ripples(this.ptr);
  }

  reveal(): number {
    return this.api.engine_reveal(this.ptr);
  }

  appear(): string {
    const index = this.api.engine_appear(this.ptr);
    const ptr = this.api.appear_name(index);
    const len = this.api.appear_name_len(index);
    const bytes = new Uint8Array(this.api.memory.buffer, ptr, len);
    return new TextDecoder().decode(bytes);
  }

  cell(): number {
    return this.api.engine_cell(this.ptr);
  }

  setMark(cells: Uint8Array, cols: number, rows: number): boolean {
    const ptr = this.api.engine_mark_prepare(this.ptr, cols, rows);
    const len = cols * rows;
    if (ptr === 0 || cells.length < len) return false;
    new Uint8Array(this.api.memory.buffer, ptr, len).set(cells.subarray(0, len));
    this.api.engine_mark_commit(this.ptr);
    this.cols = this.api.engine_logo_cols(this.ptr);
    this.rows = this.api.engine_logo_rows(this.ptr);
    return true;
  }

  resetMark(): void {
    this.api.engine_mark_reset(this.ptr);
    this.cols = this.api.engine_logo_cols(this.ptr);
    this.rows = this.api.engine_logo_rows(this.ptr);
  }

  setHex(kind: number, hex: string): boolean {
    const bytes = new TextEncoder().encode(hex);
    if (bytes.length > 16) return false;
    const ptr = this.api.engine_hex_prepare(this.ptr);
    if (ptr === 0) return false;
    new Uint8Array(this.api.memory.buffer, ptr, bytes.length).set(bytes);
    return this.api.engine_hex_commit(this.ptr, kind, bytes.length) === 1;
  }
}

async function instantiate(url: string): Promise<WebAssembly.Instance> {
  try {
    const streamed = await WebAssembly.instantiateStreaming(fetch(url), {});
    return streamed.instance;
  } catch {
    const bytes = await fetch(url).then((res) => res.arrayBuffer());
    const compiled = await WebAssembly.instantiate(bytes, {});
    return compiled.instance;
  }
}
