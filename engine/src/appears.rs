//! Step 3. What a click on the word does.
//!
//! Process:
//! 1. The first frame plays `ORDER` entry 0. That is `print`, left to right.
//! 2. A click on the word calls `next_index`, which reads `CYCLE`.
//! 3. `shows` decides which letter cells are visible. `progress` is 0 at the
//!    start of the click and 1 when the word is full. The length of that
//!    clock is `REVEAL_MS` in `motion.rs`. This file is only the shape.
//!
//! Config:
//! - `ORDER` is the click sequence. Reorder it to change what you see.
//!   Entry 0 is the first load. After the last name, the next click returns
//!   to the first.
//! - `CYCLE` is `Sequence` (walk the list) or `Random` (roll, and skip a
//!   repeat of the one just played). The page ships with `Sequence`.
//!
//! Add an appear:
//! 1. Add a name to `Appear`, and the same name as a string in `Appear::name`.
//! 2. In `shows`, say when a cell is visible.
//! 3. Push that name onto `ORDER`.
//!
//! Reduced motion holds the word finished and does not advance the list.

use crate::field;

/// How the wordmark fills in. Every variant ends on the full bitmap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Appear {
    /// Left to right, with a little noise in the column order.
    Print,
    /// Ascenders first, tails last.
    Drop,
    /// Tails first, ascenders last.
    Rise,
    /// Center of the word outward.
    Bloom,
    /// Scattered. Cells arrive in hash order.
    Spark,
}

impl Appear {
    pub fn name(self) -> &'static str {
        match self {
            Appear::Print => "print",
            Appear::Drop => "drop",
            Appear::Rise => "rise",
            Appear::Bloom => "bloom",
            Appear::Spark => "spark",
        }
    }
}

/// The click sequence. Entry 0 plays on the first frame.
/// Reorder these lines to change what each click does.
pub const ORDER: &[Appear] = &[
    Appear::Print,
    Appear::Drop,
    Appear::Rise,
    Appear::Bloom,
    Appear::Spark,
];

/// What a click does with [`ORDER`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cycle {
    /// The next entry, then wrap.
    Sequence,
    /// A roll from the clock. A repeat of the current entry is skipped.
    /// Assign this to [`CYCLE`] to turn it on.
    #[allow(dead_code)]
    Random,
}

/// `Sequence` walks `ORDER`, so the list you edit is the list you see.
/// Set this to `Cycle::Random` when the next click should be a roll.
pub const CYCLE: Cycle = Cycle::Sequence;

pub fn kind(index: usize) -> Appear {
    ORDER[index % len()]
}

pub fn name_at(index: usize) -> &'static str {
    kind(index).name()
}

pub fn len() -> usize {
    ORDER.len()
}

pub fn next_index(current: usize, now_ms: f64) -> usize {
    step(CYCLE, current, now_ms)
}

pub fn step(cycle: Cycle, current: usize, now_ms: f64) -> usize {
    let len = len().max(1);
    let current = current % len;
    match cycle {
        Cycle::Sequence => (current + 1) % len,
        Cycle::Random => {
            if len == 1 {
                return 0;
            }
            let n = roll(now_ms.to_bits(), current as u32);
            let mut index = (n as usize) % len;
            if index == current {
                index = (index + 1) % len;
            }
            index
        }
    }
}

/// `progress` runs from 0 (nothing) to 1 (the whole bitmap).
pub fn shows(kind: Appear, col: i32, row: i32, cols: i32, rows: i32, progress: f32) -> bool {
    if progress >= 1.0 {
        return true;
    }
    if progress <= 0.0 || cols <= 0 || rows <= 0 {
        return false;
    }
    let jitter = field::hash01(col, row);
    let threshold = match kind {
        Appear::Print => {
            let along = col as f32 / cols as f32;
            along * 0.82 + jitter * 0.18
        }
        Appear::Drop => {
            let along = row as f32 / rows as f32;
            along * 0.82 + jitter * 0.18
        }
        Appear::Rise => {
            let along = 1.0 - (row as f32 / rows as f32);
            along * 0.82 + jitter * 0.18
        }
        Appear::Bloom => {
            let cx = (col as f32 + 0.5) / cols as f32 - 0.5;
            let cy = (row as f32 + 0.5) / rows as f32 - 0.5;
            let dist = (cx * cx + cy * cy).sqrt() * 1.6;
            (dist * 0.92 + jitter * 0.08).min(0.99)
        }
        Appear::Spark => jitter,
    };
    progress >= threshold
}

