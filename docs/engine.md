# The pixel engine

Read `README.md` and get the page running before this file. This is the map for someone changing the picture.

## One frame, in plain words

`engine/src/paint.rs` builds every picture. The numbered comments in `paint` are the order. Read that function before the rest of this file.

1. Fill the whole buffer with the background colour. Squares that fail the speckle test stay this colour.
2. Ease the pointer toward the latest mouse sample. Boxes over the nav and the sentence force that glow down so the type stays readable.
3. Advance the appear clock. The shape of the fill, left to right or top to bottom or the others, is `appears.rs`. The length of the clock is `REVEAL_MS` in `motion.rs`.
4. Collect glows: the pointer, and a slow drift when the mouse is away and motion is allowed.
5. Collect icons. The one under a held pointer, then any that were released earlier and are still alive. A slide also contributes the smaller icons it has spat out. A release on the word is not in this list. It restarts the appear instead.
6. For each square, decide. If it is a letter cell and the appear says it is visible, paint the overhead colour. Otherwise it is field: noise has to clear a threshold, helped by the glows and the icons. Inside a wide radius the pointer hoovers those speckles onto itself and the cells they left stay quiet. The noise points stay where they are. When the pointer moves off a point, that speckle is drawn on it again.

Nothing else draws pixels. `lib.rs` calls `paint` once per frame. `src/main.ts` copies the buffer onto the canvas after that call.

A click icon does not spawn on a timer. Search for `spawn_sprite` in `lib.rs`. It runs from a pointer release on the field. The roll of which icon and which play happens at the press, in `sprites.rs`, so the shape under your finger is the shape that runs. The wander stays a ball of pixels. `tick_drift` fires one trail of pixels on a random wait of at least five seconds. The trail lands at a random point inside the canvas and bursts into a random icon. `MAX_SPRITES` in `engine/src/config.rs` caps both.

## Edit order

The reading order starts at `paint.rs`, then follows the calls. This list is different. It is the order to open files when you are changing a value, because later files use the earlier ones:

1. `engine/src/config.rs` — every knob, including the wander and its trail
2. `engine/src/palette.rs` — the six colours
3. `engine/src/logo.rs` — the wordmark bitmap and the light bands
4. `engine/src/appears.rs` — what a click on the word does, in order
5. `engine/src/sprites.rs` — icons, and whether a release appears, slides, or expands
6. `engine/src/pointer.rs` — the mouse
7. `engine/src/motion.rs` — the shared appear clock, and the wander
8. `engine/src/field.rs` — noise, dither, vignette
9. `engine/src/paint.rs` — one frame, numbered in the same order it draws
10. `engine/src/lib.rs` — the `Engine` that runs those, then the wasm exports

`paint.rs` is annotated as the frame sequence: background, pointer, appear clock, glows, sprites, then each cell.

## The Rust is the source

`engine/src/*.rs`, `engine/src/mark.txt`, and `engine/mark.py` are the project. `public/engine.wasm` is the release build of that crate. The demo host commits that file so it can publish without Rust. Rebuild it when the Rust changes, and commit the new file. Do not edit it, and do not try to recover source from it.

```bash
npm run build:wasm
```

That is `cargo build --release` for `wasm32-unknown-unknown`, then a copy to `public/engine.wasm`. The crate has no wasm-bindgen dependency. Host tests call `Engine` directly. The exports exist only when the target is wasm32, so `cargo test` does not need that target.

```bash
cargo test --manifest-path engine/Cargo.toml
npm test
```

`npm test` runs the Rust tests, rebuilds the wasm, and checks it with `scripts/smoke.mjs` (logo size, a crest after the print, a pointer lift, one stamp).

## Letters

`engine/mark.py` holds the hand-drawn glyphs for `p e s t y . h`. Every other printable ASCII character is in `engine/alphabet.py`, on the same 19-row grid. Capitals use the lowercase letter. The script adds one blank row above and below and two empty columns between letters, and writes `engine/src/mark.txt` from `pesty.sh` only.

```bash
python3 engine/mark.py
```

`#` is ink, `.` is empty, one row per line. `logo.rs` loads that file with `include_str!` and checks the size: 80×21, 465 ink pixels. A letter change that adds or removes ink has to update that count in the test.

