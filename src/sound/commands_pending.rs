//! `sound::commands_pending` (0x409C01): whether commands still wait for the audio
//! thread; without audio (`g_noAudio` not 0) the count is cleared first.
//!
//! In:  nothing
//! Out: zero flag set = none waiting (`cmp g_soundCommandCount, 0`), carry clear; every
//!      register preserved
//! One caller, `sound::wait_for_commands` (`jz`), whose Rust version uses
//! `sound::queue::pending` itself.
//! Shadow mode runs the original alone (the count is the audio thread's as well).
//! Self-test: a copy whose two variables are the test's, against `queue::pending` on
//! the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::sound::queue::{Queue, pending};

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let waiting = unsafe { pending(&Queue::game()) };
    r.set_zero(!waiting);
    r.set_carry(false);
    Ok(())
}
