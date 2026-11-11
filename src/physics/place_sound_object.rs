//! `physics::place_sound_object` (0x45CD60): when the current car is listed: a sound is picked by
//! the kind of the object `face_object` (`g_objectKinds`, its material record's byte +0x24 & 0xF,
//! then `object_kind_sounds[]`), and `physics::sound_band_2` gives a band; a band not 0 goes to
//! the car's sound object with that sound and band, the place offset by `object_point_offset`
//! and the update flag set (see `physics/car_sounds.rs`). A reading: the sound of a roadside
//! object hit.
//!
//! Caller: `physics::touch_face`. Refuses (before anything) a car number past 7, a null sound
//! object when it is written, an object past 0x100 or a kind past the 0x4B0 records.

use crate::errors::Refused;
use crate::physics::car_sounds::{cue_plain, listed, object};
use crate::physics::sound_band_2::TWIN;
use crate::physics::sound_bands::band_value_plain;
use crate::physics::step_families::SoundInputs;
use rc2k_core::physics::sound_places::{FIRST, object_sound};

// Converted: `rc2k_core::physics::sound_places::{object_sound, place}`.
pub fn place_sound_object(
    c: &SoundInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let contact = c.contact;
    if !listed(c.frame_work, c.work.current_car.get())? {
        return Ok(());
    }
    let trap = crate::physics::cells::Trap::new();
    let sound = object_sound(
        contact.face_object.get(),
        |thing| scene.objects.tables.object_kind(thing),
        |kind| {
            // The material's byte +0x24 (the low byte of its tenth word).
            trap.keep(
                crate::scene::polygon_store::POLYGON_TABLES
                    .object_materials
                    .get(kind as usize)
                    .map(|m| m.words[9].get() as u8)
                    .ok_or(crate::errors::BadInput("an object kind past the materials")),
            )
        },
        |n| c.spots.object_kind_sounds[n as usize].get(),
    )?;
    trap.finish()?;
    let b = band_value_plain(c, TWIN);
    if b == 0 {
        return Ok(());
    }
    let offset = Some(
        contact
            .object_point_offset
            .each_ref()
            .map(super::super::session::Dword::bits),
    );
    cue_plain(c, object(), FIRST, sound, b, offset)?;
    Ok(())
}