The cell size on screen is the wordmark box divided by the bitmap. The page measures the box from the DOM spacer behind the bitmap.

## Live title

The public page draws the baked `pesty.sh`. It has no title field, no face menu, and no colour boxes.

Demo mode is the live page. The dev server is demo. A published build is production until the address has `?demo` (or `?demo=1`). `?demo=0` is the public page, including on the dev server.

Demo adds four strings under the word:

| Box | String |
| --- | --- |
| Title | up to 24 characters. `pesty.sh` in the Pesty face is the baked bitmap |
| Face | `pesty`, `plex`, `georgia`, or `impact` |
| Tint | a hex colour, default `#d6ee6a` |
| Background | a hex colour, default `#12140f` |

Pesty composes letters from `engine/mark.py` and `engine/alphabet.py`. Run `python3 engine/mark.py` after a letter change. That rewrites `engine/src/mark.txt` and `src/glyphs.json`. Printable ASCII is covered. A character outside that set is drawn as a small box.

Plex Mono, Georgia, and Impact are set in the browser at the cell grid, then the ink cells are handed to the engine. The engine stores that grid in place of the baked mark and plays the appear again.

Tint and background are hex strings (`#rgb`, `rgb`, `#rrggbb`, or `rrggbb`). The engine parses them. The default tint and the default background leave the six baked colours alone. Any other tint keeps each band's lightness and takes hue and saturation from the string. Any other background replaces the ground. An unreadable string is ignored and the previous colour stays.

A shared link carries the strings. `?demo&text=hello&face=georgia&tint=ff6040&bg=101820` opens that title, face, and those two colours on a published build. `text` is at most 24 characters. The grid is capped at 160 columns and must be at least 8 rows.

Rows are lit from above:

| Band | Row fraction |
| --- | --- |
| Crest | under 0.26 |
| Hover | under 0.37 |
| Lit | under 0.58 |
| Mid | under 0.74 |
| Dim | the rest |

A pointer near a letter mixes that band toward crest. The field itself never uses crest. Its cells are dim, mid, or lit.

## Colours

`engine/src/palette.rs` is the ink.

| Name | Hex |
| --- | --- |
| BG | `#12140f` |
| DIM | `#3e4f28` |
| MID | `#739238` |
| LIT | `#d6ee6a` |
| HOVER | `#e7f7a4` |
| CREST | `#f7ffd4` |

`src/style.css` repeats BG, LIT, MID, and CREST as `--bg`, `--lit`, `--mid`, and `--crest` for the page chrome. Change a shared colour in both files. `--text`, `--muted`, and `--line` are page type, not field ink.

Demo hex strings do not edit this table. They override the ground and recolour the ink for that visit. Clearing a box, or typing the default hex again, returns to the table.

## Click appears

The first load plays the first name in `ORDER` inside `engine/src/appears.rs`. That name is `print`, the original left-to-right fill. Each click on the wordmark advances one entry and starts the clock again. After the last name, the next click returns to the first.

The list that ships:

| Order | Name | What you see |
| --- | --- | --- |
| 1 | print | Left to right, with a little noise in the columns |
| 2 | drop | Ascenders first, tails last |
| 3 | rise | Tails first, ascenders last |
| 4 | bloom | Center of the word outward |
| 5 | spark | Cells arrive in hash order |

Every appear starts empty and ends on the full bitmap. They share one clock: 1100ms, ease-out cubic, `REVEAL_MS` in `motion.rs`. Change the duration there. Change the shape in `appears.rs`.

`CYCLE` is the option.

- `Cycle::Sequence` walks `ORDER`. This is the setting that ships, so the list you are editing is the list you see.
- `Cycle::Random` rolls a name from the clock of the click, and skips a repeat of the one just played.

Add an appear:

1. Add a variant to `Appear`, and its name in `Appear::name`.
2. In `shows`, say when a cell is visible. `progress` is 0 at the start and 1 when the bitmap is complete.
3. Push the variant onto `ORDER`. Reorder that array to change the click sequence.

Reduced motion holds the mark finished and does not advance the list.

## Icons

