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
use crate::marks::store::{Slot, game_read, game_write};

const INDEX_RVA: Slot = Slot::at(0x84460C); // dword_84460C
const TABLE_RVA: Slot = Slot::at(0x844610); // dword_844610: 256 entries
/// (the flag, the shape): `dword_844604` / `g_marksShapesA`, `dword_844608` /
/// `g_marksShapesB`.
const PAIRS: [(Slot, Slot); 2] = [
    (Slot::at(0x844604), Slot::at(0x82FC56)),
    (Slot::at(0x844608), Slot::at(0x82FC7A)),
];

/// The Rust version (see the header).
// Converted: `rc2k_core::marks::steps::{shape_entry, pick_shapes}`
pub unsafe fn pick_shapes() -> Result<(), Refused> {
    use rc2k_core::marks::steps;
    unsafe {
        let index = steps::shape_entry(game_read(INDEX_RVA))?;
        for (flag, shape) in PAIRS {
            let picked = steps::pick_shapes(game_read(TABLE_RVA + 4 * index));
            game_write(flag, picked.flag);
            game_write(shape, picked.shapes);
        }
    }
    Ok(())
}
