//! `marks::run_splines` (0x4B978A): the splines' frame. `g_marksSplinesMade` stepped by
//! half `g_marksFrameTicks`: toward 0x80 while the settings' +0x2700 is set (+0x26FC then
//! 0); else up to 0xFF and back to 0 while +0x26FC is set (cleared there); else on away
//! from 0x80 (up when at or above it, to 0 past 0xFF; down below it, to 0 under 1). Then
//! the wave: `flt_7695B6` plus (the marks' random's low byte less 0x80) / `flt_788F0D` by
//! `flt_788F11`, and `flt_7695BA` its sine by `flt_788F15`; then `marks::place_spline` for
//! each of `g_marksSplinePlace` splines, the last first.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: the registers as the last `place_spline` leaves them (eax the random's byte less
//!      0x80, esi the settings, with none)
//!
//! Caller: `sub_4C1270`. Refuses null settings and more than 8 splines (before anything),
//! and `place_spline`'s refusals (all it may write is put back first). Self-test: the
//! game's settings (skipped without them), records of the test's in the game's tables
//! (`place_spline::fill_test_records`), switches, counts and ticks by lot.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::lines::SETTINGS_RVA;
use crate::marks::place_spline::{self, place_spline};
use crate::marks::setup::SEED_RVA;
use crate::marks::splines::{MADE_RVA, PLACE_RVA};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::run_splines", rva(0x4B978A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TICKS_RVA: usize = rva(0x764FD0); // g_marksFrameTicks
const ANGLE_RVA: usize = rva(0x7695B6); // flt_7695B6
const WAVE_RVA: usize = rva(0x7695BA); // flt_7695BA
/// flt_788F0D, flt_788F11, flt_788F15.
const WAVE_K: [usize; 3] = [rva(0x788F0D), rva(0x788F11), rva(0x788F15)];
const MOST_SPLINES: i32 = 8;

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// What it may write.
pub fn regions(settings: u32) -> Vec<Region> {
    let mut r: Vec<Region> = [MADE_RVA, SEED_RVA, ANGLE_RVA, WAVE_RVA].map(at).to_vec();
    if settings != 0 {
        r.push((settings + 0x26FC, 4));
    }
    let places: i32 = unsafe { game_read(PLACE_RVA) };
    for n in 0..places.clamp(0, MOST_SPLINES) as u32 {
        r.extend(place_spline::regions(n));
    }
    r
}

