//! Step 5. The mouse. This file does not choose the icon or the letter fill.
//!
//! Process:
//! 1. `place` stores the latest sample. It does not move the glow yet.
//! 2. `smooth` eases the glow toward that sample. `paint` calls this once per frame.
//! 3. `press` remembers whether the finger landed on the word (`glyph`).
//! 4. `release` returns one of three results:
//!    - `Replay` — pressed on the word and released still on the word.
//!      `lib.rs` plays the next appear. No icon.
//!    - `Stamp` — pressed on the field. `lib.rs` plays the icon chosen at press.
//!    - `None` — there was no press, or the finger slid off the word.
//!
//! Quiet rectangles are the nav and the sentence. `quiet_factor` is 0 inside
//! them and 1 once `reach` away, so the field does not paint through the type.
//!
//! Config, in `smooth`:
//! - `FOLLOW` is how fast the glow position catches the cursor. `1.0` snaps.
//!   Lower lags behind.
//! - `STRENGTH` is how fast the glow fades in and out. Lower is slower.
//! - Strength under `0.004` is treated as off, so the glow can finish.

/// How fast the glow position catches the cursor. 1.0 snaps. Lower lags.
const FOLLOW: f32 = 0.38;
/// How fast the glow strength fades in and out. Lower is slower.
const STRENGTH: f32 = 0.22;

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Debug)]
pub struct Pointer {
    pub raw_x: f32,
    pub raw_y: f32,
    pub raw_inside: bool,
    pub x: f32,
    pub y: f32,
    pub strength: f32,
    pub has: bool,
    pub down: bool,
    pub glyph: bool,
    pub down_at: f64,
    pub down_x: f32,
    pub down_y: f32,
}

pub enum Release {
    None,
    Replay,
    Stamp { x: f32, y: f32, held_ms: f64 },
}

impl Pointer {
    pub fn new() -> Self {
        Self {
            raw_x: 0.0,
            raw_y: 0.0,
            raw_inside: false,
            x: 0.0,
            y: 0.0,
            strength: 0.0,
            has: false,
            down: false,
            glyph: false,
            down_at: 0.0,
            down_x: 0.0,
            down_y: 0.0,
        }
    }

    pub fn place(&mut self, x: f32, y: f32, inside: bool) {
        self.raw_x = x;
        self.raw_y = y;
        self.raw_inside = inside;
    }

    pub fn press(&mut self, x: f32, y: f32, now: f64, on_logo: bool) {
        if self.down {
            return;
        }
        self.down = true;
        self.glyph = on_logo;
        self.down_at = now;
        self.down_x = x;
        self.down_y = y;
        self.raw_x = x;
        self.raw_y = y;
        self.raw_inside = true;
    }

    pub fn release(&mut self, x: f32, y: f32, now: f64, still_on_logo: bool) -> Release {
        if !self.down {
            return Release::None;
        }
        self.down = false;
        let held_ms = (now - self.down_at).max(0.0);
        if self.glyph {
            return if still_on_logo {
                Release::Replay
            } else {
                Release::None
            };
        }
        Release::Stamp { x, y, held_ms }
    }

    pub fn cancel(&mut self) {
        self.down = false;
        self.glyph = false;
    }

    /// Ease toward the latest sample. `quiet` is 0 inside a type block.
    pub fn smooth(&mut self, quiet: f32) {
        if self.raw_inside {
            if !self.has {
                self.x = self.raw_x;
                self.y = self.raw_y;
                self.has = true;
            } else {
                self.x += (self.raw_x - self.x) * FOLLOW;
                self.y += (self.raw_y - self.y) * FOLLOW;
            }
        }
        let target = if self.raw_inside {
            quiet.clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.strength += (target - self.strength) * STRENGTH;
        if self.strength < 0.004 {
            self.strength = 0.0;
        }
    }
}

impl Default for Pointer {
    fn default() -> Self {
        Self::new()
    }
}

/// 0 inside `rects`, 1 once `reach` away. Empty list means the whole field is live.
pub fn quiet_factor(x: f32, y: f32, rects: &[Rect], reach: f32) -> f32 {
    if rects.is_empty() || reach <= 0.0 {
        return 1.0;
    }
    let mut best = f32::MAX;
    for r in rects {
        let dx = (r.x - x).max(0.0).max(x - (r.x + r.w));
        let dy = (r.y - y).max(0.0).max(y - (r.y + r.h));
        best = best.min((dx * dx + dy * dy).sqrt());
    }
    if best >= reach {
        1.0
    } else {
        (best / reach).powi(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_rects_kill_the_center_and_recover() {
        let rects = [Rect {
            x: 10.0,
            y: 10.0,
            w: 40.0,
            h: 20.0,
        }];
        assert_eq!(quiet_factor(20.0, 20.0, &rects, 30.0), 0.0);
        assert_eq!(quiet_factor(200.0, 200.0, &rects, 30.0), 1.0);
        let edge = quiet_factor(70.0, 20.0, &rects, 40.0);
        assert!(edge > 0.05 && edge < 0.5, "{edge}");
        assert_eq!(quiet_factor(0.0, 0.0, &[], 30.0), 1.0);
    }

    #[test]
    fn glyph_press_replays_and_field_press_stamps() {
        let mut p = Pointer::new();
        p.press(4.0, 4.0, 10.0, true);
        assert!(matches!(p.release(4.0, 4.0, 40.0, true), Release::Replay));
        p.press(4.0, 4.0, 10.0, false);
        match p.release(8.0, 9.0, 210.0, false) {
            Release::Stamp { x, y, held_ms } => {
                assert_eq!((x, y), (8.0, 9.0));
                assert!((held_ms - 200.0).abs() < 0.1);
            }
            Release::None | Release::Replay => panic!("expected a stamp"),
        }
        assert!(matches!(p.release(0.0, 0.0, 0.0, false), Release::None));
    }

    #[test]
    fn sliding_off_the_word_cancels_the_replay() {
        let mut p = Pointer::new();
        p.press(1.0, 1.0, 0.0, true);
        assert!(matches!(p.release(9.0, 9.0, 20.0, false), Release::None));
    }
}
