//! `race::draw_depth_entries` (0x454E84): each entry of `g_depthEntries` (in its order) drawn by
//! `race::draw_list_entry` of its second word (see `race/frame_draw.rs`).
//!
//! In:  nothing
//! Out: ecx the count, esi the last entry, ebx its word (none: kept) (a `call` follows)
//!
//! Caller: `sub_4557AB` (0x455876), each view drawn. Refuses (before anything) an entry
//! of 0x4000000 or more.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::race::frame_draw::*;

const ENTRIES: u32 = 0x9D3730; // g_depthEntries

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let done = unsafe { draw_entries(r, DEPTH_COUNT_RVA, ENTRIES, &mut |e| draw_list_entry(e)) };
    {
        let after = done?;
        let esp = r.esp;
        *r = after;
        r.esp = esp;
    }
    Ok(())
}
