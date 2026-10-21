//! Where the view is drawn on the screen (converted 2026-10-21 from
//! `scene::choose_view_window` 0x44FE1A): the whole screen, or in the framed layout
//! (`dword_621514`) 800 x 600 at 0, 0 (with `dword_621480`, or mode `word_636526` 2) or
//! 720 x 540 at 0x50, 0.

/// The settings the choice reads (the screen's size only when not framed).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewSettings {
    pub framed: bool,
    pub wide: bool,
    pub mode: u16,
}

/// The window chosen: its size, its offset, and the scale (`dword_624080`, 0x100).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewWindow {
    pub size: [u32; 2],
    pub offset: [u32; 2],
    pub scale: u32,
}

/// The scale every choice sets.
pub const SCALE: u32 = 0x100;
/// The mode `scene::full_view_window` sets around the choice.
pub const FULL_MODE: u16 = 2;

/// `choose_view_window`'s window; `screen` (width, height) is asked only when not framed.
pub fn choose(settings: ViewSettings, screen: impl FnOnce() -> [u32; 2]) -> ViewWindow {
    let (size, x) = if !settings.framed {
        (screen(), 0)
    } else if settings.wide || settings.mode == FULL_MODE {
        ([800, 600], 0)
    } else {
        ([720, 540], 0x50)
    };
    ViewWindow {
        size,
        offset: [x, 0],
        scale: SCALE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows() {
        let s = |framed, wide, mode| ViewSettings { framed, wide, mode };
        assert_eq!(
            choose(s(false, true, 2), || [640, 480]),
            ViewWindow {
                size: [640, 480],
                offset: [0, 0],
                scale: 0x100
            }
        );
        assert_eq!(
            choose(s(true, false, 0), || unreachable!()).offset,
            [0x50, 0]
        );
        assert_eq!(
            choose(s(true, false, 2), || unreachable!()).size,
            [800, 600]
        );
        assert_eq!(choose(s(true, true, 1), || unreachable!()).size, [800, 600]);
        assert_eq!(
            choose(s(true, false, 1), || unreachable!()).size,
            [720, 540]
        );
    }
}
