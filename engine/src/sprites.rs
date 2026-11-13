//! Step 4. The icons.
//!
//! A press on the field still chooses one, and the release plays it.
//! Separately, the wander stays a ball of pixels. On a random wait of at
//! least five seconds it fires one trail of pixels. The trail lands at a
//! random point inside the canvas and bursts into a random icon.
//! The wait, the trail, and the burst are the first group in `config.rs`.
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
//!   was long (`Sprite::spawn`, `life`). A slide lives about 1.3s, plus up to
//!   about 0.8s more, so its landings can finish.
//! - Five live icons is the cap. That limit is in `lib.rs`, `spawn_sprite`.
//!   The smaller icons a slide spits out are drawn with that slide. They are
//!   not extra entries, and they do not come from the clock.
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
//! - `Slide` spins across the header. On the way it spits smaller random
//!   icons, and it bursts each time it lands. Those spat icons fly to their
//!   own landing and burst too.
//! - `Expand` grows in place and thins out. This is the original stamp.

/// `Random` rolls an icon and a play. `First` is bug + expand, every time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cast {
    Random,
    /// Assign this to [`CAST`] to drop the bug with [`Play::Expand`] every time.
    #[allow(dead_code)]
    First,
}

/// `CAST_RANDOM` in `config.rs` picks this. `Random` rolls. `First` is the bug.
pub const CAST: Cast = if crate::config::CAST_RANDOM {
    Cast::Random
} else {
    Cast::First
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Play {
    Appear,
    Slide,
    Expand,
    /// Flies to a random landing and bursts. The drifter pops these.
    /// A click does not roll it.
    Pop,
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
    /// Radians. Zero leaves the glyph unrotated, which is the sample the
    /// appear, the expand, and the charge still use.
    pub angle: f32,
    /// A single field cell, not a glyph. The wander's trail uses this.
    pub pixels: bool,
}

impl Stamp {
    pub fn amp_at(&self, x: f32, y: f32) -> f32 {
        if self.cell_px <= 0.25 || self.amp <= 0.0 {
            return 0.0;
        }
        if self.pixels {
            let dx = (x - self.x).abs();
            let dy = (y - self.y).abs();
            return if dx <= self.cell_px * 0.55 && dy <= self.cell_px * 0.55 {
                self.amp
            } else {
                0.0
            };
        }
        let side = ICONS
            .get(self.icon)
            .map(|icon| icon.rows.len())
            .unwrap_or(0) as f32;
        if side <= 0.0 {
            return 0.0;
        }
        let half = side / 2.0;
        let (dx, dy) = if self.angle == 0.0 {
            (x - self.x, y - self.y)
        } else {
            let dx = x - self.x;
            let dy = y - self.y;
            let c = self.angle.cos();
            let s = self.angle.sin();
            (dx * c + dy * s, -dx * s + dy * c)
        };
        let col = (dx / self.cell_px + half).floor() as i32;
        let row = (dy / self.cell_px + half).floor() as i32;
        if ink(self.icon, col, row) {
            self.amp
        } else {
            0.0
        }
    }
}

use crate::config::CHARGE_MS;

/// The icon under a finger that has not let go yet. It grows with the hold.
pub fn charge_stamp(x: f32, y: f32, held_ms: f64, cell: f32, icon: usize) -> Stamp {
    let t = (held_ms as f32 / CHARGE_MS).clamp(0.0, 1.0);
    Stamp {
        x,
        y,
        cell_px: cell * (0.34 + t * 1.15),
        amp: 0.5 + t * 0.45,
        icon,
        angle: 0.0,
        pixels: false,
    }
}

pub(crate) fn mix_point(now: f64, x: f32, y: f32) -> u32 {
    roll(now.to_bits(), x.to_bits(), y.to_bits())
}

use crate::config::{
    CLICK_LIFE_HOLD, CLICK_LIFE_MS, POP_CELL, POP_EDGE, POP_LAND_AT, POP_LIFE_MS, SLIDE_HOPS,
    SLIDE_LAND_AT, SLIDE_LIFE_HOLD, SLIDE_LIFE_MS, SLIDE_SPIT, TRAIL_DOTS, TRAIL_SPREAD,
};

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
    field_w: f32,
    field_h: f32,
    land_x: f32,
    land_y: f32,
}

