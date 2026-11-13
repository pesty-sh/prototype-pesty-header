// The page. It does not draw pixels. engine/src/paint.rs does that.
// src/engine.ts loads the wasm and copies the finished picture.
//
// Process, every frame (`draw`):
// 1. Measure the canvas in device pixels. CSS size times `dpr()`, and `dpr()`
//    is capped at `DPR_CAP`.
// 2. Set `canvas.width` and `canvas.height` to that size, then resize the
//    wasm buffer to the same size. Both have to match or the word is clipped.
//    A canvas stuck at 300×150 means this step did not run.
// 3. Send the word box, then the quiet boxes (nav, eyebrow, sentence).
// 4. Ask the engine for one frame, then copy the pixels.
//
// Pointer process (`boot`):
// - Move updates the glow.
// - Press on the field charges an icon. Release plays it.
// - Press on the word and release still on the word plays the next appear.
//   The click handler skips that case so the appear does not play twice.
//
// Config, just below the imports:
// - `DPR_CAP` is 2. Higher is sharper and heavier.
// - A frame is drawn about every `FRAME_MS` (28), or every `REDUCED_FRAME_MS`
//   (80) when reduced motion is on.
// - Quiet boxes grow by `QUIET_PAD_CSS` (12) CSS pixels so the speckle stays off the type.
// - Demo is on for this dev server. `?demo=0` hides the title, face, and hex
//   boxes. On a published site those boxes appear only with `?demo`.

import "@fontsource/ibm-plex-mono/400.css";
import "@fontsource/ibm-plex-mono/500.css";
import "./style.css";
import { Field } from "./engine";
import {
  DEFAULT_BG,
  DEFAULT_TINT,
  hexLooksOpen,
  hexWhileTyping,
  isDemo,
  luma,
  mixHex,
  normalizeHex,
  readableAccent,
} from "./live";
import { cellsFor, faceById, FACES } from "./title";

function required<T>(value: T | null | undefined, name: string): T {
  if (value == null) throw new Error(`${name} is missing`);
  return value;
}

const hero = required(document.querySelector<HTMLElement>("#home"), "hero");
const canvas = required(document.querySelector<HTMLCanvasElement>("#field"), "canvas");
const statusEl = required(document.querySelector<HTMLElement>("#status"), "status");
const markSlot = required(document.querySelector<HTMLElement>("#mark-slot"), "wordmark");
const mark = required(document.querySelector<HTMLButtonElement>("#mark"), "wordmark button");
const nav = required(document.querySelector<HTMLElement>("#nav"), "nav");
const eyebrow = required(document.querySelector<HTMLElement>("#eyebrow"), "eyebrow");
const copy = required(document.querySelector<HTMLElement>("#copy"), "copy");
const wordLabel = required(document.querySelector<HTMLElement>("#mark .sr-only"), "word label");
const themeColor = document.querySelector<HTMLMetaElement>('meta[name="theme-color"]');
const demo = isDemo(location.search, import.meta.env.DEV);
const live = document.querySelector<HTMLFormElement>("#live");
if (!demo) live?.remove();
else live?.removeAttribute("hidden");
const pointerEl = document.querySelector<HTMLElement>("#m-pointer");
const printEl = document.querySelector<HTMLElement>("#m-print");
const stampsEl = document.querySelector<HTMLElement>("#m-stamps");
const cellEl = document.querySelector<HTMLElement>("#m-cell");
const ctx = required(canvas.getContext("2d", { alpha: false }), "canvas context");
ctx.imageSmoothingEnabled = false;

// The canvas backing store starts opaque black. Paint the field colour
// into it, and onto the page, before wasm arrives. `?bg` wins over the
// baked ground so a white field does not flash black while loading.
let ground = normalizeHex(new URLSearchParams(location.search).get("bg") ?? "") ?? DEFAULT_BG;

function paintField(color: string) {
  ctx.fillStyle = color;
  ctx.fillRect(0, 0, canvas.width, canvas.height);
}

