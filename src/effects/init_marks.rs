//! `effects::init_marks`: the marks set up for a stage: the settings and the seed kept, then
//! the fifteen steps, the screen's middle and the kind's float.
//!
//! Refuses a kind (the settings' `mark_car_file_number`, which step 1 makes the effects kind) of
//! 0x22 or more (the kinds' tables); both before anything. Kinds 0x17 .. 0x21 are past the kind
//! float table's 0x17 entries: they get 18240.0 (wiki/Fixed-bugs.md: "Some cars' marks lose their
//! depth bias").

use crate::effects::owner::Effects;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::owner::Race;

/// The table's entries; the kinds go to 0x21 (the other kinds' tables have 0x22).
const KIND_FLOATS: u32 = 0x17;
const KINDS: u32 = rc2k_core::effects::kind_values::KINDS;
/// 18240.0, the value of most of the table's kinds: the kinds past it get it.
const KIND_FLOAT_PAST: f32 = 18240.0;

/// `seed` is the seed. The race hands its one settings record (`race::effects_settings_store`).
pub fn init_marks(
    race: &'static Race,
    effects: &Effects,
    seed: u32,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let course = &race.course;
    use crate::effects::*;
    let (ring, masks) = (&effects.ring, &effects.masks);
    let weather = &effects.weather;
    let paths = &effects.paths;
    {
        // Every read of the settings is a field of the race's record.
        let marks = race.effects_settings();
        if marks.mark_car_file_number.get() >= KINDS {
            return Err(BadInput("a marks kind of 0x22 or more").into());
        }
        effects.setup.marks.set(Some(marks));
        effects.setup.car_blocks.set(Some(race.car_blocks()));
        effects.setup.car_parts.set(Some(&race.shared.car_parts));
        effects.setup.stage.set(Some(&race.course.stage));
        effects.setup.settings.is_set.set(1);
        effects.setup.work.borrow_mut().seed = seed;
        effects.setup.clock.borrow_mut().started = 0;
        crate::logfile::load_times::mark("load_kind_data");
        edges::load_kind_data(race, effects, scene, gfx, system)?;
        crate::logfile::load_times::mark("make_wave_tables");
        setup::make_wave_tables(paths)?;
        crate::logfile::load_times::mark("make_profiles");
        profiles::make_profiles(scene, effects)?;
        crate::logfile::load_times::mark("seed_noise");
        setup::seed_noise(weather, effects);
        crate::logfile::load_times::mark("make_noise_curve");
        setup::make_noise_curve(weather, scene, effects)?;
        crate::logfile::load_times::mark("allocate_places");
        blocks::allocate_places(effects)?;
        crate::logfile::load_times::mark("set_kind_values");
        kind_values::set_kind_values(&kind_values::first(effects), effects)?;
        kind_values::set_kind_values(&kind_values::second(effects), effects)?;
        crate::logfile::load_times::mark("load_mask_pictures");
        load_mask_pictures::load_mask_pictures(masks, scene, gfx, effects, system)?;
        setup::clear_slots(effects);
        crate::logfile::load_times::mark("set_up_surface");
        set_up_surface::set_up_surface(ring, masks, scene, gfx, effects, system)?;
        crate::logfile::load_times::mark("allocate_ring");
        blocks::allocate_ring(&effects.particles, ring)?;
        crate::logfile::load_times::mark("set_up_blocks");
        lines::set_up_blocks(course, race.faces(), paths, scene, effects)?;
        marks.fresh_marks_flags[1].set(1);
        effects.setup.clock.borrow_mut().settings_copies = [1; 2];
        crate::logfile::load_times::mark("tick_clock");
        setup::tick_clock(&race.shared, effects)?;
        effects.setup.frame.steps.set(0);
        let less = effects.setup.floats.one;
        for (n, side) in [
            &system.session.screen.get().width,
            &system.session.screen.get().height,
        ]
        .into_iter()
        .enumerate()
        {
            let half = side.get() >> 1;
            // The screen's middle x, it less `MarksFloats.one`, the same for y.
            let mut clip = effects.clipping.clip.borrow_mut();
            clip.edges[2 * n] = x87::from_int(half as i32);
            clip.edges[2 * n + 1] = x87::int_less(half as i32, less);
        }
        let kind: u32 = effects.kinds.marks_kind.get();
        // Kinds 0x17 .. 0x21 are past the table and get `KIND_FLOAT_PAST` (wiki/Fixed-bugs.md:
        // "Some cars' marks lose their depth bias").
        let float = if kind < KIND_FLOATS {
            effects.setup.kind_floats.borrow().kind_floats[kind as usize]
        } else {
            KIND_FLOAT_PAST
        };
        effects.setup.kind_floats.borrow_mut().kind_float = float;
    }
    Ok(())
}
