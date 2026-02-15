/// Hex ↔ RGB ↔ HSL conversion and color manipulation utilities.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

struct Hsl {
    h: f64, // 0..360
    s: f64, // 0..1
    l: f64, // 0..1
}

/// Parse `#RRGGBB` or `RRGGBB` into an `Rgb`.
pub fn parse_hex(input: &str) -> Option<Rgb> {
    let hex = input.strip_prefix('#').unwrap_or(input);
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Rgb { r, g, b })
}

/// Format as `#rrggbb`.
pub fn to_hex(rgb: &Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", rgb.r, rgb.g, rgb.b)
}

/// Format as `rgba(r, g, b, alpha)`.
pub fn to_rgba(rgb: &Rgb, alpha: f64) -> String {
    format!("rgba({}, {}, {}, {})", rgb.r, rgb.g, rgb.b, alpha)
}

/// Validate that a string is a valid hex color.
pub fn is_valid_hex(input: &str) -> bool {
    parse_hex(input).is_some()
}

/// Increase HSL lightness by `amount` (0.0–1.0), clamped.
pub fn lighten(rgb: &Rgb, amount: f64) -> Rgb {
    let mut hsl = rgb_to_hsl(rgb);
    hsl.l = (hsl.l + amount).min(1.0);
    hsl_to_rgb(&hsl)
}

/// Decrease HSL lightness by `amount` (0.0–1.0), clamped.
pub fn darken(rgb: &Rgb, amount: f64) -> Rgb {
    let mut hsl = rgb_to_hsl(rgb);
    hsl.l = (hsl.l - amount).max(0.0);
    hsl_to_rgb(&hsl)
}

/// Set HSL lightness to a target value (for generating pale tints).
pub fn tint(rgb: &Rgb, target_lightness: f64) -> Rgb {
    let mut hsl = rgb_to_hsl(rgb);
    hsl.l = target_lightness.clamp(0.0, 1.0);
    hsl_to_rgb(&hsl)
}

// --- internal ---

fn rgb_to_hsl(rgb: &Rgb) -> Hsl {
    let r = rgb.r as f64 / 255.0;
    let g = rgb.g as f64 / 255.0;
    let b = rgb.b as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f64::EPSILON {
        return Hsl { h: 0.0, s: 0.0, l };
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - r).abs() < f64::EPSILON {
        let mut h = (g - b) / d;
        if g < b {
            h += 6.0;
        }
        h
    } else if (max - g).abs() < f64::EPSILON {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };

    Hsl { h: h * 60.0, s, l }
}

fn hsl_to_rgb(hsl: &Hsl) -> Rgb {
    if hsl.s.abs() < f64::EPSILON {
        let v = (hsl.l * 255.0).round() as u8;
        return Rgb { r: v, g: v, b: v };
    }

    let q = if hsl.l < 0.5 {
        hsl.l * (1.0 + hsl.s)
    } else {
        hsl.l + hsl.s - hsl.l * hsl.s
    };
    let p = 2.0 * hsl.l - q;
    let h = hsl.h / 360.0;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    Rgb {
        r: (r * 255.0).round() as u8,
        g: (g * 255.0).round() as u8,
        b: (b * 255.0).round() as u8,
    }
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_with_hash() {
        let c = parse_hex("#6366f1").unwrap();
        assert_eq!(
            c,
            Rgb {
                r: 99,
                g: 102,
                b: 241
            }
        );
    }

    #[test]
    fn parse_hex_without_hash() {
        let c = parse_hex("f97316").unwrap();
        assert_eq!(
            c,
            Rgb {
                r: 249,
                g: 115,
                b: 22
            }
        );
    }

    #[test]
    fn parse_hex_invalid() {
        assert!(parse_hex("xyz").is_none());
        assert!(parse_hex("#12345").is_none());
        assert!(parse_hex("#gggggg").is_none());
    }

    #[test]
    fn round_trip_hex() {
        let original = "#6366f1";
        let rgb = parse_hex(original).unwrap();
        assert_eq!(to_hex(&rgb), original);
    }

    #[test]
    fn rgba_formatting() {
        let rgb = parse_hex("#6366f1").unwrap();
        assert_eq!(to_rgba(&rgb, 0.12), "rgba(99, 102, 241, 0.12)");
        assert_eq!(to_rgba(&rgb, 0.4), "rgba(99, 102, 241, 0.4)");
    }

    #[test]
    fn lighten_color() {
        let primary = parse_hex("#6366f1").unwrap();
        let lighter = lighten(&primary, 0.10);
        // Should be brighter — higher lightness means higher component values on average
        let avg_original = (primary.r as u16 + primary.g as u16 + primary.b as u16) / 3;
        let avg_lighter = (lighter.r as u16 + lighter.g as u16 + lighter.b as u16) / 3;
        assert!(
            avg_lighter > avg_original,
            "lighter should have higher average: {avg_lighter} vs {avg_original}"
        );
    }

    #[test]
    fn darken_color() {
        let primary = parse_hex("#6366f1").unwrap();
        let darker = darken(&primary, 0.10);
        let avg_original = (primary.r as u16 + primary.g as u16 + primary.b as u16) / 3;
        let avg_darker = (darker.r as u16 + darker.g as u16 + darker.b as u16) / 3;
        assert!(
            avg_darker < avg_original,
            "darker should have lower average: {avg_darker} vs {avg_original}"
        );
    }

    #[test]
    fn tint_to_pale() {
        let primary = parse_hex("#6366f1").unwrap();
        let pale = tint(&primary, 0.95);
        // 95% lightness should produce a very light color
        assert!(
            pale.r > 220 && pale.g > 220 && pale.b > 220,
            "tint(0.95) should be very light: got {:?}",
            pale
        );
    }

    #[test]
    fn achromatic_round_trip() {
        // Pure gray should survive conversions
        let gray = Rgb {
            r: 128,
            g: 128,
            b: 128,
        };
        let lighter = lighten(&gray, 0.1);
        let darker = darken(&lighter, 0.1);
        // Should be close to original (allow ±1 for rounding)
        assert!((darker.r as i16 - gray.r as i16).unsigned_abs() <= 1);
        assert!((darker.g as i16 - gray.g as i16).unsigned_abs() <= 1);
        assert!((darker.b as i16 - gray.b as i16).unsigned_abs() <= 1);
    }

    #[test]
    fn is_valid_hex_works() {
        assert!(is_valid_hex("#6366f1"));
        assert!(is_valid_hex("6366f1"));
        assert!(!is_valid_hex("nope"));
        assert!(!is_valid_hex("#123"));
    }

    #[test]
    fn black_and_white() {
        let black = parse_hex("#000000").unwrap();
        let white = parse_hex("#ffffff").unwrap();
        assert_eq!(to_hex(&lighten(&black, 1.0)), "#ffffff");
        assert_eq!(to_hex(&darken(&white, 1.0)), "#000000");
    }
}
