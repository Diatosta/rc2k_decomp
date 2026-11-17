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
}
