//! Step 4. The icons. They do not enter the screen on their own.
//!
//! Process:
//! 1. A press on the field calls `pick`. That rolls the icon and the play
//!    once, so the shape under your finger is the shape that runs.
//! 2. While the finger is down, `charge_stamp` draws that icon growing.
//! 3. The release calls `Sprite::spawn`. `place` moves it until its life ends.
//! 4. A press on the word never reaches this file. It restarts the letter fill.
//!
//! Config:
//! - `CAST` is `Random` (roll an icon and a play) or `First` (always the bug,
//!   expanding in place). The page ships with `Random`.
//! - `ICONS` entry 0 is what `First` uses. Draw order is also the roll order.
//! - A charge finishes at 700ms (`charge_stamp`). Longer holds do not grow more.
//! - A released icon lives 560ms, plus up to about 660ms more when the press
//!   was long (`Sprite::spawn`, `life`).
//! - Five live icons is the cap. That limit is in `lib.rs`, `spawn_sprite`.
//!
//! Add an icon:
//! 1. Draw a 15×15 pattern. `#` is ink, `.` is empty. Every row is 15 characters.
//! 2. Add it to `ICONS`.
//!
//! The build refuses a row that is the wrong length or uses another character.
//!
//! Add a play:
//! 1. Add a name to `Play`.
//! 2. In `Sprite::place`, say where it is, how big it is, and how strong it is.
//! 3. Teach `pick_with` to roll it, or leave it for a cast you set by hand.
//!
//! Plays:
//! - `Appear` pops in, holds, then fades. It does not travel.
//! - `Slide` holds its size and drifts on a random heading.
//! - `Expand` grows in place and thins out. This is the original stamp.

/// `Random` rolls an icon and a play. `First` is bug + expand, every time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cast {
    Random,
    /// Assign this to [`CAST`] to drop the bug with [`Play::Expand`] every time.
    #[allow(dead_code)]
    First,
}

/// `Random` rolls an icon and a play on each press.
/// Set this to `Cast::First` to drop the bug with `Play::Expand` every time.
pub const CAST: Cast = Cast::Random;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Play {
    Appear,
    Slide,
    Expand,
}

pub struct Icon {
    /// Used by the docs and the tests. Paint reads [`Icon::rows`].
    #[allow(dead_code)]
    pub name: &'static str,
    pub rows: &'static [&'static str],
}

const BUG: [&str; 15] = [
    ".......#.......",
    "......###......",
    ".....#####.....",
    "....###.###....",
    "...#########...",
    "..###.###.###..",
    ".####.###.####.",
    "..###.###.###..",
    "...#########...",
    "....###.###....",
    "....##...##....",
    "...##.....##...",
    "...#.......#...",
    "...............",
    "...............",
];

const SEED: [&str; 15] = [
    "...............",
    "...............",
    "...............",
    "...............",
    "...............",
    "......#####....",
    ".....#######...",
    ".....#######...",
    "......#####....",
    ".......###.....",
    "...............",
    "...............",
    "...............",
    "...............",
    "...............",
];

const SPARK: [&str; 15] = [
    "...............",
    ".......#.......",
    "......###......",
    ".......#.......",
    "..#....#....#..",
    "...#...#...#...",
    "....#..#..#....",
    "......#####....",
    "....#..#..#....",
    "...#...#...#...",
    "..#....#....#..",
    ".......#.......",
    "......###......",
    ".......#.......",
    "...............",
];

const MITE: [&str; 15] = [
    "...............",
    "...............",
    "...............",
    "...............",
    "......#####....",
    ".....##...##...",
    "....##.....##..",
    "....##.###.##..",
    "....##.....##..",
    ".....##...##...",
    "......#####....",
    ".......#.#.....",
    ".......#.#.....",
    "...............",
    "...............",
];

/// Draw order is also the roll order. Entry 0 is what [`Cast::First`] uses.
pub const ICONS: &[Icon] = &[
    Icon {
        name: "bug",
        rows: &BUG,
    },
    Icon {
        name: "seed",
        rows: &SEED,
    },
    Icon {
        name: "spark",
        rows: &SPARK,
    },
    Icon {
        name: "mite",
        rows: &MITE,
    },
];

pub(crate) const fn glyphs_ok() -> bool {
    glyph_ok(&BUG) && glyph_ok(&SEED) && glyph_ok(&SPARK) && glyph_ok(&MITE)
}

const fn glyph_ok(rows: &[&str]) -> bool {
    if rows.len() != 15 {
        return false;
    }
    let mut i = 0;
    while i < rows.len() {
        let bytes = rows[i].as_bytes();
        if bytes.len() != 15 {
            return false;
        }
        let mut j = 0;
        while j < bytes.len() {
            if bytes[j] != b'#' && bytes[j] != b'.' {
                return false;
            }
            j += 1;
        }
        i += 1;
    }
    true
}

