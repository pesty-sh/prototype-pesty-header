//! One picture. If you are new, read the numbered steps in `paint` first.
//!
//! Nothing else writes pixels. `lib.rs` calls `paint` once per frame.
//! The page copies the buffer onto the canvas after that call returns.
//!
//! Process, in the order the numbers in `paint` run:
//! 1. Fill every pixel with the background colour. Cells that fail the
//!    speckle test stay this colour.
//! 2. Ease the pointer toward the latest mouse sample. Quiet boxes (the nav
//!    and the sentence) pull that glow down so the type stays readable.
//! 3. Advance the letter clock. The length is `REVEAL_MS` in `motion.rs`.
//!    The shape of the fill is `appears.rs`.
//! 4. Collect glows: the pointer, then a slow drift when motion is allowed.
//! 5. Collect icons. First the one charging under a held finger, then any
//!    released earlier that are still alive. A slide also contributes the
//!    smaller icons it has spat out. A release on the word is not in this
//!    list. It restarts the letter fill instead.
//! 6. For each cell, decide. A letter cell the appear has revealed takes the
//!    overhead colour, lifted toward crest by a nearby glow. Every other cell
//!    is the field. Noise has to clear a threshold, and the pointer, the
//!    drift, and the icons are what push it up into mid and lit. Inside a
//!    wide radius the pointer slides those speckles toward itself. They
//!    slide back out as the pointer moves away.
//!
//! Glow sizes are in `config.rs` (`QUIET_REACH_CELLS`, `POINTER_REACH_CELLS`,
//! `DRIFT_GLOW`, and the rest of that group).

use crate::appears;
use crate::config::{
    DRIFT_GLOW, DRIFT_GLOW_DIM, DRIFT_GLOW_REACH_CELLS, NOISE_WELL_CELLS, NOISE_WELL_PULL,
    POINTER_REACH_CELLS, POINTER_REACH_EXTRA, QUIET_REACH_CELLS,
};
use crate::field::{self, Noise, Well};
use crate::logo::Grid;
use crate::motion;
use crate::palette::{self, Band};
use crate::pointer::{self, Rect};
use crate::sprites::{self, Stamp};
use crate::Engine;

struct Glow {
    x: f32,
    y: f32,
    strength: f32,
    reach: f32,
}

