//! `marks::init_marks`' (0x4D12F7) first leaves (IDA, 2026-10-16): the noise tables, the
//! wave tables (made once), the slots cleared, and the marks' clock (also run each frame by
//! `marks::run_frame`, 0x4D1549).

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::math::x87;

pub const SEED_RVA: usize = rva(0x71C8CC); // g_marksSeed
pub const NOISE_RVAS: [usize; 3] = [rva(0x8BC511), rva(0x8BC911), rva(0x8BCD11)]; // g_marksNoiseA ..
pub const NOISE_COUNT_RVA: usize = rva(0x8BC405); // dword_8BC405
pub const WAVE_DONE_RVA: usize = rva(0x9206C2); // g_marksWaveDone: 0, or where to go instead
const WAVE_DONE_TARGET_RVA: usize = rva(0x4C4EA0); // nullsub_23
pub const WAVE_LEVELS_RVA: usize = rva(0x9205F6); // g_marksWaveLevels: whole numbers, then floats
pub const WAVE_LEVELS: u32 = 0x11;
const WAVE_SCALE_RVA: usize = rva(0x9206C6); // flt_9206C6, 256.0
const WAVE_LESS_RVA: usize = rva(0x9206CA); // flt_9206CA, 1.0
const SINE_STEP_RVA: usize = rva(0x9206CE); // flt_9206CE, pi / 128
pub const SINES_RVA: usize = rva(0x92001E); // g_marksSines
pub const SINES: u32 = 0x140;
pub const SLOTS_RVA: usize = rva(0x934F7C); // g_markSlots: 0x20 of 0x14 bytes
pub const SLOTS: u32 = 0x20;
pub const SLOT_LEN: u32 = 0x14;
pub const CLOCK_RVA: usize = rva(0x613830); // g_sceneClock
pub const SETTINGS_RVA: usize = rva(0x767A28); // g_marksSettings
/// `g_marksClockStarted` .. `flt_764FE4`, `g_marksClockNow`, `g_marksClockBefore`.
pub const CLOCK_VARS_RVA: usize = rva(0x764FC8);
const STARTED: usize = 0;
const LAST: usize = 4;
const TICKS: usize = 8;
const TICKS_SCALED: usize = 0xC; // flt_764FD4
const SIXTEENTH: usize = 0x10; // dword_764FD8
const QUARTER: usize = 0x14; // dword_764FDC
const RECIPROCAL: usize = 0x18; // flt_764FE0
const RECIPROCAL_SCALED: usize = 0x1C; // flt_764FE4
const NOW: usize = 0x20;
const BEFORE: usize = 0x24;
const FIRST_RECIPROCAL_RVA: usize = rva(0x93CC10); // flt_93CC10
const RECIPROCAL_SCALE_RVA: usize = rva(0x7651BC); // flt_7651BC
const TICKS_SCALE_RVA: usize = rva(0x93CC14); // flt_93CC14

/// `marks::seed_noise` (0x4C1697): 0x100 steps of `g_marksSeed` (times 0x9D plus 0x47),
/// each giving entry 0xFF .. 0 of the three noise tables its low 18 bits, its low 18 bits
/// turned right 10 and turned right 20; `dword_8BC405` 0.
pub unsafe fn seed_noise() {
    // Converted: `rc2k_core::marks::noise::seed_noise`.
    unsafe {
        let (tables, seed) = rc2k_core::marks::noise::seed_noise(game_read(SEED_RVA));
        for (table, values) in NOISE_RVAS.into_iter().zip(tables) {
            game_write(table, values);
        }
        game_write(SEED_RVA, seed);
        game_write(NOISE_COUNT_RVA, 0u32);
    }
}

/// `marks::make_wave_tables` (0x4C4E2F): once (`g_marksWaveDone` 0): the 0x11 levels,
/// whole numbers, become floats (over `flt_9206C6`, less `flt_9206CA`), the 0x140 sines of
/// 0, step, 2 step .. (`math::x87::sine_table`); then `g_marksWaveDone` = `nullsub_23`.
/// Done before: nothing. Refuses a mark that is neither 0 nor `nullsub_23` (the original
/// jumps to it).
pub unsafe fn make_wave_tables() -> Result<(), Refused> {
    unsafe {
        let done: u32 = game_read(WAVE_DONE_RVA);
        let target = game_addr(WAVE_DONE_TARGET_RVA) as u32;
        if done == target {
            return Ok(());
        }
        if done != 0 {
            return Err(BadInput("the wave tables' mark is neither 0 nor nullsub_23").into());
        }
        let (scale, less): (u32, u32) = (game_read(WAVE_SCALE_RVA), game_read(WAVE_LESS_RVA));
        for n in (0..WAVE_LEVELS as usize).rev() {
            let level: i32 = game_read(WAVE_LEVELS_RVA + 4 * n);
            game_write(
                WAVE_LEVELS_RVA + 4 * n,
                x87::int_div_less(level, scale, less),
            );
        }
        let sines = game_addr(SINES_RVA) as u32;
        x87::sine_table(game_read(SINE_STEP_RVA), sines as *mut u32, SINES);
        game_write(WAVE_DONE_RVA, target);
    }
    Ok(())
}