function applyGround(color: string) {
  ground = color;
  const light = luma(color) > 0.62;
  const root = document.documentElement;
  root.style.setProperty("--bg", color);
  root.style.backgroundColor = color;
  root.style.colorScheme = light ? "light" : "dark";
  if (light) {
    root.style.setProperty("--text", "#1a1c14");
    root.style.setProperty("--muted", "#4d5244");
    root.style.setProperty("--line", "#d5d3c6");
  } else {
    root.style.removeProperty("--text");
    root.style.removeProperty("--muted");
    root.style.removeProperty("--line");
  }
  hero.style.backgroundColor = color;
  hero.classList.toggle("is-light", light);
  canvas.style.backgroundColor = color;
  themeColor?.setAttribute("content", color);
  document
    .querySelector('meta[name="color-scheme"]')
    ?.setAttribute("content", light ? "light" : "dark");
  paintField(color);
}

applyGround(ground);

// Picture knobs (the self-moving sprite, pops, clock, glow) are in
// engine/src/config.rs. Rebuild the wasm after editing that file.
// Bump this when that wasm changes so a reload skips a cached copy.
const WASM_REV = "0.1.7.1";

// Device pixel ratio cap. 2 stays sharp without a 3× buffer.
const DPR_CAP = 2;
// How often a frame is drawn, in milliseconds.
const FRAME_MS = 28;
const REDUCED_FRAME_MS = 80;
// Extra CSS pixels around the nav and the sentence. The field does not paint there.
const QUIET_PAD_CSS = 12;

const reduceQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
let reduced = reduceQuery.matches;
reduceQuery.addEventListener("change", () => {
  reduced = reduceQuery.matches;
});

const quietEls = [nav, eyebrow, copy];
let pointerLabel = "away";
let sawPointerReplay = false;
let running = true;
let raf = 0;
let lastDraw = 0;
let lastHud = 0;

function dpr(): number {
  return Math.min(window.devicePixelRatio || 1, DPR_CAP);
}

function boxOf(el: Element, origin: DOMRect, scale: number) {
  const rect = el.getBoundingClientRect();
  return {
    x: (rect.left - origin.left) * scale,
    y: (rect.top - origin.top) * scale,
    w: rect.width * scale,
    h: rect.height * scale,
  };
}

function local(event: PointerEvent) {
  const origin = canvas.getBoundingClientRect();
  const scale = dpr();
  const inside =
    event.clientX >= origin.left &&
    event.clientX <= origin.right &&
    event.clientY >= origin.top &&
    event.clientY <= origin.bottom;
  return {
    x: (event.clientX - origin.left) * scale,
    y: (event.clientY - origin.top) * scale,
    inside,
    cssX: Math.round(event.clientX - origin.left),
    cssY: Math.round(event.clientY - origin.top),
  };
}

function draw(field: Field, now: number) {
  // 1. Device-pixel size. CSS box times dpr, capped at DPR_CAP.
  const origin = canvas.getBoundingClientRect();
  const scale = dpr();
  const width = Math.max(1, Math.round(origin.width * scale));
  const height = Math.max(1, Math.round(origin.height * scale));
  // 2. The canvas and the wasm buffer have to be the same size.
  if (canvas.width !== width || canvas.height !== height) {
    // Resizing clears the bitmap to black. Put the ground back before the
    // frame is copied, so a white field never shows that clear.
    canvas.width = width;
    canvas.height = height;
    ctx.imageSmoothingEnabled = false;
    paintField(ground);
    field.resize(width, height);
  }
  // 3. Word box, then the quiet boxes over the type.
  const markBox = boxOf(markSlot, origin, scale);
  field.clearQuiet();
  const pad = QUIET_PAD_CSS * scale;
  for (const el of quietEls) {
    const box = boxOf(el, origin, scale);
    field.pushQuiet(box.x - pad, box.y - pad, box.w + pad * 2, box.h + pad * 2);
  }
  // 4. One frame, then copy the pixels. The copy has to happen after frame.
  field.frame(now, markBox.x, markBox.y, markBox.w, markBox.h, reduced);
  field.blit(ctx);
  if (now - lastHud > 180) {
    lastHud = now;
    const reveal = field.reveal();
    if (printEl) {
      const appear = field.appear();
      printEl.textContent = reveal >= 0.999 ? appear : `${appear} ${Math.round(reveal * 100)}%`;
    }
    if (stampsEl) stampsEl.textContent = String(field.ripples());
    if (cellEl) cellEl.textContent = `${(field.cell() / scale).toFixed(1)} px`;
    if (pointerEl) pointerEl.textContent = pointerLabel;
  }
}

