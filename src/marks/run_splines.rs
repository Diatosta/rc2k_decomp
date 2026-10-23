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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::marks::lines::SETTINGS_RVA;
use crate::marks::place_spline::{self, place_spline};
use crate::marks::setup::SEED_RVA;
use crate::marks::splines::{MADE_RVA, PLACE_RVA};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, state_entry};

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

    unsafe { run(entry, settings, places) }
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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
