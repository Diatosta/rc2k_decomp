//! `scene::fill_light_tables`: the five tables of colours the lighting picks from.
//!
//! With `scene_set_up` and the stage's `lit_b` both 0 (the plain way): the light table a ramp
//! between the two `plain_light_colours`, and the four lamp tables that first colour scaled. Else:
//! the light table a ramp between the stage's `dark_colour` and `lit_colour`, and the four lamp
//! tables the stage's lamp colours scaled.

use crate::scene::light_tables::{fill_light_ramp, fill_light_scale};

// Logic: `rc2k_core::scene::lamps::plain_light`
/// `N` dwords of the stage's prefs from byte `offset` on (a colour read on into the fields
/// after it).
fn prefs_dwords<const N: usize>(
    course: &crate::race::owner_course::Course,
    offset: usize,
) -> Result<[u32; N], crate::errors::BadInput> {
    let mut bytes = vec![0; 4 * N];
    crate::memory::cells::load_cells(course.stage_prefs.get(), offset, &mut bytes)?;
    let mut d = [0; N];
    for (v, b) in d.iter_mut().zip(bytes.as_chunks::<4>().0) {
        *v = u32::from_le_bytes(*b);
    }
    Ok(d)
}

pub fn fill_light_tables(
    scene: &crate::scene::owner::Scene,
    race: &crate::race::owner::Race,
) -> Result<(), crate::errors::BadInput> {
    use crate::race::prefs_store::StagePrefs;
    let course = &race.course;
    let mut t = scene.light_tables.borrow_mut();
    use core::mem::offset_of;
    let plain =
        rc2k_core::scene::lamps::plain_light(race.shared.switches.get().scene_set_up.get(), || {
            course.stage_prefs.get().lit_b.get()
        });
    // The plain colours are two (the light and the dark); the stage's are two (the dark one
    // and the lit).
    let p = &crate::scene::light_work_store::PLAIN_LIGHT_COLOURS;
    let ramp: [u32; 6] = if plain {
        std::array::from_fn(|k| p[k])
    } else {
        prefs_dwords(course, offset_of!(StagePrefs, dark_colour))?
    };
    fill_light_ramp(ramp, &mut t.light_table);
    // The four lamp tables and, the other way, their colours.
    for (which, colour) in [
        (0, offset_of!(StagePrefs, lamp_table_a)),
        (1, offset_of!(StagePrefs, lamp_table_b)),
        (2, offset_of!(StagePrefs, car_light_table)),
    ] {
        let table = match which {
            0 => &mut t.lamp_table_a,
            1 => &mut t.lamp_table_b,
            _ => &mut t.car_light_table,
        };
        let colour = if plain {
            [p[0], p[1], p[2]]
        } else {
            prefs_dwords(course, colour)?
        };
        fill_light_scale(colour, table);
    }
    // The third lamp table is only kept in the stage's own preferences; its colours are still
    // read, so a bad stage is refused as before.
    if !plain {
        prefs_dwords::<3>(course, offset_of!(StagePrefs, lamp_table_c))?;
    }
    Ok(())
}
