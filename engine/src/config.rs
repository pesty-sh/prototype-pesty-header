//! Every knob for the picture. Edit this file, then rebuild the wasm
//! (`npm run build:wasm`) and reload the page.
//!
//! The page's own knobs — how often it draws, the pixel-ratio cap — sit at
//! the top of `src/main.ts`. Colours stay in `palette.rs` and `src/style.css`.
//! Letter shapes stay in `engine/mark.py`.

// =============================================================================
// The wander. It stays a ball of pixels. It is not an icon, and it is not
// the mouse. It fires one trail on a random wait of at least five seconds.
// =============================================================================

/// `false` keeps the ball and stops the firing.
pub const DRIFT_FIRE: bool = true;

/// Shortest and longest wait between fires, in milliseconds.
/// Each wait is rolled at random. Nothing fires faster than once per five seconds.
pub const DRIFT_GAP_MIN_MS: f64 = 5_000.0;
pub const DRIFT_GAP_MAX_MS: f64 = 8_000.0;

/// How many pixels are in the trail.
pub const TRAIL_DOTS: usize = 18;

/// How wide the trail is, in letter-cells, at the tail.
pub const TRAIL_SPREAD: f32 = 2.8;

/// How long the trail flies and then bursts, in milliseconds.
pub const POP_LIFE_MS: f64 = 1_500.0;

/// Share of that life spent as pixels. The rest is the sprite bursting
/// on the landing.
pub const POP_LAND_AT: f32 = 0.72;

/// Size of the burst sprite, in letter-cells per glyph pixel, before it grows.
pub const POP_CELL: f32 = 0.7;

/// The burst opens at least this far inside the canvas, as a fraction of
/// the smaller side. The landing is still a random point. This stops the
/// icon from opening on the top edge and painting off the picture.
pub const POP_EDGE: f32 = 0.42;

/// How many icons can be alive at once. Clicks and pops share this cap.
/// The oldest one is dropped.
pub const MAX_SPRITES: usize = 8;

// The route that sprite follows. Speeds are radians per second.
// Swings are fractions of the canvas, out from the middle.
pub const WANDER_SPEED_X: f32 = 0.57;
pub const WANDER_SPEED_Y: f32 = 0.31;
pub const WANDER_SWING_X: f32 = 0.33;
pub const WANDER_SWING_Y: f32 = 0.26;
/// Where the path sits, as a fraction of the canvas. `0.5` is the middle.
pub const WANDER_CENTER_X: f32 = 0.5;
pub const WANDER_CENTER_Y: f32 = 0.48;
/// Phase offset so x and y do not peak together.
pub const WANDER_PHASE_Y: f32 = 1.15;

// =============================================================================
// Letter fill
// =============================================================================

/// How long every appear takes, in milliseconds.
pub const REVEAL_MS: f64 = 1100.0;

/// `false` walks the list in `appears.rs`. `true` rolls the next fill.
pub const APPEAR_RANDOM: bool = false;

// Where each letter colour ends, as a fraction of the letter height.
// `0` is the top. The last band, dim, is whatever is left.
pub const BAND_CREST: f32 = 0.26;
pub const BAND_HOVER: f32 = 0.37;
pub const BAND_LIT: f32 = 0.58;
pub const BAND_MID: f32 = 0.74;

// =============================================================================
// Click icons
// =============================================================================

/// `true` rolls the icon and the play. `false` is always the bug, expanding.
pub const CAST_RANDOM: bool = true;

/// Milliseconds until a held icon reaches full size.
pub const CHARGE_MS: f32 = 700.0;

/// A click icon lives this long, plus `CLICK_LIFE_HOLD` times the hold
/// (the hold is capped at 0.85 seconds). A slide uses the slide pair.
pub const CLICK_LIFE_MS: f64 = 560.0;
pub const CLICK_LIFE_HOLD: f64 = 780.0;
pub const SLIDE_LIFE_MS: f64 = 1280.0;
pub const SLIDE_LIFE_HOLD: f64 = 820.0;

/// How many times a slide sets down.
pub const SLIDE_HOPS: usize = 3;
/// Smaller icons a slide spits while it travels.
pub const SLIDE_SPIT: usize = 6;
/// Fraction of each hop spent flying. The rest is the burst.
pub const SLIDE_LAND_AT: f32 = 0.78;

// =============================================================================
// Pointer
// =============================================================================

/// How fast the glow position catches the cursor. `1` snaps. Lower lags.
pub const POINTER_FOLLOW: f32 = 0.38;
/// How fast the glow fades in and out. Lower is slower.
pub const POINTER_STRENGTH: f32 = 0.22;
/// Strength below this is treated as off.
pub const POINTER_OFF: f32 = 0.004;

// =============================================================================
// One frame: glow sizes, in letter-cells
// =============================================================================

/// How far past a quiet box the glow fades back in.
pub const QUIET_REACH_CELLS: f32 = 7.0;
/// Pointer glow radius, plus `POINTER_REACH_EXTRA` more at full strength.
pub const POINTER_REACH_CELLS: f32 = 6.5;
pub const POINTER_REACH_EXTRA: f32 = 8.0;
/// The drift glow. It dims by `DRIFT_GLOW_DIM` while the pointer is strong.
pub const DRIFT_GLOW: f32 = 0.24;
pub const DRIFT_GLOW_DIM: f32 = 0.7;
pub const DRIFT_GLOW_REACH_CELLS: f32 = 8.0;

// =============================================================================
// Speckle
// =============================================================================

/// Cells per second the noise scrolls.
pub const NOISE_SCROLL_X: f32 = 0.14;
pub const NOISE_SCROLL_Y: f32 = 0.055;
/// Overall density. Lower is sparser.
pub const NOISE_DENSITY: f32 = 0.78;
/// Radius of the cursor well, in letter-cells. Speckles inside are
/// picked up. Speckles outside stay on their own point.
pub const NOISE_WELL_CELLS: f32 = 20.0;
/// How hard the well pulls. `0` leaves every point where it is.
/// Higher packs the disk onto the cursor and leaves the rest of the
/// radius empty. Kept under 1 so a point can fall back as the pointer leaves.
pub const NOISE_WELL_PULL: f32 = 0.88;
/// Cap on how much gathered noise can brighten the cursor. The points
/// themselves are not moved. This only keeps the hoovered clump visible.
pub const NOISE_WELL_MASS: f32 = 3.2;
/// Radius where the edges start to fill in.
pub const VIGNETTE_RADIUS: f32 = 0.34;
/// Fraction of the header height the top fade takes.
pub const VIGNETTE_TOP: f32 = 0.22;
