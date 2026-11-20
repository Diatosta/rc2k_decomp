//! `effects::frame_marks`: the effects' frame step. The cycle counter read at its start and again:
//! the cycles since the last frame kept (with the last count); the settings' `listener_near`,
//! `after_loading_copy` and `other_marks_shown` copied to the frame's switches;
//! `effects::scale_vectors_by`, `effects::morph_object_points`, `effects::view_frame`; the
//! settings' second `fresh_marks_flags` 0, the step counter plus 1; the cycles this step took over
//! the frame's, by a constant, kept as the step's share.
//!
//! Refuses null settings before anything, and its steps' refusals (all it may write is put back
//! first).

use crate::effects::owner::Effects;
use crate::effects::run_frame::FRAME_K;
use crate::effects::{morph_object_points, scale_vectors_by, view_frame};
use crate::errors::Refused;
use crate::math::x87;
use crate::memory::session::PlainDword;
use crate::race::effects_settings_store::EffectsSettings;
use crate::race::owner::Race;
/// (the settings' field, the copy): `listener_near`, `after_loading_copy` and
/// `other_marks_shown`.
fn copies<'a>(
    effects: &'a Effects,
    marks: &'a EffectsSettings,
) -> [(&'a PlainDword, &'a PlainDword); 3] {
    let setup = &effects.setup;
    [
        (&marks.listener_near, &setup.kind_floats.fading),
        (&marks.after_loading_copy, &setup.kind_floats.other),
        (&marks.other_marks_shown, &setup.kind_floats.running),
    ]
}

pub fn frame_marks(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    effects: &Effects,
) -> Result<(), Refused> {
    let shared = &race.shared;
    let course = &race.course;
    let marks = effects.settings()?;
    {
        let start = crate::effects::cycles::start();
        if let Some(now) = start {
            let last: u32 = effects.setup.frame.last_count.get();
            effects.setup.frame.last_count.set(now);
            effects
                .setup
                .frame
                .frame_cycles
                .set(now.wrapping_sub(last) as i32);
        }
        for (from, to) in copies(effects, marks) {
            to.set(from.get());
        }
        effects.setup.clock.borrow_mut().settings_copies[0] = marks.fresh_marks_flags[1].get();
        scale_vectors_by::scale_vectors_by(effects);
        morph_object_points::morph_object_points(
            course,
            race.faces(),
            &effects.paths,
            scene,
            effects,
        )?;
        view_frame::view_frame(shared, effects)?;
        marks.fresh_marks_flags[1].set(0);
        effects
            .setup
            .frame
            .steps
            .set(effects.setup.frame.steps.get().wrapping_add(1));
        if let Some(took) = crate::effects::cycles::since(start) {
            let share = x87::int_over_int_times(
                took,
                effects.setup.frame.frame_cycles.get(),
                FRAME_K.share_k,
            );
            effects.setup.frame.step_share.set(share);
        }
    }
    Ok(())
}