/// The made counter's step (see the header).
unsafe fn step_made(settings: u32, half: u32) {
    unsafe {
        let mut m: i32 = game_read(MADE_RVA);
        let h = half as i32;
        if get(settings + 0x2700) != 0 {
            put(settings + 0x26FC, 0);
            if m < 0x80 {
                m = m.wrapping_add(h);
                if m > 0x80 {
                    m = 0x80;
                }
            } else if m > 0x80 {
                m = m.wrapping_sub(h);
                if m < 0x80 {
                    m = 0x80;
                }
            }
        } else if get(settings + 0x26FC) != 0 {
            m = m.wrapping_add(h);
            if m > 0xFF {
                put(settings + 0x26FC, 0);
                m = 0;
            }
        } else if m != 0 {
            if m >= 0x80 {
                m = m.wrapping_add(h);
                if m > 0xFF {
                    m = 0;
                }
            } else {
                m = m.wrapping_sub(h);
                if m < 1 {
                    m = 0;
                }
            }
        }
        game_write(MADE_RVA, m);
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn run_splines(entry: &Regs) -> Result<Regs, Refused> {
    let settings: u32 = unsafe { game_read(SETTINGS_RVA) };
    if settings == 0 {
        return Err(BadInput("no marks settings").into());
    }
    let places: i32 = unsafe { game_read(PLACE_RVA) };
    if places > MOST_SPLINES {
        return Err(BadInput("more than 8 splines").into());
    }
    let written = regions(settings);
    let saved = capture(&written);
    let done = unsafe { run(entry, settings, places) };
    if done.is_err() {
        restore(&written, &saved);
    }
    done
}

unsafe fn run(entry: &Regs, settings: u32, places: i32) -> Result<Regs, Refused> {
    let mut g = Regs {
        esi: settings,
        ..*entry
    };
    unsafe {
        step_made(settings, game_read::<u32>(TICKS_RVA) >> 1);
        let s = game_read::<u32>(SEED_RVA)
            .wrapping_mul(0x9D)
            .wrapping_add(0x47);
        game_write(SEED_RVA, s);
        g.eax = (s & 0xFF).wrapping_sub(0x80);
        let k = WAVE_K.map(|r| game_read::<u32>(r));
        let (angle, wave) =
            x87::angle_and_wave(game_read(ANGLE_RVA), g.eax as i32, k[0], k[1], k[2]);
        game_write(ANGLE_RVA, angle);
        game_write(WAVE_RVA, wave);
        for n in (0..places.max(0) as u32).rev() {
            g.eax = n;
            g = place_spline(&g)?;
        }
    }
    Ok(g)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { run_splines(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let settings: u32 = unsafe { game_read(SETTINGS_RVA) };
    dispatch_state(&HOOK, r, call_original, &regions(settings), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    use crate::marks::splines::{CHOSEN_RVA, MOST_POINTS, TestSpline, work_regions};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let settings: u32 = unsafe { game_read(SETTINGS_RVA) };
    if settings == 0 {
        return t.skip("no marks settings");
    }
    let spline = TestSpline::new();
    let targets = vec![0u32; 3 * MOST_POINTS as usize];
    let base = targets.as_ptr() as u32;
    let mut fixed: Vec<Region> = (0..2).flat_map(place_spline::record_regions).collect();
    fixed.extend(
        [
            MADE_RVA,
            SEED_RVA,
            ANGLE_RVA,
            WAVE_RVA,
            PLACE_RVA,
            TICKS_RVA,
            CHOSEN_RVA,
            rva(0x76A20A),
            rva(0x7695CA),
            rva(0x71C814),
        ]
        .map(at),
    );
    fixed.extend(work_regions());
    fixed.push((game_read_addr(rva(0x76A1D2)), 0x30));
    fixed.push((settings + 0x26DC, 4));
    fixed.push((settings + 0x26FC, 8));
    fixed.push((base, 4 * targets.len()));
    let saved = capture(&fixed);
    // (the switches +0x2700, +0x26FC, the made count, the splines, the ticks)
    for (case, (on, up, made, places, ticks)) in [
        (1u32, 0u32, 0x20i32, 2i32, 6u32),
        (1, 1, 0xA0, 1, 3),
        (0, 1, 0xF0, 0, 40),
        (0, 0, 0x90, 2, 9),
        (0, 0, 0x10, 1, 50),
        (0, 0, 0, 0, 2),
    ]
    .into_iter()
    .enumerate()
    {
        let prepare = || unsafe {
            restore(&fixed, &saved);
            put(settings + 0x2700, on);
            put(settings + 0x26FC, up);
            game_write(MADE_RVA, made);
            game_write(PLACE_RVA, places);
            game_write(TICKS_RVA, ticks);
            game_write(SEED_RVA, case as u32 * 0x2545_F491);
            for n in 0..2 {
                place_spline::fill_test_records(&spline, n, case as u32 + 1, base);
            }
            std::ptr::write_bytes(base as *mut u8, 0xAA, 4 * targets.len());
        };
        prepare();
        let regions = regions(settings);
        check_state(
            &mut t,
            &format!("on {on}, up {up}, made {made:#x}, splines {places}, ticks {ticks}"),
            call_original,
            Regs::SENTINEL,
            &prepare,
            &regions,
            &rust,
        );
    }
    restore(&fixed, &saved);
    drop(targets);
    t.finish()
}

/// A game variable's address.
fn game_read_addr(r: usize) -> u32 {
    crate::framework::game_addr(r) as u32
}
