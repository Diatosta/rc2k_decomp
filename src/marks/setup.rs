//! `marks::init_marks`' (0x4D12F7) first leaves (IDA, 2026-10-16): the noise tables, the
//! wave tables (made once), the slots cleared, and the marks' clock (also run each frame by
//! `marks::run_frame`, 0x4D1549).

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::snow::{SNOW, SNOW_CURVE, SNOW_LIGHT, SNOW_TABLES, SNOW_VARS};
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87;
use crate::session::{Bytes, Dword, Float};

/// The marks' clock (`marks::tick_clock`) (IDA 0x764FC8 .. 0x764FFC, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct MarksClock {
    /// 0x764FC8: `g_marksClockStarted`: 0 until `marks::tick_clock` has run once (`init_marks` clears it).
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
    /// 0x764FF0: `dword_764FF0`: a copy of the settings' +0x18 (`marks::frame_marks`), 1 from `init_marks`; `dword_764FF4`: 1 from `init_marks` (nothing ported reads it).
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
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        &CLOCK as *const MarksClock as usize
    }
}

pub const SEED_RVA: usize = rva(0x71C8CC); // g_marksSeed
const WAVE_DONE_TARGET_RVA: usize = rva(0x4C4EA0); // nullsub_23
pub const WAVE_LEVELS: u32 = 0x11;
pub const SINES: u32 = 0x140;
pub const CLOCK_RVA: usize = rva(0x613830); // g_sceneClock
const FIRST_RECIPROCAL_RVA: Slot = Slot::at(0x93CC10); // flt_93CC10
const RECIPROCAL_SCALE_RVA: Slot = Slot::at(0x7651BC); // flt_7651BC
const TICKS_SCALE_RVA: Slot = Slot::at(0x93CC14); // flt_93CC14

/// A test buffer standing for the marks' block at `mk` (IDA 0x764F40): the clock in it.
#[cfg(test)]
pub fn clock_at(mk: usize) -> &'static MarksClock {
    unsafe {
        &*(mk
            .wrapping_add(MarksClock::START)
            .wrapping_sub(crate::marks::store::START) as *const MarksClock)
    }
}

/// `marks::seed_noise` (0x4C1697): 0x100 steps of `g_marksSeed` (times 0x9D plus 0x47),
/// each giving entry 0xFF .. 0 of the three noise tables its low 18 bits, its low 18 bits
/// turned right 10 and turned right 20; `dword_8BC405` 0.
pub unsafe fn seed_noise() {
    // Converted: `rc2k_core::marks::noise::seed_noise`.
    unsafe {
        let (tables, seed) = rc2k_core::marks::noise::seed_noise(game_read(SEED_RVA));
        for (table, values) in crate::marks::rain::RAIN.noise.iter().zip(tables) {
            for (cell, v) in table.iter().zip(values) {
                cell.set(v);
            }
        }
        game_write(SEED_RVA, seed);
        crate::marks::rain::RAIN.drawn.set(0);
    }
}

/// `marks::make_wave_tables` (0x4C4E2F): once (`g_marksWaveDone` 0): the 0x11 levels,
/// whole numbers, become floats (over `flt_9206C6`, less `flt_9206CA`), the 0x140 sines of
/// 0, step, 2 step .. (`math::x87::sine_table`); then `g_marksWaveDone` = `nullsub_23`.
/// Done before: nothing. Refuses a mark that is neither 0 nor `nullsub_23` (the original
/// jumps to it).
pub unsafe fn make_wave_tables() -> Result<(), Refused> {
    use crate::marks::waves::{SINES as SINE_TABLE, WAVE, WAVE_LEVELS as LEVELS};
    unsafe {
        let done = WAVE.done.get();
        let target = game_addr(WAVE_DONE_TARGET_RVA) as u32;
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
        x87::sine_table(
            WAVE.sine_step.bits(),
            SINE_TABLE.table.as_ptr() as *mut u32,
            SINES,
        );
        WAVE.done.set(target);
    }
    Ok(())
}

/// `marks::clear_slots` (0x4C967B): the first dword of the 0x20 slots 0.
pub fn clear_slots() {
    for slot in &crate::marks::mark_slots::SLOTS.slots {
        slot.life.set(0);
    }
}

