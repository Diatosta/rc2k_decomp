//! `physics::place_sound_object` (0x45CD60): when the current car is listed: a sound by the kind of the
//! object `dword_6D321C` (`g_objectKinds`, its material record's byte +0x24 & 0xF, then
//! `dword_6D1A90[]`) in eax, `physics::sound_band_2` (ebx); a band not 0 goes to the car's
//! sound object with +0x238 the sound, +0x23C the band, the place offset by
//! `dword_6D3B4C` .. at +0x240 .. and +0x10 1 (see `physics/car_sounds.rs`). A reading:
//! the sound of a roadside object hit.
//!
//! Caller: `physics::touch_face`. Refuses (before anything) a car number past 7, a null sound
//! object when it is written, an object past 0x100 or a kind past the 0x4B0 records.

use crate::errors::Refused;
use crate::physics::car_sounds::{cue_plain, listed, object};
use crate::physics::sound_band_2::TWIN;
use crate::physics::sound_bands::band_value_plain;
use crate::race::owner::Race;
use rc2k_core::physics::sound_places::{FIRST, object_sound};

/// The Rust version.
// Converted: `rc2k_core::physics::sound_places::{object_sound, place}`.
pub fn place_sound_object(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let physics_store = &race.physics;
    if !listed(&race.frame, race.shared.work.current_car.get())? {
        return Ok(());
    }
    let trap = crate::physics::cells::Trap::new();
    let sound = object_sound(
        physics_store.contact.face_object.get(),
        |thing| scene.objects.tables.object_kind(thing),
        |kind| {
            // The material's byte +0x24 (the low byte of its tenth word).
            trap.keep(
                crate::scene::polygon_store::POLYGON_WORK
                    .object_materials
                    .get(kind as usize)
                    .map(|m| m.words[9].get() as u8)
                    .ok_or(crate::errors::BadInput("an object kind past the materials")),
            )
        },
        |n| race.frame.sound_spots.object_kind_sounds[n as usize].get(),
    )?;
    trap.finish()?;
    let b = band_value_plain(race, TWIN);
    if b == 0 {
        return Ok(());
    }
    let offset = Some(
        physics_store
            .contact
            .object_point_offset
            .each_ref()
            .map(super::super::session::Dword::bits),
    );
    cue_plain(race, object(), FIRST, sound, b, offset)?;
    Ok(())
}
