//! `marks::frame_marks` (0x4D14B4): the marks' frame step. The time stamp counter read
//! at its start (kept on the stack) and again: `dword_765114` the cycles since the last
//! frame (`dword_765110` the last count); `dword_76523C` / `dword_765238` / `dword_7652A0`
//! / `dword_764FF0` the settings' +0x50 / +0x4C / +0x10 / +0x18; `marks::scale_vectors_by`,
//! `marks::morph_object_points` (through the jumps at 0x4D074D, 0x4D0815),
//! `marks::view_frame`; the settings' +0x18 0, `dword_76513C` plus 1; `flt_765128` the
//! cycles this step took over the frame's, by `flt_93C8C0`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: the registers as `view_frame` leaves them (edx before the calls the counter's high
//!      half)
//!
//! Callers: race init (0x446403) and 0x456464. Refuses null settings before anything, and
//! its steps' refusals (all it may write is put back first). Shadow mode and the
//! self-test compare all but the three timing variables (`rdtsc`: no two runs agree).
//! Self-test: as `view_frame`'s, when no spline is placed.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::marks::lines::SETTINGS_RVA;
use crate::marks::{morph_object_points, scale_vectors_by, view_frame};
use crate::math::x87;
use crate::service::harness::{Region, at, capture, restore};

const LAST_RVA: usize = rva(0x765110); // dword_765110
const FRAME_CYCLES_RVA: usize = rva(0x765114); // dword_765114
const SHARE_RVA: usize = rva(0x765128); // flt_765128
const SHARE_K_RVA: usize = rva(0x93C8C0); // flt_93C8C0
/// (the settings' field, the copy).
const COPIES: [(u32, usize); 4] = [
    (0x50, rva(0x76523C)),
    (0x4C, rva(0x765238)),
    (0x10, rva(0x7652A0)),
    (0x18, rva(0x764FF0)),
];
const STEPS_RVA: usize = rva(0x76513C); // dword_76513C

fn rdtsc() -> (u32, u32) {
    let t = unsafe { core::arch::x86::_rdtsc() };
    (t as u32, (t >> 32) as u32)
}

/// What it may write but the timing variables.
fn regions() -> Vec<Region> {
    let settings: u32 = unsafe { game_read(SETTINGS_RVA) };
    let mut r: Vec<Region> = COPIES.map(|c| at(c.1)).to_vec();
    r.push(at(STEPS_RVA));
    if settings != 0 {
        r.push((settings + 0x18, 4));
    }
    r.extend(scale_vectors_by::regions());
    r.extend(morph_object_points::regions(settings));
    r.extend(view_frame::regions());
    r
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn frame_marks(entry: &Regs) -> Result<Regs, Refused> {
    let settings: u32 = unsafe { game_read(SETTINGS_RVA) };
    if settings == 0 {
        return Err(BadInput("no marks settings").into());
    }
    let mut written = regions();
    written.extend([LAST_RVA, FRAME_CYCLES_RVA, SHARE_RVA].map(at));
    let saved = capture(&written);
    let done = unsafe { run(entry, settings) };
    if done.is_err() {
        restore(&written, &saved);
    }
    done
}

unsafe fn run(entry: &Regs, settings: u32) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let (start, _) = rdtsc();
        let (now, high) = rdtsc();
        g.ebx = game_read(LAST_RVA);
        game_write(LAST_RVA, now);
        g.eax = now.wrapping_sub(g.ebx);
        g.edx = high;
        game_write(FRAME_CYCLES_RVA, g.eax);
        g.esi = settings;
        for (from, to) in COPIES {
            g.eax = ((settings + from) as *const u32).read_unaligned();
            game_write(to, g.eax);
        }
        scale_vectors_by::scale_vectors_by();
        g = morph_object_points::morph_object_points(&g)?;
        g = view_frame::view_frame(&g)?;
        g.esi = settings;
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
    Ok(g)
}
