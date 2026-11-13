//! The engine. Read this after the page is running (README, "First run").
//!
//! The picture has three jobs: the pointer, the motion, and the word.
//! This file holds them and talks to the page. It does not choose colours,
//! letter shapes, or icons. Those live in the files below.
//!
//! Read in this order. Each file's top comment is the tutorial for that step,
//! with the settings you can change:
//!
//! 1. `palette.rs` — the six colours.
//! 2. `logo.rs` — the letter grid, and which rows are bright.
//! 3. `appears.rs` — what a click on the word does. The list is `ORDER`.
//! 4. `sprites.rs` — the icons. They run only after a release on the field.
//! 5. `pointer.rs` — the mouse.
//! 6. `motion.rs` — how long a letter fill takes (`REVEAL_MS`), and the drift.
//! 7. `field.rs` — the speckled background.
//! 8. `paint.rs` — one picture, numbered in the order it is drawn.
//!    Start there if you want to see a frame happen.
//!
//! Process, once per frame. The page calls `frame`:
//! 1. Store the time, the word box, and whether reduced motion is on.
//! 2. On the first frame, start the letter clock. Reduced motion starts it
//!    already finished.
//! 3. Call `paint`, which fills `pixels`.
//!
//! The page copies `pixels` after that call. It never draws the field itself.
//!
//! Config that lives in this file:
//! - A live title must be 1–160 columns and 8–48 rows. Anything else is refused.
//! - At most 5 released icons stay alive. The oldest is dropped.
//! - Tint `#d6ee6a` and background `#12140f` clear the demo override and
//!   return to the colours in `palette.rs`. Kind `0` is the tint. Kind `1`
//!   is the background.
//!
//! An icon is created in one place only: `spawn_sprite`, and only after a
//! release on the field. A release on the word calls `replay` instead.

mod appears;
mod field;
mod logo;
mod motion;
mod paint;
mod palette;
mod pointer;
mod sprites;

use pointer::{Pointer, Rect, Release};
use sprites::Sprite;

/// How many released icons can be alive at once.
const MAX_SPRITES: usize = 5;

pub struct Engine {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    noise: field::Noise,
    pointer: Pointer,
    /// Icon and play chosen at press, so the charge matches the release.
    pending: sprites::Pick,
    sprites: Vec<Sprite>,
    /// Index into [`appears::ORDER`]. The first frame plays entry 0.
    appear_ix: usize,
    quiet: Vec<Rect>,
    grid: logo::Grid,
    /// Bytes the page fills before [`Engine::commit_mark`].
    scratch: Vec<u8>,
    scratch_cols: i32,
    scratch_rows: i32,
    /// Recolour of the baked ramp. `None` keeps the six colours.
    tint: Option<[u8; 4]>,
    /// Ground that failed the dither test. `None` keeps [`palette::BG`].
    background: Option<[u8; 4]>,
    /// Bytes the page fills before [`Engine::commit_hex`].
    hex: Vec<u8>,
    mark: Rect,
    cell_w: f32,
    cell_h: f32,
    now: f64,
    reveal_from: f64,
    reveal: f32,
    reduced: bool,
    booted: bool,
}

