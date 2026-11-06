//! `effects::expire_windscreen_flakes` (0x4C06AD): each live windscreen flake item (`g_marksWindscreenFlakeItems`, 0x800 of
//! 0x3C bytes, +0 1 live) ends when its cell's byte in `g_marksMaskA` (or, with
//! `g_marksMaskRunCount` above 1, in `g_marksMaskB`) is the low byte of `g_marksShapesA` (the
//! cell from its place +0x20 / +0x24, `fistp`ed through `dword_81471C` / `dword_814720`,
//! >> 3).
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `effects::windscreen_flakes_step` (0x4C0682).

use crate::effects::mask_store::Masks;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;

pub const ITEMS: u32 = 0x800;
pub const ITEM_LEN: u32 = 0x3C;

/// The Rust version (see the header).
// Converted: `rc2k_core::effects::expiry::{cell, square_ends}`
pub fn expire_windscreen_flakes(masks: &Masks, drops: &WindscreenDrops, fl: &WindscreenFlakes) {
    use rc2k_core::effects::expiry;

    let al = masks.vars.shapes[0].step.get() as u8;
    for item in &fl.items.items {
        if item.live.get() != 1 {
            continue;
        }
        let c = expiry::cell(item.place[0].get(), item.place[1].get());
        drops.state.cell[1].set(c.z as u32);
        drops.state.cell[0].set(c.x as u32);
        if expiry::square_ends(
            al,
            || masks.vars.run_count.get() as i32,
            |n| [&masks.vars.mask_a, &masks.vars.mask_b][n].get(c.at as usize),
        ) {
            item.live.set(0);
        }
    }
}
