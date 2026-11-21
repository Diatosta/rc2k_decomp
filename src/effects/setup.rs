//! `effects::init_marks`' first leaves: the noise tables, the wave tables (made once), the slots
//! cleared, and the marks' clock (also run each frame by `effects::run_frame`).

use crate::effects::paths_store::Paths;
use crate::effects::rain_store::Weather;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::race::owner_shared::RaceShared;
const WAVE_LEVELS: u32 = 0x11;

/// `effects::seed_noise`: 0x100 steps of the marks' seed (times 0x9D plus 0x47), each giving
/// entry 0xFF .. 0 of the three noise tables its low 18 bits, its low 18 bits turned right
/// 10 and turned right 20; `RainState.drawn` 0.
pub fn seed_noise(weather: &Weather, effects: &crate::effects::owner::Effects) {
    let setup = &effects.setup;
    // The logic is in `rc2k_core::effects::noise::seed_noise`.
    let (tables, seed) = rc2k_core::effects::noise::seed_noise(setup.work.borrow().seed);
    {
        let mut rain = weather.rain.borrow_mut();
        for (table, values) in rain.noise.iter_mut().zip(tables) {
            table.copy_from_slice(&values);
        }
        rain.drawn = 0;
    }
    setup.work.borrow_mut().seed = seed;
}

/// `effects::make_wave_tables`: once: the 0x11 levels, whole numbers, become floats (over 256,
/// less 1). Done before: nothing.
pub(super) fn make_wave_tables(paths: &Paths) -> Result<(), Refused> {
    let mut waves = paths.wave_levels.borrow_mut();
    if waves.made {
        return Ok(());
    }
    let (scale, less) = (256.0f32, 1.0f32);
    for level in waves.levels.iter_mut().take(WAVE_LEVELS as usize).rev() {
        *level = bh::st(bh::int(*level as i32) / bh::ld(scale) - bh::ld(less));
    }
    waves.made = true;
    Ok(())
}

/// `effects::clear_slots`: the first dword of the 0x20 slots 0.
pub(super) fn clear_slots(effects: &crate::effects::owner::Effects) {
    for slot in &mut effects.blocks.slots.borrow_mut().slots {
        slot.life = 0;
    }
}

/// `effects::tick_clock`. The first time (not started yet): started, the last, now and
/// before the scene clock, the ticks 0, the reciprocal `CLOCK_FIRST_RECIPROCAL`. Then:
/// before the last, now and the last the clock, the ticks the difference (0 unless the
/// settings' `other_marks_shown` is 1), a quarter and a sixteenth of them (shifted), the reciprocal one
/// over them and that times `MarksFloats.quarter`, and the ticks' bits taken as a float
/// (a slip kept as is) times `CLOCK_TICKS_SCALE`.
pub(super) fn tick_clock(
    shared: &RaceShared,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    let mut c = setup.clock.borrow_mut();
    {
        let clock: u32 = shared.run().scene_clock;
        if c.started == 0 {
            c.started = 1;
            c.last = clock;
            c.now = clock;
            c.before = clock;
            c.frame_ticks = 0;
            c.reciprocal = x87::loaded(super::setup_store::CLOCK_FIRST_RECIPROCAL);
            return Ok(());
        }
        c.now = clock;
        let last = c.last;
        c.before = last;
        c.last = clock;
        let marks = crate::effects::setup::effects_settings(effects)?;
        let ticks = if marks.other_marks_shown.get() == 1 {
            clock.wrapping_sub(last)
        } else {
            0
        };
        c.frame_ticks = ticks;
        c.quarter = ticks >> 2;
        c.sixteenth = ticks >> 4;
        let (reciprocal, scaled) =
            x87::reciprocal_int_and_times(ticks as i32, setup.floats.quarter);
        c.reciprocal = reciprocal;
        c.reciprocal_scaled = scaled;
        c.ticks_scaled = x87::mul(ticks, super::setup_store::CLOCK_TICKS_SCALE);
    }
    Ok(())
}

pub const CURVE_LEN: u32 = 0x1000;
const CURVE_RECORDS: u32 = 0x20;

