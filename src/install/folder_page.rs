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
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::installer::{PATH_PROBLEM, g, p, rb, wb};
use rc2k_core::install::pages;

/// `state` the list action's (ebx), `list` the page's.
pub fn folder_page(state: u32, list: u32) {
    if state == 0 {
        crate::install::next_edit::next_edit(0, list);
    }
    if (state as i32) < 0 {
        p(PATH_PROBLEM, 0);
    }
    // Converted: `rc2k_core::install::pages::hidden_notices`.
    let hide = |offsets: [u32; 2]| {
        for o in offsets {
            wb(list + o, rb(list + o) | pages::HIDDEN);
        }
    };
    if pages::hidden_notices(g(PATH_PROBLEM)).0 {
        hide([0x38, 0x48]);
    }
    if pages::hidden_notices(g(PATH_PROBLEM)).1 {
        hide([0x58, 0x68]);
    }
}
