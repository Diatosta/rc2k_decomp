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

use crate::errors::Refused;
use crate::framework::{game_read, rva};
use crate::menu::load_car_background::shown_car;

// ---- game data ----

// dword_755990
const STREAMS_IDA: usize = 0x75A31E; // off_75A31E: `MENU_LISTS.car_streams`
const VOLUME_RVA: usize = rva(0x755698); // dword_755698

// ---- Rust version ----

/// `start` is `sound::start_stream` (the file, the volume).
pub unsafe fn play_car_stream(start: &mut dyn FnMut(u32, u32)) -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::set_up::car_stream`.
    unsafe {
        let car = shown_car()?;
        let s = rc2k_core::sound::set_up::car_stream(
            // `MENU_LISTS.car_streams` (off_75A31E) has 24 entries; the original reads any
            // car's, past them what the image has after (`MOVED` resolves the address).
            game_read(rva(STREAMS_IDA + car as usize * 4)),
            |list| (list as *const u32).read_unaligned(),
            game_read(VOLUME_RVA),
        );
        crate::sound::store::CAR_STREAM.on.set(s.on);
        crate::sound::store::CAR_STREAM.stream.set(s.list);
        crate::sound::store::CAR_STREAM.place.set(s.place);
        start(s.first, s.volume);
        Ok(())
    }
}
