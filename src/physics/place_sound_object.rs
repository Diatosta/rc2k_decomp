//! `physics::place_sound_object` (0x45CD60): when the current car is listed: a sound by the kind of the
//! object `dword_6D321C` (`g_objectKinds`, its material record's byte +0x24 & 0xF, then
//! `dword_6D1A90[]`) in eax, `physics::sound_band_2` (ebx); a band not 0 goes to the car's
//! sound object with +0x238 the sound, +0x23C the band, the place offset by
//! `dword_6D3B4C` .. at +0x240 .. and +0x10 1 (see `physics/car_sounds.rs`). A reading:
//! the sound of a roadside object hit.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_45E9E0`. Refuses (before anything) a car number past 7, a null sound object
//! when it is written, an object past 0x100 or a kind past the 0x4B0 records.

use crate::errors::Refused;
use crate::framework::{game_addr, rva};
use crate::physics::car_sounds::{cue_plain, g, listed, object};
use crate::physics::sound_band_2::TWIN;
use crate::physics::sound_bands::band_value_plain;
use rc2k_core::physics::sound_places::{FIRST, object_sound};

/// The Rust version.
// Converted: `rc2k_core::physics::sound_places::{object_sound, place}`.
pub unsafe fn place_sound_object() -> Result<(), Refused> {
    if !listed(g(0x71B3F8))? {
        return Ok(());
    }
    let sound = object_sound(
        g(0x6D321C),
        |thing| g(0x5CD27C + 4 * thing as usize),
        |kind| unsafe { *(game_addr(rva(0x97DF54 + 0x80 * kind as usize)) as *const u8) },
        |n| g(0x6D1A90 + 4 * n as usize),
    )?;
    let scratch = g(0x6D1AEC);
    let b = unsafe { band_value_plain(TWIN) };
    if b == 0 {
        return Ok(());
    }
    // The band left `dword_6D1AEC` written: put back if the object refuses.
    let o = match object() {
        Ok(o) => o,
        Err(e) => {
            unsafe { crate::framework::game_write(rva(0x6D1AEC), scratch) };
            return Err(e);
        }
    };
    let offset = Some([g(0x6D3B4C), g(0x6D3B50), g(0x6D3B54)]);
    unsafe { cue_plain(o, FIRST, sound, b, offset) };
    Ok(())
}
