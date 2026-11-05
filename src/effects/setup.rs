//! `effects::init_marks`' (0x4D12F7) first leaves (IDA, 2026-10-16): the noise tables, the
//! wave tables (made once), the slots cleared, and the marks' clock (also run each frame by
//! `effects::run_frame`, 0x4D1549).

use crate::effects::snow::{SNOW, SNOW_CURVE, SNOW_LIGHT, SNOW_TABLES, SNOW_VARS};
use crate::effects::store::game_read;
use crate::effects::tuning::MARKS_FLOATS;
use crate::effects::work_store::MARKS_WORK;
use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::math::x87;
use crate::physics::tables_store::PHYSICS;
use crate::session::{Bytes, Dword, Float};

/// The marks' clock (`effects::tick_clock`) (IDA 0x764FC8 .. 0x764FFC, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MarksClock {
    /// 0x764FC8: `g_marksClockStarted`: 0 until `effects::tick_clock` has run once (`init_marks` clears it).
    pub started: Dword,
    /// 0x764FCC: `g_marksLastClock`: the scene clock at the last tick.
    pub last: Dword,
    /// 0x764FD0: `g_marksFrameTicks`: the clock's ticks since the last frame (0 unless the settings' +0x10 is 1).
    pub frame_ticks: Dword,
    /// 0x764FD4: `flt_764FD4`: the ticks' bits taken as a float, times `flt_93CC14` (a game bug, section 16).
    pub ticks_scaled: Float,
    /// 0x764FD8: `dword_764FD8`: the ticks shifted right 4.
    pub sixteenth: Dword,
    /// 0x764FDC: `dword_764FDC`: the ticks shifted right 2.
    pub quarter: Dword,
    /// 0x764FE0: `flt_764FE0`: one over the ticks (`fidiv`); the first tick's `flt_93CC10`.
    pub reciprocal: Float,
    /// 0x764FE4: `flt_764FE4`: `reciprocal` times `flt_7651BC`.
    pub reciprocal_scaled: Float,
    /// 0x764FE8: `g_marksClockNow`: the scene clock now.
    pub now: Dword,
    /// 0x764FEC: `g_marksClockBefore`: the clock at the tick before.
    pub before: Dword,
    /// 0x764FF0: `dword_764FF0`: a copy of the settings' +0x18 (`effects::frame_marks`), 1 from `init_marks`; `dword_764FF4`: 1 from `init_marks` (nothing ported reads it).
    pub settings_copies: [Dword; 2],
    /// 0x764FF8: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_764ff8: Bytes<4>,
}

pub static CLOCK: MarksClock = MarksClock {
    started: Dword::new(0x0),
    last: Dword::new(0x0),
    frame_ticks: Dword::new(0x0),
    ticks_scaled: Float::from_bits(0x0),
    sixteenth: Dword::new(0x0),
    quarter: Dword::new(0x0),
    reciprocal: Float::from_bits(0x0),
    reciprocal_scaled: Float::from_bits(0x0),
    now: Dword::new(0x0),
    before: Dword::new(0x0),
    settings_copies: [Dword::new(0x0), Dword::new(0x0)],
    _gap_764ff8: Bytes::new([0; 4]),
};

impl MarksClock {
    pub const START: usize = 0x764FC8;
    pub const LEN: usize = 0x34;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const CLOCK) as usize
    }
}

const WAVE_DONE_TARGET: u32 = 0x4C4EA0; // nullsub_23
pub const WAVE_LEVELS: u32 = 0x11;
pub const SINES: u32 = 0x140;

/// A test buffer standing for the marks' block at `mk` (IDA 0x764F40): the clock in it.
#[cfg(all(test, target_arch = "x86"))]
pub fn clock_at(mk: usize) -> &'static MarksClock {
    unsafe {
        &*(crate::pointers::host_at(
            mk.wrapping_add(MarksClock::START)
                .wrapping_sub(crate::effects::store::START),
        ) as *const MarksClock)
    }
}

