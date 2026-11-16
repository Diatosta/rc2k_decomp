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
    let t = scene.light_tables.get();
    use core::mem::offset_of;
    let plain =
        rc2k_core::scene::lamps::plain_light(race.shared.switches.get().scene_set_up.get(), || {
            course.stage_prefs.get().lit_b.get()
        });
    // The plain colours are two (the light and the dark); the stage's are two (the dark one
    // and the lit).
    let p = &scene.lights.work.get().plain_light_colours;
    let ramp: [u32; 6] = if plain {
        std::array::from_fn(|k| p[k].get())
    } else {
        prefs_dwords(course, offset_of!(StagePrefs, dark_colour))?
    };
    fill_light_ramp(ramp, &t.light_table);
    // The four lamp tables and, the other way, their colours.
    let lamps = [
        (&t.lamp_table_a, offset_of!(StagePrefs, lamp_table_a)),
        (&t.lamp_table_b, offset_of!(StagePrefs, lamp_table_b)),
        (&t.car_light_table, offset_of!(StagePrefs, car_light_table)),
        (&t.lamp_table_c, offset_of!(StagePrefs, lamp_table_c)),
    ];
    for (table, colour) in lamps {
        let colour = if plain {
            [p[0].get(), p[1].get(), p[2].get()]
        } else {
            prefs_dwords(course, colour)?
        };
        fill_light_scale(colour, table);
    }
    Ok(())
}
