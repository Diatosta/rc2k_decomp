//! Where a frame is shown in a window of another size: the frame keeps its shape, centred, and
//! the window's rest is bars.

/// A frame's place in the window, in the window's pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fit {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    /// The frame is magnified by a whole number (or not at all): shown with no filter it stays
    /// sharp, so no filter is wanted.
    pub sharp: bool,
}

impl Fit {
    /// Whether the frame covers the whole window.
    pub fn is_whole(&self, window: (u32, u32)) -> bool {
        (self.x, self.y, self.width, self.height) == (0, 0, window.0, window.1)
    }
}

/// The largest place of the `frame`'s shape in the `window`, centred. A frame or window with no
/// size gets the whole window.
pub fn fit(frame: (u32, u32), window: (u32, u32)) -> Fit {
    let (frame_width, frame_height) = (u64::from(frame.0), u64::from(frame.1));
    let (window_width, window_height) = (u64::from(window.0), u64::from(window.1));
    if frame.0 == 0 || frame.1 == 0 || window.0 == 0 || window.1 == 0 {
        return Fit {
            x: 0,
            y: 0,
            width: window.0,
            height: window.1,
            sharp: false,
        };
    }
    // The window is the frame's shape or wider: the height decides.
    let (width, height) = if window_width * frame_height >= window_height * frame_width {
        let width = (frame_width * window_height * 2 + frame_height) / (frame_height * 2);
        (width.min(window_width), window_height)
    } else {
        let height = (frame_height * window_width * 2 + frame_width) / (frame_width * 2);
        (window_width, height.min(window_height))
    };
    let sharp = width % frame_width == 0
        && height % frame_height == 0
        && width / frame_width == height / frame_height;
    Fit {
        x: ((window_width - width) / 2) as u32,
        y: ((window_height - height) / 2) as u32,
        width: width as u32,
        height: height as u32,
        sharp,
    }
}

/// Where a frame is shown on a screen whose edges are partly unusable (a phone's notch, rounded
/// corners and home indicator): `safe` is the usable part of the `window` (x, y, width, height).
/// A race frame (`menu` false) has the window's shape and covers all of it, edge to edge. A
/// menu's 4:3 page is as tall as the window and centred across the usable part, so its items and
/// texts keep clear of the notch while the background picture fills the rest.
pub fn place(frame: (u32, u32), window: (u32, u32), safe: [u32; 4], menu: bool) -> Fit {
    let mut fit = fit(frame, window);
    if menu && window.0 > 0 && fit.width < window.0 {
        let spare = i64::from(safe[2]) - i64::from(fit.width);
        let x = i64::from(safe[0]) + spare / 2;
        fit.x = x.clamp(0, i64::from(window.0 - fit.width)) as u32;
    }
    fit
}

/// The frame pixel under a window pixel, for a frame shown at `fit`: the one function that maps
/// touches and the pointer to the frame. None outside the frame's place.
pub fn window_to_frame(fit: &Fit, frame: (u32, u32), point: (f64, f64)) -> Option<(f64, f64)> {
    if fit.width == 0 || fit.height == 0 {
        return None;
    }
    let x = (point.0 - f64::from(fit.x)) * f64::from(frame.0) / f64::from(fit.width);
    let y = (point.1 - f64::from(fit.y)) * f64::from(frame.1) / f64::from(fit.height);
    (x >= 0.0 && y >= 0.0 && x < f64::from(frame.0) && y < f64::from(frame.1)).then_some((x, y))
}

/// The usable part of a window (`safe`: x, y, width, height in window pixels) in the pixels of a
/// `frame` that covers the window (a race frame): the part the HUD keeps to. The whole frame
/// where the window's size is unknown.
pub fn safe_in_frame(safe: [u32; 4], window: (u32, u32), frame: (u32, u32)) -> [u32; 4] {
    if window.0 == 0 || window.1 == 0 {
        return [0, 0, frame.0, frame.1];
    }
    let across = |v: u32, round_up: bool| {
        let n = u64::from(v) * u64::from(frame.0);
        let d = u64::from(window.0);
        (if round_up { n.div_ceil(d) } else { n / d }) as u32
    };
    let down = |v: u32, round_up: bool| {
        let n = u64::from(v) * u64::from(frame.1);
        let d = u64::from(window.1);
        (if round_up { n.div_ceil(d) } else { n / d }) as u32
    };
    let left = across(safe[0], true).min(frame.0);
    let top = down(safe[1], true).min(frame.1);
    let right = across(safe[0] + safe[2], false).clamp(left, frame.0);
    let bottom = down(safe[1] + safe[3], false).clamp(top, frame.1);
    [left, top, right - left, bottom - top]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_of_the_windows_size_covers_it_sharp() {
        let f = fit((800, 600), (800, 600));
        assert!(f.is_whole((800, 600)) && f.sharp);
    }

    #[test]
    fn a_four_three_frame_in_a_wide_window_has_bars_at_the_sides() {
        let f = fit((800, 600), (1920, 1080));
        assert_eq!((f.x, f.y, f.width, f.height), (240, 0, 1440, 1080));
        // 1.8 times: filtered.
        assert!(!f.sharp);
        assert!(!f.is_whole((1920, 1080)));
    }

    #[test]
    fn whole_factors_are_sharp() {
        let f = fit((800, 600), (1600, 1200));
        assert!(f.is_whole((1600, 1200)) && f.sharp);
        let f = fit((800, 600), (2400, 1800));
        assert!(f.sharp);
    }

    #[test]
    fn a_tall_window_has_bars_above_and_below() {
        let f = fit((800, 600), (800, 900));
        assert_eq!((f.x, f.y, f.width, f.height), (0, 150, 800, 600));
        assert!(f.sharp);
    }

    #[test]
    fn a_small_window_shrinks_the_frame() {
        let f = fit((800, 600), (640, 480));
        assert!(f.is_whole((640, 480)) && !f.sharp);
    }

    #[test]
    fn no_size_gives_the_whole_window() {
        let f = fit((0, 0), (640, 480));
        assert!(f.is_whole((640, 480)));
    }
    #[test]
    fn a_race_frame_covers_the_window_and_a_menu_page_centres_in_the_safe_part() {
        let window = (2868, 1320);
        let safe = [186, 0, 2496, 1266];
        let race = place((1434, 660), window, safe, false);
        assert!(race.is_whole(window));
        let page = place((800, 600), window, [186, 0, 2400, 1266], true);
        assert_eq!((page.y, page.width, page.height), (0, 1760, 1320));
        assert_eq!(page.x, 186 + (2400 - 1760) / 2);
        // Never pushed out of the window.
        assert_eq!(
            place((800, 600), window, [2000, 0, 868, 1320], true).x,
            1108
        );
    }

    #[test]
    fn touches_map_to_the_frame_and_the_safe_part_to_the_hud() {
        let fit = place((800, 600), (1600, 1200), [0, 0, 1600, 1200], true);
        assert_eq!(
            window_to_frame(&fit, (800, 600), (800.0, 600.0)),
            Some((400.0, 300.0))
        );
        assert_eq!(
            window_to_frame(
                &place((800, 600), (1920, 1080), [0; 4], false),
                (800, 600),
                (0.0, 5.0)
            ),
            None
        );
        assert_eq!(
            safe_in_frame([186, 0, 2496, 1266], (2868, 1320), (1434, 660)),
            [93, 0, 1248, 633]
        );
        assert_eq!(safe_in_frame([0, 0, 10, 10], (0, 0), (8, 6)), [0, 0, 8, 6]);
    }
}
