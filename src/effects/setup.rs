//! `effects::init_marks`' (0x4D12F7) first leaves (IDA): the noise tables, the
//! wave tables (made once), the slots cleared, and the marks' clock (also run each frame by
//! `effects::run_frame`, 0x4D1549).

use crate::effects::paths::Paths;
use crate::effects::rain::Weather;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::race::owner_shared::RaceShared;
use crate::session::{PlainDword, PlainFloat};

/// The marks' clock (`effects::tick_clock`) (IDA 0x764FC8 .. 0x764FFC).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MarksClock {
    /// 0x764FC8: `g_marksClockStarted`: 0 until `effects::tick_clock` has run once (`init_marks` clears it).
    pub started: PlainDword,
    /// 0x764FCC: `g_marksLastClock`: the scene clock at the last tick.
    pub last: PlainDword,
    /// 0x764FD0: `g_marksFrameTicks`: the clock's ticks since the last frame (0 unless the settings' +0x10 is 1).
    pub frame_ticks: PlainDword,
    /// 0x764FD4: `flt_764FD4`: the ticks' bits taken as a float, times `flt_93CC14` (a game bug, section 16).
    pub ticks_scaled: PlainFloat,
    /// 0x764FD8: `dword_764FD8`: the ticks shifted right 4.
    pub sixteenth: PlainDword,
    /// 0x764FDC: `dword_764FDC`: the ticks shifted right 2.
    pub quarter: PlainDword,
    /// 0x764FE0: `flt_764FE0`: one over the ticks (`fidiv`); the first tick's `flt_93CC10`.
    pub reciprocal: PlainFloat,
    /// 0x764FE4: `flt_764FE4`: `reciprocal` times `flt_7651BC`.
    pub reciprocal_scaled: PlainFloat,
    /// 0x764FE8: `g_marksClockNow`: the scene clock now.
    pub now: PlainDword,
    /// 0x764FEC: `g_marksClockBefore`: the clock at the tick before.
    pub before: PlainDword,
    /// 0x764FF0: `dword_764FF0`: a copy of the settings' +0x18 (`effects::frame_marks`), 1 from `init_marks`; `dword_764FF4`: 1 from `init_marks` (nothing ported reads it).
    pub settings_copies: [PlainDword; 2],
}

impl MarksClock {
    pub const fn new() -> Self {
        Self {
            started: PlainDword::new(0x0),
            last: PlainDword::new(0x0),
            frame_ticks: PlainDword::new(0x0),
            ticks_scaled: PlainFloat::new(0.0),
            sixteenth: PlainDword::new(0x0),
            quarter: PlainDword::new(0x0),
            reciprocal: PlainFloat::new(0.0),
            reciprocal_scaled: PlainFloat::new(0.0),
            now: PlainDword::new(0x0),
            before: PlainDword::new(0x0),
            settings_copies: [PlainDword::new(0x0), PlainDword::new(0x0)],
        }
    }
}

const WAVE_DONE_TARGET: u32 = 0x4C4EA0; // nullsub_23
pub const WAVE_LEVELS: u32 = 0x11;
pub const SINES: u32 = 0x140;

/// `effects::seed_noise` (0x4C1697): 0x100 steps of `g_marksSeed` (times 0x9D plus 0x47),
/// each giving entry 0xFF .. 0 of the three noise tables its low 18 bits, its low 18 bits
/// turned right 10 and turned right 20; `RainState.drawn` 0.
pub fn seed_noise(weather: &Weather, effects: &crate::effects::owner::Effects) {
    let setup = &effects.setup;
    // Converted: `rc2k_core::effects::noise::seed_noise`.
    let (tables, seed) = rc2k_core::effects::noise::seed_noise(setup.work.seed.get());
    for (table, values) in weather.rain.noise.iter().zip(tables) {
        for (cell, v) in table.iter().zip(values) {
            cell.set(v);
        }
    }
    setup.work.seed.set(seed);
    weather.rain.drawn.set(0);
}

/// `effects::make_wave_tables` (0x4C4E2F): once (`g_marksWaveDone` 0): the 0x11 levels,
/// whole numbers, become floats (over `WaveState.scale`, less `WaveState.less`), the 0x140 sines of
/// 0, step, 2 step .. (`math::x87::sine_table`); then `g_marksWaveDone` = `nullsub_23`.
/// Done before: nothing. Refuses a mark that is neither 0 nor `nullsub_23` (the original
/// jumps to it).
pub fn make_wave_tables(paths: &Paths) -> Result<(), Refused> {
    let done = paths.wave.done.get();
    let target = WAVE_DONE_TARGET;
    if done == target {
        return Ok(());
    }
    if done != 0 {
        return Err(BadInput("the wave tables' mark is neither 0 nor nullsub_23").into());
    }
    let (scale, less) = (paths.wave.scale.get(), paths.wave.less.get());
    for level in paths
        .wave_levels
        .levels
        .iter()
        .take(WAVE_LEVELS as usize)
        .rev()
    {
        level.set(bh::st(
            bh::int(level.get() as i32) / bh::ld(scale) - bh::ld(less),
        ));
    }
    let mut sines = vec![0f32; SINES as usize];
    x87::sine_table(&paths.wave.sine_step, &mut sines);
    for (cell, &v) in paths.sines.table.iter().zip(&sines) {
        cell.set(v);
    }
    paths.wave.done.set(target);
    Ok(())
}

