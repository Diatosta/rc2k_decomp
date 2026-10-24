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
use crate::marks::lines::SETTINGS_RVA;
use crate::marks::store::{Slot, game_read, game_write};
use crate::marks::{morph_object_points, scale_vectors_by, view_frame};
use crate::math::x87;

const LAST_RVA: Slot = Slot::at(0x765110); // dword_765110
const FRAME_CYCLES_RVA: Slot = Slot::at(0x765114); // dword_765114
const SHARE_RVA: Slot = Slot::at(0x765128); // flt_765128
const SHARE_K_RVA: Slot = Slot::at(0x93C8C0); // flt_93C8C0
/// (the settings' field, the copy).
const COPIES: [(u32, Slot); 4] = [
    (0x50, Slot::at(0x76523C)),
    (0x4C, Slot::at(0x765238)),
    (0x10, Slot::at(0x7652A0)),
    (0x18, Slot::at(0x764FF0)),
];
const STEPS_RVA: Slot = Slot::at(0x76513C); // dword_76513C

fn rdtsc() -> (u32, u32) {
    let t = unsafe { core::arch::x86::_rdtsc() };
    (t as u32, (t >> 32) as u32)
}

/// The Rust version (see the header).
pub unsafe fn frame_marks() -> Result<(), Refused> {
    let settings: u32 = unsafe { game_read(SETTINGS_RVA) };
    if settings == 0 {
        return Err(BadInput("no marks settings").into());
    }
    unsafe {
        let (start, _) = rdtsc();
        let (now, _) = rdtsc();
        let last: u32 = game_read(LAST_RVA);
        game_write(LAST_RVA, now);
        game_write(FRAME_CYCLES_RVA, now.wrapping_sub(last));
        for (from, to) in COPIES {
            game_write(to, ((settings + from) as *const u32).read_unaligned());
        }
        scale_vectors_by::scale_vectors_by();
        morph_object_points::morph_object_points()?;
        view_frame::view_frame()?;
        ((settings + 0x18) as *mut u32).write_unaligned(0);
        game_write(STEPS_RVA, game_read::<u32>(STEPS_RVA).wrapping_add(1));
        let (end, _) = rdtsc();
        let share = x87::int_over_int_times(
            end.wrapping_sub(start) as i32,
            game_read(FRAME_CYCLES_RVA),
            game_read(SHARE_K_RVA),
        );
        game_write(SHARE_RVA, share);
    }
    Ok(())
}
