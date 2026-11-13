---
name: pesty-header
description: Update the pesty.sh pixel header — the Rust pointer, motion, and wordmark engine, the generated wasm, the canvas page, or the dev server on port 43123. Use when editing the hero, the mark, engine sources, the wasm build, or the port-release scripts.
---

# pesty.sh header

The pixel header is a Rust engine (`engine/`) compiled to WebAssembly and blitted by a Vite page (`src/`). The engine has three scopes only: pointer, motion, and the `pesty.sh` wordmark.

Read `README.md` for the first-timer path, then `AGENTS.md` and `docs/engine.md`. This skill is the short version of those files. `docs/engine.md` opens with one frame in plain words. Picture knobs are in `engine/src/config.rs`. A click icon spawns from a field release. The wander stays a pixel ball. `DRIFT_FIRE` lets it shoot one pixel trail on a random wait of at least five seconds. The trail lands at a random point inside the canvas and bursts into a random icon.

## Source

The `.rs` files are the source. `public/engine.wasm` is the release build. The demo host commits it so a publish does not install Rust. Do not edit the wasm or decompile it back into Rust. After a Rust change, rebuild it and commit the new file.

```bash
cargo test --manifest-path engine/Cargo.toml
npm run build:wasm
```

`npm test` runs those tests, rebuilds the wasm, and runs `scripts/smoke.mjs`.

## Edit map

Edit in this order. The catalogs are `appears.rs` and `sprites.rs`.

- Colours: `engine/src/palette.rs` and `--bg`, `--lit`, `--mid`, `--crest` in `src/style.css`
- Letter shapes: hand-drawn `p e s t y . h` in `engine/mark.py`. The rest of printable ASCII is `engine/alphabet.py`. Then `python3 engine/mark.py` (rewrites `mark.txt` from `pesty.sh` only, and `src/glyphs.json` from the whole set). Capitals use the lowercase glyph.
- Live title: demo only. Dev server, or `?demo` on a published build. `?demo=0` is the public page and removes the title, face, tint, and background boxes. `?text=` and `?face=` (`pesty`, `plex`, `georgia`, `impact`). Tint and background are hex strings (`?tint=` and `?bg=`, no `#`). Defaults `#d6ee6a` and `#12140f` keep the baked palette. The engine takes the cells through `engine_mark_prepare` / `engine_mark_commit`, and the hex through `engine_hex_prepare` / `engine_hex_commit` (kind 0 tint, kind 1 background).
- Bitmap and light bands: `engine/src/logo.rs`, `engine/src/mark.txt`
- Click appears: `engine/src/appears.rs`. Reorder `ORDER`. Add a variant, a name, and a branch in `shows`. `CYCLE` is `Sequence` or `Random`.
- Icons: `engine/src/sprites.rs`. 15×15, `#` and `.` only. `CAST` is `Random` or `First`. Plays are `Appear`, `Slide`, `Expand`. Slide spins across the header, spits smaller random icons, and bursts on each landing. A new play is a variant plus a branch in `Sprite::place`.
- Pointer, quiet rects, charge: `engine/src/pointer.rs`
- Appear clock (`REVEAL_MS`, 1100ms) and wander: `engine/src/motion.rs`
- Noise, Bayer, vignette: `engine/src/field.rs`. A wide cursor well (`NOISE_WELL_CELLS`) hoovers speckles onto the pointer. They fall back when the pointer leaves. The noise points themselves are not moved.
- Frame, numbered in `paint`: `engine/src/paint.rs`
- `Engine` and wasm exports: `engine/src/lib.rs` (exports are `cfg`'d to wasm32)
- Canvas size and pointer events: `src/main.ts`
- Wasm instantiate and blit: `src/engine.ts`
- Public `dist/`: `docs/release.md`. `npm test` then `npm run build`. `npm run build` only bundles the page. The demo host is static: `public/engine.wasm` is committed, and Vercel runs `vite build`. Do not commit `dist/`.

The bitmap test expects 80×21 and 465 ink pixels. Change the assertion when the ink count changes. `mark.py` overwrites `mark.txt`.

`src/main.ts` must set `canvas.width` and `canvas.height` to the device-pixel size (dpr capped at 2) and resize the engine to that same size. The blit copies wasm memory after `engine_frame`, because the buffer can move.

## Port 43123

`npm run dev` is `node scripts/dev.mjs`. Ctrl+C must print `Port 43123 is free.` and leave the port bindable. `npm run stop` is the same release when the terminal is gone.

Only signal a process whose cwd is this repo and whose command is Vite or `scripts/dev.mjs`. Do not kill a foreign listener. `.pesty/` is gitignored.

## Done

Run the Rust tests after an engine change, `npx tsc --noEmit` after a TypeScript change, and `npm test` when the wasm or the page contract changed. Look at the hero: the wordmark has to be on screen, not clipped to a 300×150 canvas.