/// `effects::clear_slots` (0x4C967B): the first dword of the 0x20 slots 0.
pub fn clear_slots(effects: &crate::effects::owner::Effects) {
    for slot in &effects.blocks.slots.slots {
        slot.life.set(0);
    }
}

/// `effects::tick_clock` (0x4D2BDC). The first time (`g_marksClockStarted` 0): started, the
/// last, now and before `g_sceneClock`, the ticks 0, the reciprocal `ClockK.first_reciprocal`. Then:
/// before the last, now and the last the clock, the ticks the difference (0 unless
/// `g_marksSettings` +0x10 is 1), a quarter and a sixteenth of them (shifted), the
/// reciprocal one over them (`fidiv`) and that times `MarksFloats.quarter`, and the ticks' bits
/// taken as a float (section 16) times `ClockK.ticks_scale`.
pub fn tick_clock(
    shared: &RaceShared,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    let c = &setup.clock;
    {
        let clock: u32 = shared.physics_tables.get().scene_clock.get();
        if c.started.get() == 0 {
            c.started.set(1);
            for r in [&c.last, &c.now, &c.before] {
                r.set(clock);
            }
            c.frame_ticks.set(0);
            c.reciprocal
                .set(x87::loaded(&setup.clock_k.first_reciprocal));
            return Ok(());
        }
        c.now.set(clock);
        let last = c.last.get();
        c.before.set(last);
        c.last.set(clock);
        let marks = crate::effects::setup::effects_settings(effects)?;
        let ticks = if marks.other_marks_shown.get() == 1 {
            clock.wrapping_sub(last)
        } else {
            0
        };
        c.frame_ticks.set(ticks);
        c.quarter.set(ticks >> 2);
        c.sixteenth.set(ticks >> 4);
        let (reciprocal, scaled) =
            x87::reciprocal_int_and_times(ticks as i32, &setup.floats.quarter);
        c.reciprocal.set(reciprocal);
        c.reciprocal_scaled.set(scaled);
        c.ticks_scaled
            .set(x87::mul(ticks, &setup.clock_k.ticks_scale));
    }
    Ok(())
}

pub const CURVE_LEN: u32 = 0x1000;
pub const CURVE_RECORDS: u32 = 0x20;

