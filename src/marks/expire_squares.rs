//! `marks::expire_squares` (0x4C06AD): each live square item (`g_marksSquareItems`, 0x800 of
//! 0x3C bytes, +0 1 live) ends when its cell's byte in `g_marksMaskA` (or, with
//! `g_marksMaskRunCount` above 1, in `g_marksMaskB`) is the low byte of `g_marksShapesA` (the
//! cell from its place +0x20 / +0x24, `fistp`ed through `dword_81471C` / `dword_814720`,
//! >> 3).
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `marks::squares_step` (0x4C0682).

use crate::marks::hexagon_state::HEXAGON_STATE;
use crate::marks::mask_vars::MASKS;
use crate::marks::squares_store::SQUARE_ITEMS;

pub const ITEMS: u32 = 0x800;
pub const ITEM_LEN: u32 = 0x3C;

/// The Rust version (see the header).
// Converted: `rc2k_core::marks::expiry::{cell, square_ends}`
pub unsafe fn expire_squares() {
    use rc2k_core::marks::expiry;

    let al = MASKS.shapes[0].step.get() as u8;
    for item in &SQUARE_ITEMS.items {
        if item.live.get() != 1 {
            continue;
        }
        let c = expiry::cell(item.place[0].get(), item.place[1].get());
        HEXAGON_STATE.cell[1].set(c.z as u32);
        HEXAGON_STATE.cell[0].set(c.x as u32);
        if expiry::square_ends(
            al,
            || MASKS.run_count.get() as i32,
            |n| [&MASKS.mask_a, &MASKS.mask_b][n].get(c.at as usize),
        ) {
            item.live.set(0);
        }
    }
}
