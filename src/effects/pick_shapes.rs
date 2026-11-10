//! `effects::pick_shapes` (0x4BE8C3): `g_marksShapesA` and `g_marksShapesB` both get
//! |`dword_844610[dword_84460C]`| (the same entry read twice: section 16), with
//! `dword_844604` / `dword_844608` 2, or 0 when the entry is negative.
//!
//! Callers: `effects::run_shapes` (0x4BDF82), `effects::run_shapes` (0x4BE09D),
//! `effects::windscreen_flakes_step` (0x4C067D). Refuses (before anything) an index of 0x100 or
//! more (the table's 256 entries; its writer masks it with 0xFF).

use crate::effects::mask_store::Masks;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::Refused;

// Converted: `rc2k_core::effects::steps::{shape_entry, pick_shapes}`
pub fn pick_shapes(masks: &Masks, drops: &WindscreenDrops) -> Result<(), Refused> {
    use rc2k_core::effects::steps;
    let index = steps::shape_entry(drops.picks.index.get())?;
    // (the flag, the shape): `dword_844604` / `g_marksShapesA`, `dword_844608` /
    // `g_marksShapesB`.
    for n in 0..2 {
        let picked = steps::pick_shapes(drops.picks.table[index].get() as i32);
        drops.picks.flags[n].set(picked.flag);
        masks.vars.shapes[n].step.set(picked.shapes);
    }
    Ok(())
}