impl Engine {
    pub fn new() -> Self {
        const {
            assert!(sprites::glyphs_ok());
        }
        Self {
            pixels: Vec::new(),
            width: 0,
            height: 0,
            noise: field::Noise::new(0x0E57_5A11),
            pointer: Pointer::new(),
            pending: sprites::Pick::first(),
            sprites: Vec::new(),
            appear_ix: 0,
            quiet: Vec::new(),
            grid: logo::Grid::baked(),
            scratch: Vec::new(),
            scratch_cols: 0,
            scratch_rows: 0,
            tint: None,
            background: None,
            hex: vec![0; 16],
            mark: Rect {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
            },
            cell_w: 0.0,
            cell_h: 0.0,
            now: 0.0,
            reveal_from: 0.0,
            reveal: 0.0,
            reduced: false,
            booted: false,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let width = width.min(4096);
        let height = height.min(4096);
        if width == self.width && height == self.height {
            return;
        }
        self.width = width;
        self.height = height;
        self.pixels
            .resize((width as usize) * (height as usize) * 4, 0);
    }

    pub fn clear_quiet(&mut self) {
        self.quiet.clear();
    }

    pub fn push_quiet(&mut self, x: f32, y: f32, w: f32, h: f32) {
        if w > 0.0 && h > 0.0 {
            self.quiet.push(Rect { x, y, w, h });
        }
    }

    pub fn pointer(&mut self, x: f32, y: f32, inside: bool) {
        self.pointer.place(x, y, inside);
    }

    pub fn pointer_down(&mut self, x: f32, y: f32, now: f64) {
        // The roll is fixed for this press. Paint shows it while the finger is down.
        self.pending = sprites::pick(now, x, y);
        self.pointer.press(x, y, now, self.hit(x, y));
    }

    /// `true` when the release landed on the wordmark and the print restarted.
    pub fn pointer_up(&mut self, x: f32, y: f32, now: f64) -> bool {
        match self.pointer.release(x, y, now, self.hit(x, y)) {
            Release::None => false,
            Release::Replay => {
                self.replay(now);
                true
            }
            Release::Stamp { x, y, held_ms } => {
                if !self.reduced {
                    self.spawn_sprite(x, y, now, held_ms);
                }
                false
            }
        }
    }

    pub fn pointer_cancel(&mut self) {
        self.pointer.cancel();
    }

    /// Advance to the next appear and start its clock.
    /// [`appears::CYCLE`] decides whether that is the next name or a roll.
    pub fn replay(&mut self, now: f64) {
        if self.reduced {
            return;
        }
        self.appear_ix = appears::next_index(self.appear_ix, now);
        self.reveal_from = now;
        self.now = now;
    }

    pub fn hit(&self, x: f32, y: f32) -> bool {
        logo::in_rect(x, y, self.mark.x, self.mark.y, self.mark.w, self.mark.h)
    }

    pub fn frame(
        &mut self,
        now: f64,
        mark_x: f32,
        mark_y: f32,
        mark_w: f32,
        mark_h: f32,
        reduced: bool,
    ) {
        self.now = now;
        self.reduced = reduced;
        self.mark = Rect {
            x: mark_x,
            y: mark_y,
            w: mark_w,
            h: mark_h,
        };
        if !self.booted {
            self.booted = true;
            self.reveal_from = if reduced {
                now - motion::REVEAL_MS
            } else {
                now
            };
        }
        paint::paint(self);
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Live icons. The wasm export keeps the name `engine_ripples`.
    pub fn ripple_count(&self) -> u32 {
        self.sprites.len() as u32
    }

    pub fn appear_index(&self) -> u32 {
        self.appear_ix as u32
    }

    pub fn appear_name(&self) -> &'static str {
        appears::name_at(self.appear_ix)
    }

    pub fn reveal(&self) -> f32 {
        self.reveal
    }

    pub fn cell(&self) -> f32 {
        (self.cell_w + self.cell_h) * 0.5
    }

    pub fn logo_cols(&self) -> i32 {
        self.grid.cols
    }

    pub fn logo_rows(&self) -> i32 {
        self.grid.rows
    }

    /// Replace the baked word with `cells`, one byte per cell, `1` for ink.
    /// Starts the appear again when the engine is already running.
    pub fn set_mark(&mut self, cols: i32, rows: i32, cells: &[u8]) -> bool {
        if !(1..=160).contains(&cols) || !(8..=48).contains(&rows) {
            return false;
        }
        if cells.len() != (cols as usize) * (rows as usize) {
            return false;
        }
        self.grid = logo::Grid::from_cells(cols, rows, cells.to_vec());
        self.restart_appear();
        true
    }

    /// Hand the page a buffer of `cols * rows` bytes. `0` means the size was refused.
    pub fn prepare_mark(&mut self, cols: i32, rows: i32) -> *mut u8 {
        if !(1..=160).contains(&cols) || !(8..=48).contains(&rows) {
            return std::ptr::null_mut();
        }
        let n = (cols as usize) * (rows as usize);
        self.scratch_cols = cols;
        self.scratch_rows = rows;
        self.scratch.resize(n, 0);
        self.scratch.as_mut_ptr()
    }

    pub fn commit_mark(&mut self) {
        let cols = self.scratch_cols;
        let rows = self.scratch_rows;
        let cells = std::mem::take(&mut self.scratch);
        self.set_mark(cols, rows, &cells);
    }

    pub fn reset_mark(&mut self) {
        self.grid = logo::Grid::baked();
        self.scratch.clear();
        self.scratch_cols = 0;
        self.scratch_rows = 0;
        self.restart_appear();
    }

    /// Hand the page 16 bytes for a hex string. The page writes, then commits.
    pub fn prepare_hex(&mut self) -> *mut u8 {
        if self.hex.len() < 16 {
            self.hex.resize(16, 0);
        }
        self.hex.as_mut_ptr()
    }

    /// `kind` 0 is the text tint, 1 is the background. `len` is the byte count written.
    pub fn commit_hex(&mut self, kind: u32, len: u32) -> bool {
        let n = (len as usize).min(self.hex.len());
        let mut buf = [0u8; 16];
        buf[..n].copy_from_slice(&self.hex[..n]);
        let Ok(text) = std::str::from_utf8(&buf[..n]) else {
            return false;
        };
        let text = text.trim().to_owned();
        self.apply_hex(kind, &text)
    }

    /// Tint or background from a hex string. Empty, or the baked hex, clears the override.
    pub fn set_tint_hex(&mut self, text: &str) -> bool {
        self.apply_hex(0, text)
    }

    pub fn set_background_hex(&mut self, text: &str) -> bool {
        self.apply_hex(1, text)
    }

    fn apply_hex(&mut self, kind: u32, text: &str) -> bool {
        let text = text.trim();
        if text.is_empty() {
            return self.clear_hex(kind);
        }
        let Some(color) = palette::parse_hex(text) else {
            return false;
        };
        match kind {
            0 => {
                self.tint = if color == palette::LIT {
                    None
                } else {
                    Some(color)
                };
                true
            }
            1 => {
                self.background = if color == palette::BG {
                    None
                } else {
                    Some(color)
                };
                true
            }
            _ => false,
        }
    }

    fn clear_hex(&mut self, kind: u32) -> bool {
        match kind {
            0 => self.tint = None,
            1 => self.background = None,
            _ => return false,
        }
        true
    }

    fn restart_appear(&mut self) {
        if self.booted && !self.reduced {
            self.reveal_from = self.now;
        }
    }

    /// The only icon spawn. Called from a field release, never from the clock.
    fn spawn_sprite(&mut self, x: f32, y: f32, now: f64, held_ms: f64) {
        let cell = (self.cell_w + self.cell_h).max(2.0) * 0.5;
        // Five is the cap. The oldest icon is dropped so a long hold-release
        // session cannot fill the header.
        if self.sprites.len() >= MAX_SPRITES {
            self.sprites.remove(0);
        }
        self.sprites
            .push(Sprite::spawn(x, y, now, held_ms, cell, self.pending));
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_arch = "wasm32")]
mod ffi {
    use super::Engine;