pub fn paint(engine: &mut Engine) {
    let width = engine.width;
    let height = engine.height;
    if width == 0 || height == 0 {
        return;
    }
    // 1. Background. Cells that fail the speckle test leave this colour.
    //    `background` is the demo hex. Empty means palette::BG.
    let tint = engine.tint;
    let ground = engine.background.unwrap_or(palette::BG);
    fill_bg(&mut engine.pixels, ground);

    let mark = engine.mark;
    if !(mark.w.is_finite() && mark.h.is_finite()) || mark.w < 1.0 || mark.h < 1.0 {
        return;
    }
    let cw = mark.w / engine.grid.cols as f32;
    let ch = mark.h / engine.grid.rows as f32;
    if cw < 0.5 || ch < 0.5 {
        return;
    }
    engine.cell_w = cw;
    engine.cell_h = ch;

    let cell = (cw + ch) * 0.5;
    // How far past a quiet box the glow fades back in. 7 letter-cells.
    let quiet_reach = cell * QUIET_REACH_CELLS;
    let q_pointer = pointer::quiet_factor(
        engine.pointer.raw_x,
        engine.pointer.raw_y,
        &engine.quiet,
        quiet_reach,
    );
    // 2. Pointer. `smooth` eases toward the latest sample. `q_pointer` is 0
    //    inside type and 1 once `quiet_reach` away. The follow speeds are in pointer.rs.
    engine.pointer.smooth(q_pointer);

    // 3. Letter clock. `reveal` returns 0 at the start and 1 when the word is full.
    //    The shape (left to right, top to bottom, and the rest) is appears.rs.
    let now = engine.now;
    let reduced = engine.reduced;
    let progress = motion::reveal(now, engine.reveal_from, reduced);
    engine.reveal = progress;
    let appear = appears::kind(engine.appear_ix);
    let time_s = if reduced { 0.0 } else { (now / 1000.0) as f32 };

    // 4. Glows. The pointer first, then the drift when motion is allowed.
    //    Reach and strength here are the glow config. See the file comment.
    let mut glows = Vec::with_capacity(2);
    if engine.pointer.strength > 0.01 {
        glows.push(Glow {
            x: engine.pointer.x,
            y: engine.pointer.y,
            strength: engine.pointer.strength,
            reach: cell * (POINTER_REACH_CELLS + POINTER_REACH_EXTRA * engine.pointer.strength),
        });
    }
    if !reduced {
        let (wx, wy) = motion::wander(now, width as f32, height as f32);
        let dim = 1.0 - engine.pointer.strength * DRIFT_GLOW_DIM;
        glows.push(Glow {
            x: wx,
            y: wy,
            strength: DRIFT_GLOW * dim,
            reach: cell * DRIFT_GLOW_REACH_CELLS,
        });
    }

    // 5. Icons. The charge under a held pointer, then icons already released.
    //    Reduced motion drops the list. A press on the word (`glyph`) is not an icon.
    let mut stamps: Vec<Stamp> = Vec::with_capacity(engine.sprites.len() * 8 + 1);
    if reduced {
        engine.sprites.clear();
    } else {
        // The wander ball fires one pixel trail. The mouse is not involved.
        engine.tick_drift(cell);
        if engine.pointer.down && !engine.pointer.glyph {
            let held = (now - engine.pointer.down_at).max(0.0);
            stamps.push(sprites::charge_stamp(
                engine.pointer.x,
                engine.pointer.y,
                held,
                cell,
                engine.pending.icon,
            ));
        }
        engine.sprites.retain(|sprite| sprite.live(now).is_some());
        for sprite in &engine.sprites {
            sprite.push_stamps(now, &mut stamps);
        }
    }

    // The well follows the pointer. A speckle's slide is its distance to
    // that point, so it travels in as the cursor approaches and back out
    // as the cursor moves away. Strength only fades the pull once the
    // pointer has left the picture.
    let well = if !reduced && engine.pointer.strength > 0.02 {
        Some(Well {
            col: (engine.pointer.x - mark.x) / cw,
            row: (engine.pointer.y - mark.y) / ch,
            radius: NOISE_WELL_CELLS,
            pull: NOISE_WELL_PULL * engine.pointer.strength.min(1.0),
        })
    } else {
        None
    };

    let col0 = ((-mark.x) / cw).floor() as i32 - 1;
    let row0 = ((-mark.y) / ch).floor() as i32 - 1;
    let col1 = ((width as f32 - mark.x) / cw).ceil() as i32 + 2;
    let row1 = ((height as f32 - mark.y) / ch).ceil() as i32 + 2;
    if !(0..2_000).contains(&(col1 - col0)) || !(0..2_000).contains(&(row1 - row0)) {
        return;
    }

    for row in row0..row1 {
        for col in col0..col1 {
            // 6. One cell. `shows` decides a letter cell. Everything else is field.
            let color = cell_color(
                &engine.grid,
                &CellIn {
                    noise: &engine.noise,
                    quiet: &engine.quiet,
                    glows: &glows,
                    stamps: &stamps,
                    appear,
                    col,
                    row,
                    cw,
                    ch,
                    mark,
                    width,
                    height,
                    quiet_reach,
                    progress,
                    time_s,
                    tint,
                    well,
                },
            );
            if let Some(color) = color {
                let (x0, x1) = span(mark.x, col, cw);
                let (y0, y1) = span(mark.y, row, ch);
                fill_cell(
                    &mut engine.pixels,
                    width,
                    height,
                    PxBox { x0, y0, x1, y1 },
                    color,
                );
            }
        }
    }
}

