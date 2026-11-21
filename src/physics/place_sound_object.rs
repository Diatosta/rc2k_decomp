//! When the current car is listed: a sound is picked by the kind of the object `face_object` (its
//! material record's kind bits, then `OBJECT_KIND_SOUNDS`), and the object band (`sound_bands::OBJECT_BAND`) gives a
//! band; a band not 0 goes to the car's sound object with that sound and band, the place offset by
//! `object_point_offset` and the update flag set (see `physics/car_sounds.rs`). A reading: the
//! sound of a roadside object hit.
//!
//! Refuses (before anything) a car number past 7, a null sound object when it is written, an object
//! past 0x100 or a kind past the 0x4B0 records.

use crate::errors::Refused;
use crate::physics::car_sounds::{cue_plain, listed, object};
use crate::physics::sound_bands::{OBJECT_BAND, band_value_plain};
use crate::physics::step_families::SoundInputs;
use rc2k_core::physics::sound_places::{FIRST, object_sound};

/// The sound by object kind, a 4-bit index.
const OBJECT_KIND_SOUNDS: [u32; 16] = [
    0xD7, 0xD7, 0xD8, 0xD9, 0xDA, 0xDB, 0xDC, 0xDD, 0xDE, 0xDF, 0xE0, 0xE1, 0xD7, 0xD7, 0xD7, 0xD7,
];

// Core: `rc2k_core::physics::sound_places::{object_sound, place}`.
pub fn place_sound_object(
    c: &SoundInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    if !listed(c.frame_work, c.work.current_car.get())? {
        return Ok(());
    }
    let face_object = c.contact.borrow().face_object;
    let trap = crate::physics::cells::Trap::new();
    let sound = object_sound(
        face_object,
        |thing| scene.objects.tables.borrow().object_kind(thing),
        |kind| {
            // The material's kind byte (the low byte of its tenth word).
            trap.keep(crate::scene::polygon::material_run_byte(
                c.polygon_tables,
                kind,
                0x24,
            ))
        },
        |n| OBJECT_KIND_SOUNDS[n as usize],
    )?;
    trap.finish()?;
    let b = band_value_plain(c, &OBJECT_BAND);
    if b == 0 {
        return Ok(());
    }
    let offset = Some(c.contact.borrow().object_point_offset);
    cue_plain(c, object(c), FIRST, sound, b, offset)?;
    Ok(())
}
