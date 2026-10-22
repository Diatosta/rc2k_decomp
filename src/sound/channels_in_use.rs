//! `sound::channels_in_use` (0x409BA1): `cmp g_channelsUsed, 0`: the zero flag set when
//! no channel is in use.
//!
//! In:  nothing
//! Out: the flags of the compare (carry clear, zero as said); every register preserved
//! One caller, `sound::send_command_7`, whose `popa` / `retn` do not look at them.
//! Shadow mode: both, the two flags compared.
//! Self-test: a copy whose variable is the test's, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_read, rva};

/// g_channelsUsed.
const CHANNELS_RVA: usize = rva(0x506350);
// ---- Rust version ----

/// The zero flag: no channel in use.
pub fn channels_in_use(channels: u32) -> bool {
    channels == 0
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let channels: u32 = unsafe { game_read(CHANNELS_RVA) };
    r.set_zero(channels_in_use(channels));
    r.set_carry(false);
    Ok(())
}