struct CellIn<'a> {
    noise: &'a Noise,
    quiet: &'a [Rect],
    glows: &'a [Glow],
    stamps: &'a [Stamp],
    appear: appears::Appear,
    col: i32,
    row: i32,
    cw: f32,
    ch: f32,
    mark: Rect,
    width: u32,
    height: u32,
    quiet_reach: f32,
    progress: f32,
    time_s: f32,
    tint: Option<[u8; 4]>,
    well: Option<Well>,
}

fn cell_color(grid: &Grid, cell: &CellIn<'_>) -> Option<[u8; 4]> {
    let (x0, x1) = span(cell.mark.x, cell.col, cell.cw);
    let (y0, y1) = span(cell.mark.y, cell.row, cell.ch);
    if x1 <= 0 || y1 <= 0 || x0 >= cell.width as i32 || y0 >= cell.height as i32 {
        return None;
    }
    let cx = (x0 + x1) as f32 * 0.5;
    let cy = (y0 + y1) as f32 * 0.5;
    let q = pointer::quiet_factor(cx, cy, cell.quiet, cell.quiet_reach);
    let glow = glow_at(cx, cy, cell.glows) * q;
    let mut stamp: f32 = 0.0;
    for sample in cell.stamps {
        stamp = stamp.max(sample.amp_at(cx, cy));
    }
    stamp *= q;
    let influence = (glow + stamp).clamp(0.0, 1.4);

    if grid.on(cell.col, cell.row)
        && appears::shows(
            cell.appear,
            cell.col,
            cell.row,
            grid.cols,
            grid.rows,
            cell.progress,
        )
    {
        let base = palette::ink(grid.band(cell.row), cell.tint);
        return Some(palette::mix(
            base,
            palette::ink(Band::Crest, cell.tint),
            (influence * 0.85).min(1.0),
        ));
    }

    let vig = field::vignette(cx, cy, cell.width as f32, cell.height as f32) * q;
    let (ncol, nrow, mass) = match field::sample_at(cell.col, cell.row, cell.well) {
        field::Sample::Own => (cell.col as f32, cell.row as f32, 1.0),
        field::Sample::Shifted { col, row, mass } => (col, row, mass),
    };
    let base = field::amount(cell.noise, ncol, nrow, cell.time_s, vig) * mass;
    if base + influence * 0.9 <= field::threshold(ncol.round() as i32, nrow.round() as i32) {
        return None;
    }
    Some(field::tier(influence, cell.tint))
}

fn glow_at(x: f32, y: f32, glows: &[Glow]) -> f32 {
    let mut best: f32 = 0.0;
    for g in glows {
        if g.reach <= 0.0 || g.strength <= 0.0 {
            continue;
        }
        let dx = x - g.x;
        let dy = y - g.y;
        let d = (dx * dx + dy * dy).sqrt();
        if d >= g.reach {
            continue;
        }
        let t = 1.0 - d / g.reach;
        best = best.max(t * t * g.strength);
    }
    best
}

fn span(origin: f32, index: i32, size: f32) -> (i32, i32) {
    let a = (origin + index as f32 * size).round() as i32;
    let b = (origin + (index as f32 + 1.0) * size).round() as i32;
    if b <= a {
        (a, a + 1)
    } else {
        (a, b)
    }
}

fn fill_bg(buf: &mut [u8], color: [u8; 4]) {
    for px in buf.chunks_exact_mut(4) {
        px.copy_from_slice(&color);
    }
}

struct PxBox {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
}

fn fill_cell(buf: &mut [u8], width: u32, height: u32, rect: PxBox, color: [u8; 4]) {
    let x0 = rect.x0.max(0) as u32;
    let y0 = rect.y0.max(0) as u32;
    let x1 = (rect.x1.max(0) as u32).min(width);
    let y1 = (rect.y1.max(0) as u32).min(height);
    if x0 >= x1 || y0 >= y1 {
        return;
    }
    for y in y0..y1 {
        let start = ((y * width + x0) * 4) as usize;
        let end = ((y * width + x1) * 4) as usize;
        for px in buf[start..end].chunks_exact_mut(4) {
            px.copy_from_slice(&color);
        }
    }
}