impl Sprite {
    pub fn spawn(
        x: f32,
        y: f32,
        now: f64,
        held_ms: f64,
        cell: f32,
        field_w: f32,
        field_h: f32,
        pick: Pick,
    ) -> Self {
        let held = (held_ms / 1000.0).clamp(0.0, 0.85) as f32;
        let n = roll(now.to_bits(), x.to_bits(), y.to_bits());
        let heading = (n as f32 / u32::MAX as f32) * std::f32::consts::TAU;
        let (from, to, travel) = match pick.play {
            Play::Appear => (cell * 0.22, cell * 0.86, 0.0),
            Play::Slide => (cell * 0.62, cell * 0.62, cell * (8.0 + held * 12.0)),
            Play::Expand | Play::Pop => (cell * 0.42, cell * (1.7 + held * 3.4), 0.0),
        };
        let life = match pick.play {
            Play::Slide => SLIDE_LIFE_MS + f64::from(held) * SLIDE_LIFE_HOLD,
            _ => CLICK_LIFE_MS + f64::from(held) * CLICK_LIFE_HOLD,
        };
        Self {
            icon: pick.icon % ICONS.len(),
            x,
            y,
            born: now,
            life,
            play: pick.play,
            heading,
            travel,
            from,
            to,
            field_w,
            field_h,
            land_x: x,
            land_y: y,
        }
    }

    /// A trail the wander fires. Pixels fly to a random point, then a random
    /// icon bursts there.
    pub fn pop(
        x: f32,
        y: f32,
        now: f64,
        cell: f32,
        field_w: f32,
        field_h: f32,
        icon: usize,
    ) -> Self {
        let n = roll(now.to_bits(), x.to_bits(), y.to_bits());
        let mut sprite = Self {
            icon: icon % ICONS.len(),
            x,
            y,
            born: now,
            life: POP_LIFE_MS,
            play: Play::Pop,
            heading: unit(n) * std::f32::consts::TAU,
            travel: cell * 8.0,
            from: cell * POP_CELL,
            to: cell * POP_CELL,
            field_w,
            field_h,
            land_x: x,
            land_y: y,
        };
        let (lx, ly) = sprite.pop_landing();
        sprite.land_x = lx;
        sprite.land_y = ly;
        sprite
    }

    pub fn live(&self, now: f64) -> Option<Stamp> {
        self.age(now).map(|t| self.place(t).stamp())
    }

    /// The icon, and for a slide the smaller icons it has spat out.
    pub fn push_stamps(&self, now: f64, out: &mut Vec<Stamp>) {
        let Some(t) = self.age(now) else {
            return;
        };
        if self.play == Play::Pop {
            self.push_pop(t, out);
            return;
        }
        out.push(self.place(t).stamp());
        if self.play == Play::Slide {
            self.push_spit(t, out);
        }
    }

    /// Pixels along the path, then the icon on the landing.
    fn push_pop(&self, t: f32, out: &mut Vec<Stamp>) {
        let fly = (t / POP_LAND_AT).clamp(0.0, 1.0);
        let ease = 1.0 - (1.0 - fly).powi(3);
        let boom = ((t - POP_LAND_AT) / (1.0 - POP_LAND_AT)).clamp(0.0, 1.0);
        if boom <= 0.0 {
            let speck = self.from / POP_CELL.max(0.01);
            for i in 0..TRAIL_DOTS {
                let n = self.mix(i + 40, 0xB529_7A4D);
                let slot = (i as f32 + 0.5) / TRAIL_DOTS as f32;
                let along = ease * slot;
                let want = if i % 2 == 0 { 0.28 } else { -0.28 };
                let curl = self.bounded_curl(want);
                let (bx, by) = arc(self.x, self.y, self.land_x, self.land_y, curl, along);
                let spread = speck * TRAIL_SPREAD * (1.0 - along);
                let ox = (unit(n) - 0.5) * 2.0 * spread;
                let oy = (unit(n >> 8) - 0.5) * 2.0 * spread;
                out.push(Stamp {
                    x: bx + ox,
                    y: by + oy,
                    cell_px: speck * 0.95,
                    amp: 0.75 + unit(n >> 16) * 0.45,
                    icon: 0,
                    angle: 0.0,
                    pixels: true,
                });
            }
            return;
        }
        let cell_px = self.from * (1.15 + boom * boom * 4.2);
        let amp = (1.0 - boom).powf(0.35) * 1.4;
        out.push(Stamp {
            x: self.land_x,
            y: self.land_y,
            cell_px,
            amp,
            icon: self.icon,
            angle: self.heading + boom * std::f32::consts::TAU,
            pixels: false,
        });
    }

    fn age(&self, now: f64) -> Option<f32> {
        if self.life <= 0.0 {
            return None;
        }
        let t = ((now - self.born) / self.life).clamp(0.0, 1.0) as f32;
        if t >= 1.0 {
            None
        } else {
            Some(t)
        }
    }

