//! `scene::fill_light_tables` (0x44F0AB): the five tables of colours the lighting
//! picks from.
//!
//! With `dword_621480` and `dword_60931C` both 0 (the plain way): `g_lightTable` a ramp between
//! the two colours at `unk_62DB10`, and the four lamp tables (`dword_635518`, `dword_635918`,
//! `g_carLightTable`, `unk_636118`) that first colour scaled. Else: `g_lightTable` a ramp
//! between the two colours at `dword_6092B4` (the dark and the lit colour of the scene), and
//! the four lamp tables the colours at `unk_60A54C`, `unk_60A558`, `dword_60A564`,
//! `dword_60A570` scaled. Four callers.

use crate::scene::light_tables::{fill_light_ramp, fill_light_scale};

// ---- Rust version ----

// Converted: `rc2k_core::scene::lamps::plain_light`
/// `N` dwords of the stage's prefs from byte `offset` on (a colour read on into the
/// fields after it, as the original's pointers do).
fn prefs_dwords<const N: usize>(
    course: &crate::race::owner_course::Course,
    offset: usize,
) -> [u32; N] {
    let d = course.stage_prefs.dwords();
    std::array::from_fn(|k| d[offset / 4 + k].get())
}

pub fn fill_light_tables(
    scene: &crate::scene::owner::Scene,
    race: &crate::race::owner::Race,
) -> Result<(), crate::errors::BadInput> {
    use crate::race::prefs_store::StagePrefs;
    let course = &race.course;
    let t = &scene.light_tables;
    use core::mem::offset_of;
    let plain =
        rc2k_core::scene::lamps::plain_light(race.shared.switches.scene_set_up.get(), || {
            course.stage_prefs.lit_b.get()
        });
    // `unk_62DB10`: two colours; `dword_6092B4`: two colours (the dark one and the lit).
    let p = &scene.lights.work.plain_light_colours;
    let ramp: [u32; 6] = if plain {
        std::array::from_fn(|k| p[k].get())
    } else {
        prefs_dwords(course, offset_of!(StagePrefs, dark_colour))
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
            prefs_dwords(course, colour)
        };
        fill_light_scale(colour, table);
    }
    Ok(())
}
