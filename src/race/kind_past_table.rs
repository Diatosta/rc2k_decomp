//! `race::kind_past_table` (0x4651AC): carry for a car kind (eax) of 0x17 or more.
//!
//! In:  eax = the kind
//! Out: carry set for 0x17 or more; zero as `cmp eax, 17h` left it; every register
//!      preserved
//!
//! A `jnb` in its first bytes: private copy (the whole 9 bytes), manual patch.
//! See `race/race_shell.rs`. One caller, `race::kind_unusable` (`jb` after it). Its
//! flags are its result, so dispatch sets carry and zero. Shadow mode: the original, its
//! flags compared with the Rust version's. Self-test: kinds round 6, 0x17 and the sign,
//! with the flags clear and set before.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::race_shell::kind_past_table;

fn set(r: &mut Regs, (carry, z): (bool, bool)) {
    r.set_carry(carry);
    r.set_zero(z);
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    set(r, kind_past_table(r.eax));
    Ok(())
}
