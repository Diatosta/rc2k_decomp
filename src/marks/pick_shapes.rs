//! `marks::pick_shapes` (0x4BE8C3): `g_marksShapesA` and `g_marksShapesB` both get
//! |`dword_844610[dword_84460C]`| (the same entry read twice: section 16), with
//! `dword_844604` / `dword_844608` 2, or 0 when the entry is negative.
//!
//! In:  nothing
//! Out: eax the entry made positive, ebp `g_marksShapesB`; the rest kept
//!
//! Callers: 0x4BDF82, 0x4BE09D, 0x4C067D. Refuses (before anything) an index of 0x100 or
//! more (the table's 256 entries; its writer masks it with 0xFF). Self-test: the index,
//! the entry and the flags by lot (negative entries and 0x80000000 among them), compared
//! on the four results and the registers; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, state_entry};

const INDEX_RVA: usize = rva(0x84460C); // dword_84460C
const TABLE_RVA: usize = rva(0x844610); // dword_844610: 256 entries
/// (the flag, the shape): `dword_844604` / `g_marksShapesA`, `dword_844608` /
/// `g_marksShapesB`.
const PAIRS: [(usize, usize); 2] = [
    (rva(0x844604), rva(0x82FC56)),
    (rva(0x844608), rva(0x82FC7A)),
];

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::marks::steps::{shape_entry, pick_shapes}`
pub unsafe fn pick_shapes(entry: &Regs) -> Result<Regs, Refused> {
    use rc2k_core::marks::steps;
    let mut g = *entry;
    unsafe {
        let index = steps::shape_entry(game_read(INDEX_RVA))?;
        for (flag, shape) in PAIRS {
            let picked = steps::pick_shapes(game_read(TABLE_RVA + 4 * index));
            game_write(flag, picked.flag);
            game_write(shape, picked.shapes);
            g.eax = picked.shapes;
            g.ebp = game_addr(shape) as u32;
        }
    }
    Ok(g)
}

pub fn regions() -> Vec<Region> {
    PAIRS.iter().flat_map(|&(f, s)| [at(f), at(s)]).collect()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { pick_shapes(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