    fn engine(ptr: u32) -> &'static mut Engine {
        unsafe { &mut *(ptr as usize as *mut Engine) }
    }

    #[no_mangle]
    pub extern "C" fn engine_new() -> u32 {
        Box::into_raw(Box::new(Engine::new())) as usize as u32
    }

    #[no_mangle]
    pub extern "C" fn engine_free(ptr: u32) {
        if ptr == 0 {
            return;
        }
        unsafe {
            drop(Box::from_raw(ptr as usize as *mut Engine));
        }
    }

    #[no_mangle]
    pub extern "C" fn engine_resize(ptr: u32, width: u32, height: u32) {
        engine(ptr).resize(width, height);
    }

    #[no_mangle]
    pub extern "C" fn engine_clear_quiet(ptr: u32) {
        engine(ptr).clear_quiet();
    }

    #[no_mangle]
    pub extern "C" fn engine_push_quiet(ptr: u32, x: f32, y: f32, w: f32, h: f32) {
        engine(ptr).push_quiet(x, y, w, h);
    }

    #[no_mangle]
    pub extern "C" fn engine_pointer(ptr: u32, x: f32, y: f32, inside: u32) {
        engine(ptr).pointer(x, y, inside != 0);
    }

    #[no_mangle]
    pub extern "C" fn engine_pointer_down(ptr: u32, x: f32, y: f32, now: f64) {
        engine(ptr).pointer_down(x, y, now);
    }

