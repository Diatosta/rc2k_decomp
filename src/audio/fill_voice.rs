//! `audio::fill_voice` (0x40B06E): a new voice filled for its sample record.
//!
//! In:  eax = the voice it shares data with (0: none), ebx = the record, esi = the voice
//! Out: every register preserved
//!
//! Without a shared voice: data, length, format, frames and rate from `g_sampleFile`, no
//! source, no next copy, the record's `data_key`. With one: the same five from it, it as
//! the source, the voice added at the end of the record's chain (unless it is on it
//! already), no next copy and no `data_key`. Then from the record: flags 1 with
//! DirectSound's buffers (`g_soundMode` bit 0x10000) else 2, sound, kind, scale,
//! priority, volume, the pan's low byte (signed), pan sign 0, the frequency (or the
//! voice's own for `OWN_FREQUENCY`), the place, the minimum distance from its own, the volume
//! factor (0x40, or what the record's points at); state 0, dirty bits 0xF more, no
//! channel, no list link. One caller, command 3 (`sub_40AEB9`).
//! Refuses a shared voice for a record without voices, a chain without its end.
//! Compared in shadow mode. Self-test: a redirected copy whose sample file and mode are the
//! test's, records and voices of the test's, 48 cases.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::samples::*;
use crate::audio::voices::*;
use crate::errors::Refused;
use crate::framework::{game_addr, game_read};

pub unsafe fn fill_voice_in_game(at: u32, record: u32, shared: u32) -> Result<(), Refused> {
    unsafe {
        let file = *(game_addr(SAMPLE_FILE_RVA) as *const SampleFile);
        fill_voice(at, record, shared, &file, game_read(SOUND_MODE_RVA))
    }
}
