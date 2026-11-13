//! Step 7. The speckled ground under the word.
//!
//! Process, for one field cell (`paint` step 6 calls these):
//! 1. `amount` asks the noise how much this cell wants to be ink.
//! 2. `threshold` is an 8×8 Bayer pattern plus a little hash, so neighbours
//!    do not all turn on together.
//! 3. If amount, plus the glow and the icons, clears that threshold, the
//!    cell is painted. Otherwise it stays the background from step 1.
//! 4. `vignette` keeps the middle quieter than the edges, so the letters
//!    stay the loudest thing. The top eases in so the bar is not a wall of cells.
//! 5. `tier` picks dim, mid, or lit from how strong the glow is.
//!
//! A cursor inside the picture opens a wide well. Speckles in that radius
//! are hoovered onto the pointer, and the cells they left stay quiet.
//! Each speckle still belongs to its own cell. The well does not store a
//! new point. When the pointer leaves that cell, the speckle is drawn
//! there again.
//!
//! `NOISE_SCROLL_X`, `NOISE_SCROLL_Y`, `NOISE_DENSITY`, `NOISE_WELL_CELLS`,
//! `NOISE_WELL_PULL`, `NOISE_WELL_MASS`, `VIGNETTE_RADIUS`, and
//! `VIGNETTE_TOP` are in `config.rs`.

use crate::config::{
    NOISE_DENSITY, NOISE_SCROLL_X, NOISE_SCROLL_Y, NOISE_WELL_MASS, VIGNETTE_RADIUS, VIGNETTE_TOP,
};
use crate::palette::{self, Band};

const N: i32 = 64;

pub struct Noise {
    grid: Vec<f32>,
}

impl Noise {
    pub fn new(seed: u32) -> Self {
        let mut grid = vec![0.0; (N * N) as usize];
        let mut s = seed;
        for v in &mut grid {
            s = s.wrapping_mul(1664525).wrapping_add(1013904223);
            *v = (s >> 8) as f32 / 16_777_216.0;
        }
        Self { grid }
    }

    pub fn sample(&self, x: f32, y: f32) -> f32 {
        let x = x.rem_euclid(N as f32);
        let y = y.rem_euclid(N as f32);
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let tx = fade(x - x0 as f32);
        let ty = fade(y - y0 as f32);
        let v00 = self.at(x0, y0);
        let v10 = self.at(x0 + 1, y0);
        let v01 = self.at(x0, y0 + 1);
        let v11 = self.at(x0 + 1, y0 + 1);
        let a = v00 + (v10 - v00) * tx;
        let b = v01 + (v11 - v01) * tx;
        a + (b - a) * ty
    }

    fn at(&self, x: i32, y: i32) -> f32 {
        let x = x.rem_euclid(N) as usize;
        let y = y.rem_euclid(N) as usize;
        self.grid[y * N as usize + x]
    }
}

fn fade(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

const BAYER: [u8; 64] = [
    0, 32, 8, 40, 2, 34, 10, 42, 48, 16, 56, 24, 50, 18, 58, 26, 12, 44, 4, 36, 14, 46, 6, 38, 60,
    28, 52, 20, 62, 30, 54, 22, 3, 35, 11, 43, 1, 33, 9, 41, 51, 19, 59, 27, 49, 17, 57, 25, 15,
    47, 7, 39, 13, 45, 5, 37, 63, 31, 55, 23, 61, 29, 53, 21,
];

pub fn hash01(x: i32, y: i32) -> f32 {
    let n = (x as u32)
        .wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(668_265_263));
    let mut n = n;
    n ^= n >> 16;
    n = n.wrapping_mul(0x7feb_352d);
    n ^= n >> 15;
    n = n.wrapping_mul(0x846c_a68b);
    n ^= n >> 16;
    (n & 0x00ff_ffff) as f32 / 16_777_215.0
}

/// Where the cursor is, in the same cell grid the field uses.
/// `col` and `row` are continuous. `0.5` is the middle of the first cell.
#[derive(Clone, Copy, Debug)]
pub struct Well {
    pub col: f32,
    pub row: f32,
    pub radius: f32,
    pub pull: f32,
}

/// Which noise point a cell should draw.
///
/// `Own` is the cell itself. `Shifted` is a point from farther out in the
/// well, drawn here because the cursor is holding it. `mass` keeps that
/// clump visible. `Held` means this cell's point is in the clump, so the
/// cell stays quiet until the cursor lets it go. The point's own cell is
/// not given a new home.
#[derive(Clone, Copy, Debug)]
pub enum Sample {
    Own,
    Shifted { col: f32, row: f32, mass: f32 },
    Held,
}