/// `effects::make_noise_curve` (0x4C2247), `init_marks`' step 5, with `g_marksSeed` started
/// at 0x7FFFFFF:
/// - a walk of 0x1000 steps into `g_marksCurve` (each the value << 4 times 8.5; the step
///   moved by 2 up, or down when the running seed sum carries, held to -0x1000 .. 0x1000;
///   the value plus the step times 0.9);
/// - 0xA0 times: the curve copied, and each of 1 .. 0xFFD made 0.3, 0.4, 0.3 of its
///   neighbours and itself (`x87::weighted3_to_int`);
/// - the copy's 0 .. 0xFFD less a ramp from 0 by (copy 0xFFF - copy 0) / 0x1000;
/// - 0x800 times three 20-bit randoms and (the seed >> 30, three bits) times 0.0625 plus
///   `SnowLight.spread_base`; then 0x800 times two remainders (`idiv` by 0xC00);
/// - `SnowState.flag` 0, the 0x20 records' four quarters, the three quads.
///
/// Refuses a divisor that is 0 or 1 or -1 (the original's `idiv` traps).
// Converted: `rc2k_core::effects::noise::{walk, smooth, random_tables, record_values}`
pub fn make_noise_curve(
    weather: &Weather,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    use rc2k_core::effects::noise as core;
    let divisors = weather
        .snow_light
        .divisors
        .each_ref()
        .map(|d| d.get() as i32);
    if !divisors.into_iter().all(core::divisor_fits) {
        return Err(BadInput("a remainder's divisor is 0, 1 or -1: idiv traps").into());
    }
    for (cell, v) in weather.snow_vars.curve_vars.iter().zip([
        0u32,
        core::LEAST_STEP as u32,
        core::MOST_STEP as u32,
        core::CHANGE as u32,
        0,
    ]) {
        cell.set(v);
    }
    // The roundings follow the x87's control word: the x87's.
    let (scale, damp) = (
        weather.snow_light.curve_scale.get(),
        weather.snow_light.curve_damp.get(),
    );
    let w = core::walk(
        weather.snow_vars.curve_vars[5].get(),
        |v| x87::int_times_to_int(v, scale),
        |v| x87::int_times_to_int(v, damp),
    );
    weather.snow_vars.curve_vars[5].set(w.sum as u32);
    weather.snow_vars.curve_vars[0].set(w.value as u32);
    weather.snow_vars.curve_vars[4].set(w.step as u32);
    let mut curve = w.curve;
    let mut copy = vec![0; CURVE_LEN as usize];
    let weights = weather
        .snow_light
        .smooth_weights
        .each_ref()
        .map(super::super::session::PlainFloat::get);
    for _ in 0..core::SMOOTHINGS {
        core::smooth(&mut curve, &mut copy, |a, b, c| {
            x87::weighted3_to_int(a, b, c, weights[0], weights[1], weights[2])
        });
    }
    for (cell, v) in weather.snow_curve.curve.iter().zip(&curve) {
        cell.set(*v as u32);
    }
    for (cell, v) in weather.snow_curve.copy.iter().zip(&copy) {
        cell.set(*v as u32);
    }
    let mut copy: Vec<i32> = weather
        .snow_curve
        .copy
        .iter()
        .map(|c| c.get() as i32)
        .collect();
    let (last, first) = (copy[0xFFF], copy[0]);
    x87::ramp_down(
        &mut copy[..0xFFE],
        last,
        first,
        weather.snow_light.ramp_by.get(),
    );
    for (cell, &v) in weather.snow_curve.copy.iter().zip(&copy) {
        cell.set(v as u32);
    }
    let (spread_scale, spread_base): (f32, f32) = (
        weather.snow_light.spread_scale.get(),
        weather.snow_light.spread_base.get(),
    );
    let t = core::random_tables(w.seed, divisors);
    for (cells, table) in weather.snow_tables.randoms.iter().zip(&t.randoms) {
        for (cell, v) in cells.iter().zip(table) {
            cell.set(*v);
        }
    }
    let spread = t
        .threes
        .map(|three| x87::int_times_plus(three, spread_scale, spread_base));
    for (cell, v) in weather.snow_tables.spreads.iter().zip(spread) {
        cell.set(v);
    }
    for (cells, table) in weather.snow_tables.remainders.iter().zip(&t.remainders) {
        for (cell, v) in cells.iter().zip(table) {
            cell.set(*v);
        }
    }
    effects.setup.work.seed.set(t.seed);
    weather.snow.flag.set(0);
    let values = weather
        .snow_light
        .record_values
        .each_ref()
        .map(super::super::session::PlainFloat::bits);
    let layout = core::record_values(values);
    for n in 0..CURVE_RECORDS as usize {
        for &(o, v) in &layout {
            weather.snow_vars.records[n][o as usize / 4].set(v);
        }
    }
    let first = x87::loaded(&weather.snow_light.quad_first);
    for cell in &weather.snow.quad_a {
        cell.set(first);
    }
    // The view's centre (`SceneDraw.view_centre[0]`, `SceneDraw.view_centre[1]`).
    let centre = &crate::scene::polygon_work::front_end().window.view_centre;
    for (quad, from) in [
        (&weather.snow.quad_b, &centre[0]),
        (&weather.snow.quad_c, &centre[1]),
    ] {
        let v = from.get();
        for cell in quad {
            cell.set(v);
        }
    }
    Ok(())
}

/// The marks' settings flag (IDA 0x767A28, phase 7, typed state): `g_marksSettings`, which
/// held the address of the settings the game hands `effects::init_marks`; it is 1 once they are
/// handed over and 0 before (nothing runs without them). The settings are the race's one
/// record, `EFFECTS_SETTINGS` (the original kept its address).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SettingsPointer {
    /// 0x767A28: `g_marksSettings`: 1 when the settings are set, else 0.
    pub is_set: PlainDword,
}

impl SettingsPointer {
    pub const fn new() -> Self {
        Self {
            is_set: PlainDword::new(0),
        }
    }
}

/// The marks' settings the race handed `effects::init_marks` (`race::effects_settings_store`,
/// the one record `g_marksSettings` points at); refused while it points at none, where the
/// original's reads through it would fault.
pub fn effects_settings(
    effects: &crate::effects::owner::Effects,
) -> Result<&'static crate::race::effects_settings_store::EffectsSettings, BadInput> {
    effects.settings()
}

/// The marks' clock constants (IDA 0x93CC10 .. 0x93CC18).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ClockK {
    /// 0x93CC10: `flt_93CC10`: the reciprocal the first tick starts with
    pub first_reciprocal: PlainFloat,
    /// 0x93CC14: `flt_93CC14`: the ticks' scale
    pub ticks_scale: PlainFloat,
}

impl ClockK {
    /// ClockK's first values.
    pub const fn new() -> Self {
        Self {
            first_reciprocal: PlainFloat::new(1e-6),
            ticks_scale: PlainFloat::new(0.1),
        }
    }
}
