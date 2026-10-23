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
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::sound::spot_voices::{load_voices, store_voices};

// ---- Rust version ----

pub fn exclusive_spot_voices() {
    // Converted: `rc2k_core::sound::spot_voices::keep_exclusive`.
    let mut voices = load_voices();
    rc2k_core::sound::spot_voices::keep_exclusive(&mut voices);
    store_voices(&voices);
}