A press on the field charges one icon. The release plays it. The press rolls the icon and the play once, so the shape under your finger is the shape that runs when you let go. Five live icons is the cap. Quiet rectangles still hide them under the type.

`CAST` in `engine/src/sprites.rs` is the option.

- `Cast::Random` rolls an icon and a play. This is the setting that ships.
- `Cast::First` is the original stamp: the bug, expanding in place, every time.

The icons are 15×15. `#` is ink, `.` is empty.

| Name | Shape |
| --- | --- |
| bug | The original stamp |
| seed | A small pebble |
| spark | A glint |
| mite | A compact mark with two feet |

The plays:

| Play | Motion |
| --- | --- |
| appear | Pops in, holds, fades. It does not travel. |
| slide | Spins across the header, spits smaller random icons, and bursts on each random landing. Those spat icons fly out and burst too. They belong to that one release. |
| expand | Grows in place and thins out. |

Add an icon:

1. Draw a 15×15 pattern next to the others. Every row is 15 characters, and the only marks are `#` and `.`.
2. Add an `Icon` to `ICONS`.

The build checks the row length and the characters. A bad row does not compile.

Add a play:

1. Add a variant to `Play`.
2. In `Sprite::place`, say where it is, how big it is, and how strong it is.
3. Teach `pick_with` to roll it, or leave the variant for a cast you set by hand.

## Pointer

Move sets a position and a strength that eases toward the latest sample. Quiet rectangles (the nav, the eyebrow, the sentence, padded on the page) force that strength to zero inside the type and recover it over a short reach, so the field does not paint through the words.

A press on the wordmark is a hit. Releasing it plays the next appear and does not drop an icon. A press on the field charges. The icon catalog lives in `sprites.rs`, not in this file.

## Motion

`reveal` is the clock every appear shares. The wander is a ball of pixels, not the mouse. It fires one trail of pixels on a random wait of at least five seconds. That trail lands at a random point inside the canvas and bursts into a random icon. Reduced motion sets the clock to finished, skips the wander, and drops no icons. The pointer glow still draws.

Every number for that sprite, and the other picture knobs, is in `engine/src/config.rs`.

## The page

`src/engine.ts` instantiates `public/engine.wasm` and copies the RGBA buffer into an `ImageData` after each frame. The copy has to happen after the call, because the wasm memory can move.

`src/main.ts` sizes the canvas backing store to the CSS box times the device pixel ratio, capped at 2. The wasm buffer is resized to that same width and height. Setting one without the other clips the wordmark. The page and the canvas are filled with the field background before the first wasm frame. `?bg` is that colour while the file is still loading, so a white ground does not start black. Resizing the canvas clears it to black, and the page paints the ground back before the frame is copied.

The frame is skipped while the header is off screen. The meter under the header reads the same engine: pointer, the appear name, live icons, cell size.

Publishing the built site is `docs/release.md`.

## Wasm exports

These are the C symbols in `engine/src/lib.rs`. Bools cross the boundary as `u32` 0 or 1. Pointers are `u32` because the wasm target is 32-bit.

`engine_new`, `engine_free`, `engine_resize`, `engine_clear_quiet`, `engine_push_quiet`, `engine_pointer`, `engine_pointer_down`, `engine_pointer_up`, `engine_pointer_cancel`, `engine_replay`, `engine_hit`, `engine_frame`, `engine_pixels`, `engine_byte_len`, `engine_width`, `engine_height`, `engine_ripples`, `engine_reveal`, `engine_cell`, `engine_appear`, `appear_len`, `appear_name`, `appear_name_len`, `logo_cols`, `logo_rows`, `engine_mark_prepare`, `engine_mark_commit`, `engine_mark_reset`, `engine_logo_cols`, `engine_logo_rows`, `engine_hex_prepare`, `engine_hex_commit`.

`engine_ripples` counts live icons. The name stayed so the page and the smoke test did not have to grow a second word for the same number. `engine_appear` is the index into `ORDER`. `appear_name` points at that name in wasm memory. `engine_hex_prepare` hands back 16 bytes. The page writes a hex string into them and `engine_hex_commit` reads it. Kind `0` is the tint. Kind `1` is the background.