#[derive(Clone, Copy, Debug)]
pub struct Pick {
    pub icon: usize,
    pub play: Play,
}

impl Pick {
    pub fn first() -> Self {
        Self {
            icon: 0,
            play: Play::Expand,
        }
    }
}

pub fn pick(now_ms: f64, x: f32, y: f32) -> Pick {
    pick_with(CAST, now_ms, x, y)
}

pub fn pick_with(cast: Cast, now_ms: f64, x: f32, y: f32) -> Pick {
    match cast {
        Cast::First => Pick::first(),
        Cast::Random => {
            let n = roll(now_ms.to_bits(), x.to_bits(), y.to_bits());
            let icon = (n as usize) % ICONS.len();
            let play = match (n / 16) % 3 {
                0 => Play::Appear,
                1 => Play::Slide,
                _ => Play::Expand,
            };
            Pick { icon, play }
        }
    }
}

pub fn ink(icon: usize, col: i32, row: i32) -> bool {
    if col < 0 || row < 0 {
        return false;
    }
    let Some(icon) = ICONS.get(icon) else {
        return false;
    };
    let Some(line) = icon.rows.get(row as usize) else {
        return false;
    };
    line.as_bytes().get(col as usize).copied() == Some(b'#')
}

#[derive(Clone, Copy, Debug)]
pub struct Stamp {
    pub x: f32,
    pub y: f32,
    pub cell_px: f32,
    pub amp: f32,
    pub icon: usize,
}

impl Stamp {
    pub fn amp_at(&self, x: f32, y: f32) -> f32 {
        if self.cell_px <= 0.25 || self.amp <= 0.0 {
            return 0.0;
        }
        let side = ICONS
            .get(self.icon)
            .map(|icon| icon.rows.len())
            .unwrap_or(0) as f32;
        if side <= 0.0 {
            return 0.0;
        }
        let half = side / 2.0;
        let col = ((x - self.x) / self.cell_px + half).floor() as i32;
        let row = ((y - self.y) / self.cell_px + half).floor() as i32;
        if ink(self.icon, col, row) {
            self.amp
        } else {
            0.0
        }
    }
}

/// Milliseconds until a held icon reaches full size. Longer presses stay full.
const CHARGE_MS: f32 = 700.0;