/// `effects::seed_noise` (0x4C1697): 0x100 steps of `g_marksSeed` (times 0x9D plus 0x47),
/// each giving entry 0xFF .. 0 of the three noise tables its low 18 bits, its low 18 bits
/// turned right 10 and turned right 20; `dword_8BC405` 0.
pub fn seed_noise() {
    // Converted: `rc2k_core::effects::noise::seed_noise`.
    let (tables, seed) = rc2k_core::effects::noise::seed_noise(MARKS_WORK.seed.get());
    for (table, values) in crate::effects::rain::RAIN.noise.iter().zip(tables) {
        for (cell, v) in table.iter().zip(values) {
            cell.set(v);
        }
    }
    MARKS_WORK.seed.set(seed);
    crate::effects::rain::RAIN.drawn.set(0);
}

/// `effects::make_wave_tables` (0x4C4E2F): once (`g_marksWaveDone` 0): the 0x11 levels,
/// whole numbers, become floats (over `flt_9206C6`, less `flt_9206CA`), the 0x140 sines of
/// 0, step, 2 step .. (`math::x87::sine_table`); then `g_marksWaveDone` = `nullsub_23`.
/// Done before: nothing. Refuses a mark that is neither 0 nor `nullsub_23` (the original
/// jumps to it).
pub fn make_wave_tables() -> Result<(), Refused> {
    use crate::effects::waves::{SINES as SINE_TABLE, WAVE, WAVE_LEVELS as LEVELS};
    let done = WAVE.done.get();
    let target = WAVE_DONE_TARGET;
    if done == target {
        return Ok(());
    }
    if done != 0 {
        return Err(BadInput("the wave tables' mark is neither 0 nor nullsub_23").into());
    }
    let (scale, less) = (WAVE.scale.bits(), WAVE.less.bits());
    for level in LEVELS.levels.iter().take(WAVE_LEVELS as usize).rev() {
        level.set(x87::int_div_less(level.get() as i32, scale, less));
    }
    let mut sines = vec![0u32; SINES as usize];
    x87::sine_table(WAVE.sine_step.bits(), &mut sines);
    for (cell, &v) in SINE_TABLE.table.iter().zip(&sines) {
        cell.set_bits(v);
    }
    WAVE.done.set(target);
    Ok(())
}

/// `effects::clear_slots` (0x4C967B): the first dword of the 0x20 slots 0.
pub fn clear_slots() {
    for slot in &crate::effects::mark_slots::SLOTS.slots {
        slot.life.set(0);
    }
}

/// `effects::tick_clock` (0x4D2BDC). The first time (`g_marksClockStarted` 0): started, the
/// last, now and before `g_sceneClock`, the ticks 0, the reciprocal `flt_93CC10`. Then:
/// before the last, now and the last the clock, the ticks the difference (0 unless
/// `g_marksSettings` +0x10 is 1), a quarter and a sixteenth of them (shifted), the
/// reciprocal one over them (`fidiv`) and that times `flt_7651BC`, and the ticks' bits
/// taken as a float (section 16) times `flt_93CC14`.
pub fn tick_clock() -> Result<(), Refused> {
    let c = &CLOCK;
    {
        let clock: u32 = PHYSICS.scene_clock.bits();
        if c.started.get() == 0 {
            c.started.set(1);
            for r in [&c.last, &c.now, &c.before] {
                r.set(clock);
            }
            c.frame_ticks.set(0);
            c.reciprocal
                .set_bits(x87::loaded(CLOCK_K.first_reciprocal.bits()));
            return Ok(());
        }
        c.now.set(clock);
        let last = c.last.get();
        c.before.set(last);
        c.last.set(clock);
        let marks = crate::effects::setup::marks()?;
        let ticks = if marks.other_marks_shown.get() == 1 {
            clock.wrapping_sub(last)
        } else {
            0
        };
        c.frame_ticks.set(ticks);
        c.quarter.set(ticks >> 2);
        c.sixteenth.set(ticks >> 4);
        let (reciprocal, scaled) =
            x87::reciprocal_int_and_times(ticks as i32, MARKS_FLOATS.quarter.bits());
        c.reciprocal.set_bits(reciprocal);
        c.reciprocal_scaled.set_bits(scaled);
        c.ticks_scaled
            .set_bits(x87::mul(ticks, CLOCK_K.ticks_scale.bits()));
    }
    Ok(())
}

pub const CURVE_LEN: u32 = 0x1000;
pub const CURVE_RECORDS: u32 = 0x20;
const QUAD_VALUE_RVAS: (usize, usize) = (rva(0x62409C), rva(0x6240A0));

