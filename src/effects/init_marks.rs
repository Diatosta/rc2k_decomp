//! `effects::init_marks` (0x4D12F7): the marks set up for a stage: the settings and the
//! seed kept, then the fifteen steps (each after an empty call with its number), the
//! screen's middle and the kind's float.
//!
//! One caller, the end of the "init mark" step. Refuses a kind (the settings'
//! +0x26A8, which step 1 makes the effects kind) of 0x22 or more (the kinds' tables); both
//! before anything. Kinds 0x17 .. 0x21 are past the kind float table's 0x17 entries:
//! fixed (section 16), they get 18240.0.

use crate::effects::owner::Effects;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::owner::Race;
use crate::system::owner::system;

/// The table's entries; the kinds go to 0x21 (the other kinds' tables have 0x22).
const KIND_FLOATS: u32 = 0x17;
const KINDS: u32 = rc2k_core::effects::kind_values::KINDS;
/// 18240.0, the value of most of the table's kinds: the kinds past it get it (section 16).
const KIND_FLOAT_PAST: u32 = 0x468E_8000;

/// `seed` the seed (eax). The original took the settings'
/// address in esi; the race hands its one record (`race::effects_settings_store`), so none is
/// passed. The steps (`nullsub_1` before each, `nullsub_26` for step 7: nothing).
pub fn init_marks(
    race: &Race,
    effects: &Effects,
    seed: u32,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    let course = &race.course;
    let system = system();
    use crate::effects::*;
    let (ring, masks) = (&effects.ring, &effects.masks);
    let weather = &effects.weather;
    let paths = &effects.paths;
    {
        // Every read through `g_marksSettings` is a field of the race's record.
        let marks = race.effects_settings();
        if marks.mark_car_file_number.get() >= KINDS {
            return Err(BadInput("a marks kind of 0x22 or more").into());
        }
        effects.setup.settings.is_set.set(1);
        effects.setup.work.seed.set(seed);
        effects.setup.clock.started.set(0);
        edges::load_kind_data(course, effects, scene, gfx)?;
        setup::make_wave_tables(paths)?;
        profiles::make_profiles(course, paths, profiles::RESUME, effects)?;
        setup::seed_noise(weather, effects);
        setup::make_noise_curve(weather, effects)?;
        blocks::allocate_places(effects)?;
        kind_values::set_kind_values(&kind_values::first(effects), effects)?;
        kind_values::set_kind_values(&kind_values::second(effects), effects)?;
        load_mask_pictures::load_mask_pictures(masks, scene, gfx, effects)?;
        setup::clear_slots(effects);
        set_up_surface::set_up_surface(ring, masks, scene, gfx, effects)?;
        blocks::allocate_ring(&effects.particles, ring)?;
        lines::set_up_blocks(course, paths, scene, effects)?;
        marks.fresh_marks_flags[1].set(1);
        for r in &effects.setup.clock.settings_copies {
            r.set(1);
        }
        setup::tick_clock(&race.shared, effects)?;
        effects.setup.frame.steps.set_bits(0u32);
        let less = effects.setup.floats.one.get();
        for (n, side) in [&system.session.screen.width, &system.session.screen.height]
            .into_iter()
            .enumerate()
        {
            let half = side.get() >> 1;
            // The screen's middle x, it less `MarksFloats.one`, the same for y.
            effects.clipping.clip.edges[2 * n].set(x87::from_int(half as i32));
            effects.clipping.clip.edges[2 * n + 1].set(x87::int_less(half as i32, less));
        }
        let kind: u32 = effects.kinds.marks_kind.kind.get();
        // Fixed (section 16): the original reads past the table for kinds
        // 0x17 .. 0x21 (the next variables: `g_marksKindFloat` itself, a 0/1 flag, 9, ..),
        // so those cars' marks lose their depth bias.
        let float = if kind < KIND_FLOATS {
            effects.setup.kind_floats.kind_floats[kind as usize].bits()
        } else {
            KIND_FLOAT_PAST
        };
        effects.setup.kind_floats.kind_float.set_bits(float);
    }
    Ok(())
}