fn roll(a: u64, b: u32) -> u32 {
    let mut n = (a as u32)
        .wrapping_mul(374_761_393)
        .wrapping_add(b.wrapping_mul(668_265_263));
    n ^= n >> 16;
    n = n.wrapping_mul(0x7feb_352d);
    n ^= n >> 15;
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    fn covered(kind: Appear, progress: f32, top: bool) -> usize {
        let mut n = 0;
        for row in 0..21 {
            let in_band = if top { row < 10 } else { row >= 10 };
            if !in_band {
                continue;
            }
            for col in 0..80 {
                if shows(kind, col, row, 80, 21, progress) {
                    n += 1;
                }
            }
        }
        n
    }

    #[test]
    fn the_list_is_the_sequence_and_it_wraps() {
        assert!(ORDER.len() >= 2);
        assert_eq!(kind(0), Appear::Print);
        assert_eq!(name_at(0), "print");
        assert_eq!(step(Cycle::Sequence, 0, 0.0), 1);
        assert_eq!(step(Cycle::Sequence, ORDER.len() - 1, 0.0), 0);
        let rolled = step(Cycle::Random, 0, 12.0);
        assert_ne!(rolled, 0);
        assert!(rolled < ORDER.len());
    }

    #[test]
    fn every_appear_starts_empty_and_finishes_full() {
        for kind in ORDER {
            assert!(!shows(*kind, 10, 8, 80, 21, 0.0));
            assert!(shows(*kind, 10, 8, 80, 21, 1.0));
            assert!(shows(*kind, 79, 20, 80, 21, 1.0));
        }
    }

    #[test]
    fn print_drop_and_rise_travel_in_their_own_direction() {
        let progress = 0.45;
        let mut left = 0;
        let mut right = 0;
        for row in 0..21 {
            for col in 0..80 {
                if !shows(Appear::Print, col, row, 80, 21, progress) {
                    continue;
                }
                if col < 40 {
                    left += 1;
                } else {
                    right += 1;
                }
            }
        }
        assert!(left > right * 2, "print left {left} right {right}");

        let drop_top = covered(Appear::Drop, progress, true);
        let drop_bottom = covered(Appear::Drop, progress, false);
        assert!(
            drop_top > drop_bottom * 2,
            "drop {drop_top} vs {drop_bottom}"
        );

        let rise_top = covered(Appear::Rise, progress, true);
        let rise_bottom = covered(Appear::Rise, progress, false);
        assert!(
            rise_bottom > rise_top * 2,
            "rise {rise_bottom} vs {rise_top}"
        );
    }

    #[test]
    fn bloom_opens_from_the_middle_and_spark_scatters() {
        assert!(shows(Appear::Bloom, 40, 10, 80, 21, 0.2));
        assert!(!shows(Appear::Bloom, 0, 0, 80, 21, 0.2));
        assert!(shows(Appear::Bloom, 0, 0, 80, 21, 1.0));

        let mut shown = 0;
        let mut left = 0;
        let mut right = 0;
        for row in 0..21 {
            for col in 0..80 {
                if !shows(Appear::Spark, col, row, 80, 21, 0.5) {
                    continue;
                }
                shown += 1;
                if col < 40 {
                    left += 1;
                } else {
                    right += 1;
                }
            }
        }
        assert!(shown > 400 && shown < 1_200, "{shown}");
        assert!(left > 100 && right > 100, "left {left} right {right}");
    }
}
