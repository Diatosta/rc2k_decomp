//! `sound::let_sounds_play` (0x409B76): `g_soundsHeldBack` 0.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! Its twin `sound::hold_sounds_back` (0x409B6B) sets it to 1; `sound::play_sound` plays
//! nothing while it is 1. Five callers (`race::start_race_sounds` and four more).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{game_write, rva};

pub const HELD_BACK_RVA: usize = rva(0x506C84); // g_soundsHeldBack

pub fn let_sounds_play() {
    unsafe { game_write(HELD_BACK_RVA, 0u32) };
}
