//! `marks::pick_shapes` (0x4BE8C3): `g_marksShapesA` and `g_marksShapesB` both get
//! |`dword_844610[dword_84460C]`| (the same entry read twice: section 16), with
//! `dword_844604` / `dword_844608` 2, or 0 when the entry is negative.
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Callers: 0x4BDF82, 0x4BE09D, 0x4C067D. Refuses (before anything) an index of 0x100 or
//! more (the table's 256 entries; its writer masks it with 0xFF).

use crate::errors::Refused;
use crate::marks::mask_picks::PICKS;
use crate::marks::mask_vars::MASKS;

/// The Rust version (see the header).
// Converted: `rc2k_core::marks::steps::{shape_entry, pick_shapes}`
pub fn pick_shapes() -> Result<(), Refused> {
    use rc2k_core::marks::steps;
    let index = steps::shape_entry(PICKS.index.get())?;
    // (the flag, the shape): `dword_844604` / `g_marksShapesA`, `dword_844608` /
    // `g_marksShapesB`.
    for n in 0..2 {
        let picked = steps::pick_shapes(PICKS.table[index].get() as i32);
        PICKS.flags[n].set(picked.flag);
        MASKS.shapes[n].step.set(picked.shapes);
    }
    Ok(())
}
