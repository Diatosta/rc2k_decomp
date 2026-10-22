//! `sound::open_cd_audio` (0x40F9DE): the CD audio's opening, cut out of this build.
//!
//! In:  eax = anything
//! Out: every register preserved (`pusha` .. `popa`); carry clear, zero set if eax is 0
//!      (`test eax, eax` on the entry's eax)
//! The function begins `pusha; jmp` to its own end: the opening behind the jump ("CDA: Open
//! cd audio", `mciSendCommandA` open (0x803) and set (0x80D) on a CD audio device into
//! `mciId`, `dword_518108` 1, `sub_40FDE4` the volume, the time in `dword_518114`; failing:
//! "CDA: Cannot open CD audio .." and `sub_40FA8B`) is never reached, so `dword_518108`
//! stays 0 and the CD audio functions that test it do nothing. One caller, `start`
//! (0x4010E4), which reads no flag.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;

// ---- game data ----

// ---- Rust version ----

/// (carry, zero) for the entry's eax.
pub fn open_cd_audio(eax: u32) -> (bool, bool) {
    (false, eax == 0)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let mine = open_cd_audio(r.eax);
    r.set_carry(mine.0);
    r.set_zero(mine.1);
    Ok(())
}
