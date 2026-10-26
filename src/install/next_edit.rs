//! `install::next_edit` (0x41530F): with ebx 0, the highlight (`g_listChoice`) moved on to
//! the first edit item (kind 4) from it, which begins to be edited (`install::start_edit`);
//! none before the list's end: the highlight back to 0. Any other ebx: nothing.
//!
//! In:  ebx 0 (a list action's start), edi the list
//! Out: every register kept (`pusha` / `popa`); the flags of what it did last (no caller
//!      reads them)
//! Callers `install::folder_page` (the folder page's action), `install::mark_page`.
//! Shadow mode compares. Self-test: lists of the test's against the original.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{CHOICE, cmp, g, logic, p, rb};
use crate::race::viewer::regs_entry;

pub fn next_edit(entry: &Regs) -> Regs {
    let mut r = *entry;
    if entry.ebx != 0 {
        logic(&mut r, entry.ebx);
        return r;
    }
    let count = rb(entry.edi + 1) as u32;
    let item = |n: u32| entry.edi + 0x18 + 0x10 * n;
    // Converted: `rc2k_core::install::items::next_edit` (the highlight stored once, where
    // it ends).
    let (n, found) = rc2k_core::install::items::next_edit(g(CHOICE), count, |n| rb(item(n)));
    if found {
        p(CHOICE, n);
        r = crate::install::start_edit::start_edit(&Regs { esi: item(n), ..r });
    } else {
        cmp(&mut r, count, n);
        p(CHOICE, 0);
    }
    let flags = r.eflags;
    Regs {
        eflags: flags,
        ..*entry
    }
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(next_edit(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}
