# Publish the site

The public site is a static folder. The browser loads one HTML page, one wasm module, and the font. Nothing in this project runs on a server after the build.

## Build

From a clean checkout:

```bash
rustup target add wasm32-unknown-unknown
npm install
npm test
npm run build
```

`npm test` runs the Rust tests, builds `public/engine.wasm`, and checks that module. `npm run build` builds the wasm again and writes the site to `dist/`.

Upload the contents of `dist/`. Do not upload `node_modules`, `engine/target`, or a wasm you edited by hand. The Rust in `engine/src` is the source. The build compiles it.

`dist/` is gitignored. Generate it on the machine that publishes. `public/engine.wasm` is committed so a host without Rust can still publish.

## Demo host

The demo site does not install Rust. Vite copies `public/engine.wasm` into `dist/`.

On Vercel, import the repo as Vite:

| Setting | Value |
| --- | --- |
| Framework | Vite |
| Root Directory | `./` |
| Install Command | `npm install` |
| Build Command | `vite build` |
| Output Directory | `dist` |

No environment variables. `vercel.json` sets the install command, the build command, and the output directory. If the project settings still show a rustup install, replace that command with `npm install` and set the build command to `vite build`.

`npm run build` compiles the wasm before Vite runs, so a host without Rust cannot use it.

## Look at the build

The dev server on port 43123 is the working copy. The built site uses the same port, so stop the dev server first.

```bash
npm run stop
npm run preview
```

Open <http://127.0.0.1:43123>. Ctrl+C prints `Port 43123 is free.` Start `npm run dev` again when you want the working copy.

## What the host has to do

Serve the files from the domain root. A request for `/engine.wasm` must return the file in `dist/engine.wasm`. The page asks for that path.

If the site lives under a subpath, set `base` in `vite.config.ts` to that path and rebuild.

Serve `engine.wasm` as `application/wasm`. `public/_headers` sets that for Netlify and Cloudflare Pages, and the build copies it into `dist/`. If a host sends a different type, the page falls back from streaming compile to a buffered compile. Both paths are in `src/engine.ts`.

No environment variables. No database. No accounts. The palette and the default wordmark are in the wasm and the CSS that the build already bundled.

The public address has no title, face, or colour boxes. The live page is the same build with `demo` on the query. Hand someone that link, or add the strings:

```text
https://example.com/?demo&text=hello&face=georgia&tint=ff6040&bg=101820
```

`face` is `pesty`, `plex`, `georgia`, or `impact`. `tint` and `bg` are hex strings without a leading `#`. The dev server is already demo. Add `?demo=0` there to see the public page.

## After a change

Edit the Rust, then run the same two commands:

```bash
npm test
npm run build
```

Upload the new `dist/`. A change to a click appear or an icon is a Rust change. The map of those files is `docs/engine.md`.