/// The point that belongs on this cell, or the point the cursor is holding.
/// The noise grid is not written. A later call with the cursor gone
/// returns `Own` again.
pub fn sample_at(col: i32, row: i32, well: Option<Well>) -> Sample {
    let Some(well) = well else {
        return Sample::Own;
    };
    let pull = well.pull.clamp(0.0, 0.92);
    if well.radius < 0.75 || pull < 0.02 {
        return Sample::Own;
    }
    let dx = col as f32 + 0.5 - well.col;
    let dy = row as f32 + 0.5 - well.row;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist >= well.radius {
        return Sample::Own;
    }
    // The whole disk is packed into a small core on the cursor.
    // Everything outside that core has been picked up.
    let inner = (1.0 - pull).clamp(0.06, 1.0);
    let v = dist / well.radius;
    if v > inner {
        return Sample::Held;
    }
    let mass = (1.0 / inner).min(NOISE_WELL_MASS);
    if dist < 0.04 {
        return Sample::Shifted {
            col: col as f32,
            row: row as f32,
            mass,
        };
    }
    let scale = 1.0 / inner;
    Sample::Shifted {
        col: well.col + dx * scale - 0.5,
        row: well.row + dy * scale - 0.5,
        mass,
    }
}

/// How much of this cell wants to be ink, before the Bayer test.
/// `col` and `row` are the noise point, not necessarily the cell on screen.
pub fn amount(noise: &Noise, col: f32, row: f32, time_s: f32, vignette: f32) -> f32 {
    let x = col / 9.0;
    let y = row / 9.0;
    let i = 0.62 * noise.sample(x + time_s * NOISE_SCROLL_X, y - time_s * NOISE_SCROLL_Y)
        + 0.38 * noise.sample(x * 0.55 - time_s * 0.08, y * 0.55 + time_s * 0.06);
    let spark = 0.5
        + 0.5
            * (time_s * 1.1 + hash01(col.round() as i32, row.round() as i32) * std::f32::consts::TAU)
                .sin();
    vignette * (0.30 + 0.52 * i * i + 0.18 * spark) * NOISE_DENSITY
}

pub fn threshold(col: i32, row: i32) -> f32 {
    let b = BAYER[((row & 7) * 8 + (col & 7)) as usize] as f32;
    0.78 * ((b + 0.5) / 64.0) + 0.22 * hash01(col.wrapping_mul(3), row.wrapping_mul(5))
}

/// Center stays a little quieter than the corners. The top of the header
/// eases in so the bar is not a wall of cells.
pub fn vignette(cx: f32, cy: f32, w: f32, h: f32) -> f32 {
    if w <= 1.0 || h <= 1.0 {
        return 0.0;
    }
    let nx = (cx / w) * 2.0 - 1.0;
    let ny = (cy / h) * 2.0 - 1.0;
    let rad = (nx * nx + ny * ny * 0.82).sqrt();
    let edge = ((rad - VIGNETTE_RADIUS) / 0.9).clamp(0.0, 1.0);
    let radial = 0.18 + 0.82 * edge * edge;
    let vert = (cy / h / VIGNETTE_TOP).clamp(0.18, 1.0);
    radial * vert
}

pub fn tier(influence: f32, tint: Option<[u8; 4]>) -> [u8; 4] {
    let band = if influence > 0.42 {
        Band::Lit
    } else if influence > 0.12 {
        Band::Mid
    } else {
        Band::Dim
    };
    palette::ink(band, tint)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn well_at(col: f32, row: f32, pull: f32) -> Well {
        Well {
            col,
            row,
            radius: 20.0,
            pull,
        }
    }

    fn dist(col: f32, row: f32, well: Well) -> f32 {
        let dx = col + 0.5 - well.col;
        let dy = row + 0.5 - well.row;
        (dx * dx + dy * dy).sqrt()
    }

    #[test]
    fn a_well_hoovers_a_wide_disk_and_releases_it() {
        let near = well_at(10.5, 10.5, 0.88);
        assert!(matches!(sample_at(10, 10, None), Sample::Own));
        assert!(matches!(sample_at(40, 10, Some(near)), Sample::Own));
        assert!(matches!(sample_at(11, 10, Some(well_at(10.5, 10.5, 0.0))), Sample::Own));

        let Sample::Shifted { col, row, mass } = sample_at(11, 10, Some(near)) else {
            panic!("the core should hold the gathered points");
        };
        let here = dist(11.0, 10.0, near);
        let home = dist(col, row, near);
        assert!(home > here + 1.0, "home {home} cell {here}");
        assert!(home <= near.radius + 0.05, "home left the well {home}");
        assert!(mass > 1.0);

        assert!(matches!(sample_at(18, 10, Some(near)), Sample::Held));
        let gone = well_at(80.5, 80.5, 0.88);
        assert!(matches!(sample_at(18, 10, Some(gone)), Sample::Own));
    }
}
