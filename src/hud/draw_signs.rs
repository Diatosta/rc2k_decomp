//! `hud::draw_signs` (0x4B4F2B): up to four signs of a view's marks (`Panel::marks`; 0 none): sign
//! v as rectangle 0x4C + (v - 1) & 0xF of the sprite sheet, centred in its 0x40 wide place by its
//! width (`sign_widths`[(v - 1) & 7]), the places 0x40 apart from the marks' `base`. Each layout
//! and player has a `SignStyle` for whose marks they are and the row they are drawn at.

use crate::errors::Refused;
use crate::hud::owner::{Hud, Pen};
use crate::race::owner_frame::Frame;

/// The row: `y`, or 0x20 higher with `Panel::flat` 0.
pub fn row(frame: &Frame, y: u32) -> u32 {
    row_by(frame, y, 0x20)
}

/// The row: `y`, or `lift` higher with `Panel::flat` 0.
pub fn row_by(frame: &Frame, y: u32, lift: u32) -> u32 {
    rc2k_core::hud::signs::row(y, lift, frame.panel_and_overlay.panel.flat.get())
}

/// Where one player's signs are: whose marks they are and where they are drawn.
pub struct SignStyle {
    /// The view whose marks they are.
    view: usize,
    /// Added to the marks' base (the places' start).
    shift: u32,
    /// The row.
    y: u32,
    /// How much higher the row is with `Panel::flat` 0.
    lift: u32,
}

/// Layout 0's signs (0x4B4F2B).
pub static LAYOUT_0: SignStyle = SignStyle {
    view: 0,
    shift: 0,
    y: 0x36,
    lift: 0x20,
};
/// The first player's signs in layout 1 (0x4B52F7).
pub static LAYOUT_1A: SignStyle = SignStyle {
    view: 0,
    shift: 0xFFFFFF38,
    y: 0x70,
    lift: 0x20,
};
/// The second player's signs in layout 1 (0x4B5451).
pub static LAYOUT_1B: SignStyle = SignStyle {
    view: 1,
    shift: 0xC8,
    y: 0x70,
    lift: 0x20,
};
/// The first player's signs in layout 2 (0x4B507F).
pub static LAYOUT_2A: SignStyle = SignStyle {
    view: 0,
    shift: 0,
    y: 0x8,
    lift: 0,
};
/// The second player's signs in layout 2 (0x4B51A3).
pub static LAYOUT_2B: SignStyle = SignStyle {
    view: 1,
    shift: 0,
    y: 0x143,
    lift: 0x10,
};

/// Draws the four signs of `style`.
// Converted: `rc2k_core::hud::signs::{sign, place, row}`
pub fn draw_signs(frame: &Frame, hud: &Hud, pen: Pen, style: &SignStyle) -> Result<(), Refused> {
    use rc2k_core::hud::signs;
    let marks = &frame.panel_and_overlay.panel.marks[style.view];
    let places = marks.base.get().wrapping_add(style.shift);
    for (n, mark) in (0..signs::SIGNS as u32).zip(&marks.marks) {
        let v = mark.get();
        let Some(sign) = signs::sign(places, n, v, |k| hud.block.sign_widths[k as usize].get())
        else {
            continue;
        };
        let rect = crate::hud::store::RectAt::offset(sign.rectangle);
        crate::hud::add_sprite::add_sprite(
            frame,
            hud,
            pen,
            sign.x,
            row_by(frame, style.y, style.lift),
            rect,
        )?;
    }
    Ok(())
}
