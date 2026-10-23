//! `sound::play_car_stream` (0x469AA1): the car shown's sound stream started.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The car (as `menu::load_car_background` finds it, 0x21 counting as 0x17):
//! `dword_A8C188` = 1, `dword_A8C184` = `off_75A31E[car]`, `dword_A8C180` = 0, and
//! `sound::start_stream` with eax = the first dword there and ecx = `dword_755698`.
//! Five call sites, next to `load_car_background`'s.
//! Refuses what the record's lookup refuses.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::menu::load_car_background::shown_car;

// ---- game data ----

// dword_755990
const STREAMS_RVA: usize = rva(0x75A31E); // off_75A31E
const ON_RVA: usize = rva(0xA8C188); // dword_A8C188
const STREAM_RVA: usize = rva(0xA8C184); // dword_A8C184
const PLACE_RVA: usize = rva(0xA8C180); // dword_A8C180
const VOLUME_RVA: usize = rva(0x755698); // dword_755698

// ---- Rust version ----

/// `start` is `sound::start_stream` (the file, the volume).
pub unsafe fn play_car_stream(start: &mut dyn FnMut(u32, u32)) -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::set_up::car_stream`.
    unsafe {
        let car = shown_car()?;
        let s = rc2k_core::sound::set_up::car_stream(
            game_read(STREAMS_RVA + car as usize * 4),
            |list| (list as *const u32).read_unaligned(),
            game_read(VOLUME_RVA),
        );
        game_write(ON_RVA, s.on);
        game_write(STREAM_RVA, s.list);
        game_write(PLACE_RVA, s.place);
        start(s.first, s.volume);
        Ok(())
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { play_car_stream(&mut crate::sound::start_stream::start_stream) }
}
