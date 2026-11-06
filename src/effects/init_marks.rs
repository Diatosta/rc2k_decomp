//! `effects::init_marks` (0x4D12F7): the marks set up for a stage: the settings and the
//! seed kept, then the fifteen steps (each after `nullsub_1` with its number in eax), the
//! screen's middle and the kind's float.
//!
//! In:  the seed (eax; the original also took the settings in esi, here always the race's
//!      one; its ecx and edi are read by no step)
//! Out: nothing (no caller reads the registers it leaves)
//!
//! One caller, init mark's end (0x457030). Refuses a kind (the settings'
//! +0x26A8, which step 1 makes `g_marksKind`) of 0x22 or more (the kinds' tables); both
//! before anything. Kinds 0x17 .. 0x21 are past `g_marksKindFloatTable`'s 0x17 entries:
//! fixed (section 16, 2026-10-23), they get 18240.0.

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

/// The Rust version (see the header): `seed` the seed (eax). The original took the settings'
/// address in esi; the race hands its one record (`race::marks_settings_store`), so none is
/// passed. The steps (`nullsub_1` before each, `nullsub_26` for step 7: nothing).
pub fn init_marks(race: &Race, effects: &Effects, seed: u32) -> Result<(), Refused> {
    let course = &race.course;
    let system = system();
    use crate::effects::*;
    let (ring, masks) = (&effects.ring, &effects.masks);
    let weather = &effects.weather;
    let paths = &effects.paths;
    {
        // Every read through `g_marksSettings` is a field of the race's record.
        let marks = &*crate::race::marks_settings_store::MARKS_SETTINGS;
        if marks.mark_car_file_number.get() >= KINDS {
            return Err(BadInput("a marks kind of 0x22 or more").into());
        }
        effects.setup.settings.is_set.set(1);
        effects.setup.work.seed.set(seed);
        effects.setup.clock.started.set(0);
        edges::load_kind_data(course, effects)?;
        setup::make_wave_tables(paths)?;
        profiles::make_profiles(course, paths, profiles::RESUME)?;
        setup::seed_noise(weather);
        setup::make_noise_curve(weather)?;
        blocks::allocate_places()?;
        kind_values::set_kind_values(&kind_values::FIRST)?;
        kind_values::set_kind_values(&kind_values::SECOND)?;
        load_mask_pictures::load_mask_pictures(masks)?;
        setup::clear_slots();
        set_up_surface::set_up_surface(ring, masks)?;
        blocks::allocate_ring(&effects.particles, ring)?;
        lines::set_up_blocks(course, paths)?;
        marks.unknown_67c[1].set(1);
        for r in &effects.setup.clock.settings_copies {
            r.set(1);
        }
        setup::tick_clock(&race.shared)?;
        effects.setup.frame.steps.set_bits(0u32);
        let less: u32 = effects.setup.floats.one.bits();
        for (n, side) in [&system.session.screen.width, &system.session.screen.height]
            .into_iter()
            .enumerate()
        {
            let half = side.get() >> 1;
            // The screen's middle x, it less `flt_76516C`, the same for y.
            effects.clipping.clip.edges[2 * n].set_bits(x87::from_int(half as i32));
            effects.clipping.clip.edges[2 * n + 1].set_bits(x87::int_less(half as i32, less));
        }
        let kind: u32 = effects.kinds.marks_kind.kind.bits();
        // Fixed (section 16, 2026-10-23): the original reads past the table for kinds
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