/// `marks::tick_clock` (0x4D2BDC). The first time (`g_marksClockStarted` 0): started, the
/// last, now and before `g_sceneClock`, the ticks 0, the reciprocal `flt_93CC10`. Then:
/// before the last, now and the last the clock, the ticks the difference (0 unless
/// `g_marksSettings` +0x10 is 1), a quarter and a sixteenth of them (shifted), the
/// reciprocal one over them (`fidiv`) and that times `flt_7651BC`, and the ticks' bits
/// taken as a float (section 16) times `flt_93CC14`.
pub unsafe fn tick_clock() {
    let c = &CLOCK;
    unsafe {
        let clock: u32 = game_read(CLOCK_RVA);
        if c.started.get() == 0 {
            c.started.set(1);
            for r in [&c.last, &c.now, &c.before] {
                r.set(clock);
            }
            c.frame_ticks.set(0);
            c.reciprocal
                .set_bits(x87::loaded(game_read(FIRST_RECIPROCAL_RVA)));
            return;
        }
        c.now.set(clock);
        let last = c.last.get();
        c.before.set(last);
        c.last.set(clock);
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        let ticks = if ((settings + 0x10) as *const u32).read_unaligned() == 1 {
            clock.wrapping_sub(last)
        } else {
            0
        };
        c.frame_ticks.set(ticks);
        c.quarter.set(ticks >> 2);
        c.sixteenth.set(ticks >> 4);
        let (reciprocal, scaled) =
            x87::reciprocal_int_and_times(ticks as i32, game_read(RECIPROCAL_SCALE_RVA));
        c.reciprocal.set_bits(reciprocal);
        c.reciprocal_scaled.set_bits(scaled);
        c.ticks_scaled
            .set_bits(x87::mul(ticks, game_read(TICKS_SCALE_RVA)));
    }
}

pub const CURVE_LEN: u32 = 0x1000;
pub const CURVE_RECORDS: u32 = 0x20;
const QUAD_VALUE_RVAS: (usize, usize) = (rva(0x62409C), rva(0x6240A0));

/// `marks::make_noise_curve` (0x4C2247), `init_marks`' step 5, with `g_marksSeed` started
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
// Converted: `rc2k_core::marks::noise::{walk, smooth, random_tables, record_values}`
pub unsafe fn make_noise_curve() -> Result<(), Refused> {
    use rc2k_core::marks::noise as core;
    unsafe {
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
        let weights = SNOW_LIGHT.smooth_weights.each_ref().map(|f| f.bits());
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
        let copy = SNOW_CURVE.copy.as_ptr() as *mut i32;
        x87::ramp_down(
            copy,
            0xFFE,
            copy.add(0xFFF).read_unaligned(),
            copy.read_unaligned(),
            SNOW_LIGHT.ramp_by.get() as i32,
        );
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
        game_write(SEED_RVA, t.seed);
        SNOW.flag.set(0);
        let values = SNOW_LIGHT.record_values.each_ref().map(|f| f.bits());
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
    }
    Ok(())
}

/// The marks' settings pointer (IDA 0x767A28, phase 7, typed state): `g_marksSettings`, the
/// address of the settings the game hands `marks::init_marks` (0 before; nothing runs
/// without it).
#[repr(C)]
pub struct MarksSettings {
    /// 0x767A28: `g_marksSettings`.
    pub pointer: Dword,
}

pub static SETTINGS: MarksSettings = MarksSettings {
    pointer: Dword::new(0),
};

impl MarksSettings {
    pub const START: usize = 0x767A28;
    pub const LEN: usize = 4;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        &SETTINGS as *const MarksSettings as usize
    }
}

#[cfg(test)]
mod clock_tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn the_clock_starts_as_the_games() {
        assert_eq!(size_of::<MarksClock>(), MarksClock::LEN);
        let bytes =
            unsafe { std::slice::from_raw_parts(MarksClock::home() as *const u8, MarksClock::LEN) };
        assert!(
            bytes == &crate::moved::image_bytes(MarksClock::START, MarksClock::LEN)[..],
            "the clock starts differently from the image"
        );
        let s = MarksClock::START;
        assert_eq!(offset_of!(MarksClock, frame_ticks), 0x764FD0 - s);
        assert_eq!(offset_of!(MarksClock, reciprocal), 0x764FE0 - s);
        assert_eq!(offset_of!(MarksClock, now), 0x764FE8 - s);
        assert_eq!(offset_of!(MarksClock, settings_copies), 0x764FF0 - s);
        assert_eq!(size_of::<MarksSettings>(), MarksSettings::LEN);
        assert_eq!(
            crate::moved::place(MarksSettings::START),
            SETTINGS.pointer.addr()
        );
    }
}