function loop(field: Field, now: number) {
  raf = 0;
  if (!running) return;
  if (now - lastDraw >= (reduced ? REDUCED_FRAME_MS : FRAME_MS)) {
    lastDraw = now;
    draw(field, now);
  }
  raf = requestAnimationFrame((t) => loop(field, t));
}

function boot(field: Field) {
  if (demo) wireDemo(field);

  markSlot.style.aspectRatio = `${field.cols} / ${field.rows}`;
  statusEl.textContent = "live";

  window.addEventListener("pointermove", (event) => {
    const point = local(event);
    field.pointer(point.x, point.y, point.inside);
    pointerLabel = point.inside ? `${point.cssX}  ${point.cssY}` : "away";
  });

  window.addEventListener("pointerdown", (event) => {
    if (event.button !== 0) return;
    const target = event.target;
    if (target instanceof Element && target.closest("a, input, textarea, select, label")) return;
    const point = local(event);
    if (!point.inside) return;
    field.pointerDown(point.x, point.y, event.timeStamp);
  });

  window.addEventListener("pointerup", (event) => {
    const point = local(event);
    sawPointerReplay = field.pointerUp(point.x, point.y, event.timeStamp);
  });

  window.addEventListener("pointercancel", () => {
    field.pointerCancel();
  });

  mark.addEventListener("click", () => {
    if (sawPointerReplay) {
      sawPointerReplay = false;
      return;
    }
    field.replay(performance.now());
  });

  const observer = new IntersectionObserver(
    ([entry]) => {
      running = entry.isIntersecting;
      if (running && raf === 0) raf = requestAnimationFrame((t) => loop(field, t));
    },
    { rootMargin: "80px" },
  );
  observer.observe(hero);

  draw(field, performance.now());
  hero.classList.add("is-ready", "is-live");
  raf = requestAnimationFrame((t) => loop(field, t));
}

