//! `sound::exclusive_spot_voices` (0x40EAF3): at most one voice of a kind starts: a kind
//! already playing, or started by an earlier voice in the table, silences the others.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! The kinds' bits (+0x1C) of the voices heard and playing gathered; then each voice
//! heard and not playing whose bit is among them is silenced (+4 0), and its bit added
//! either way. One caller, `sound::spot_sounds`.
//! Shadow mode: both, the voices compared.
//! Self-test: the original against the Rust version on the game's voices (put back).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::sound::spot_voices::{load_voices, store_voices};
use crate::sound::turn_parts::{Part, turn_entry};

// ---- Rust version ----

pub fn exclusive_spot_voices_in(_: &mut Regs, _: Part) -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::spot_voices::keep_exclusive`.
    let mut voices = load_voices();
    rc2k_core::sound::spot_voices::keep_exclusive(&mut voices);
    store_voices(&voices);
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, exclusive_spot_voices_in)
}
