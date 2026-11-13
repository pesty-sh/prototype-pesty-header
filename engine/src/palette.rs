//! Step 1. The six colours. Change a colour here first.
//!
//! Process:
//! 1. The constants below are the only inks the field paints.
//! 2. Letters use them from crest (top of the letters) down to dim (the tails).
//!    The row cuts are in `logo.rs`.
//! 3. The speckled ground uses dim, mid, and lit. It never uses crest.
//! 4. A demo tint keeps each colour's brightness and takes the hue of the hex
//!    you type. Typing the default tint or background clears that override.
//!
//! Config. Each value is red, green, blue, then 255. Leave the 255.
//! The canvas does not use transparency. A byte is 0–255, so hex `12` is `0x12`.
//!
//! Copy BG, LIT, MID, and CREST into `src/style.css` or the page and the
//! picture will disagree:
//! - `BG` `#12140f` is `--bg`. It is also the background that means "no override".
//! - `DIM` `#3e4f28` is the darkest ink. Letter tails, and quiet field cells.
//! - `MID` `#739238` is `--mid`.
//! - `LIT` `#d6ee6a` is `--lit`. It is also the tint that means "no override".
//! - `HOVER` `#e7f7a4` is the letter rows just under the crest. No page variable.
//! - `CREST` `#f7ffd4` is `--crest`. The brightest letter tops.

/// Ground. Page variable `--bg`. Typing this hex clears a demo background.
pub const BG: [u8; 4] = [0x12, 0x14, 0x0f, 0xff];
/// Darkest ink. Letter tails, and field cells with little glow.
pub const DIM: [u8; 4] = [0x3e, 0x4f, 0x28, 0xff];
/// Middle ink. Page variable `--mid`.
pub const MID: [u8; 4] = [0x73, 0x92, 0x38, 0xff];
/// Bright ink. Page variable `--lit`. Typing this hex clears a demo tint.
pub const LIT: [u8; 4] = [0xd6, 0xee, 0x6a, 0xff];
/// Letter rows just under the crest. The field does not use this.
pub const HOVER: [u8; 4] = [0xe7, 0xf7, 0xa4, 0xff];
/// Brightest letter tops. Page variable `--crest`.
pub const CREST: [u8; 4] = [0xf7, 0xff, 0xd4, 0xff];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Band {
    Crest,
    Hover,
    Lit,
    Mid,
    Dim,
}

pub fn band_color(band: Band) -> [u8; 4] {
    match band {
        Band::Crest => CREST,
        Band::Hover => HOVER,
        Band::Lit => LIT,
        Band::Mid => MID,
        Band::Dim => DIM,
    }
}

pub fn mix(a: [u8; 4], b: [u8; 4], t: f32) -> [u8; 4] {
    let t = t.clamp(0.0, 1.0);
    let chan = |i: usize| (a[i] as f32 + (b[i] as f32 - a[i] as f32) * t).round() as u8;
    [chan(0), chan(1), chan(2), 0xff]
}

/// A band recoloured by `tint`, or the baked band when `tint` is absent.
/// Lightness stays with the band. Hue and saturation come from the tint.
pub fn ink(band: Band, tint: Option<[u8; 4]>) -> [u8; 4] {
    match tint {
        None => band_color(band),
        Some(tint) => recolor(band_color(band), tint),
    }
}

/// `#rgb`, `rgb`, `#rrggbb`, or `rrggbb`. Anything else is refused.
pub fn parse_hex(input: &str) -> Option<[u8; 4]> {
    let s = input.trim();
    let s = s.strip_prefix('#').unwrap_or(s);
    let bytes = s.as_bytes();
    if bytes.len() == 3 {
        let r = nibble(bytes[0])?;
        let g = nibble(bytes[1])?;
        let b = nibble(bytes[2])?;
        return Some([r * 17, g * 17, b * 17, 0xff]);
    }
    if bytes.len() != 6 {
        return None;
    }
    let r = byte_at(bytes, 0)?;
    let g = byte_at(bytes, 2)?;
    let b = byte_at(bytes, 4)?;
    Some([r, g, b, 0xff])
}

fn nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

fn byte_at(bytes: &[u8], i: usize) -> Option<u8> {
    Some(nibble(bytes[i])? * 16 + nibble(bytes[i + 1])?)
}

fn recolor(sample: [u8; 4], tint: [u8; 4]) -> [u8; 4] {
    let (_, _, light) = rgb_hsl(sample);
    let (hue, sat, _) = rgb_hsl(tint);
    hsl_rgb(hue, sat, light)
}

fn rgb_hsl(rgb: [u8; 4]) -> (f32, f32, f32) {
    let r = rgb[0] as f32 / 255.0;
    let g = rgb[1] as f32 / 255.0;
    let b = rgb[2] as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let light = (max + min) * 0.5;
    let delta = max - min;
    if delta < 1.0e-6 {
        return (0.0, 0.0, light);
    }
    let sat = if light > 0.5 {
        delta / (2.0 - max - min)
    } else {
        delta / (max + min)
    };
    let hue = if max == r {
        (g - b) / delta + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    (hue / 6.0, sat, light)
}

fn hsl_rgb(hue: f32, sat: f32, light: f32) -> [u8; 4] {
    if sat <= 0.0 {
        let v = (light * 255.0).round() as u8;
        return [v, v, v, 0xff];
    }
    let q = if light < 0.5 {
        light * (1.0 + sat)
    } else {
        light + sat - light * sat
    };
    let p = 2.0 * light - q;
    let chan = |offset: f32| {
        let mut t = hue + offset;
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        let v = if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        };
        (v * 255.0).round() as u8
    };
    [chan(1.0 / 3.0), chan(0.0), chan(-1.0 / 3.0), 0xff]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_strings_parse() {
        assert_eq!(parse_hex("#d6ee6a"), Some(LIT));
        assert_eq!(parse_hex("12140F"), Some(BG));
        assert_eq!(parse_hex("  #abc "), Some([0xaa, 0xbb, 0xcc, 0xff]));
        assert_eq!(parse_hex("#ff6040"), Some([255, 96, 64, 0xff]));
        assert_eq!(parse_hex("nope"), None);
        assert_eq!(parse_hex("#12"), None);
        assert_eq!(parse_hex(""), None);
    }

    #[test]
    fn a_tint_keeps_the_band_lightness() {
        assert_eq!(ink(Band::Crest, None), CREST);
        let red = [255, 0, 0, 255];
        let tinted = ink(Band::Crest, Some(red));
        assert_ne!(tinted, CREST);
        assert!(tinted[0] > tinted[1] && tinted[0] > tinted[2]);
        let (_, _, crest_l) = rgb_hsl(CREST);
        let (_, _, tinted_l) = rgb_hsl(tinted);
        assert!((crest_l - tinted_l).abs() < 0.02);
    }
}