/// `effects::make_noise_curve`, `init_marks`' step 5, with the marks' seed started
/// at 0x7FFFFFF:
/// - a walk of 0x1000 steps into the curve (each the value << 4 times 8.5; the step
///   moved by 2 up, or down when the running seed sum carries, held to -0x1000 .. 0x1000;
///   the value plus the step times 0.9);
/// - 0xA0 times: the curve copied, and each of 1 .. 0xFFD made 0.3, 0.4, 0.3 of its
///   neighbours and itself (`x87::weighted3_to_int`);
/// - the copy's 0 .. 0xFFD less a ramp from 0 by (copy 0xFFF - copy 0) / 0x1000;
/// - 0x800 times three 20-bit randoms and (the seed >> 30, three bits) times 0.0625 plus
///   `SnowLight.spread_base`; then 0x800 times two remainders (of a division by 0xC00);
/// - `SnowState.flag` 0, the 0x20 records' four quarters, the three quads.
///
/// Refuses a divisor that is 0 or 1 or -1 (the division would trap).
// The logic is in `rc2k_core::effects::noise::{walk, smooth, random_tables, record_values}`
pub(super) fn make_noise_curve(
    weather: &Weather,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    use rc2k_core::effects::noise as core;
    let light = weather.snow_light.borrow();
    let divisors = light.divisors.map(|d| d as i32);
    if !divisors.into_iter().all(core::divisor_fits) {
        return Err(BadInput("a remainder's divisor is 0, 1 or -1: idiv traps").into());
    }
    {
        let mut vars = weather.snow_vars.borrow_mut();
        for (cell, v) in vars.curve_vars.iter_mut().zip([
            0u32,
            core::LEAST_STEP as u32,
            core::MOST_STEP as u32,
            core::CHANGE as u32,
            0,
        ]) {
            *cell = v;
        }
    }
    // The roundings follow the x87's control word: the x87's.
    let (scale, damp) = (light.curve_scale, light.curve_damp);
    let w = core::walk(
        weather.snow_vars.borrow().curve_vars[5],
        |v| x87::int_times_to_int(v, scale),
        |v| x87::int_times_to_int(v, damp),
    );
    {
        let mut vars = weather.snow_vars.borrow_mut();
        vars.curve_vars[5] = w.sum as u32;
        vars.curve_vars[0] = w.value as u32;
        vars.curve_vars[4] = w.step as u32;
    }
    let mut curve = w.curve;
    let mut copy = vec![0; CURVE_LEN as usize];
    let weights = light.smooth_weights;
    for _ in 0..core::SMOOTHINGS {
        core::smooth(&mut curve, &mut copy, |a, b, c| {
            x87::weighted3_to_int(a, b, c, weights[0], weights[1], weights[2])
        });
    }
    let mut table = weather.snow_curve.borrow_mut();
    for (cell, v) in table.curve.iter_mut().zip(&curve) {
        *cell = *v as u32;
    }
    for (cell, v) in table.copy.iter_mut().zip(&copy) {
        *cell = *v as u32;
    }
    let mut copy: Vec<i32> = table.copy.iter().map(|&c| c as i32).collect();
    let (last, first) = (copy[0xFFF], copy[0]);
    x87::ramp_down(&mut copy[..0xFFE], last, first, light.ramp_by);
    for (cell, &v) in table.copy.iter_mut().zip(&copy) {
        *cell = v as u32;
    }
    drop(table);
    let (spread_scale, spread_base): (f32, f32) = (light.spread_scale, light.spread_base);
    let t = core::random_tables(w.seed, divisors);
    {
        let mut tables = weather.snow_tables.borrow_mut();
        for (cells, table) in tables.randoms.iter_mut().zip(&t.randoms) {
            cells.copy_from_slice(table);
        }
        let spread = t
            .threes
            .map(|three| x87::int_times_plus(three, spread_scale, spread_base));
        tables.spreads.copy_from_slice(&spread);
        for (cells, table) in tables.remainders.iter_mut().zip(&t.remainders) {
            cells.copy_from_slice(table);
        }
    }
    effects.setup.work.borrow_mut().seed = t.seed;
    weather.snow.borrow_mut().flag = 0;
    let values = light.record_values.map(f32::to_bits);
    let layout = core::record_values(values);
    for record in weather
        .snow_vars
        .borrow_mut()
        .records
        .iter_mut()
        .take(CURVE_RECORDS as usize)
    {
        for &(o, v) in &layout {
            record[o as usize / 4] = v;
        }
    }
    let first = x87::loaded(light.quad_first);
    weather.snow.borrow_mut().quad_a = [first; 4];
    // The view's centre (`SceneDraw.view_centre[0]`, `SceneDraw.view_centre[1]`).
    let centre = &scene.scratch.front_end_polygon().window.view_centre;
    {
        let mut snow = weather.snow.borrow_mut();
        snow.quad_b = [centre[0].get(); 4];
        snow.quad_c = [centre[1].get(); 4];
    }
    Ok(())
}

/// The marks' settings the race handed `effects::init_marks` (`race::effects_settings_store`,
/// the one record the settings flag stands for); refused while it is not set.
pub fn effects_settings(
    effects: &crate::effects::owner::Effects,
) -> Result<&'static crate::race::effects_settings_store::EffectsSettings, BadInput> {
    effects.settings()
}
