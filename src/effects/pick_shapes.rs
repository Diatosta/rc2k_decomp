//! `effects::pick_shapes`: both shape sets A and B get |the picks' `flags` entry at the
//! index| (the same entry read twice: a slip kept as is), with their flags set to 2, or 0
//! when the entry is negative.
//!
//! Refuses (before anything) an index of 0x100 or more (the table's 256 entries; its writer masks
//! it with 0xFF).

use crate::effects::mask_store::Masks;
use crate::effects::windscreen_drop_vars_store::WindscreenDrops;
use crate::errors::Refused;

// The logic is in `rc2k_core::effects::steps::{shape_entry, pick_shapes}`
pub fn pick_shapes(masks: &Masks, drops: &WindscreenDrops) -> Result<(), Refused> {
    use rc2k_core::effects::steps;
    let index = steps::shape_entry(drops.picks.index.get())?;
    // (the flag, the shape): `MaskPicks.flags[0]` / shape set A, `MaskPicks.flags[1]` /
    // shape set B.
    for n in 0..2 {
        let picked = steps::pick_shapes(drops.picks.table[index].get() as i32);
        drops.picks.flags[n].set(picked.flag);
        masks.vars.shapes[n].ease_step.set(picked.shapes);
    }
    Ok(())
}
