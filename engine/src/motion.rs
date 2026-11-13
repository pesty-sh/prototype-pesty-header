//! Step 6. Two clocks. This file does not read the mouse or choose a picture.
//!
//! Process:
//! 1. `reveal` is the letter clock. Every click in `appears.rs` shares it.
//!    `progress` runs from 0 to 1. The curve is ease-out: fast at the start,
//!    then it settles. Reduced motion returns 1 immediately, so the word is
//!    already finished.
//! 2. `wander` is a slow drift used when the pointer is away. `paint` skips
//!    it when reduced motion is on.
//!
//! The shape of a letter fill is `appears.rs`. How a released icon moves is
//! `sprites.rs`.
//!
//! Config:
//! - `REVEAL_MS` is the length of every appear, in milliseconds. Change it
//!   here, not per appear. 1100 is a little over a second.
//! - The two speeds inside `wander` (`0.57` and `0.31`) are radians per
//!   second along x and y. Raise them and the drift moves faster.
//!   `0.33` and `0.26` are how far it swings, as a fraction of the canvas.

/// Every appear uses this long, ease-out cubic. Change it here, not per appear.
pub const REVEAL_MS: f64 = 1100.0;

pub fn reveal(now: f64, start: f64, reduced: bool) -> f32 {
    if reduced {
        return 1.0;
    }
    let t = ((now - start) / REVEAL_MS).clamp(0.0, 1.0) as f32;
    1.0 - (1.0 - t).powi(3)
}

pub fn wander(now_ms: f64, w: f32, h: f32) -> (f32, f32) {
    let t = (now_ms / 1000.0) as f32;
    // 0.57 and 0.31 are the drift speeds. 0.33 and 0.26 are the swing.
    let x = w * (0.5 + 0.33 * (t * 0.57).sin());
    let y = h * (0.48 + 0.26 * (t * 0.31 + 1.15).sin());
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reveal_eases_and_reduced_motion_is_already_there() {
        assert_eq!(reveal(0.0, 0.0, true), 1.0);
        assert_eq!(reveal(0.0, 0.0, false), 0.0);
        let mid = reveal(REVEAL_MS * 0.5, 0.0, false);
        assert!(mid > 0.5 && mid < 1.0, "{mid}");
        assert_eq!(reveal(REVEAL_MS, 0.0, false), 1.0);
    }
}