    #[no_mangle]
    pub extern "C" fn engine_pointer_up(ptr: u32, x: f32, y: f32, now: f64) -> u32 {
        u32::from(engine(ptr).pointer_up(x, y, now))
    }

    #[no_mangle]
    pub extern "C" fn engine_pointer_cancel(ptr: u32) {
        engine(ptr).pointer_cancel();
    }

    #[no_mangle]
    pub extern "C" fn engine_replay(ptr: u32, now: f64) {
        engine(ptr).replay(now);
    }

    #[no_mangle]
    pub extern "C" fn engine_hit(ptr: u32, x: f32, y: f32) -> u32 {
        u32::from(engine(ptr).hit(x, y))
    }

    #[no_mangle]
    pub extern "C" fn engine_frame(
        ptr: u32,
        now: f64,
        mark_x: f32,
        mark_y: f32,
        mark_w: f32,
        mark_h: f32,
        reduced: u32,
    ) {
        engine(ptr).frame(now, mark_x, mark_y, mark_w, mark_h, reduced != 0);
    }

    #[no_mangle]
    pub extern "C" fn engine_pixels(ptr: u32) -> u32 {
        engine(ptr).pixels().as_ptr() as usize as u32
    }

    #[no_mangle]
    pub extern "C" fn engine_byte_len(ptr: u32) -> u32 {
        engine(ptr).pixels().len() as u32
    }

    #[no_mangle]
    pub extern "C" fn engine_width(ptr: u32) -> u32 {
        engine(ptr).width()
    }

    #[no_mangle]
    pub extern "C" fn engine_height(ptr: u32) -> u32 {
        engine(ptr).height()
    }

    #[no_mangle]
    pub extern "C" fn engine_ripples(ptr: u32) -> u32 {
        engine(ptr).ripple_count()
    }

    #[no_mangle]
    pub extern "C" fn engine_appear(ptr: u32) -> u32 {
        engine(ptr).appear_index()
    }

    #[no_mangle]
    pub extern "C" fn appear_len() -> u32 {
        super::appears::len() as u32
    }

    #[no_mangle]
    pub extern "C" fn appear_name(index: u32) -> u32 {
        super::appears::name_at(index as usize).as_ptr() as u32
    }

    #[no_mangle]
    pub extern "C" fn appear_name_len(index: u32) -> u32 {
        super::appears::name_at(index as usize).len() as u32
    }

    #[no_mangle]
    pub extern "C" fn engine_reveal(ptr: u32) -> f32 {
        engine(ptr).reveal()
    }

    #[no_mangle]
    pub extern "C" fn engine_cell(ptr: u32) -> f32 {
        engine(ptr).cell()
    }

    #[no_mangle]
    pub extern "C" fn engine_mark_prepare(ptr: u32, cols: i32, rows: i32) -> u32 {
        engine(ptr).prepare_mark(cols, rows) as u32
    }

    #[no_mangle]
    pub extern "C" fn engine_mark_commit(ptr: u32) {
        engine(ptr).commit_mark();
    }

    #[no_mangle]
    pub extern "C" fn engine_mark_reset(ptr: u32) {
        engine(ptr).reset_mark();
    }

    #[no_mangle]
    pub extern "C" fn engine_logo_cols(ptr: u32) -> u32 {
        engine(ptr).logo_cols() as u32
    }

    #[no_mangle]
    pub extern "C" fn engine_logo_rows(ptr: u32) -> u32 {
        engine(ptr).logo_rows() as u32
    }

    #[no_mangle]
    pub extern "C" fn engine_hex_prepare(ptr: u32) -> u32 {
        engine(ptr).prepare_hex() as u32
    }

    #[no_mangle]
    pub extern "C" fn engine_hex_commit(ptr: u32, kind: u32, len: u32) -> u32 {
        u32::from(engine(ptr).commit_hex(kind, len))
    }

    #[no_mangle]
    pub extern "C" fn logo_cols() -> u32 {
        super::logo::COLS as u32
    }

