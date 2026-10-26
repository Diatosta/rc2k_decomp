//! `marks::frame_marks` (0x4D14B4): the marks' frame step. The time stamp counter read
//! at its start (kept on the stack) and again: `dword_765114` the cycles since the last
//! frame (`dword_765110` the last count); `dword_76523C` / `dword_765238` / `dword_7652A0`
//! / `dword_764FF0` the settings' +0x50 / +0x4C / +0x10 / +0x18; `marks::scale_vectors_by`,
//! `marks::morph_object_points` (through the jumps at 0x4D074D, 0x4D0815),
//! `marks::view_frame`; the settings' +0x18 0, `dword_76513C` plus 1; `flt_765128` the
//! cycles this step took over the frame's, by `flt_93C8C0`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads the registers it leaves: none after 0x446403, 0x456464)
//!
//! Callers: race init (0x446403) and 0x456464. Refuses null settings before anything, and
//! its steps' refusals (all it may write is put back first).

use crate::errors::{BadInput, Refused};
use crate::marks::run_frame::FRAME_K;
use crate::marks::setup::CLOCK;
use crate::marks::tuning::{MARKS_FRAME, MARKS_KIND_FLOATS};
use crate::marks::{morph_object_points, scale_vectors_by, view_frame};
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::session::Dword;
/// (the settings' field, the copy).
static COPIES: [(u32, &Dword); 3] = [
    (0x50, &MARKS_KIND_FLOATS.fading),
    (0x4C, &MARKS_KIND_FLOATS.other),
    (0x10, &MARKS_KIND_FLOATS.running),
];

fn rdtsc() -> (u32, u32) {
    let t = unsafe { crate::arch::_rdtsc() };
    (t as u32, (t >> 32) as u32)
}

/// The Rust version (see the header).
pub fn frame_marks() -> Result<(), Refused> {
    let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
    if settings == 0 {
        return Err(BadInput("no marks settings").into());
    }
    {
        let (start, _) = rdtsc();
        let (now, _) = rdtsc();
        let last: u32 = MARKS_FRAME.last_count.bits();
        MARKS_FRAME.last_count.set_bits(now);
        MARKS_FRAME.frame_cycles.set_bits(now.wrapping_sub(last));
        for &(from, to) in &COPIES {
            to.set(GamePtr::new(settings + from).read::<u32>("a mark's field")?);
        }
        CLOCK.settings_copies[0].set(GamePtr::new(settings + 0x18).read::<u32>("a mark's field")?);
        scale_vectors_by::scale_vectors_by();
        morph_object_points::morph_object_points()?;
        view_frame::view_frame()?;
        GamePtr::new(settings + 0x18).write::<u32>(0, "a mark's field")?;
        MARKS_FRAME
            .steps
            .set(MARKS_FRAME.steps.get().wrapping_add(1));
        let (end, _) = rdtsc();
        let share = x87::int_over_int_times(
            end.wrapping_sub(start) as i32,
            MARKS_FRAME.frame_cycles.bits() as i32,
            FRAME_K.share_k.bits(),
        );
        MARKS_FRAME.step_share.set_bits(share);
    }
    Ok(())
}
