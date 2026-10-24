//! `sound::hold_sounds_back` (0x409B6B): `g_soundsHeldBack` 1.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! Its twin `sound::let_sounds_play` (0x409B76) sets it to 0; `sound::play_sound` plays
//! nothing while it is 1. One caller (0x40D388).
//! Self-test: the variable from 1 and from 0, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::sound::let_sounds_play::HELD_BACK_RVA;
use crate::store::game_write;

pub fn hold_sounds_back() {
    unsafe { game_write(HELD_BACK_RVA, 1u32) };
}
