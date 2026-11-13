//! Step 2. The word, and which rows are bright.
//!
//! Process:
//! 1. `mark.txt` is baked into the program at compile time. `#` is a filled
//!    cell. `.` is empty. One row per line.
//! 2. `Grid::baked` copies those cells into memory. A live title can replace
//!    them. `reset_mark` in `lib.rs` puts this file back.
//! 3. `band_at` picks the colour of a row. `0` is the top of the letters.
//!    `1` is the tail. The cuts below are fractions of that height.
//!
//! Config:
//! - Draw the letters in `engine/mark.py`, then run `python3 engine/mark.py`.
//!   That rewrites `mark.txt`. Do not edit `mark.txt` by hand.
//! - The tests in this file expect 80 columns, 21 rows, and 465 filled cells.
//!   Update that count when a letter change moves it.
//! - Band cuts, top to bottom: crest, hover, lit, mid, then dim.
//!   Raise a cut and that colour reaches farther down the letters.

use crate::palette::Band;

const MARK: &str = include_str!("mark.txt");

const fn dims() -> (i32, i32) {
    let b = MARK.as_bytes();
    let mut cols = 0;
    while cols < b.len() && b[cols] != b'\n' {
        cols += 1;
    }
    let mut rows = 0;
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\n' {
            rows += 1;
        }
        i += 1;
    }
    (cols as i32, rows)
}

pub const COLS: i32 = dims().0;
pub const ROWS: i32 = dims().1;

pub fn on(col: i32, row: i32) -> bool {
    if col < 0 || row < 0 || col >= COLS || row >= ROWS {
        return false;
    }
    let i = row as usize * (COLS as usize + 1) + col as usize;
    MARK.as_bytes().get(i).copied() == Some(b'#')
}

/// The cells the frame reads. Starts as the baked pesty.sh mark.
pub struct Grid {
    pub cols: i32,
    pub rows: i32,
    cells: Vec<u8>,
}

impl Grid {
    pub fn baked() -> Self {
        let mut cells = Vec::with_capacity((COLS * ROWS) as usize);
        for row in 0..ROWS {
            for col in 0..COLS {
                cells.push(u8::from(on(col, row)));
            }
        }
        Self {
            cols: COLS,
            rows: ROWS,
            cells,
        }
    }

    pub fn on(&self, col: i32, row: i32) -> bool {
        if col < 0 || row < 0 || col >= self.cols || row >= self.rows {
            return false;
        }
        self.cells[(row * self.cols + col) as usize] != 0
    }

    pub fn band(&self, row: i32) -> Band {
        band_at(row, self.rows)
    }

    pub(crate) fn from_cells(cols: i32, rows: i32, cells: Vec<u8>) -> Self {
        Self { cols, rows, cells }
    }
}

/// Where each colour ends, as a fraction of the letter height.
/// `0` is the top. `1` is the tail. The last band, dim, is whatever is left.
const CREST_UNTIL: f32 = 0.26;
const HOVER_UNTIL: f32 = 0.37;
const LIT_UNTIL: f32 = 0.58;
const MID_UNTIL: f32 = 0.74;

/// Light band for a bitmap row. `t` is 0 at the ascender line and 1 at the tail.
pub fn band_at(row: i32, rows: i32) -> Band {
    let span = (rows - 1).max(1) as f32;
    let t = (row as f32 / span).clamp(0.0, 1.0);
    if t < CREST_UNTIL {
        Band::Crest
    } else if t < HOVER_UNTIL {
        Band::Hover
    } else if t < LIT_UNTIL {
        Band::Lit
    } else if t < MID_UNTIL {
        Band::Mid
    } else {
        Band::Dim
    }
}

pub fn in_rect(x: f32, y: f32, mark_x: f32, mark_y: f32, mark_w: f32, mark_h: f32) -> bool {
    mark_w > 0.0
        && mark_h > 0.0
        && x >= mark_x
        && y >= mark_y
        && x < mark_x + mark_w
        && y < mark_y + mark_h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitmap_is_the_pesty_mark() {
        assert_eq!((COLS, ROWS), (80, 21));
        assert_eq!(MARK.len(), (COLS as usize + 1) * ROWS as usize);
        let ink = (0..ROWS)
            .flat_map(|r| (0..COLS).filter(move |c| on(*c, r)))
            .count();
        assert_eq!(ink, 465);
        assert!(!on(-1, 0));
        assert!(!on(0, 0));
        assert!(on(2, 16), "p stem");
    }

    #[test]
    fn light_falls_from_crest_to_dim() {
        assert_eq!(band_at(1, ROWS), Band::Crest);
        assert_eq!(band_at(ROWS - 2, ROWS), Band::Dim);
        assert_ne!(band_at(1, ROWS), band_at(ROWS - 2, ROWS));
    }

    #[test]
    fn descenders_are_the_tails_only() {
        let bottom = ROWS - 2;
        let ink: Vec<i32> = (0..COLS).filter(|c| on(*c, bottom)).collect();
        assert!(!ink.is_empty());
        assert!(
            ink.len() < 12,
            "tails should be two short strokes, got {ink:?}"
        );
    }
}
