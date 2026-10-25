//! `audio::set_eax_room` (0x40D26B): the EAX room asked for set.
//!
//! In:  nothing
//! Out: eax = the room's entry in `g_eaxRooms`; every other register preserved
//!
//! See `audio/property_sets.rs`: the entry of `g_eaxRoom` (an EAX 1 reverb, 0x10 bytes)
//! given to `g_eaxProperties`' `Set` when that is there (its result not read). The `Set`
//! is a chunk (0x40D325) entered by `jmp`. Callers: `audio::update_voices` (twice),
//! `audio::make_buffer`. Refuses a room past the table's ten (the original reads past it).

use crate::audio::property_sets::set_eax_room;
use crate::audio::store::AUDIO;
use crate::errors::Refused;

/// On the game's variables.
pub fn in_game() -> Result<u32, Refused> {
    set_eax_room(AUDIO.eax_room.get())
}
