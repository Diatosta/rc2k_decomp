//! `race::draw_sorted_entries` (0x454EAB): each entry of `g_sortEntries` (in its order) drawn by
//! `race::draw_list_entry` of its second word (see `race/frame_draw.rs`).
//!
//! In:  nothing
//! Out: ecx the count, esi the last entry, ebx its word (none: kept) (a `call` follows)
//!
//! Caller: `sub_4557AB` (0x45583A), each view drawn. Refuses (before anything) an entry
//! of 0x4000000 or more. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose entry call goes to a stand-in that notes ebx, esi and ecx, against the Rust
//! version with the same; lists of 0, 1 and 5 entries; all put back.

use crate::framework::Regs;
use crate::race::frame_draw::*;

const ENTRIES: u32 = 0x9DB730; // g_sortEntries

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let done = unsafe {
        draw_entries(r, SORT_COUNT_RVA, ENTRIES, &mut |e| {
            draw_list_entry(e, &mut entry_part)
        })
    };
    {
        let after = done?;
        let esp = r.esp;
        *r = after;
        r.esp = esp;
    }
    Ok(())
}