/// `effects::make_noise_curve` (0x4C2247), `init_marks`' step 5, with `g_marksSeed` started
/// at 0x7FFFFFF:
/// - a walk of 0x1000 steps into `g_marksCurve` (each the value << 4 times 8.5; the step
///   moved by 2 up, or down when the running seed sum carries, held to -0x1000 .. 0x1000;
///   the value plus the step times 0.9);
/// - 0xA0 times: the curve copied, and each of 1 .. 0xFFD made 0.3, 0.4, 0.3 of its
///   neighbours and itself (`x87::weighted3_to_int`);
/// - the copy's 0 .. 0xFFD less a ramp from 0 by (copy 0xFFF - copy 0) / 0x1000;
/// - 0x800 times three 20-bit randoms and (the seed >> 30, three bits) times 0.0625 plus
///   `flt_8DB4E8`; then 0x800 times two remainders (`idiv` by 0xC00);
/// - `dword_8D215C` 0, the 0x20 records' four quarters, the three quads.
///
/// Refuses a divisor that is 0 or 1 or -1 (the original's `idiv` traps).
// Converted: `rc2k_core::effects::noise::{walk, smooth, random_tables, record_values}`
pub fn make_noise_curve() -> Result<(), Refused> {
    use rc2k_core::effects::noise as core;
    let divisors = SNOW_LIGHT.divisors.each_ref().map(|d| d.get() as i32);
    if !divisors.into_iter().all(core::divisor_fits) {
        return Err(BadInput("a remainder's divisor is 0, 1 or -1: idiv traps").into());
    }
    for (cell, v) in SNOW_VARS.curve_vars.iter().zip([
        0u32,
        core::LEAST_STEP as u32,
        core::MOST_STEP as u32,
        core::CHANGE as u32,
        0,
    ]) {
        cell.set(v);
    }
    // The roundings follow the x87's control word: the x87's.
    let (scale, damp) = (SNOW_LIGHT.curve_scale.bits(), SNOW_LIGHT.curve_damp.bits());
    let w = core::walk(
        SNOW_VARS.curve_vars[5].get(),
        |v| x87::int_times_to_int(v, scale),
        |v| x87::int_times_to_int(v, damp),
    );
    SNOW_VARS.curve_vars[5].set(w.sum as u32);
    SNOW_VARS.curve_vars[0].set(w.value as u32);
    SNOW_VARS.curve_vars[4].set(w.step as u32);
    let mut curve = w.curve;
    let mut copy = vec![0; CURVE_LEN as usize];
    let weights = SNOW_LIGHT
        .smooth_weights
        .each_ref()
        .map(super::super::session::Float::bits);
    for _ in 0..core::SMOOTHINGS {
        core::smooth(&mut curve, &mut copy, |a, b, c| {
            x87::weighted3_to_int(a, b, c, weights[0], weights[1], weights[2])
        });
    }
    for (cell, v) in SNOW_CURVE.curve.iter().zip(&curve) {
        cell.set(*v as u32);
    }
    for (cell, v) in SNOW_CURVE.copy.iter().zip(&copy) {
        cell.set(*v as u32);
    }
    let mut copy: Vec<i32> = SNOW_CURVE.copy.iter().map(|c| c.get() as i32).collect();
    let (last, first) = (copy[0xFFF], copy[0]);
    x87::ramp_down(
        &mut copy[..0xFFE],
        last,
        first,
        SNOW_LIGHT.ramp_by.get() as i32,
    );
    for (cell, &v) in SNOW_CURVE.copy.iter().zip(&copy) {
        cell.set(v as u32);
    }
    let (spread_scale, spread_base): (u32, u32) = (
        SNOW_LIGHT.spread_scale.bits(),
        SNOW_LIGHT.spread_base.bits(),
    );
    let t = core::random_tables(w.seed, divisors);
    for (cells, table) in SNOW_TABLES.randoms.iter().zip(&t.randoms) {
        for (cell, v) in cells.iter().zip(table) {
            cell.set(*v);
        }
    }
    let spread = t
        .threes
        .map(|three| x87::int_times_plus(three, spread_scale, spread_base));
    for (cell, v) in SNOW_TABLES.spreads.iter().zip(spread) {
        cell.set_bits(v);
    }
    for (cells, table) in SNOW_TABLES.remainders.iter().zip(&t.remainders) {
        for (cell, v) in cells.iter().zip(table) {
            cell.set(*v);
        }
    }
    MARKS_WORK.seed.set(t.seed);
    SNOW.flag.set(0);
    let values = SNOW_LIGHT
        .record_values
        .each_ref()
        .map(super::super::session::Float::bits);
    let layout = core::record_values(values);
    for n in 0..CURVE_RECORDS as usize {
        for &(o, v) in &layout {
            SNOW_VARS.records[n][o as usize / 4].set(v);
        }
    }
    let first = x87::loaded(SNOW_LIGHT.quad_first.bits());
    for cell in &SNOW.quad_a {
        cell.set_bits(first);
    }
    for (quad, rva) in [
        (&SNOW.quad_b, QUAD_VALUE_RVAS.0),
        (&SNOW.quad_c, QUAD_VALUE_RVAS.1),
    ] {
        let v: u32 = game_read(rva);
        for cell in quad {
            cell.set_bits(v);
        }
    }
    Ok(())
}

