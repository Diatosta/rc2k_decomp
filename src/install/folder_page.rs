//! `install::folder_page` (0x4142F5): the install-folder page. At the start (ebx 0) the
//! first edit item is begun (`install::next_edit`). At the end (ebx negative) `dword_51BA00`
//! is 0. Then by `dword_51BA00`: not 1, the items at +0x38 and +0x48 get 0x40 (hidden);
//! not 2, +0x58 and +0x68 get 0x40.
//!
//! In:  ebx the state, edi the list
//! Out: every register kept but what `next_edit` leaves; the flags of the last `cmp`
//! A page action. Caller `install::run_page`. Its call through `part`. Shadow mode runs
//! the original alone (`next_edit` begins an edit). Self-test: a copy whose call goes to a
//! stand-in, against the Rust version, every state and problem.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{NEXT_EDIT, PATH_PROBLEM, at, cmp, g, p, rb, wb};
use crate::race::viewer::regs_entry;
use rc2k_core::install::pages;

pub fn folder_page(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    if r.ebx == 0 {
        r.ebx = 0;
        crate::install::installer::game_part(at(NEXT_EDIT), &mut r)?;
    }
    if (entry.ebx as i32) < 0 {
        p(PATH_PROBLEM, 0);
    }
    // Converted: `rc2k_core::install::pages::hidden_notices`.
    let hide = |r: &Regs, offsets: [u32; 2]| {
        for o in offsets {
            wb(r.edi + o, rb(r.edi + o) | pages::HIDDEN);
        }
    };
    cmp(&mut r, g(PATH_PROBLEM), pages::BAD_FOLDER);
    if pages::hidden_notices(g(PATH_PROBLEM)).0 {
        hide(&r, [0x38, 0x48]);
    }
    cmp(&mut r, g(PATH_PROBLEM), pages::NO_ROOM);
    if pages::hidden_notices(g(PATH_PROBLEM)).1 {
        hide(&r, [0x58, 0x68]);
    }
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| folder_page(r))
}
