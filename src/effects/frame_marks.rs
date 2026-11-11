//! `effects::frame_marks` (0x4D14B4): the effects' frame step. The time stamp counter read at
//! its start (kept on the stack) and again: the cycles since the last frame kept (with the
//! last count); the settings' +0x50 / +0x4C / +0x10 / +0x18 copied to the frame's switches;
//! `effects::scale_vectors_by`, `effects::morph_object_points`, `effects::view_frame`; the
//! settings' +0x18 0, the step counter plus 1; the cycles this step took over the frame's,
//! by a constant, kept as the step's share.
//!
//! Works on the effects settings.
//!
//! Callers: race init (`drive::race_init`) and `race::run_view_marks`. Refuses
//! null settings before anything, and its steps' refusals (all it may write is put back first).

use crate::effects::owner::Effects;
use crate::effects::owner::effects;
use crate::effects::run_frame::FRAME_K;
use crate::effects::{morph_object_points, scale_vectors_by, view_frame};
use crate::errors::Refused;
use crate::math::x87;
use crate::race::effects_settings_store::EffectsSettings;
use crate::race::owner::Race;
use crate::session::Dword;
/// (the settings' field, the copy): +0x50, +0x4C and +0x10.
fn copies(marks: &EffectsSettings) -> [(&Dword, &Dword); 3] {
    let setup = &effects().setup;
    [
        (&marks.listener_near, &setup.kind_floats.fading),
        (&marks.after_loading_copy, &setup.kind_floats.other),
        (&marks.other_marks_shown, &setup.kind_floats.running),
    ]
}

fn rdtsc() -> (u32, u32) {
    let t = crate::arch::rdtsc();
    (t as u32, (t >> 32) as u32)
}

pub fn frame_marks(race: &Race, effects: &Effects) -> Result<(), Refused> {
    let shared = &race.shared;
    let course = &race.course;
    let marks = effects.settings()?;
    {
        let (start, _) = rdtsc();
        let (now, _) = rdtsc();
        let last: u32 = effects.setup.frame.last_count.get();
        effects.setup.frame.last_count.set_bits(now);
        effects
            .setup
            .frame
            .frame_cycles
            .set_bits(now.wrapping_sub(last));
        for (from, to) in copies(marks) {
            to.set(from.get());
        }
        effects.setup.clock.settings_copies[0].set(marks.fresh_marks_flags[1].get());
        scale_vectors_by::scale_vectors_by(effects);
        morph_object_points::morph_object_points(course, &effects.paths, effects)?;
        view_frame::view_frame(shared, effects)?;
        marks.fresh_marks_flags[1].set(0);
        effects
            .setup
            .frame
            .steps
            .set(effects.setup.frame.steps.get().wrapping_add(1));
        let (end, _) = rdtsc();
        let share = x87::int_over_int_times(
            end.wrapping_sub(start) as i32,
            effects.setup.frame.frame_cycles.bits() as i32,
            FRAME_K.share_k.to_bits(),
        );
        effects.setup.frame.step_share.set_bits(share);
    }
    Ok(())
}
