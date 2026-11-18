//! The screen sizes the graphics page offers for the race: the game's own list (kept so old
//! configurations still read), the monitor's modes, the usual sizes of the common ratios that fit
//! the desktop, and a size set by hand in `rc2k.toml`. Each is shown with its ratio.

/// Sizes of the usual ratios (4:3, 16:10, 16:9, 21:9, 32:9), offered when they fit the desktop.
pub const COMMON: [(u32, u32); 21] = [
    (1280, 960),
    (1400, 1050),
    (1280, 800),
    (1440, 900),
    (1680, 1050),
    (1920, 1200),
    (2560, 1600),
    (1280, 720),
    (1366, 768),
    (1600, 900),
    (1920, 1080),
    (2560, 1440),
    (3840, 2160),
    (2560, 1080),
    (3440, 1440),
    (3840, 1600),
    (3840, 1080),
    (5120, 1440),
    (5120, 2160),
    (7680, 2160),
    (7680, 4320),
];

/// The ratios named, as (across, down).
const NAMED: [(u32, u32); 7] = [(4, 3), (5, 4), (3, 2), (16, 10), (16, 9), (21, 9), (32, 9)];

/// The smallest size offered at all (the game's own smallest).
pub const SMALLEST: (u32, u32) = (640, 480);

/// The ratio of a size as a text, e.g. "16:9": the nearest named ratio when the size is within
/// 6 percent of one (3440 x 1440 is "21:9"), else the size's own reduced ratio.
pub fn ratio(width: u32, height: u32) -> String {
    if width == 0 || height == 0 {
        return "0:0".into();
    }
    let value = f64::from(width) / f64::from(height);
    let near = NAMED
        .iter()
        .map(|&(a, b)| {
            let v = f64::from(a) / f64::from(b);
            ((value - v).abs() / v, (a, b))
        })
        .min_by(|x, y| x.0.total_cmp(&y.0));
    match near {
        Some((off, (a, b))) if off < 0.06 => format!("{a}:{b}"),
        _ => {
            let (mut x, mut y) = (width, height);
            while y != 0 {
                (x, y) = (y, x % y);
            }
            format!("{}:{}", width / x, height / x)
        }
    }
}

/// The text a size is shown as: "2560 x 1080 (21:9)", or "custom 2560 x 1080" (`word` is the
/// language's "custom") for the size set by hand.
pub fn label(width: u32, height: u32, custom: Option<(u32, u32)>, word: &[u8]) -> Vec<u8> {
    if custom == Some((width, height)) {
        let mut text = word.to_vec();
        text.extend(format!(" {width} x {height}").bytes());
        text
    } else {
        format!("{width} x {height} ({})", ratio(width, height)).into_bytes()
    }
}

/// The sizes to offer besides the game's own `table`: the monitor's `modes`, the usual sizes that
/// fit the `desktop` (none when it is not known), and `custom`; none twice, none the table has,
/// smallest first.
pub fn extra_sizes(
    table: &[(u32, u32)],
    modes: &[(u32, u32)],
    desktop: Option<(u32, u32)>,
    custom: Option<(u32, u32)>,
) -> Vec<(u32, u32)> {
    let fits = |(w, h): (u32, u32)| desktop.is_some_and(|(dw, dh)| w <= dw && h <= dh);
    let mut found: Vec<(u32, u32)> = Vec::new();
    let all = modes
        .iter()
        .copied()
        .chain(COMMON.into_iter().filter(|&s| fits(s)))
        .chain(custom);
    for size in all {
        let (w, h) = size;
        if w < SMALLEST.0 || h < SMALLEST.1 || table.contains(&size) || found.contains(&size) {
            continue;
        }
        found.push(size);
    }
    sort(&mut found);
    found
}

/// The widest frame a phone's own size is cut down to by default: the menus' canvas is drawn by
/// the processor and shown whole each frame, which a screen of three million pixels makes slow.
pub const PHONE_WIDEST: u32 = 1920;

/// The shares (numerator, denominator) of a screen's own size a small screen offers beside it:
/// fewer pixels to draw where the frame rate needs it.
const FRACTIONS: [(u32, u32); 4] = [(1, 1), (3, 4), (2, 3), (1, 2)];

/// A size scaled by a fraction, each side down to an even number.
fn scaled((w, h): (u32, u32), (n, d): (u32, u32)) -> (u32, u32) {
    let side = |v: u32| (u64::from(v) * u64::from(n) / u64::from(d)) as u32 & !1;
    (side(w), side(h))
}

