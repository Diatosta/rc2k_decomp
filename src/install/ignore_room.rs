//! `install::ignore_room` (0x4142DA): the not-enough-room page's "go on anyway" action:
//! `dword_51BA04` 1, the answer (`byte_51D415`) 1 and the page left.
//!
//! In:  nothing
//! Out: every register kept; zero set, carry clear (`cmp eax, eax`)
//! An item action (kind 6). Caller `install::item_action` through a page's item. Shadow
//! mode compares.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{IGNORE_ROOM, LIST_ANSWER, LIST_LEAVE, p, pb};
use crate::race::viewer::regs_entry;

pub fn ignore_room(entry: &Regs) -> Regs {
    let mut r = *entry;
    p(IGNORE_ROOM, 1);
    pb(LIST_ANSWER, 1);
    pb(LIST_LEAVE, 1);
    r.set_zero(true);
    r.set_carry(false);
    r
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(ignore_room(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}