/// The marks' settings flag (IDA 0x767A28, phase 7, typed state): `g_marksSettings`, which
/// held the address of the settings the game hands `effects::init_marks`; it is 1 once they are
/// handed over and 0 before (nothing runs without them). The settings are the race's one
/// record, `MARKS_SETTINGS` (the original kept its address).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SettingsPointer {
    /// 0x767A28: `g_marksSettings`: 1 when the settings are set, else 0.
    pub is_set: Dword,
}

pub static SETTINGS: SettingsPointer = SettingsPointer {
    is_set: Dword::new(0),
};

/// The marks' settings the race handed `effects::init_marks` (`race::marks_settings_store`,
/// the one record `g_marksSettings` points at); refused while it points at none, where the
/// original's reads through it would fault.
pub fn marks() -> Result<&'static crate::race::marks_settings_store::MarksSettings, BadInput> {
    if SETTINGS.is_set.get() == 0 {
        return Err(BadInput("no marks settings"));
    }
    Ok(&crate::race::marks_settings_store::MARKS_SETTINGS)
}

impl SettingsPointer {
    pub const START: usize = 0x767A28;
    pub const LEN: usize = 4;
}

/// The marks' clock constants (IDA 0x93CC10 .. 0x93CC18, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ClockK {
    /// 0x93CC10: `flt_93CC10`: the reciprocal the first tick starts with
    pub first_reciprocal: Float,
    /// 0x93CC14: `flt_93CC14`: the ticks' scale
    pub ticks_scale: Float,
}

/// ClockK's first values.
pub static CLOCK_K: ClockK = ClockK {
    first_reciprocal: Float::from_bits(0x358637BD),
    ticks_scale: Float::from_bits(0x3DCCCCCD),
};

impl ClockK {
    pub const START: usize = 0x93CC10;
    pub const LEN: usize = 0x8;
}

#[cfg(test)]
mod clock_k_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn clock_k_starts_as_the_games() {
        assert_eq!(size_of::<ClockK>(), ClockK::LEN);
        assert!(
            bytes(&CLOCK_K) == crate::moved::image_bytes(ClockK::START, ClockK::LEN),
            "CLOCK_K starts differently from the image"
        );
        assert_eq!(
            offset_of!(ClockK, first_reciprocal),
            0x93CC10 - ClockK::START
        );
        assert_eq!(offset_of!(ClockK, ticks_scale), 0x93CC14 - ClockK::START);
    }
}

#[cfg(test)]
mod clock_tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn the_clock_starts_as_the_games() {
        assert_eq!(size_of::<MarksClock>(), MarksClock::LEN);
        let bytes = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(MarksClock::home()).cast_const(),
                MarksClock::LEN,
            )
        };
        assert!(
            bytes == &crate::moved::image_bytes(MarksClock::START, MarksClock::LEN)[..],
            "the clock starts differently from the image"
        );
        let s = MarksClock::START;
        assert_eq!(offset_of!(MarksClock, frame_ticks), 0x764FD0 - s);
        assert_eq!(offset_of!(MarksClock, reciprocal), 0x764FE0 - s);
        assert_eq!(offset_of!(MarksClock, now), 0x764FE8 - s);
        assert_eq!(offset_of!(MarksClock, settings_copies), 0x764FF0 - s);
        assert_eq!(size_of::<SettingsPointer>(), SettingsPointer::LEN);
        assert_eq!(
            crate::moved::place(SettingsPointer::START),
            SETTINGS.is_set.addr() as usize
        );
    }
}
