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
//! The numbers are in `config.rs`: `REVEAL_MS`, and the `WANDER_*` speeds.

pub use crate::config::REVEAL_MS;

pub fn reveal(now: f64, start: f64, reduced: bool) -> f32 {
    if reduced {
        return 1.0;
    }
    let t = ((now - start) / REVEAL_MS).clamp(0.0, 1.0) as f32;
    1.0 - (1.0 - t).powi(3)
}

pub fn wander(now_ms: f64, w: f32, h: f32) -> (f32, f32) {
    let t = (now_ms / 1000.0) as f32;
    let x = w * (crate::config::WANDER_CENTER_X
        + crate::config::WANDER_SWING_X * (t * crate::config::WANDER_SPEED_X).sin());
    let y = h
        * (crate::config::WANDER_CENTER_Y
            + crate::config::WANDER_SWING_Y
                * (t * crate::config::WANDER_SPEED_Y + crate::config::WANDER_PHASE_Y).sin());
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