/// The sizes a screen with no mode list of its own (a phone's, which is one size in pixels)
/// offers: the `native` size and its fractions, none below the smallest size.
pub fn native_fractions(native: (u32, u32)) -> Vec<(u32, u32)> {
    let mut sizes: Vec<(u32, u32)> = FRACTIONS
        .into_iter()
        .map(|f| scaled(native, f))
        .filter(|&(w, h)| w >= SMALLEST.0 && h >= SMALLEST.1)
        .collect();
    sizes.dedup();
    sort(&mut sizes);
    sizes
}

/// The size a phone's race starts at: the largest of the `native` size's fractions that is no
/// wider than `PHONE_WIDEST` (the native size itself when none is).
pub fn native_default(native: (u32, u32)) -> (u32, u32) {
    native_fractions(native)
        .into_iter()
        .rev()
        .find(|&(w, _)| w <= PHONE_WIDEST)
        .unwrap_or(native)
}

/// Sizes smallest first (by the pixels, then the width).
pub fn sort(sizes: &mut [(u32, u32)]) {
    sizes.sort_by_key(|&(w, h)| (u64::from(w) * u64::from(h), w));
}

/// The monitor's modes as sizes: each size once (its highest refresh being the mode that would
/// be set), smallest first. `modes` are (width, height, refresh).
pub fn modes_as_sizes(modes: &[(u32, u32, f32)]) -> Vec<(u32, u32)> {
    let mut sizes: Vec<(u32, u32)> = Vec::new();
    for &(w, h, _) in modes {
        if w > 0 && h > 0 && !sizes.contains(&(w, h)) {
            sizes.push((w, h));
        }
    }
    sort(&mut sizes);
    sizes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratios_are_named() {
        for (w, h, r) in [
            (1920, 1080, "16:9"),
            (1366, 768, "16:9"),
            (1280, 800, "16:10"),
            (2560, 1080, "21:9"),
            (3440, 1440, "21:9"),
            (3840, 1080, "32:9"),
            (5120, 1440, "32:9"),
            (1024, 768, "4:3"),
            (1280, 1024, "5:4"),
            (10, 1, "10:1"),
        ] {
            assert_eq!(ratio(w, h), r, "{w} x {h}");
        }
    }

    #[test]
    fn labels() {
        assert_eq!(label(2560, 1080, None, b"custom"), b"2560 x 1080 (21:9)");
        assert_eq!(
            label(2000, 700, Some((2000, 700)), b"custom"),
            b"custom 2000 x 700"
        );
    }

    #[test]
    fn a_phones_own_size_and_its_default() {
        let native = (2556, 1179);
        let all = native_fractions(native);
        assert_eq!(all.last(), Some(&(2556, 1178)));
        assert!(all.contains(&(1278, 588)) && all.contains(&(1704, 786)));
        // The widest of the fractions that fits 1920: three quarters.
        assert_eq!(native_default(native), (1917 & !1, 884));
        assert_eq!(native_default((1280, 720)), (1280, 720));
        assert_eq!(native_default((640, 480)), (640, 480));
    }

    #[test]
    fn the_extra_sizes_fit_and_are_not_repeated() {
        let table = [(800, 600), (1024, 768)];
        let modes = [(1920, 1080), (1024, 768), (1920, 1080), (1680, 1050)];
        let extra = extra_sizes(&table, &modes, Some((1920, 1200)), Some((2000, 500)));
        assert!(extra.contains(&(1280, 720)) && extra.contains(&(1920, 1200)));
        assert!(extra.contains(&(2000, 500)) && extra.contains(&(1680, 1050)));
        assert!(!extra.contains(&(2560, 1440)) && !extra.contains(&(1024, 768)));
        assert_eq!(extra.iter().filter(|&&s| s == (1920, 1080)).count(), 1);
        assert!(extra.windows(2).all(|p| p[0].0 * p[0].1 <= p[1].0 * p[1].1));
        // A desktop that is not known offers the monitor's modes and the custom size only.
        let extra = extra_sizes(&table, &modes, None, None);
        assert_eq!(extra, vec![(1680, 1050), (1920, 1080)]);
        assert_eq!(
            modes_as_sizes(&[(800, 600, 60.0), (800, 600, 75.0)]),
            vec![(800, 600)]
        );
    }
}
