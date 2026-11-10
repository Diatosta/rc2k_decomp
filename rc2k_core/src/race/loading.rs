//! The loading bar drawn while a stage loads (converted 2026-10-21 from
//! `race::loading_bar_quad` 0x446861): a red rectangle along the bottom of the screen, as
//! long as the share loaded, laid out on 800 x 600 and scaled to the screen; and the flat
//! rectangles' corners (converted 2026-10-21 from `race::draw_screen_quad` 0x44B364);
//! a frame's way through converted 2026-10-21 from `race::loading_bar_frame` 0x4467C7.

/// The bar's colour.
pub const BAR_COLOUR: u32 = 0xFF_0000;

/// The bar on the screen: its left and right edges, its top and bottom, and the rest of
/// the right edge's division (the original's edx).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bar {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
    pub rest: u32,
}

/// `v` x `size` / `of` (`mul`, `div`): the quotient and the rest.
fn scaled(v: u32, size: u32, of: u32) -> (u32, u32) {
    let n = u64::from(v) * u64::from(size);
    ((n / u64::from(of)) as u32, (n % u64::from(of)) as u32)
}

/// The share loaded held to 0 .. 100 (signed).
pub fn held_share(share: u32) -> u32 {
    (share as i32).clamp(0, 0x64) as u32
}

/// The bar for a screen of `width` x `height` and the share loaded (0 .. 100): from x 0
/// to the width times the share / 100, y 0x24D .. 0x258 of 600.
pub fn loading_bar(width: u32, height: u32, share: u32) -> Bar {
    let left = scaled(0, width, 0x320).0;
    let (right, rest) = scaled(width, held_share(share), 0x64);
    Bar {
        left,
        right: right.wrapping_add(left),
        top: scaled(0x24D, height, 0x258).0,
        bottom: scaled(0x258, height, 0x258).0,
        rest,
    }
}

/// `race::draw_screen_quad`'s four corners (x, y) of the rectangle from x `x0` .. `x1`
/// and y `y0` .. `y1` (whole numbers made floats: `fild` / `fstp`, one rounding at any
/// precision), round from the top left.
pub fn quad_corners(x0: i32, x1: i32, y0: i32, y1: i32) -> [(f32, f32); 4] {
    let (x0, x1, y0, y1) = (x0 as f32, x1 as f32, y0 as f32, y1 as f32);
    [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
}

/// `loading_bar_frame`'s mark (`dword_6365CC`) while the bar is on.
pub const FRAME_MARK: u32 = 8;

/// A frame is tried with the window active and the surfaces not lost (`dword_518380`
/// not 1).
pub fn frame_tried(active: u32, lost: u32) -> bool {
    active != 0 && lost != 1
}

pub fn restored(lost_before: u32, lost_after: u32) -> (u32, bool, bool) {
    let ebx = (lost_before ^ 1) | lost_after;
    (ebx, ebx < 1, ebx == 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames() {
        assert!(frame_tried(1, 0) && !frame_tried(0, 0) && !frame_tried(1, 1));
        assert!(frame_tried(1, 2));
        assert_eq!(restored(1, 0), (0, true, false));
        assert_eq!(restored(0, 0), (1, false, true));
        assert_eq!(restored(1, 1), (1, false, true));
        assert_eq!(restored(2, 0), (3, false, false));
    }

    #[test]
    fn bar() {
        let b = loading_bar(800, 600, 50);
        assert_eq!(
            (b.left, b.right, b.top, b.bottom, b.rest),
            (0, 400, 0x24D, 0x258, 0)
        );
        assert_eq!(loading_bar(1024, 768, 33).right, 1024 * 33 / 100);
        assert_eq!(loading_bar(1024, 768, 33).rest, 1024 * 33 % 100);
        assert_eq!(held_share(0xFFFF_FFFF), 0);
        assert_eq!(held_share(500), 100);
        assert_eq!(
            quad_corners(0, 800, 589, 600),
            [(0.0, 589.0), (800.0, 589.0), (800.0, 600.0), (0.0, 600.0)]
        );
        assert_eq!(quad_corners(0x0100_0001, 0, 0, 0)[0].0, 16777216.0);
    }
}
