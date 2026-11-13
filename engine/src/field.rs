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
//! Config inside `amount` and `vignette`:
//! - The noise scrolls at `0.14` and `0.055` cells per second. Raise them
//!   and the speckle drifts faster. Reduced motion freezes time, so this stops.
//! - `0.78` near the end of `amount` is the overall density. Lower is sparser.
//! - `vignette` starts opening up at radius `0.34`. The vertical fade is
//!   the first `0.22` of the header height.

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

/// How much of this cell wants to be ink, before the Bayer test.
pub fn amount(noise: &Noise, col: i32, row: i32, time_s: f32, vignette: f32) -> f32 {
    let x = col as f32 / 9.0;
    let y = row as f32 / 9.0;
    // 0.14 and 0.055 are the scroll speeds. 0.78 at the end is the density.
    let i = 0.62 * noise.sample(x + time_s * 0.14, y - time_s * 0.055)
        + 0.38 * noise.sample(x * 0.55 - time_s * 0.08, y * 0.55 + time_s * 0.06);
    let spark = 0.5 + 0.5 * (time_s * 1.1 + hash01(col, row) * std::f32::consts::TAU).sin();
    vignette * (0.30 + 0.52 * i * i + 0.18 * spark) * 0.78
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
    // 0.34 is where the edges start to fill in. 0.22 below is the top fade.
    let edge = ((rad - 0.34) / 0.9).clamp(0.0, 1.0);
    let radial = 0.18 + 0.82 * edge * edge;
    let vert = (cy / h / 0.22).clamp(0.18, 1.0);
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
