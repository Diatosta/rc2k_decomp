//! The race sounds' turn (stream B, 2026-10-18): `sound::race_turn` (0x40D5F3) and the
//! trees under it, which work on the eight cars' sound objects (`g_raceSoundObjects`,
//! 0x4A8 bytes each), the shared spots' voices (`unk_5168C0`) and one request
//! (`unk_5124BC`, ten dwords, see `sound::play_placed_sound`) they send each sound with.
//! This file: what they share, and the three ways a sound goes to the audio thread.
//!
//! Plain functions since 2026-10-23 (phase 3): every function of the turn keeps the
//! registers (`pusha` / `popa`, or none changed that a caller reads), so they take what
//! they read as arguments. The registers a sound command is queued with besides those
//! `play_sound` sets were the callers' leftovers; the audio thread reads none of them
//! (a stop is the sound and a volume of 0: `rc2k_core::audio::voices::plan`), so they
//! are 0 now.

use crate::errors::Refused;
use crate::framework::{game_addr, rva};

/// unk_5124BC: the request the turn's sounds go with.
pub const REQUEST: usize = 0x5124BC;
/// dword_5124E8: the time of the sounds' last turn (`sound/race_clock.rs`).
pub const CLOCK: usize = 0x5124E8;
/// g_raceSoundObjects: eight pointers.
pub const OBJECTS: usize = 0x512A10;
/// dword_514F7C: the sound objects in use.
pub const COUNT: usize = 0x514F7C;
/// dword_5183CC: the scratch dword the x87 code goes through.
pub const SCRATCH: usize = 0x5183CC;

/// The request's address.
pub fn request() -> u32 {
    game_addr(rva(REQUEST)) as u32
}

pub use crate::sound::play_sound::play_sound as play;

/// `sound::stop_sound`: the request's sound stopped unless it has bit 0x8000.
pub fn stop(request: u32) -> Result<(), Refused> {
    Ok(unsafe { crate::sound::stop_sound::stop_sound(request) }?)
}

/// `sound::play_placed_sound` with the request.
pub fn placed(request: u32) -> Result<(), Refused> {
    Ok(crate::sound::play_placed_sound::play_placed_sound_in_game(
        request,
    )?)
}

/// `sound::step_race_clock`: true when a turn is due (the original's carry clear).
pub fn step_clock() -> bool {
    unsafe { crate::sound::race_clock::step_race_clock(crate::system::tick_count::tick_count()) }
}
