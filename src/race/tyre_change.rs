//! `race::tyre_change` (0x444F7A): copies `dword_5D1444` / `dword_5D2444` to
//! `dword_5BEBE0` / `dword_5BEBE4`, then runs the tyre change's gauge `race::tyre_change_gauge` for car
//! 0 (block at 0x70F3EC, "c1 tyrec" in the field table at 0x5ED068).
//!
//! In:  nothing
//! Out: eax (the gauge's result, or +0x298 & 0x1E when it is not called), esi 0 when it is
//!      called; the rest kept (the caller goes on with a `call`: no flags read)
//!
//! Starts the gauge (eax 0, esi 0; then +0x26C = 1) when not all four bits 1 .. 4 of
//! +0x298 are set, +0x26C is 0 and +0x268 is at least 300. While +0x26C is 1 it runs it
//! (eax 1); a time (not 0) is added to +0x88, +0x26C becomes -301, and each of the bits
//! not set before is set with its dword of +0x354 .. +0x360 cleared. A reading (the
//! gauge's on-screen text not looked at): the roadside tyre change, timed by a button
//! press on a gauge; see `notes/history.md`.
//! The gauge is `race::tyre_change_gauge`'s Rust version (since 2026-10-19; before, a call
//! into the game). Shadow mode runs the original alone (the gauge reads the clock
//! and the buttons and has its own state: it must not run twice). Self-test: a copy whose
//! variables are the test's and whose gauge call goes to a stand-in that notes eax and
//! esi and gives a time, against the Rust version on the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::frame::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let after = unsafe { tyre_change_at(r, &tyre_variables(), &mut tyre_part) }?;
        r.eax = after.eax;
        r.esi = after.esi;
    }
    Ok(())
}
