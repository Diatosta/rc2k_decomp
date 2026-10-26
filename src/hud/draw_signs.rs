//! `hud::draw_signs` (0x4B4F2B): up to four signs of the HUD's record (+0xA4 .. +0xB0, 0
//! none): sign v as rectangle 0x4C + (v - 1) & 0xF of `dword_75B940`, centred in its 0x40
//! wide place by its width (`dword_75BF80`[(v - 1) & 7]), the places from +0xB4 on 0x40
//! apart, at 0x36 (0x16 with `dword_6D1502` 0).
//!
//! In:  ebp the HUD's record
//! Out: ebx the last place, ecx the last sign's row (0 when the last is none), eax / esi
//!      the last drawn sign's rectangle (as `add_sprite` keeps them)
//!
//! Caller: layout 0's drawer (0x4B1F27).

use crate::errors::Refused;
use crate::hud::store::HUD;
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

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::signs` (through `signs_with`)
pub fn draw_signs(record: u32) -> Result<(), Refused> {
    signs_with(record, 0xA4, 0xB4, 0, 0x36, 0x20)
}

/// Four signs from the record's `first` .. on, the places from its `start` field plus
/// `shift`, at row `y` (shared with `hud::draw_signs_1a` / `_1b`).
// Converted: `rc2k_core::hud::signs::{sign, place, row}`
pub fn signs_with(
    record: u32,
    first: u32,
    start: u32,
    shift: u32,
    y: u32,
    lift: u32,
) -> Result<(), Refused> {
    use rc2k_core::hud::signs;
    {
        let get = |o: u32| GamePtr::new(record + o).read::<u32>("a hud record's field");
        let places = get(start)?.wrapping_add(shift);
        for n in 0..signs::SIGNS as u32 {
            let v = get(first + 4 * n)?;
            let Some(sign) = signs::sign(places, n, v, |k| HUD.sign_widths[k as usize].get())
            else {
                continue;
            };
            let rect = (HUD.rects.as_ptr() as u32).wrapping_add(sign.rectangle);
            crate::hud::add_sprite::add_sprite(sign.x, row_by(y, lift), rect)?;
        }
    }
    Ok(())
}