/// `marks::clear_slots` (0x4C967B): the first dword of the 0x20 slots 0.
pub unsafe fn clear_slots() {
    for n in 0..SLOTS as usize {
        unsafe { game_write(SLOTS_RVA + SLOT_LEN as usize * n, 0u32) };
    }
}

unsafe fn clock_var(o: usize) -> u32 {
    unsafe { game_read(CLOCK_VARS_RVA + o) }
}

unsafe fn set_clock_var(o: usize, v: u32) {
    unsafe { game_write(CLOCK_VARS_RVA + o, v) }
}

/// `marks::tick_clock` (0x4D2BDC). The first time (`g_marksClockStarted` 0): started, the
/// last, now and before `g_sceneClock`, the ticks 0, the reciprocal `flt_93CC10`. Then:
/// before the last, now and the last the clock, the ticks the difference (0 unless
/// `g_marksSettings` +0x10 is 1), a quarter and a sixteenth of them (shifted), the
/// reciprocal one over them (`fidiv`) and that times `flt_7651BC`, and the ticks' bits
/// taken as a float (section 16) times `flt_93CC14`.
pub unsafe fn tick_clock() {
    unsafe {
        let clock: u32 = game_read(CLOCK_RVA);
        if clock_var(STARTED) == 0 {
            set_clock_var(STARTED, 1);
            for o in [LAST, NOW, BEFORE] {
                set_clock_var(o, clock);
            }
            set_clock_var(TICKS, 0);
            set_clock_var(RECIPROCAL, x87::loaded(game_read(FIRST_RECIPROCAL_RVA)));
            return;
        }
        set_clock_var(NOW, clock);
        let last = clock_var(LAST);
        set_clock_var(BEFORE, last);
        set_clock_var(LAST, clock);
        let settings: u32 = game_read(SETTINGS_RVA);
        let ticks = if ((settings + 0x10) as *const u32).read_unaligned() == 1 {
            clock.wrapping_sub(last)
        } else {
            0
        };
        set_clock_var(TICKS, ticks);
        set_clock_var(QUARTER, ticks >> 2);
        set_clock_var(SIXTEENTH, ticks >> 4);
        let (reciprocal, scaled) =
            x87::reciprocal_int_and_times(ticks as i32, game_read(RECIPROCAL_SCALE_RVA));
        set_clock_var(RECIPROCAL, reciprocal);
        set_clock_var(RECIPROCAL_SCALED, scaled);
        set_clock_var(TICKS_SCALED, x87::mul(ticks, game_read(TICKS_SCALE_RVA)));
    }
}

