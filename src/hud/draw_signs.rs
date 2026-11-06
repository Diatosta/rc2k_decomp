//! `hud::draw_signs` (0x4B4F2B): up to four signs of the HUD's record (0 none): sign v as
//! rectangle 0x4C + (v - 1) & 0xF of `dword_75B940`, centred in its 0x40 wide place by its
//! width (`dword_75BF80`[(v - 1) & 7]), the places 0x40 apart. Each layout and player has
//! a `SignStyle` for where the signs are in the record and the row they are drawn at.

use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::pointers::GamePtr;
use crate::race::panel_store::PANEL;

/// The row: `y`, or 0x20 higher with `dword_6D1502` 0.
pub fn row(y: u32) -> u32 {
    row_by(y, 0x20)
}

/// The row: `y`, or `lift` higher with `dword_6D1502` 0.
pub fn row_by(y: u32, lift: u32) -> u32 {
    rc2k_core::hud::signs::row(y, lift, PANEL.flat.get())
}

/// Where one player's signs are: which record fields hold them and where they are drawn.
pub struct SignStyle {
    /// The record's field of the first sign; the other three follow, a dword each.
    first: u32,
    /// The record's field of the places' start.
    start: u32,
    /// Added to the places' start.
    shift: u32,
    /// The row.
    y: u32,
    /// How much higher the row is with `dword_6D1502` 0.
    lift: u32,
}

/// Layout 0's signs (0x4B4F2B).
pub static LAYOUT_0: SignStyle = SignStyle {
    first: 0xA4,
    start: 0xB4,
    shift: 0,
    y: 0x36,
    lift: 0x20,
};
/// The first player's signs in layout 1 (0x4B52F7).
pub static LAYOUT_1A: SignStyle = SignStyle {
    first: 0xA4,
    start: 0xB4,
    shift: 0xFFFFFF38,
    y: 0x70,
    lift: 0x20,
};
/// The second player's signs in layout 1 (0x4B5451).
pub static LAYOUT_1B: SignStyle = SignStyle {
    first: 0xB8,
    start: 0xC8,
    shift: 0xC8,
    y: 0x70,
    lift: 0x20,
};
/// The first player's signs in layout 2 (0x4B507F).
pub static LAYOUT_2A: SignStyle = SignStyle {
    first: 0xA4,
    start: 0xB4,
    shift: 0,
    y: 0x8,
    lift: 0,
};
/// The second player's signs in layout 2 (0x4B51A3).
pub static LAYOUT_2B: SignStyle = SignStyle {
    first: 0xB8,
    start: 0xC8,
    shift: 0,
    y: 0x143,
    lift: 0x10,
};

/// Draws the four signs of `style`.
// Converted: `rc2k_core::hud::signs::{sign, place, row}`
pub fn draw_signs(hud: &Hud, record: GamePtr, style: &SignStyle) -> Result<(), Refused> {
    use rc2k_core::hud::signs;
    let get = |o: u32| record.offset(o).read::<u32>("a hud record's field");
    let places = get(style.start)?.wrapping_add(style.shift);
    for n in 0..signs::SIGNS as u32 {
        let v = get(style.first + 4 * n)?;
        let Some(sign) = signs::sign(places, n, v, |k| hud.block.sign_widths[k as usize].get())
        else {
            continue;
        };
        let rect = crate::hud::store::rects_at(hud, sign.rectangle);
        crate::hud::add_sprite::add_sprite(hud, sign.x, row_by(style.y, style.lift), rect)?;
    }
    Ok(())
}
