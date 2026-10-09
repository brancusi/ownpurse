//! Colour arithmetic for the theme engine: truecolor values, `mix`, WCAG contrast. Colour, never money:
//! the float arithmetic here is allowed (relative luminance needs a power function).
#![allow(clippy::float_arithmetic)]

/// A truecolor value, `0xRRGGBB`.
pub type Rgb = u32;

pub fn r(c: Rgb) -> u8 {
    (c >> 16) as u8
}
pub fn g(c: Rgb) -> u8 {
    (c >> 8) as u8
}
pub fn b(c: Rgb) -> u8 {
    c as u8
}

pub fn rgb(r: u8, g: u8, b: u8) -> Rgb {
    (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
}

/// `#rrggbb` (or `rrggbb`) → value.
pub fn parse_hex(s: &str) -> Option<Rgb> {
    let h = s.strip_prefix('#').unwrap_or(s);
    if h.len() != 6 || !h.bytes().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    u32::from_str_radix(h, 16).ok()
}

pub fn hex(c: Rgb) -> String {
    format!("#{c:06X}")
}

/// `a` blended toward `b` by `t` per mille (0 = a, 1000 = b), rounded per channel. Integer maths, so a
/// theme resolves to the same bytes everywhere.
pub fn mix(a: Rgb, b_: Rgb, t: u16) -> Rgb {
    let t = u32::from(t.min(1000));
    let ch = |x: u8, y: u8| -> u8 {
        let v = u32::from(x) * (1000 - t) + u32::from(y) * t;
        ((v + 500) / 1000) as u8
    };
    rgb(ch(r(a), r(b_)), ch(g(a), g(b_)), ch(b(a), b(b_)))
}

/// WCAG relative luminance.
pub fn luminance(c: Rgb) -> f64 {
    let f = |v: u8| {
        let s = f64::from(v) / 255.0;
        if s <= 0.03928 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * f(r(c)) + 0.7152 * f(g(c)) + 0.0722 * f(b(c))
}

/// WCAG contrast ratio (1.0–21.0).
pub fn contrast(a: Rgb, b_: Rgb) -> f64 {
    let (x, y) = (luminance(a), luminance(b_));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

/// The nearest xterm-256 index (the 6×6×6 cube or the grey ramp), for the 256-colour mode.
pub fn xterm256(c: Rgb) -> u8 {
    let steps = [0u8, 95, 135, 175, 215, 255];
    let near = |v: u8| -> (usize, u8) {
        let mut best = 0;
        for (i, s) in steps.iter().enumerate() {
            if s.abs_diff(v) < steps[best].abs_diff(v) {
                best = i;
            }
        }
        (best, steps[best])
    };
    let (ri, rv) = near(r(c));
    let (gi, gv) = near(g(c));
    let (bi, bv) = near(b(c));
    let cube = 16 + 36 * ri + 6 * gi + bi;
    let dist = |x: (u8, u8, u8)| -> u32 {
        let d = |p: u8, q: u8| u32::from(p.abs_diff(q)).pow(2);
        d(x.0, r(c)) + d(x.1, g(c)) + d(x.2, b(c))
    };
    let avg = ((u32::from(r(c)) + u32::from(g(c)) + u32::from(b(c))) / 3) as u8;
    let gi_ = (u32::from(avg.saturating_sub(8)) / 10).min(23) as u8;
    let grey = 8 + 10 * gi_;
    if dist((grey, grey, grey)) < dist((rv, gv, bv)) {
        232 + gi_
    } else {
        cube as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mix_and_hex() {
        assert_eq!(mix(0x000000, 0xFFFFFF, 0), 0x000000);
        assert_eq!(mix(0x000000, 0xFFFFFF, 1000), 0xFFFFFF);
        assert_eq!(mix(0x000000, 0xFFFFFF, 500), 0x808080);
        assert_eq!(parse_hex("#1b2226"), Some(0x1B2226));
        assert_eq!(parse_hex("1B2226"), Some(0x1B2226));
        assert_eq!(parse_hex("#1b22"), None);
        assert_eq!(hex(0x1B2226), "#1B2226");
    }

    #[test]
    fn contrast_bounds() {
        assert!((contrast(0x000000, 0xFFFFFF) - 21.0).abs() < 1e-9);
        assert!((contrast(0x777777, 0x777777) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn xterm_nearest() {
        assert_eq!(xterm256(0x000000), 16);
        assert_eq!(xterm256(0xFFFFFF), 231);
        assert_eq!(xterm256(0x808080), 244);
    }
}
