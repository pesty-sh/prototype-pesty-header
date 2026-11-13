# pesty.sh

A web page with a pixel header. The letters, the mouse glow, and the little icons are drawn by a Rust program that runs in the browser.

You edit the Rust and the TypeScript. A command turns the Rust into a file named `engine.wasm`. The page loads that file and copies each finished picture onto a canvas. Do not edit `engine.wasm`. The demo host ships the copy in `public/`. Rebuild that file when the Rust changes.

If you only want the page running, follow [Install](#install) and [Getting started](#getting-started) and stop there. If you need to change the picture, the knobs are in [Config](#config). The file map is [Read the code](#read-the-code).

## What the picture is doing

Three jobs, and only these three. Pointer, motion, and the logo share one RGBA buffer.

- **Pointer.** Where the cursor sits, and how long a press has been held. Inside a wide radius it hoovers speckles onto itself and holds that clump. Move away and each speckle returns to its own cell. The bar and the sentence are quiet rectangles, so the type stays put.
- **Motion.** The mark prints from the left on the first load. Each click runs the next appear. A ball of pixels wanders the field. Every few seconds it fires one trail of pixels. The trail lands at a random point inside the canvas and bursts into a random icon. A release on the field still drops one icon of its own: it may appear, slide, or expand. Reduced motion holds the mark finished, skips the wander, and drops no icons.
- **Logo.** The word `pesty.sh` is a bitmap, crest on the ascenders and dim on the tails. The demo redraws that bitmap from a title, a face, and two hex colours. The page uses the background hex while the wasm file is still loading.

The speckled ground is not a fourth job. The cursor can hoover it, and the wander can fire into it. Most squares stay the background colour. The mouse, the drift, and a released icon are what light a square up.

## Words this repo uses

| Word | Meaning |
| --- | --- |
| Canvas | The rectangle in the page that holds the picture. |
| Wasm | WebAssembly. The browser runs `public/engine.wasm` the way it runs JavaScript. |
| Cell | One square of the picture. The word is a grid of cells. |
| Appear | One way the letters fill in. The first is left to right. A click on the word plays the next one. |
| Quiet rectangle | A box over the nav and the sentence. The field does not paint through that type. |
| Demo | The page with the title, face, and hex boxes. The dev server is demo. A published site is demo only at `?demo`. |
| Production | The public page. No title box, no face menu, no hex boxes. The word is the hand-drawn `pesty.sh`. |

## Install

Node 20 or newer, and Rust with the browser target. Once per machine.

```bash
node --version
npm --version
rustc --version
rustup target add wasm32-unknown-unknown
```

If `node` or `npm` is missing, install Node from <https://nodejs.org> and open a new terminal. If `rustc` is missing, install Rust from <https://rustup.rs> and open a new terminal. The `wasm32-unknown-unknown` target is what the page build uses.

## Getting started

From this folder. Leave the last command running, then open the page.

```bash
npm install
npm test
npm run dev
```

1. `npm install` downloads the JavaScript packages. `node_modules/` is not committed.
2. `npm test` runs the Rust tests, builds `public/engine.wasm`, and checks that file. The last line should contain `pesty engine ok`.
3. `npm run dev` builds the wasm again and serves the page.

Open <http://127.0.0.1:43123>. You should see the word `pesty.sh` in pale pixels, a speckled field, and `live` at the top right. The port stays **43123** unless you set `PESTY_PORT` for one command.

The page reloads when you save a TypeScript or CSS file. A Rust change is picked up the next time you start `npm run dev`, because that command builds the wasm before it serves.

Move across the header. The cursor hoovers speckles from a wide radius and holds them. Leave and they fall back onto their own points. Hold on the field, then release, and an icon runs. Click the word, or tab to it and press Enter, and the next appear plays: print, drop, rise, bloom, spark, then print again. A ball of pixels wanders on its own. Every few seconds it fires one trail. That trail lands inside the canvas and bursts into a random icon.

On this dev server the boxes under the sentence are live. Title is the string drawn in the field. Face is Pesty, Plex Mono, Georgia, or Impact. Tint and Background are hex colours. `#d6ee6a` and `#12140f` are the original tint and ground. `?demo=0` is the public page, with those boxes gone. A published site opens them with `?demo`. A shared link can carry the strings: `?demo&text=hello&face=georgia&tint=ff6040&bg=ffffff`. The page paints that background while the wasm file is still loading.

If the browser asks for reduced motion, the letters are already finished, the wander does not fire, and a release drops no icon. The pointer still lights the cells under it.

Press Ctrl+C once in the dev terminal and wait until it prints `Port 43123 is free.` A second Ctrl+C force-stops it, then checks the port again. If that terminal is already gone, `npm run stop` frees a Vite process, or `scripts/dev.mjs`, whose working directory is this folder. A different program on 43123 is left running. Do not kill a pid you do not recognise.

## Config

Picture settings live in `engine/src/config.rs`. Edit that file, run `npm run build:wasm`, and reload. The page does not read the file on its own.

**Wander.** `DRIFT_FIRE` turns the trail on. The ball of pixels stays either way. `DRIFT_GAP_MIN_MS` and `DRIFT_GAP_MAX_MS` are the wait, five to eight seconds. `TRAIL_DOTS` and `TRAIL_SPREAD` shape the shot. `POP_EDGE` keeps the burst inside the canvas.

**Noise.** `NOISE_WELL_CELLS` is how far the cursor hoovers, in letter-cells. `NOISE_WELL_PULL` is how hard it packs those speckles onto itself. `NOISE_WELL_MASS` is how bright the clump can get. The points keep their own places. `NOISE_DENSITY` is how many speckles the field keeps.

**Pointer and icons.** `POINTER_FOLLOW` is how fast the glow catches the cursor. `CAST_RANDOM` rolls the icon and the play. `SLIDE_HOPS` and `SLIDE_SPIT` belong to a travelling click. `MAX_SPRITES` caps clicks and fired trails together.

**Page.** How often the page draws, and the pixel-ratio cap, are `FRAME_MS` and `DPR_CAP` at the top of `src/main.ts`. Colours are the six values in `engine/src/palette.rs`, repeated as `--bg`, `--lit`, `--mid`, and `--crest` in `src/style.css`. Letter shapes stay in `engine/mark.py`.

## Read the code

Read in this order. The comment at the top of each file is a short tutorial: the steps that file runs, then the settings you can change.

1. `engine/src/config.rs` — every knob: the wander and its trail, the hoover, the pointer, the icons.
2. `engine/src/paint.rs` — one picture, numbered in the order it is drawn.
3. `engine/src/palette.rs` — the six colours. `src/style.css` repeats four of them for the page around the canvas.
4. `engine/src/logo.rs` and `engine/src/mark.txt` — the letter grid. `#` is ink, `.` is empty.
5. `engine/src/appears.rs` — what a click on the word does. The list is the `ORDER` array.
6. `engine/src/sprites.rs` — the icons. A field release plays one. The wander fires a trail that bursts into another.
7. `engine/src/pointer.rs` — the mouse. A release on the word is a replay. A release on the field is a stamp.
8. `engine/src/motion.rs` — how long an appear takes (`REVEAL_MS`, 1100ms) and the path of the pixel ball.
9. `engine/src/field.rs` — the speckled ground, and the well that hoovers it.
10. `engine/src/lib.rs` — the `Engine` that calls the files above, then the functions the browser calls.
11. `src/engine.ts` — loads the wasm and copies the pixels onto the canvas.
12. `src/main.ts` — the page. It measures the canvas, forwards the mouse, fills the ground before the first frame, and in demo mode reads the four boxes.
13. `src/title.ts` — turns the title string into cells. Pesty uses `src/glyphs.json`. The other faces are drawn by the browser and read back.

`docs/engine.md` is the same map with the steps for adding an appear or an icon. `docs/release.md` is how to publish. `AGENTS.md` is the short note for an editor who already knows the layout.

A small first edit: in `engine/src/palette.rs`, change `BG`, and set `--bg` in `src/style.css` to the same hex. Stop the server, run `npm run dev`, and reload. The field ground and the page behind it should match.

The hand-drawn letters live in `engine/mark.py`. The rest of printable ASCII lives in `engine/alphabet.py`. Capitals use the lowercase glyph. After you edit a glyph:

```bash
python3 engine/mark.py
```

That rewrites `engine/src/mark.txt` and `src/glyphs.json`. Then run the tests. `logo.rs` checks that the baked word is 80 by 21 with 465 ink cells. If your edit changes that count, update the test it names.

## Publish

On a machine with Rust:

```bash
npm test
npm run build
```

Upload the contents of `dist/`. The rest, including how to preview that folder on port 43123, is in `docs/release.md`.

Vercel is a static demo. The wasm is already in the repo, and `npm run build` only bundles the page. Leave the Vite defaults. `vercel.json` sets install to `npm install`, build to `vite build`, and output to `dist`. If the project settings still show a rustup command, turn that override off and redeploy.

## If something fails

- `can't find crate` or `wasm32-unknown-unknown` is not installed. Run `rustup target add wasm32-unknown-unknown` and start again.
- `npm test` fails in Rust. Read the test name. The tests live next to the code they check.
- The terminal says port 43123 is in use by another program. Something else has the port. `npm run stop` will not kill it. Free it yourself, or run once with `PESTY_PORT` set to another port.
- The page says `failed`. Open the browser's console. The usual cause is a missing `engine.wasm`. Stop the server and run `npm run dev` again, and read the Rust error in that terminal.
- The word is a tiny blur in the corner, or the canvas is 300 by 150. The page did not size the canvas. `src/main.ts` has to set `canvas.width` and `canvas.height` to the on-screen size times the pixel ratio before it resizes the engine.

## Layout

```
engine/src/config.rs    every knob: wander, hoover, pointer, icons
engine/src/palette.rs   the six colours
engine/src/logo.rs      the letter grid and the light bands
engine/src/mark.txt     the baked pesty.sh bitmap
engine/mark.py          edit the hand-drawn letters here
engine/src/appears.rs   click sequence
engine/src/sprites.rs   icons, and the trail that bursts
engine/src/pointer.rs   mouse
engine/src/motion.rs    appear length, and the pixel ball
engine/src/field.rs     speckled ground, and the hoover
engine/src/paint.rs     one picture, top to bottom
engine/src/lib.rs       Engine, and the functions the page calls
src/main.ts             the page
src/engine.ts           load the wasm and copy pixels
src/title.ts            title string to cells
scripts/dev.mjs         dev server, and freeing the port on quit
scripts/stop.mjs        free port 43123 if the terminal is gone
```

This project is MIT. See `LICENSE`. What changed is in `CHANGELOG.md`.
