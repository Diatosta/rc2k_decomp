//! `audio::set_eax_room` (0x40D26B): the EAX room asked for set.
//!
//! In:  nothing
//! Out: eax = the room's entry in `g_eaxRooms`; every other register preserved
//!
//! See `audio/property_sets.rs`: the entry of `g_eaxRoom` (an EAX 1 reverb, 0x10 bytes)
//! given to `g_eaxProperties`' `Set` when that is there (its result not read). The `Set`
//! is a chunk (0x40D325) entered by `jmp`. Callers: `audio::update_voices` (twice),
//! `audio::make_buffer`. Refuses a room past the table's ten (the original reads past it).
//! Shadow mode runs the original alone. Self-test: redirected copies of the head and the
//! chunk (the head's jump retargeted) whose room and set are the test's, 30 cases.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::property_sets::set_eax_room;
use crate::audio::voices::{EAX_ROOM_RVA, EAX_RVA, read};
use crate::errors::Refused;
use crate::framework::game_addr;

/// On the game's variables.
pub unsafe fn in_game() -> Result<u32, Refused> {
    unsafe {
        set_eax_room(
            read(game_addr(EAX_ROOM_RVA) as u32),
            game_addr(EAX_RVA) as u32,
        )
    }
}