    /// Where the icon is, how big it is, and how strong it is.
    fn place(&self, t: f32) -> Placed {
        if self.play == Play::Slide {
            let (x, y, cell_px, amp) = self.slide_at(t);
            return Placed {
                x,
                y,
                cell_px,
                amp,
                icon: self.icon,
                angle: self.heading + t * std::f32::consts::TAU * 3.0,
            };
        }
        if self.play == Play::Pop {
            let fly = (t / POP_LAND_AT).clamp(0.0, 1.0);
            let ease = 1.0 - (1.0 - fly).powi(3);
            let want = if self.heading.sin() >= 0.0 { 0.42 } else { -0.42 };
            let curl = self.bounded_curl(want);
            let (x, y) = arc(self.x, self.y, self.land_x, self.land_y, curl, ease);
            let boom = ((t - POP_LAND_AT) / (1.0 - POP_LAND_AT)).clamp(0.0, 1.0);
            let cell_px = self.from * (1.0 + boom * boom * 3.6);
            let amp = if boom <= 0.0 {
                1.0
            } else {
                (1.0 - boom).powf(0.35) * 1.4
            };
            return Placed {
                x,
                y,
                cell_px,
                amp,
                icon: self.icon,
                angle: self.heading + t * std::f32::consts::TAU * 2.5,
            };
        }
        let ease = 1.0 - (1.0 - t).powi(3);
        let (cell_px, amp) = match self.play {
            Play::Expand => {
                let cell_px = self.from + (self.to - self.from) * ease;
                let amp = (1.0 - t).powf(1.35);
                (cell_px, amp)
            }
            _ => {
                let pop = (t / 0.30).min(1.0);
                let pop_e = 1.0 - (1.0 - pop).powi(3);
                let cell_px = self.from + (self.to - self.from) * pop_e;
                let fade = if t < 0.55 {
                    1.0
                } else {
                    ((1.0 - t) / 0.45).powf(1.2)
                };
                (cell_px, fade)
            }
        };
        Placed {
            x: self.x,
            y: self.y,
            cell_px,
            amp,
            icon: self.icon,
            angle: 0.0,
        }
    }

    /// Position, size, and strength of a slide. It arcs to a random point on
    /// the header, bursts there, then takes off toward the next point.
    fn slide_at(&self, t: f32) -> (f32, f32, f32, f32) {
        let hops = SLIDE_HOPS as f32;
        let slot = (t * hops).clamp(0.0, hops - f32::EPSILON);
        let hop = slot.floor() as usize;
        let u = slot - hop as f32;
        let (x0, y0) = if hop == 0 {
            (self.x, self.y)
        } else {
            self.landing(hop - 1)
        };
        let (x1, y1) = self.landing(hop);
        let travel_u = (u / SLIDE_LAND_AT).clamp(0.0, 1.0);
        let ease = 1.0 - (1.0 - travel_u).powi(3);
        let curl = if hop % 2 == 0 { 0.48 } else { -0.48 };
        let (x, y) = arc(x0, y0, x1, y1, curl, ease);
        let boom = ((u - SLIDE_LAND_AT) / (1.0 - SLIDE_LAND_AT)).clamp(0.0, 1.0);
        let cell_px = self.from * (1.0 + boom * boom * 3.2);
        let fade = (1.0 - t).powf(0.85);
        let amp = if boom <= 0.0 {
            fade
        } else {
            let flash = (1.0 - boom).powf(0.35);
            fade * (0.2 + 1.35 * flash)
        };
        (x, y, cell_px, amp)
    }

    /// A random point that stays inside the canvas, far enough from the
    /// edge that the burst icon opens on the picture.
    fn pop_landing(&self) -> (f32, f32) {
        let n = self.mix(0, 0xC2B2_AE35);
        if self.field_w < 32.0 || self.field_h < 32.0 {
            return (self.field_w * 0.5, self.field_h * 0.5);
        }
        let margin = self.pop_margin();
        let x = margin + unit(n) * (self.field_w - margin * 2.0).max(1.0);
        let y = margin + unit(n >> 8) * (self.field_h - margin * 2.0).max(1.0);
        (x, y)
    }

    /// Half the icon at the size the burst reaches, capped so a small
    /// canvas still has a span of random landings.
    fn pop_margin(&self) -> f32 {
        let side = ICONS.first().map(|icon| icon.rows.len()).unwrap_or(15) as f32;
        let peak = self.from * (1.15 + 4.2);
        let reach = peak * side * 0.5;
        let cap = self.field_w.min(self.field_h) * POP_EDGE.clamp(0.05, 0.48);
        reach.min(cap).max(8.0)
    }

