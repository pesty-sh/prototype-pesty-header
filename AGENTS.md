# Agents

This repo is the pesty.sh pixel header. The engine does three jobs: pointer, motion, and the wordmark. Keep it that way.

## Source of truth

Edit `engine/src/*.rs`. The wasm is built from those files. It was not decompiled from `public/engine.wasm`, and that file is gitignored. Do not hand-edit the wasm, do not commit it, and do not try to recover Rust from it.

After a Rust change:

```bash
cargo test --manifest-path engine/Cargo.toml
npm run build:wasm
```

`npm test` is the Rust tests, a release wasm build, and `node scripts/smoke.mjs`. `npm run dev` rebuilds the wasm before it serves.

## Where a change goes

Read `docs/engine.md` before changing a picture. The edit order there is the order of the modules.

| Task | File |
| --- | --- |
| Colours | `engine/src/palette.rs` and the same hexes in `src/style.css` |
| Letter shapes | `engine/mark.py`, then `python3 engine/mark.py` |
| Bitmap rows, hit box, light bands | `engine/src/logo.rs` and `engine/src/mark.txt` |
| Another logo click appear, or the click order | `engine/src/appears.rs` |
| Another icon, or appear / slide / expand | `engine/src/sprites.rs` |
| Cursor, quiet rects, charge | `engine/src/pointer.rs` |
| Appear duration, wander | `engine/src/motion.rs` |
| Noise, Bayer, vignette | `engine/src/field.rs` |
| Composing a frame | `engine/src/paint.rs` |
| Engine API and wasm exports | `engine/src/lib.rs` |
| Page, canvas size, quiet DOM boxes | `src/main.ts` |
| Wasm load and blit | `src/engine.ts` |
| Publishing `dist/` | `docs/release.md` |

`logo.rs` asserts 80×21 and 465 ink pixels. Update the count when a letter change moves it. Prefer `mark.py` over editing `mark.txt` by hand. The script overwrites `mark.txt`.

The canvas backing store and the wasm buffer must both be the device-pixel size. `src/main.ts` assigns `canvas.width` and `canvas.height` before `field.resize`.

`docs/engine.md` is the human map of the same layout. Read it before changing a scope.

## Dev server

The page is served on port **43123** (`PESTY_PORT` overrides one run). `strictPort` is on.

`npm run dev` runs `scripts/dev.mjs`. That process builds the wasm, starts Vite in its own process group, and on SIGINT, SIGTERM, or SIGHUP kills that group and waits until 43123 accepts a bind. It prints `Port 43123 is free.` A second SIGINT force-kills.

`npm run stop` frees a leftover server from this repo. It may signal a process only when the cwd is this repo and the command is Vite or `scripts/dev.mjs`. Never kill a listener that fails that test. Never clear a port by killing an unrelated pid.

Leave the dev server running on 43123 when a session needs the page. `.pesty/dev.json` is the pidfile and is gitignored.

## Checks

- `cargo test --manifest-path engine/Cargo.toml` after engine edits.
- `npx tsc --noEmit` after TypeScript edits.
- `npm test` before handing the header back.
- The hero is a real page. Confirm the wordmark is visible at desktop and mobile widths. A 300×150 canvas means the backing store was not resized.

## Leave it alone

Do not add a framework, a second renderer, or a scope beyond pointer, motion, and the logo. New click fills go in `appears.rs`. New icons and plays go in `sprites.rs`. The live title is demo mode: a string, a face, and two hex strings. Production, and `?demo=0`, do not show those inputs. The hand-drawn letters stay in `engine/mark.py`. Do not invent a company story for the copy. The field palette stays the six colours in `palette.rs`. Demo hexes override them for one visit and the defaults `#d6ee6a` and `#12140f` clear the override. The license is MIT.