/// `marks::make_noise_curve`'s variables (0x8DA410 ..): the value, its least and most
/// step, the step's change, the step, and a running sum of the seeds.
pub const CURVE_VARS_RVA: usize = rva(0x8DA410);
pub const CURVE_RVA: usize = rva(0x8D2310); // g_marksCurve: 0x1000 whole numbers
pub const CURVE_COPY_RVA: usize = rva(0x8D6310); // g_marksCurveCopy
pub const CURVE_LEN: u32 = 0x1000;
const CURVE_SCALE_RVA: usize = rva(0x8DB4CC); // flt_8DB4CC, 8.5
const CURVE_DAMP_RVA: usize = rva(0x8DB4D0); // flt_8DB4D0, 0.9
const SMOOTH_RVAS: [usize; 3] = [rva(0x8DB4D4), rva(0x8DB4D8), rva(0x8DB4DC)]; // 0.3, 0.4, 0.3
const RAMP_BY_RVA: usize = rva(0x8DB4E0); // dword_8DB4E0, 0x1000
/// Three tables of 0x800 20-bit randoms, two of remainders, one of floats.
pub const RANDOMS_RVA: usize = rva(0x8C6000);
const SPREAD_SCALE_RVA: usize = rva(0x8DB4E4); // flt_8DB4E4, 0.0625
const SPREAD_BASE_RVA: usize = rva(0x8DB4E8); // flt_8DB4E8
const DIVISOR_RVAS: [usize; 2] = [rva(0x8DB4EC), rva(0x8DB4F0)]; // 0xC00 each
pub const CURVE_FLAG_RVA: usize = rva(0x8D215C); // dword_8D215C
pub const CURVE_RECORDS_RVA: usize = rva(0x8DA430); // 0x20 records of 0x80
pub const CURVE_RECORDS: u32 = 0x20;
const RECORD_VALUE_RVAS: [usize; 5] = [
    rva(0x8DB4F4),
    rva(0x8DB4F8),
    rva(0x8DB4FC),
    rva(0x8DB500),
    rva(0x8DB504),
];
pub const QUADS_RVA: usize = rva(0x8D2100); // xmmword_8D2100, then 0x8D2120, 0x8D2130
const QUAD_VALUE_RVAS: [usize; 3] = [rva(0x8DB508), rva(0x62409C), rva(0x6240A0)];

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
        let divisors = DIVISOR_RVAS.map(|a| game_read::<i32>(a));
        if !divisors.into_iter().all(core::divisor_fits) {
            return Err(BadInput("a remainder's divisor is 0, 1 or -1: idiv traps").into());
        }
        game_write(
            CURVE_VARS_RVA,
            [
                0u32,
                core::LEAST_STEP as u32,
                core::MOST_STEP as u32,
                core::CHANGE as u32,
                0,
            ],
        );
        // The roundings follow the x87's control word: the x87's.
        let (scale, damp): (u32, u32) = (game_read(CURVE_SCALE_RVA), game_read(CURVE_DAMP_RVA));
        let sum_at = CURVE_VARS_RVA + 0x14;
        let w = core::walk(
            game_read(sum_at),
            |v| x87::int_times_to_int(v, scale),
            |v| x87::int_times_to_int(v, damp),
        );
        game_write(sum_at, w.sum);
        game_write(CURVE_VARS_RVA, w.value);
        game_write(CURVE_VARS_RVA + 0x10, w.step);
        let mut curve = w.curve;
        let mut copy = vec![0; CURVE_LEN as usize];
        let weights = SMOOTH_RVAS.map(|a| game_read::<u32>(a));
        for _ in 0..core::SMOOTHINGS {
            core::smooth(&mut curve, &mut copy, |a, b, c| {
                x87::weighted3_to_int(a, b, c, weights[0], weights[1], weights[2])
            });
        }
        let to = |r: usize, v: &[i32]| {
            std::ptr::copy_nonoverlapping(v.as_ptr(), game_addr(r) as *mut i32, v.len())
        };
        to(CURVE_RVA, &curve);
        to(CURVE_COPY_RVA, &copy);
        let copy = game_addr(CURVE_COPY_RVA) as *mut i32;
        x87::ramp_down(
            copy,
            0xFFE,
            copy.add(0xFFF).read_unaligned(),
            copy.read_unaligned(),
            game_read(RAMP_BY_RVA),
        );
        let (spread_scale, spread_base): (u32, u32) =
            (game_read(SPREAD_SCALE_RVA), game_read(SPREAD_BASE_RVA));
        let t = core::random_tables(w.seed, divisors);
        for (k, table) in t.randoms.iter().enumerate() {
            game_write(RANDOMS_RVA + 0x2000 * k, *table);
        }
        let spread = t
            .threes
            .map(|three| x87::int_times_plus(three, spread_scale, spread_base));
        game_write(RANDOMS_RVA + 0xA000, spread);
        for (k, table) in t.remainders.iter().enumerate() {
            game_write(RANDOMS_RVA + 0x6000 + 0x2000 * k, *table);
        }
        game_write(SEED_RVA, t.seed);
        game_write(CURVE_FLAG_RVA, 0u32);
        let values = RECORD_VALUE_RVAS.map(|r| game_read::<u32>(r));
        let layout = core::record_values(values);
        for n in 0..CURVE_RECORDS as usize {
            for &(o, v) in &layout {
                game_write(CURVE_RECORDS_RVA + 0x80 * n + o as usize, v);
            }
        }
        let first = x87::loaded(game_read(QUAD_VALUE_RVAS[0]));
        game_write(QUADS_RVA, [first; 4]);
        game_write(QUADS_RVA + 0x20, [game_read::<u32>(QUAD_VALUE_RVAS[1]); 4]);
        game_write(QUADS_RVA + 0x30, [game_read::<u32>(QUAD_VALUE_RVAS[2]); 4]);
    }
    Ok(())
}