    /// Bend the flight, but keep the arc's control point on the canvas
    /// so the trail does not climb off the top.
    fn bounded_curl(&self, want: f32) -> f32 {
        if self.field_w < 32.0 || self.field_h < 32.0 {
            return want.clamp(-0.2, 0.2);
        }
        let dx = self.land_x - self.x;
        let dy = self.land_y - self.y;
        let mx0 = (self.x + self.land_x) * 0.5;
        let my0 = (self.y + self.land_y) * 0.5;
        let mut lo = -0.45_f32;
        let mut hi = 0.45_f32;
        if dx.abs() > 1.0 {
            let a = -my0 / dx;
            let b = (self.field_h - my0) / dx;
            lo = lo.max(a.min(b));
            hi = hi.min(a.max(b));
        }
        if dy.abs() > 1.0 {
            let a = (mx0 - self.field_w) / dy;
            let b = mx0 / dy;
            lo = lo.max(a.min(b));
            hi = hi.min(a.max(b));
        }
        if lo > hi {
            return 0.0;
        }
        want.clamp(lo, hi)
    }

    /// A random point on the header. A tiny field falls back to a step away
    /// from the release, so a sprite still travels before the canvas exists.
    fn landing(&self, hop: usize) -> (f32, f32) {
        let n = self.mix(hop, 0x9E37_79B9);
        if self.field_w < 32.0 || self.field_h < 32.0 {
            let ang = unit(n) * std::f32::consts::TAU;
            let dist = self.travel * (0.45 + unit(n >> 8) * 0.55);
            return (self.x + ang.cos() * dist, self.y + ang.sin() * dist);
        }
        let pad = (self.travel * 0.15).clamp(12.0, 48.0);
        let x = pad + unit(n) * (self.field_w - pad * 2.0).max(1.0);
        let y = pad + unit(n >> 8) * (self.field_h - pad * 2.0).max(1.0);
        (x, y)
    }

    fn push_spit(&self, t: f32, out: &mut Vec<Stamp>) {
        for i in 0..SLIDE_SPIT {
            let emit = 0.05 + (i as f32) * (0.80 / SLIDE_SPIT as f32);
            if t < emit {
                continue;
            }
            let span = 0.30;
            let age = (t - emit) / span;
            if age >= 1.0 {
                continue;
            }
            let (bx, by, _, _) = self.slide_at(emit);
            let n = self.mix(i + 16, 0xA5A5_5A5A);
            let ang = unit(n) * std::f32::consts::TAU;
            let dist = self.travel * (0.55 + unit(n >> 6) * 0.7);
            let lx = bx + ang.cos() * dist;
            let ly = by + ang.sin() * dist;
            let ease = 1.0 - (1.0 - age).powi(2);
            let curl = if i % 2 == 0 { 0.35 } else { -0.35 };
            let (x, y) = arc(bx, by, lx, ly, curl, ease);
            let boom = ((age - 0.58) / 0.42).clamp(0.0, 1.0);
            let cell_px = self.from * 0.72 * (1.0 + boom * boom * 2.8);
            let amp = (1.0 - age * 0.25).max(0.2) * (1.0 + boom * 0.65);
            out.push(Stamp {
                x,
                y,
                cell_px,
                amp,
                icon: (n as usize) % ICONS.len(),
                angle: ang + age * std::f32::consts::TAU * 2.0,
                pixels: false,
            });
        }
    }

    fn mix(&self, hop: usize, salt: u64) -> u32 {
        roll(
            self.born
                .to_bits()
                .wrapping_add((hop as u64).wrapping_mul(salt)),
            self.x
                .to_bits()
                .wrapping_add((hop as u32).wrapping_mul(0x27D4_EB2F)),
            self.y.to_bits(),
        )
    }
}

fn unit(n: u32) -> f32 {
    (n as f32) / (u32::MAX as f32)
}

fn arc(x0: f32, y0: f32, x1: f32, y1: f32, curl: f32, u: f32) -> (f32, f32) {
    let mx = (x0 + x1) * 0.5 - (y1 - y0) * curl;
    let my = (y0 + y1) * 0.5 + (x1 - x0) * curl;
    let omu = 1.0 - u;
    (
        omu * omu * x0 + 2.0 * omu * u * mx + u * u * x1,
        omu * omu * y0 + 2.0 * omu * u * my + u * u * y1,
    )
}

struct Placed {
    x: f32,
    y: f32,
    cell_px: f32,
    amp: f32,
    icon: usize,
    angle: f32,
}

