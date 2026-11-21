//! `scene::fill_light_tables`: the five tables of colours the lighting picks from.
//!
//! With `scene_set_up` and the stage's `lit_b` both 0 (the plain way): the light table a ramp
//! between the two `plain_light_colours`, and the four lamp tables that first colour scaled. Else:
//! the light table a ramp between the stage's `dark_colour` and `lit_colour`, and the four lamp
//! tables the stage's lamp colours scaled.

use crate::scene::light_tables::{fill_light_ramp, fill_light_scale};

// Logic: `rc2k_core::scene::lamps::plain_light`
/// The values of a run of the stage's prefs cells.
fn values<const N: usize>(cells: &[crate::memory::session::PlainDword; N]) -> [u32; N] {
    std::array::from_fn(|k| cells[k].get())
}

pub fn fill_light_tables(
    scene: &crate::scene::owner::Scene,
    race: &crate::race::owner::Race,
) -> Result<(), crate::errors::BadInput> {
    let course = &race.course;
    let mut t = scene.light_tables.borrow_mut();
    let prefs = course.stage_prefs.get();
    let plain =
        rc2k_core::scene::lamps::plain_light(race.shared.switches.borrow().scene_set_up, || {
            prefs.lit_b.get()
        });
    // The plain colours are two (the light and the dark); the stage's are two (the dark one
    // and the lit).
    let p = &crate::scene::light_work_store::PLAIN_LIGHT_COLOURS;
    let ramp: [u32; 6] = if plain {
        std::array::from_fn(|k| p[k])
    } else {
        // The dark colour and the lit colour, one after the other.
        let (dark, lit) = (values(&prefs.dark_colour), values(&prefs.lit_colour));
        [dark[0], dark[1], dark[2], lit[0], lit[1], lit[2]]
    };
    fill_light_ramp(ramp, &mut t.light_table);
    // The four lamp tables and, the other way, their colours.
    for (which, colour) in [
        (0, &prefs.lamp_table_a),
        (1, &prefs.lamp_table_b),
        (2, &prefs.car_light_table),
    ] {
        let table = match which {
            0 => &mut t.lamp_table_a,
            1 => &mut t.lamp_table_b,
            _ => &mut t.car_light_table,
        };
        let colour = if plain {
            [p[0], p[1], p[2]]
        } else {
            values(colour)
        };
        fill_light_scale(colour, table);
    }
    Ok(())
}
