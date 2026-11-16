//! The HUD's square icons: an entry of the HUD's record (+0x84 its texture or -1 for none, +0x88
//! its width, +0x8C its alpha, +0x90 its drop) scaled by `HudBlock.icon_scale` and centred on the
//! 800-wide screen, shifted by `HudBlock.icon_shift`, as tall as wide, under the top given.

use crate::math::geometry::fistp64;

/// The screen width the icons are centred on (the centre is half of it).
pub const MIDDLE: u32 = 0x320;

/// Whether an entry is drawn: its +0x84 is not -1.
pub fn shown(texture: u32) -> bool {
    texture != u32::MAX
}

/// A width or drop scaled by `scale` and converted back to an integer.
pub fn scaled(v: u32, scale: f32) -> u32 {
    fistp64(v as i32 as f64 * scale as f64) as u32
}

/// An icon's place: its scaled width and drop, its
/// colour (white under the entry's alpha), left and right, top and bottom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Icon {
    pub width: u32,
    pub drop: u32,
    pub colour: u32,
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

/// The icon of an entry (width, alpha, drop fields) under `top`.
pub fn icon(width: u32, alpha: u32, drop: u32, scale: f32, shift: u32, top: u32) -> Icon {
    let w = scaled(width, scale);
    let drop = scaled(drop, scale);
    let top = top.wrapping_add(drop);
    Icon {
        width: w,
        drop,
        colour: (alpha << 24) | 0xFF_FFFF,
        left: (MIDDLE.wrapping_sub(w) >> 1).wrapping_add(shift),
        right: (MIDDLE.wrapping_add(w) >> 1).wrapping_add(shift),
        top,
        bottom: top.wrapping_add(w),
    }
}

/// Its four corners in the order written: (the vertex, u, v, x, y), the texture's corners
/// 0 or 1 (left top, right top, left bottom, right bottom; the vertices go round).
pub fn corners(i: &Icon) -> [(usize, u32, u32, u32, u32); 4] {
    [
        (0, 0, 0, i.left, i.top),
        (1, 1, 0, i.right, i.top),
        (3, 0, 1, i.left, i.bottom),
        (2, 1, 1, i.right, i.bottom),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icons() {
        assert!(!shown(u32::MAX) && shown(0));
        assert_eq!(scaled(41, 1.5), 62);
        assert_eq!(scaled(40, 1.25), 50);
        let i = icon(40, 0x80, 12, 1.5, 0xFFFF_FF38, 0xC8);
        assert_eq!(
            i,
            Icon {
                width: 60,
                drop: 18,
                colour: 0x80FF_FFFF,
                left: 370 - 200,
                right: 430 - 200,
                top: 0xDA,
                bottom: 0xDA + 60
            }
        );
        assert_eq!(corners(&i)[2], (3, 0, 1, 170, 0xDA + 60));
    }
}