impl Placed {
    fn stamp(self) -> Stamp {
        Stamp {
            x: self.x,
            y: self.y,
            cell_px: self.cell_px,
            amp: self.amp,
            icon: self.icon,
            angle: self.angle,
            pixels: false,
        }
    }
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
                Play::Pop => continue,
            }] = true;
        }
        assert!(icons.iter().filter(|on| **on).count() >= 3, "{icons:?}");
        assert_eq!(plays, [true, true, true]);
    }

    #[test]
    fn expand_grows_appear_stays_and_slide_travels() {
        let cell = 8.0;
        let expand = Sprite::spawn(10.0, 20.0, 0.0, 400.0, cell, 0.0, 0.0, Pick::first());
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
            0.0,
            0.0,
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
            0.0,
            0.0,
            0.0,
            200.0,
            cell,
            480.0,
            260.0,
            Pick {
                icon: 3,
                play: Play::Slide,
            },
        );
        let start = slide.live(0.0).unwrap();
        let later = slide.live(slide.life * 0.5).unwrap();
        let moved = (later.x - start.x).hypot(later.y - start.y);
        assert!(moved > cell, "slide moved {moved}");
        // Mid-hop is flight. The burst is a separate check.
        assert!((later.cell_px - start.cell_px).abs() < 0.01);
        assert!(later.angle.abs() > 0.5, "spin {}", later.angle);
    }

    #[test]
    fn slide_spits_random_icons_and_explodes_on_a_landing() {
        let cell = 8.0;
        let slide = Sprite::spawn(
            0.0,
            0.0,
            0.0,
            200.0,
            cell,
            480.0,
            260.0,
            Pick {
                icon: 0,
                play: Play::Slide,
            },
        );
        let origin = slide.live(0.0).unwrap();
        let flight = slide.live(slide.life * 0.08).unwrap();
        let burst = slide.live(slide.life * 0.31).unwrap();
        assert!(
            burst.cell_px > flight.cell_px * 1.35,
            "burst {} flight {}",
            burst.cell_px,
            flight.cell_px
        );
        let turn = (burst.angle - flight.angle).abs();
        assert!(turn > 1.0, "spin {turn}");
        let hopped = (burst.x - origin.x).hypot(burst.y - origin.y);
        assert!(hopped > cell, "landing stayed put {hopped}");

        let mut icons = Vec::new();
        let mut travelled = false;
        for step in [0.22_f64, 0.45, 0.68, 0.88] {
            let mut stamps = Vec::new();
            slide.push_stamps(slide.life * step, &mut stamps);
            assert!(stamps.len() >= 2, "spit at {step}: {}", stamps.len());
            let parent = &stamps[0];
            for spit in stamps.iter().skip(1) {
                icons.push(spit.icon);
                let away = (spit.x - parent.x).hypot(spit.y - parent.y);
                if away > cell * 2.0 {
                    travelled = true;
                }
            }
        }
        assert!(travelled, "no spat icon left the parent");
        icons.sort_unstable();
        icons.dedup();
        assert!(icons.len() >= 2, "spit icons {icons:?}");
    }

    #[test]
    fn a_pop_flies_to_a_landing_and_bursts() {
        let cell = 8.0;
        let pop = Sprite::pop(0.0, 0.0, 0.0, cell, 480.0, 260.0, 2);
        let start = pop.live(0.0).unwrap();
        let mid = pop.live(pop.life * 0.35).unwrap();
        let burst = pop.live(pop.life * 0.9).unwrap();
        let moved = (mid.x - start.x).hypot(mid.y - start.y);
        assert!(moved > cell, "pop moved {moved}");
        assert!(burst.cell_px > mid.cell_px * 1.3, "burst {}", burst.cell_px);
        assert!(pop.live(pop.life).is_none());
        assert_eq!(start.icon, 2);

        let mut flight = Vec::new();
        pop.push_stamps(pop.life * 0.3, &mut flight);
        assert!(flight.len() > 4, "trail {}", flight.len());
        assert!(flight.iter().all(|stamp| stamp.pixels));
        let mut boom = Vec::new();
        pop.push_stamps(pop.life * 0.9, &mut boom);
        assert_eq!(boom.len(), 1);
        assert!(!boom[0].pixels);
        assert_eq!(boom[0].icon, 2);

        for i in 0..24 {
            let shot = Sprite::pop(200.0, 130.0, i as f64 * 97.0, cell, 480.0, 260.0, i % 4);
            let mut burst = Vec::new();
            shot.push_stamps(shot.life * 0.9, &mut burst);
            let stamp = burst[0];
            assert!(
                stamp.x > 40.0 && stamp.x < 440.0 && stamp.y > 40.0 && stamp.y < 220.0,
                "burst left the canvas at {}, {}",
                stamp.x,
                stamp.y
            );
        }
    }
}
