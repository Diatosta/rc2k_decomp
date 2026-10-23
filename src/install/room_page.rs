//! `install::room_page` (0x4142C7): the not-enough-room page. At the start (ebx 0)
//! `dword_51BA04` (ignore the room) is set to 0.
//!
//! In:  ebx the state
//! Out: every register kept; the flags of `test ebx, ebx` last
//! Caller `install::run_page`. Shadow mode compares.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{IGNORE_ROOM, p};
use crate::race::viewer::regs_entry;

pub fn room_page(entry: &Regs) -> Regs {
    let mut r = *entry;
    if r.ebx == 0 {
        p(IGNORE_ROOM, 0);
    }
    r.set_zero(r.ebx == 0);
    r.set_carry(false);
    r
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(room_page(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}
