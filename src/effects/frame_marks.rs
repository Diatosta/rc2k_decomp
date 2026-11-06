//! `effects::frame_marks` (0x4D14B4): the marks' frame step. The time stamp counter read
//! at its start (kept on the stack) and again: `dword_765114` the cycles since the last
//! frame (`dword_765110` the last count); `dword_76523C` / `dword_765238` / `dword_7652A0`
//! / `dword_764FF0` the settings' +0x50 / +0x4C / +0x10 / +0x18; `effects::scale_vectors_by`,
//! `effects::morph_object_points` (through the jumps at 0x4D074D, 0x4D0815),
//! `effects::view_frame`; the settings' +0x18 0, `dword_76513C` plus 1; `flt_765128` the
//! cycles this step took over the frame's, by `flt_93C8C0`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads the registers it leaves: none after 0x446403, 0x456464)
//!
//! Callers: race init (0x446403) and 0x456464. Refuses null settings before anything, and
//! its steps' refusals (all it may write is put back first).

use crate::effects::owner::Effects;
use crate::effects::owner::effects;
use crate::effects::run_frame::FRAME_K;
use crate::effects::{morph_object_points, scale_vectors_by, view_frame};
use crate::errors::Refused;
use crate::math::x87;
use crate::race::marks_settings_store::MarksSettings;
use crate::race::owner::race;
use crate::session::Dword;
/// (the settings' field, the copy): +0x50, +0x4C and +0x10.
fn copies(marks: &MarksSettings) -> [(&Dword, &Dword); 3] {
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

/// The Rust version (see the header).
pub fn frame_marks(effects: &Effects) -> Result<(), Refused> {
    let shared = &crate::race::owner::race().shared;
    let course = &race().course;
    let marks = crate::effects::setup::marks()?;
    {
        let (start, _) = rdtsc();
        let (now, _) = rdtsc();
        let last: u32 = effects.setup.frame.last_count.bits();
        effects.setup.frame.last_count.set_bits(now);
        effects
            .setup
            .frame
            .frame_cycles
            .set_bits(now.wrapping_sub(last));
        for (from, to) in copies(marks) {
            to.set(from.get());
        }
        effects.setup.clock.settings_copies[0].set(marks.unknown_67c[1].get());
        scale_vectors_by::scale_vectors_by();
        morph_object_points::morph_object_points(course, &effects.paths)?;
        view_frame::view_frame(shared, effects)?;
        marks.unknown_67c[1].set(0);
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