    #[no_mangle]
    pub extern "C" fn logo_rows() -> u32 {
        super::logo::ROWS as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette;

    fn mark_for(w: f32) -> (f32, f32, f32, f32) {
        let h = w * (logo::ROWS as f32 / logo::COLS as f32);
        (24.0, 28.0, w, h)
    }

    fn count(engine: &Engine, color: [u8; 4]) -> usize {
        engine
            .pixels()
            .chunks_exact(4)
            .filter(|px| *px == color)
            .count()
    }

    fn brightness(engine: &Engine) -> u64 {
        engine.pixels().iter().map(|b| *b as u64).sum()
    }

    #[test]
    fn wordmark_prints_then_restarts() {
        let mut engine = Engine::new();
        engine.resize(480, 260);
        let (x, y, w, h) = mark_for(400.0);
        engine.frame(0.0, x, y, w, h, false);
        let early = count(&engine, palette::CREST);
        engine.frame(2_000.0, x, y, w, h, false);
        let full = count(&engine, palette::CREST);
        assert!(full > early + 80, "early {early} full {full}");
        assert!(engine.reveal() > 0.99);
        engine.replay(2_000.0);
        engine.frame(2_000.0, x, y, w, h, false);
        let restarted = count(&engine, palette::CREST);
        assert!(restarted * 3 < full, "restarted {restarted} full {full}");
    }

    #[test]
    fn pointer_lifts_the_field() {
        let mut engine = Engine::new();
        engine.resize(360, 220);
        let (x, y, w, h) = mark_for(280.0);
        engine.pointer(-100.0, -100.0, false);
        engine.frame(1_600.0, x, y, w, h, false);
        let before = brightness(&engine);
        let cx = x + w * 0.5;
        let cy = y + h * 0.5;
        engine.pointer(cx, cy, true);
        for step in 0..12 {
            engine.frame(1_620.0 + step as f64 * 16.0, x, y, w, h, false);
        }
        let after = brightness(&engine);
        assert!(after > before + 2_000, "before {before} after {after}");
        assert!(engine.hit(cx, cy));
        assert!(!engine.hit(0.0, 0.0));
    }

    #[test]
    fn release_on_the_field_leaves_a_stamp() {
        let mut engine = Engine::new();
        engine.resize(320, 200);
        let (x, y, w, h) = mark_for(240.0);
        engine.frame(0.0, x, y, w, h, false);
        engine.pointer_down(4.0, 4.0, 10.0);
        assert!(!engine.pointer_up(4.0, 4.0, 180.0));
        assert_eq!(engine.ripple_count(), 1);
        engine.frame(4_000.0, x, y, w, h, false);
        assert_eq!(engine.ripple_count(), 0);
    }

    #[test]
    fn release_on_the_word_restarts_the_print() {
        let mut engine = Engine::new();
        engine.resize(360, 220);
        let (x, y, w, h) = mark_for(280.0);
        engine.frame(0.0, x, y, w, h, false);
        engine.frame(2_000.0, x, y, w, h, false);
        let cx = x + w * 0.5;
        let cy = y + h * 0.5;
        engine.pointer_down(cx, cy, 2_000.0);
        assert!(engine.pointer_up(cx, cy, 2_080.0));
        assert_eq!(engine.ripple_count(), 0);
        engine.frame(2_080.0, x, y, w, h, false);
        assert!(engine.reveal() < 0.05, "{}", engine.reveal());
        assert_eq!(engine.appear_name(), "drop");
    }

    #[test]
    fn clicks_walk_the_appear_list() {
        let mut engine = Engine::new();
        engine.resize(360, 220);
        let (x, y, w, h) = mark_for(280.0);
        engine.frame(0.0, x, y, w, h, false);
        assert_eq!(engine.appear_name(), "print");
        engine.replay(2_000.0);
        assert_eq!(engine.appear_name(), "drop");
        engine.replay(3_000.0);
        assert_eq!(engine.appear_name(), "rise");
        engine.replay(4_000.0);
        assert_eq!(engine.appear_name(), "bloom");
        engine.replay(5_000.0);
        assert_eq!(engine.appear_name(), "spark");
        engine.replay(6_000.0);
        assert_eq!(engine.appear_name(), "print");
    }

    #[test]
    fn quiet_rect_clears_the_ground() {
        let mut engine = Engine::new();
        engine.resize(200, 120);
        engine.frame(1_000.0, 0.0, -4_000.0, 480.0, 126.0, false);
        let open = count(&engine, palette::BG);
        let total = engine.pixels().len() / 4;
        let lit = total - open;
        assert!(lit > total / 40, "field looked empty: {lit}/{total}");
        assert!(lit < total / 3, "field looked solid: {lit}/{total}");

        engine.push_quiet(0.0, 0.0, 200.0, 120.0);
        engine.frame(1_016.0, 0.0, -4_000.0, 480.0, 126.0, false);
        let cleared = total - count(&engine, palette::BG);
        assert!(cleared < 8, "quiet rect still drew {cleared} cells");
    }

    #[test]
    fn reduced_motion_holds_the_mark_still() {
        let mut engine = Engine::new();
        engine.resize(320, 180);
        let (x, y, w, h) = mark_for(240.0);
        engine.frame(0.0, x, y, w, h, true);
        assert_eq!(engine.reveal(), 1.0);
        let crest = count(&engine, palette::CREST);
        assert!(crest > 40, "{crest}");
        engine.pointer_down(2.0, 2.0, 10.0);
        assert!(!engine.pointer_up(2.0, 2.0, 40.0));
        assert_eq!(engine.ripple_count(), 0);
    }

    #[test]
    fn a_live_title_replaces_the_baked_word() {
        let mut engine = Engine::new();
        engine.resize(220, 140);
        let (x, y, w, h) = mark_for(160.0);
        engine.frame(1_000.0, x, y, w, h, false);
        assert_eq!(engine.logo_cols(), 80);
        let mut cells = vec![0u8; 12 * 21];
        let crest_row = 1;
        cells[crest_row * 12 + 3] = 1;
        assert!(engine.set_mark(12, 21, &cells));
        assert_eq!((engine.logo_cols(), engine.logo_rows()), (12, 21));
        engine.frame(1_000.0, x, y, w, h, false);
        assert!(engine.reveal() < 0.05, "{}", engine.reveal());
        engine.frame(3_000.0, x, y, w, h, false);
        let crest = count(&engine, palette::CREST);
        assert!(crest > 0 && crest < 400, "{crest}");
        assert!(!engine.set_mark(2, 2, &[1, 1, 1, 1]));
        assert_eq!(engine.logo_cols(), 12);
        engine.reset_mark();
        assert_eq!(engine.logo_cols(), logo::COLS);
    }

    #[test]
    fn hex_strings_recolour_ink_and_ground() {
        let mut engine = Engine::new();
        engine.resize(320, 180);
        let (x, y, w, h) = mark_for(240.0);
        engine.frame(0.0, x, y, w, h, true);
        let crest = count(&engine, palette::CREST);
        let ground = count(&engine, palette::BG);
        assert!(crest > 40, "{crest}");
        assert!(ground > 40, "{ground}");

        assert!(!engine.set_tint_hex("nope"));
        assert!(!engine.set_background_hex("#12"));
        assert!(engine.set_tint_hex("#ff6040"));
        assert!(engine.set_background_hex("101820"));
        engine.frame(0.0, x, y, w, h, true);
        assert!(count(&engine, palette::CREST) * 4 < crest);
        assert_eq!(count(&engine, palette::BG), 0);
        let painted = [0x10, 0x18, 0x20, 0xff];
        assert!(count(&engine, painted) > ground / 2);

        assert!(!engine.set_background_hex("nope"));
        engine.frame(0.0, x, y, w, h, true);
        assert_eq!(count(&engine, palette::BG), 0);

        assert!(engine.set_tint_hex("#d6ee6a"));
        assert!(engine.set_background_hex("#12140f"));
        engine.frame(0.0, x, y, w, h, true);
        assert!(count(&engine, palette::CREST) > 40);
        assert!(count(&engine, palette::BG) > 40);

        let mut raw = b"#ff6040".to_vec();
        raw.resize(16, 0);
        engine.hex[..raw.len()].copy_from_slice(&raw);
        assert!(engine.commit_hex(0, 7));
        engine.frame(0.0, x, y, w, h, true);
        assert!(count(&engine, palette::CREST) * 4 < crest);
    }
}