function wireDemo(field: Field) {
  const titleInput = required(document.querySelector<HTMLInputElement>("#title"), "title");
  const faceSelect = required(document.querySelector<HTMLSelectElement>("#face"), "face");
  const tintInput = required(document.querySelector<HTMLInputElement>("#tint"), "tint");
  const bgInput = required(document.querySelector<HTMLInputElement>("#bg"), "background");
  const params = new URLSearchParams(location.search);
  const explicitDemo = params.has("demo") && params.get("demo") !== "0" && params.get("demo") !== "false";
  const asked = params.get("text");
  const askedFace = params.get("face");
  if (asked) titleInput.value = asked.slice(0, 24);
  if (askedFace && FACES.some((face) => face.id === askedFace)) faceSelect.value = askedFace;
  const askedTint = normalizeHex(params.get("tint") ?? "");
  const askedBg = normalizeHex(params.get("bg") ?? "");
  if (askedTint) tintInput.value = askedTint;
  if (askedBg) bgInput.value = askedBg;

  let tintHex = askedTint ?? DEFAULT_TINT;
  let bgHex = askedBg ?? DEFAULT_BG;
  let drawnKey = "pesty\npesty.sh";
  paintChrome(bgHex, tintHex);

  let ticket = 0;
  let queryTimer = 0;

  const pushHex = (input: HTMLInputElement, kind: number): string | null => {
    const typed = hexWhileTyping(input.value);
    input.setAttribute("aria-invalid", input.value.trim() !== "" && !hexLooksOpen(input.value) && !typed ? "true" : "false");
    if (!typed) return null;
    const raw = input.value.trim();
    const digits = raw.replace(/^#/, "");
    const overflow = /^[0-9a-fA-F]+$/.test(digits) && digits.length > 6;
    const bare = /^[0-9a-fA-F]{6}$/.test(digits) && !raw.startsWith("#");
    if (overflow || bare) {
      input.value = typed;
      input.setSelectionRange(typed.length, typed.length);
    }
    if (!field.setHex(kind, typed)) return null;
    return typed;
  };

  const applyColors = () => {
    const tint = pushHex(tintInput, 0);
    const bg = pushHex(bgInput, 1);
    if (tint) tintHex = tint;
    if (bg) bgHex = bg;
    paintChrome(bgHex, tintHex);
    if (tint || bg) draw(field, performance.now());
  };

  const commitQuery = () => {
    window.clearTimeout(queryTimer);
    const face = faceSelect.value;
    const trimmed = titleInput.value.trim();
    const baked = face === "pesty" && trimmed.toLowerCase() === "pesty.sh";
    const tintDefault = tintHex.toLowerCase() === DEFAULT_TINT;
    const bgDefault = bgHex.toLowerCase() === DEFAULT_BG;
    const next = new URLSearchParams(location.search);
    if (baked && tintDefault && bgDefault) {
      next.delete("text");
      next.delete("face");
      next.delete("tint");
      next.delete("bg");
      if (explicitDemo) next.set("demo", "1");
      else next.delete("demo");
    } else {
      next.set("demo", "1");
      if (baked) {
        next.delete("text");
        next.delete("face");
      } else {
        next.set("text", trimmed);
        next.set("face", face);
      }
      if (tintDefault) next.delete("tint");
      else next.set("tint", tintHex.slice(1));
      if (bgDefault) next.delete("bg");
      else next.set("bg", bgHex.slice(1));
    }
    const query = next.toString();
    const path = query ? `${location.pathname}?${query}` : location.pathname;
    if (`${location.pathname}${location.search}` !== path) {
      history.replaceState(null, "", path);
    }
  };

  const scheduleQuery = () => {
    window.clearTimeout(queryTimer);
    queryTimer = window.setTimeout(commitQuery, 320);
  };

  const applyLive = async () => {
    applyColors();
    const mine = ++ticket;
    const face = faceSelect.value;
    const text = titleInput.value;
    if (face !== "pesty") await document.fonts.ready;
    if (mine !== ticket) return;
    const trimmed = text.trim();
    const key = `${face}\n${trimmed}`;
    const baked = face === "pesty" && trimmed.toLowerCase() === "pesty.sh";
    if (key !== drawnKey) {
      if (!baked) {
        const drawn = cellsFor(text, face);
        if (!drawn || !field.setMark(drawn.cells, drawn.cols, drawn.rows)) return;
      } else {
        field.resetMark();
      }
      drawnKey = key;
      markSlot.style.aspectRatio = `${field.cols} / ${field.rows}`;
      wordLabel.textContent = `Play the next appear for ${trimmed || "pesty.sh"}`;
      titleInput.style.fontFamily = faceById(face).family;
    }
    scheduleQuery();
  };

  const settleHex = (input: HTMLInputElement, current: string) => {
    const body = input.value.trim().replace(/^#/, "");
    if (/^[0-9a-fA-F]{3}$/.test(body) || /^[0-9a-fA-F]{6}$/.test(body)) {
      input.value = normalizeHex(input.value) ?? current;
      return;
    }
    input.value = current;
  };

  const selectOnFocus = (input: HTMLInputElement) => {
    let armed = false;
    input.addEventListener("focus", () => {
      armed = true;
    });
    input.addEventListener("mouseup", (event) => {
      if (!armed) return;
      armed = false;
      event.preventDefault();
      input.select();
    });
  };

  titleInput.addEventListener("input", () => {
    void applyLive();
  });
  faceSelect.addEventListener("change", () => {
    void applyLive();
  });
  tintInput.addEventListener("input", () => {
    void applyLive();
  });
  bgInput.addEventListener("input", () => {
    void applyLive();
  });
  tintInput.addEventListener("blur", () => {
    settleHex(tintInput, tintHex);
    applyColors();
    commitQuery();
  });
  bgInput.addEventListener("blur", () => {
    settleHex(bgInput, bgHex);
    applyColors();
    commitQuery();
  });
  selectOnFocus(tintInput);
  selectOnFocus(bgInput);

  if (asked || (askedFace && askedFace !== "pesty") || askedTint || askedBg) void applyLive();
}

function paintAccent(el: HTMLElement, tint: string, against: string) {
  const accent = readableAccent(tint, against);
  el.style.setProperty("--lit", accent);
  el.style.setProperty("--mid", mixHex(accent, against, 0.45));
  el.style.setProperty("--on-lit", luma(accent) > 0.62 ? "#1a1c14" : "#f7ffd4");
}

function paintChrome(bg: string, tint: string) {
  paintAccent(document.documentElement, tint, DEFAULT_BG);
  paintAccent(hero, tint, bg);
  applyGround(bg);
}

Field.load(`/engine.wasm?v=${WASM_REV}`)
  .then(boot)
  .catch((error: unknown) => {
    hero.classList.add("is-error");
    statusEl.textContent = "failed";
    console.error(error);
  });
