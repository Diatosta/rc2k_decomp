//! `race::sort_depth_entries` (0x454ED4): with two entries or more, `g_depthEntries` sorted by their
//! first dword with the game's quicksort (`sub_454EF7`, recursive, covered here: its
//! callers are this function and itself), then a `jmp` to a `nullsub` (a `retn`).
//!
//! In:  nothing
//! Out: the registers as the sort leaves them (a `call` follows)
//!
//! The sort is the original's step for step (`rc2k_core::race::quick_sort`): its
//! first scan moves on only past an entry above the pivot, and equal entries after a swap
//! move the left end on; the result keeps the original's order of equal keys. Caller:
//! `sub_4557AB`.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const ENTRIES: u32 = 0x9D3730; // g_depthEntries

fn rust(r: &Regs) -> Result<Left, Refused> {
    // Converted: `rc2k_core::race::quick_sort::quick_sort`.
    Ok(all_regs(&unsafe {
        sort_entries(r, DEPTH_COUNT_RVA, ENTRIES)?
    }))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