/// The icon under a finger that has not let go yet. It grows with the hold.
pub fn charge_stamp(x: f32, y: f32, held_ms: f64, cell: f32, icon: usize) -> Stamp {
    let t = (held_ms as f32 / CHARGE_MS).clamp(0.0, 1.0);
    Stamp {
        x,
        y,
        cell_px: cell * (0.34 + t * 1.15),
        amp: 0.5 + t * 0.45,
        icon,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    icon: usize,
    x: f32,
    y: f32,
    born: f64,
    life: f64,
    play: Play,
    heading: f32,
    travel: f32,
    from: f32,
    to: f32,
}

impl Sprite {
    pub fn spawn(x: f32, y: f32, now: f64, held_ms: f64, cell: f32, pick: Pick) -> Self {
        let held = (held_ms / 1000.0).clamp(0.0, 0.85) as f32;
        let n = roll(now.to_bits(), x.to_bits(), y.to_bits());
        let heading = (n as f32 / u32::MAX as f32) * std::f32::consts::TAU;
        let (from, to, travel) = match pick.play {
            Play::Appear => (cell * 0.22, cell * 0.86, 0.0),
            Play::Slide => (cell * 0.62, cell * 0.62, cell * (6.0 + held * 10.0)),
            Play::Expand => (cell * 0.42, cell * (1.7 + held * 3.4), 0.0),
        };
        Self {
            icon: pick.icon % ICONS.len(),
            x,
            y,
            born: now,
            // 560ms, plus up to 0.85 * 780ms when the press was long.
            life: 560.0 + f64::from(held) * 780.0,
            play: pick.play,
            heading,
            travel,
            from,
            to,
        }
    }

    pub fn live(&self, now: f64) -> Option<Stamp> {
        self.place(now).map(|placed| Stamp {
            x: placed.x,
            y: placed.y,
            cell_px: placed.cell_px,
            amp: placed.amp,
            icon: self.icon,
        })
    }

    /// Where the icon is, how big it is, and how strong it is.
    fn place(&self, now: f64) -> Option<Placed> {
        if self.life <= 0.0 {
            return None;
        }
        let t = ((now - self.born) / self.life).clamp(0.0, 1.0) as f32;
        if t >= 1.0 {
            return None;
        }
        let ease = 1.0 - (1.0 - t).powi(3);
        let (cell_px, amp, dist) = match self.play {
            Play::Expand => {
                let cell_px = self.from + (self.to - self.from) * ease;
                let amp = (1.0 - t).powf(1.35);
                (cell_px, amp, 0.0)
            }
            Play::Appear => {
                let pop = (t / 0.30).min(1.0);
                let pop_e = 1.0 - (1.0 - pop).powi(3);
                let cell_px = self.from + (self.to - self.from) * pop_e;
                let fade = if t < 0.55 {
                    1.0
                } else {
                    ((1.0 - t) / 0.45).powf(1.2)
                };
                (cell_px, fade, 0.0)
            }
            Play::Slide => (self.from, (1.0 - t).powf(1.1), self.travel * ease),
        };
        Some(Placed {
            x: self.x + self.heading.cos() * dist,
            y: self.y + self.heading.sin() * dist,
            cell_px,
            amp,
        })
    }
}

struct Placed {
    x: f32,
    y: f32,
    cell_px: f32,
    amp: f32,
}

fn roll(a: u64, b: u32, c: u32) -> u32 {
    let mut n = (a as u32)
        .wrapping_mul(374_761_393)
        .wrapping_add(b.wrapping_mul(668_265_263))
        .wrapping_add(c.wrapping_mul(2_246_822_519));
    n ^= n >> 16;
    n = n.wrapping_mul(0x7feb_352d);
    n ^= n >> 15;
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ink_of(rows: &[&str]) -> usize {
        rows.iter()
            .flat_map(|row| row.chars().filter(|c| *c == '#'))
            .count()
    }

    #[test]
    fn the_catalog_is_four_sparse_icons() {
        assert!(ICONS.len() >= 2);
        assert_eq!(ICONS[0].name, "bug");
        for icon in ICONS {
            let n = ink_of(icon.rows);
            assert!(n > 8 && n < 140, "{} ink {n}", icon.name);
        }
        assert!(ink(0, 7, 0), "bug antenna");
        assert!(ink(0, 7, 7));
        assert!(!ink(0, 0, 0));
        assert!(!ink(0, -1, 3));
    }

    #[test]
    fn first_cast_is_the_original_stamp_and_random_cast_varies() {
        let pinned = pick_with(Cast::First, 10.0, 4.0, 4.0);
        assert_eq!(pinned.icon, 0);
        assert_eq!(pinned.play, Play::Expand);

        let mut icons = [false; 4];
        let mut plays = [false; 3];
        for i in 0..48 {
            let pick = pick_with(Cast::Random, i as f64 * 17.0, i as f32 * 3.0, 8.0);
            icons[pick.icon] = true;
            plays[match pick.play {
                Play::Appear => 0,
                Play::Slide => 1,
                Play::Expand => 2,
            }] = true;
        }
        assert!(icons.iter().filter(|on| **on).count() >= 3, "{icons:?}");
        assert_eq!(plays, [true, true, true]);
    }

    #[test]
    fn expand_grows_appear_stays_and_slide_travels() {
        let cell = 8.0;
        let expand = Sprite::spawn(10.0, 20.0, 0.0, 400.0, cell, Pick::first());
        let (a, amp_a) = {
            let p = expand.live(0.0).unwrap();
            (p.cell_px, p.amp)
        };
        let (b, amp_b) = {
            let p = expand.live(expand.life * 0.6).unwrap();
            (p.cell_px, p.amp)
        };
        assert!(b > a);
        assert!(amp_b < amp_a);
        assert!(expand.live(expand.life).is_none());

        let appear = Sprite::spawn(
            10.0,
            20.0,
            0.0,
            200.0,
            cell,
            Pick {
                icon: 1,
                play: Play::Appear,
            },
        );
        let born = appear.live(0.0).unwrap();
        let mid = appear.live(appear.life * 0.4).unwrap();
        assert!((born.x - 10.0).abs() < 0.01 && (mid.x - 10.0).abs() < 0.01);
        assert!(mid.cell_px > born.cell_px);

        let slide = Sprite::spawn(
            30.0,
            40.0,
            0.0,
            200.0,
            cell,
            Pick {
                icon: 3,
                play: Play::Slide,
            },
        );
        let start = slide.live(0.0).unwrap();
        let later = slide.live(slide.life * 0.5).unwrap();
        let moved = (later.x - start.x).hypot(later.y - start.y);
        assert!(moved > cell, "slide moved {moved}");
        assert!((later.cell_px - start.cell_px).abs() < 0.01);
    }
}
