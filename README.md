# pesty.sh

A web page with a pixel header. The letters, the mouse glow, and the little icons are drawn by a Rust program that runs in the browser.

You edit the Rust and the TypeScript. A command turns the Rust into a file named `engine.wasm`. The page loads that file and copies each finished picture onto a canvas. Do not edit `engine.wasm`. It is generated, and it is not committed.

If you only want the page running, follow [First run](#first-run) and stop there. If you need to change the picture, follow [Read the code](#read-the-code) after the page is up.

## What the picture is doing

Three jobs, and only these three:

- **Pointer.** Where the mouse is, how long a press has been held, and whether it landed on the letters.
- **Motion.** How the letters fill in, and a slow drift in the background when the mouse is away.
- **Logo.** The word `pesty.sh`, lit from bright at the top of the letters to dark at the tails.

The speckled background is not a fourth system. It is the ground those three jobs sit on. Most squares stay the background colour. The mouse, the drift, and a released icon are what light a square up.

Icons do not enter the screen on their own. Press on the speckled field and an icon charges under the pointer. Release, and that icon appears, slides, or expands. A slide spins across the header, drops smaller icons as it moves, and bursts each time it lands. Press on the word and the next letter animation plays instead. No icon is dropped.

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

## Install, once

You need Node and Rust. In a terminal:

```bash
node --version
npm --version
rustc --version
```

Node should be 20 or newer. If `node` or `npm` is missing, install Node from <https://nodejs.org> and open a new terminal. If `rustc` is missing, install Rust from <https://rustup.rs> and open a new terminal.

Rust needs one extra target, the one the browser build uses. Once per machine:

```bash
rustup target add wasm32-unknown-unknown
```

## First run

From this folder:

```bash
npm install
npm test
npm run dev
```

What those do:

1. `npm install` downloads the JavaScript packages into `node_modules/`. That folder is not committed.
2. `npm test` runs the Rust tests, builds `public/engine.wasm`, and asks that file a few questions. The last line should contain `pesty engine ok`.
3. `npm run dev` builds the wasm again and serves the page.

Leave that terminal open. Open <http://127.0.0.1:43123>.

You should see the word `pesty.sh` in pale green pixels, a dark speckled field, and the word `live` at the top right. The port is always **43123** unless you set `PESTY_PORT` for one command.

The page reloads when you save a TypeScript or CSS file. A Rust change is picked up the next time you start `npm run dev`, because that command builds the wasm before it serves.

## Use the page

- Move across the header. Nearby squares and the letters brighten.
- Hold on the speckled field, then release. An icon runs.
- Click the word, or tab to it and press Enter. The next appear plays. The order is print, drop, rise, bloom, spark, then print again.
- On this dev server the boxes under the sentence are live. Title is the string drawn in the field. Face is Pesty, Plex Mono, Georgia, or Impact. Tint and Background are hex colours such as `#ff0000` or `ff0000`. Six digits update the picture. `#d6ee6a` and `#12140f` are the original tint and ground.
- Add `?demo=0` to the address to see the public page, with those boxes gone.
- A published site hides the boxes until the address has `?demo`. A link can carry the strings: `?demo&text=hello&face=georgia&tint=ff6040&bg=101820`.

If the browser is set to reduced motion, the letters are already finished, the field does not drift, and a release drops no icon. The mouse still lights the squares under it.

## Stop the server

In the terminal where `npm run dev` is running, press Ctrl+C once. Wait until it prints:

```text
Port 43123 is free.
```

The next `npm run dev` uses 43123 again. A second Ctrl+C force-stops it, then checks the port again.

If that terminal is already gone:

```bash
npm run stop
```

That stops a Vite process, or `scripts/dev.mjs`, whose working directory is this folder. A different program already listening on 43123 is left running, and the command prints its pid. Do not kill a pid you do not recognise.

## Read the code

Read in this order. The comment at the top of each file is a short tutorial: the steps that file runs, then the settings you can change.

1. `engine/src/paint.rs` — one picture, numbered in the order it is drawn. Start here.
2. `engine/src/palette.rs` — the six colours. `src/style.css` repeats four of them for the page around the canvas.
3. `engine/src/logo.rs` and `engine/src/mark.txt` — the letter grid. `#` is ink, `.` is empty.
4. `engine/src/appears.rs` — what a click on the word does. The list is the `ORDER` array.
5. `engine/src/sprites.rs` — the icons. A field release plays one. The sprite that moves on its own pops more. The knobs are in `engine/src/config.rs`.
6. `engine/src/pointer.rs` — the mouse. A release on the word is a replay. A release on the field is a stamp.
7. `engine/src/motion.rs` — how long an appear takes (`REVEAL_MS`, 1100ms) and the slow drift.
8. `engine/src/field.rs` — the speckled ground.
9. `engine/src/lib.rs` — the `Engine` that calls the files above, then the functions the browser calls.
10. `src/engine.ts` — loads the wasm and copies the pixels onto the canvas.
11. `src/main.ts` — the page. It measures the canvas, forwards the mouse, and in demo mode reads the four boxes.
12. `src/title.ts` — turns the title string into cells. Pesty uses `src/glyphs.json`. The other faces are drawn by the browser and read back.

`docs/engine.md` is the same map with the steps for adding an appear or an icon. `docs/release.md` is how to publish. `AGENTS.md` is the short note for an editor who already knows the layout.

A small first edit: in `engine/src/palette.rs`, change `BG`, and set `--bg` in `src/style.css` to the same hex. Stop the server, run `npm run dev`, and reload. The field ground and the page behind it should match.

The hand-drawn letters live in `engine/mark.py`. The rest of printable ASCII lives in `engine/alphabet.py`. Capitals use the lowercase glyph. After you edit a glyph:

```bash
python3 engine/mark.py
```

That rewrites `engine/src/mark.txt` and `src/glyphs.json`. Then run the tests. `logo.rs` checks that the baked word is 80 by 21 with 465 ink cells. If your edit changes that count, update the test it names.

## Publish

```bash
npm test
npm run build
```

Upload the contents of `dist/`. The rest, including how to preview that folder on port 43123, is in `docs/release.md`.

## If something fails

- `can't find crate` or `wasm32-unknown-unknown` is not installed. Run `rustup target add wasm32-unknown-unknown` and start again.
- `npm test` fails in Rust. Read the test name. The tests live next to the code they check.
- The terminal says port 43123 is in use by another program. Something else has the port. `npm run stop` will not kill it. Free it yourself, or run once with `PESTY_PORT` set to another port.
- The page says `failed`. Open the browser's console. The usual cause is a missing `engine.wasm`. Stop the server and run `npm run dev` again, and read the Rust error in that terminal.
- The word is a tiny blur in the corner, or the canvas is 300 by 150. The page did not size the canvas. `src/main.ts` has to set `canvas.width` and `canvas.height` to the on-screen size times the pixel ratio before it resizes the engine.

## Layout

```
engine/src/palette.rs   the six colours
engine/src/logo.rs      the letter grid and the light bands
engine/src/mark.txt     the baked pesty.sh bitmap
engine/mark.py          edit the hand-drawn letters here
engine/src/appears.rs   click sequence
engine/src/sprites.rs   icons, fired only by a field release
engine/src/pointer.rs   mouse
engine/src/motion.rs    appear length, drift
engine/src/field.rs     speckled ground
engine/src/paint.rs     one picture, top to bottom
engine/src/lib.rs       Engine, and the functions the page calls
src/main.ts             the page
src/engine.ts           load the wasm and copy pixels
src/title.ts            title string to cells
scripts/dev.mjs         dev server, and freeing the port on quit
scripts/stop.mjs        free port 43123 if the terminal is gone
```

This project is MIT. See `LICENSE`. What changed is in `CHANGELOG.md`.
