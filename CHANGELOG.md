# Changelog

All notable changes to the pesty.sh header. Dates are Europe/London.

## 0.1.7 — 13 November 2026

### Changed

- Speckles slide toward the cursor by how close it is, and slide back out as the cursor moves away. Leaving the canvas is no longer the only in-and-out.

## 0.1.6 — 13 November 2026

### Changed

- The cursor hoovers speckles from a wider radius and holds them. The cells they left stay quiet until the cursor moves off, and then the speckles fall back.

## 0.1.5 — 13 November 2026

### Changed

- Speckles inside a small radius pull toward the cursor and gather there. Each point keeps its own place. When the cursor moves off that place, the speckle falls back.
- The page and the canvas use the field background while wasm loads. A white `?bg` no longer starts black.

## 0.1.4 — 13 November 2026

### Changed

- A fired trail lands at a random point inside the canvas. The burst no longer opens on the top edge.
- The wait between fires is rolled from five seconds up to eight.

## 0.1.3 — 13 November 2026

### Changed

- The wander is a ball of pixels again. Every few seconds it fires one trail of pixels. The trail lands and bursts into a random icon. The gap is rolled between the two waits in `engine/src/config.rs`.
- Titles use Satoshi Bold.

## 0.1.2 — 13 November 2026

### Changed

- The sprite that moves on its own, not the mouse, pops a random icon on a timer. That icon flies to a random point and bursts. The knobs for it, and the other picture settings, are in `engine/src/config.rs`.

## 0.1.1 — 13 November 2026

### Changed

- A slide spins across the header. As it moves it drops smaller random icons, and it bursts on each random landing. The dropped icons fly to their own landing and burst too. They are drawn with that one release, so the cap is still five live icons.

## 0.1.0 — 3 November 2026

### Added

- A Rust pixel engine, compiled to WebAssembly and blitted to one canvas. It does three jobs: pointer, motion, and the `pesty.sh` wordmark.
- A dithered field. Most cells stay the ground colour. The pointer, a slow wander, and released icons wake them.
- The hand-drawn wordmark, 80×21, lit from crest on the ascenders down to dim on the tails. Clicking it plays the next appear.
- Click appears, in order: print, drop, rise, bloom, spark, then back to print. `Cycle::Random` is there if the order should be a roll instead.
- Icons a field release can drop: bug, seed, spark, and mite. Each release appears, slides, or expands. `Cast::First` is the original bug stamp.
- Quiet rectangles over the nav and the sentence, so the type stays put.
- `prefers-reduced-motion` finishes the mark, skips the wander, and drops no icons. The pointer still lights the cells under it.
- A fixed dev server on port 43123. Ctrl+C, and `npm run stop`, free that port. A listener that is not this repo is left alone.
- Human docs in `README.md`, `docs/engine.md`, and `docs/release.md`, plus `AGENTS.md` and the pesty-header skill.
- Demo mode. The dev server is demo, and a published build opens it with `?demo`. The title is a string. The face is Pesty, Plex Mono, Georgia, or Impact. Tint and Background are hex strings. `#d6ee6a` and `#12140f` keep the baked colours. Any other hex recolours that visit.
- A shareable demo link: `?demo&text=hello&face=georgia&tint=ff6040&bg=101820`.
- The MIT license.
- A full printable ASCII set for the Pesty face. `bryter.digital` draws as letters. Capitals use the lowercase glyphs. The baked `pesty.sh` mark is unchanged.
- The demo tint colours the input boxes, their labels, the caret, focus rings, the eyebrow, the live dot, link hovers, and the scope numbers.

### Changed

- The README is the first-timer path: install, first run, what you should see, how to stop, and the order to read the files. `docs/engine.md` opens with one frame in plain words. The file headers say the same thing a new reader needs.

### Fixed

- Tint and Background apply as the hex is typed. A click selects the whole colour. Six digits redraw the field at once, and further digits keep the last six so the box does not ignore the edit.

### Changed

- Production, and `?demo=0`, draw the hand-drawn `pesty.sh` and show no title, face, tint, or background boxes.
- The public page no longer mentions anything but pesty.
