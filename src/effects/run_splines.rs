//! `effects::run_splines` (0x4B978A): the splines' frame. `g_marksSplinesMade` stepped by
//! half `g_marksFrameTicks`: toward 0x80 while the settings' +0x2700 is set (+0x26FC then
//! 0); else up to 0xFF and back to 0 while +0x26FC is set (cleared there); else on away
//! from 0x80 (up when at or above it, to 0 past 0xFF; down below it, to 0 under 1). Then
//! the wave: `flt_7695B6` plus (the marks' random's low byte less 0x80) / `flt_788F0D` by
//! `flt_788F11`, and `flt_7695BA` its sine by `flt_788F15`; then `effects::place_spline` for
//! each of `g_marksSplinePlace` splines, the last first.
//!
//! Works on `g_marksSettings`.
//!
//! Caller: `effects::view_frame`. Refuses null settings and more than 8 splines (before anything),
//! and `place_spline`'s refusals (all it may write is put back first).

use crate::effects::paths::Paths;
use crate::effects::place_spline::place_spline;
use crate::errors::{BadInput, Refused};
use crate::math::x87;

const MOST_SPLINES: i32 = 8;

/// The made counter's step (see the header).
fn step_made(
    effects: &crate::effects::owner::Effects,
    paths: &Paths,
    half: u32,
) -> Result<(), Refused> {
    let marks = effects.settings()?;
    {
        let mut m = paths.spline_head.made.get() as i32;
        let h = half as i32;
        if marks.spline_handbrake.get() != 0 {
            marks.spline_gear_changed.set(0);
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
        } else if marks.spline_gear_changed.get() != 0 {
            m = m.wrapping_add(h);
            if m > 0xFF {
                marks.spline_gear_changed.set(0);
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
        paths.spline_head.made.set(m as u32);
    }
    Ok(())
}

pub fn run_splines(paths: &Paths, effects: &crate::effects::owner::Effects) -> Result<(), Refused> {
    let setup = &effects.setup;
    let settings: u32 = setup.settings.is_set.get();
    if settings == 0 {
        return Err(BadInput("no marks settings").into());
    }
    let places = paths.spline_work.place.get() as i32;
    if places > MOST_SPLINES {
        return Err(BadInput("more than 8 splines").into());
    }
    {
        step_made(effects, paths, setup.clock.frame_ticks.get() >> 1)?;
        let s = setup.work.seed.get().wrapping_mul(0x9D).wrapping_add(0x47);
        setup.work.seed.set(s);
        let k = paths
            .spline_constants
            .wave_k
            .each_ref()
            .map(super::super::session::Float::bits);
        let (angle, wave) = x87::angle_and_wave(
            &paths.spline_head.angle,
            (s & 0xFF).wrapping_sub(0x80) as i32,
            k[0],
            k[1],
            k[2],
        );
        paths.spline_head.angle.set(angle);
        paths.spline_head.wave.set(wave);
        for n in (0..places.max(0) as u32).rev() {
            place_spline(paths, n, effects)?;
        }
    }
    Ok(())
}
