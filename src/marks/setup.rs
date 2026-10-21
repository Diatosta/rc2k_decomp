//! `marks::init_marks`' (0x4D12F7) first leaves (IDA, 2026-10-16): the noise tables, the
//! wave tables (made once), the slots cleared, and the marks' clock (also run each frame by
//! `marks::run_frame`, 0x4D1549).

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;

pub const SEED_RVA: usize = rva(0x71C8CC); // g_marksSeed
pub const NOISE_RVAS: [usize; 3] = [rva(0x8BC511), rva(0x8BC911), rva(0x8BCD11)]; // g_marksNoiseA ..
pub const NOISE_LEN: u32 = 0x100;
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
pub const CLOCK_VARS_LEN: usize = 0x28;
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
/// turned right 10 and turned right 20; `dword_8BC405` 0. Leaves eax, ebx, ecx the last
/// three, esi 0.
pub unsafe fn seed_noise(entry: &Regs) -> Regs {
    // Converted: `rc2k_core::marks::noise::seed_noise`.
    let mut g = *entry;
    unsafe {
        let (tables, seed) = rc2k_core::marks::noise::seed_noise(game_read(SEED_RVA));
        for (table, values) in NOISE_RVAS.into_iter().zip(tables) {
            game_write(table, values);
        }
        game_write(SEED_RVA, seed);
        [g.eax, g.ebx, g.ecx] = [tables[0][0], tables[1][0], tables[2][0]];
        g.esi = 0;
        game_write(NOISE_COUNT_RVA, 0u32);
    }
    g
}

/// `marks::make_wave_tables` (0x4C4E2F): once (`g_marksWaveDone` 0): the 0x11 levels,
/// whole numbers, become floats (over `flt_9206C6`, less `flt_9206CA`), the 0x140 sines of
/// 0, step, 2 step .. (`math::x87::sine_table`); then `g_marksWaveDone` = `nullsub_23`.
/// Leaves eax the mark's address, edi past the sines; done before: nothing. Refuses a mark
/// that is neither 0 nor `nullsub_23` (the original jumps to it).
pub unsafe fn make_wave_tables(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let done: u32 = game_read(WAVE_DONE_RVA);
        let target = game_addr(WAVE_DONE_TARGET_RVA) as u32;
        if done == target {
            return Ok(g);
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
        g.edi = sines + 4 * SINES;
        g.eax = game_addr(WAVE_DONE_RVA) as u32;
        game_write(WAVE_DONE_RVA, target);
    }
    Ok(g)
}

/// `marks::clear_slots` (0x4C967B): the first dword of the 0x20 slots 0. Leaves esi 0.
pub unsafe fn clear_slots(entry: &Regs) -> Regs {
    let mut g = *entry;
    for n in 0..SLOTS as usize {
        unsafe { game_write(SLOTS_RVA + SLOT_LEN as usize * n, 0u32) };
    }
    g.esi = 0;
    g
}

unsafe fn clock_var(o: usize) -> u32 {
    unsafe { game_read(CLOCK_VARS_RVA + o) }
}

unsafe fn set_clock_var(o: usize, v: u32) {
    unsafe { game_write(CLOCK_VARS_RVA + o, v) }
}

/// `marks::tick_clock` (0x4D2BDC). The first time (`g_marksClockStarted` 0): started, the
/// last, now and before `g_sceneClock`, the ticks 0, the reciprocal `flt_93CC10`; the
/// registers kept. Then: before the last, now and the last the clock, the ticks the
/// difference (0 unless `g_marksSettings` +0x10 is 1), a quarter and a sixteenth of them
/// (shifted), the reciprocal one over them (`fidiv`) and that times `flt_7651BC`, and the
/// ticks' bits taken as a float (section 16) times `flt_93CC14`. Leaves eax a sixteenth,
/// ebx the last clock before, esi the settings.
pub unsafe fn tick_clock(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        let clock: u32 = game_read(CLOCK_RVA);
        if clock_var(STARTED) == 0 {
            set_clock_var(STARTED, 1);
            for o in [LAST, NOW, BEFORE] {
                set_clock_var(o, clock);
            }
            set_clock_var(TICKS, 0);
            set_clock_var(RECIPROCAL, x87::loaded(game_read(FIRST_RECIPROCAL_RVA)));
            return g;
        }
        set_clock_var(NOW, clock);
        g.ebx = clock_var(LAST);
        set_clock_var(BEFORE, g.ebx);
        set_clock_var(LAST, clock);
        g.esi = game_read(SETTINGS_RVA);
        let ticks = if ((g.esi + 0x10) as *const u32).read_unaligned() == 1 {
            clock.wrapping_sub(g.ebx)
        } else {
            0
        };
        set_clock_var(TICKS, ticks);
        set_clock_var(QUARTER, ticks >> 2);
        set_clock_var(SIXTEENTH, ticks >> 4);
        g.eax = ticks >> 4;
        let (reciprocal, scaled) =
            x87::reciprocal_int_and_times(ticks as i32, game_read(RECIPROCAL_SCALE_RVA));
        set_clock_var(RECIPROCAL, reciprocal);
        set_clock_var(RECIPROCAL_SCALED, scaled);
        set_clock_var(TICKS_SCALED, x87::mul(ticks, game_read(TICKS_SCALE_RVA)));
    }
    g
}

/// `marks::make_noise_curve`'s variables (0x8DA410 ..): the value, its least and most
/// step, the step's change, the step, and a running sum of the seeds.
pub const CURVE_VARS_RVA: usize = rva(0x8DA410);
pub const CURVE_VARS_LEN: usize = 0x18;
pub const CURVE_RVA: usize = rva(0x8D2310); // g_marksCurve: 0x1000 whole numbers
pub const CURVE_COPY_RVA: usize = rva(0x8D6310); // g_marksCurveCopy
pub const CURVE_LEN: u32 = 0x1000;
const CURVE_SCALE_RVA: usize = rva(0x8DB4CC); // flt_8DB4CC, 8.5
const CURVE_DAMP_RVA: usize = rva(0x8DB4D0); // flt_8DB4D0, 0.9
const SMOOTH_RVAS: [usize; 3] = [rva(0x8DB4D4), rva(0x8DB4D8), rva(0x8DB4DC)]; // 0.3, 0.4, 0.3
const SMOOTHINGS: u32 = 0xA0;
const RAMP_BY_RVA: usize = rva(0x8DB4E0); // dword_8DB4E0, 0x1000
/// Three tables of 0x800 20-bit randoms, two of remainders, one of floats.
pub const RANDOMS_RVA: usize = rva(0x8C6000);
pub const RANDOMS: u32 = 0x800;
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

unsafe fn next_seed() -> u32 {
    unsafe {
        let seed = game_read::<u32>(SEED_RVA)
            .wrapping_mul(0x9D)
            .wrapping_add(0x47);
        game_write(SEED_RVA, seed);
        seed
    }
}

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
/// Leaves eax the last quad's value, ebx, ecx, edx the records' values, esi 0, edi past
/// the records. Refuses a divisor that is 0 or 1 or -1 (the original's `idiv` traps).
pub unsafe fn make_noise_curve(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let divisors = DIVISOR_RVAS.map(|a| game_read::<i32>(a));
        if divisors.iter().any(|d| d.unsigned_abs() < 2) {
            return Err(BadInput("a remainder's divisor is 0, 1 or -1: idiv traps").into());
        }
        let (least, most, change) = (-0x1000i32, 0x1000i32, 2i32);
        game_write(
            CURVE_VARS_RVA,
            [0u32, least as u32, most as u32, change as u32, 0],
        );
        game_write(SEED_RVA, 0x7FF_FFFFu32);
        let (scale, damp): (u32, u32) = (game_read(CURVE_SCALE_RVA), game_read(CURVE_DAMP_RVA));
        let curve = game_addr(CURVE_RVA) as *mut i32;
        let copy = game_addr(CURVE_COPY_RVA) as *mut i32;
        let sum_at = CURVE_VARS_RVA + 0x14;
        let (mut value, mut step) = (0i32, 0i32);
        for n in 0..CURVE_LEN as usize {
            curve
                .add(n)
                .write_unaligned(x87::int_times_to_int(value << 4, scale));
            let mut next = step.wrapping_add(change);
            let seed = next_seed();
            let (sum, carry) = game_read::<u32>(sum_at).overflowing_add(seed);
            game_write(sum_at, sum);
            if carry {
                next = next.wrapping_sub(change).wrapping_sub(change);
            }
            step = next.clamp(least, most);
            value = x87::int_times_to_int(value.wrapping_add(step), damp);
        }
        game_write(CURVE_VARS_RVA, value);
        game_write(CURVE_VARS_RVA + 0x10, step);
        let weights = SMOOTH_RVAS.map(|a| game_read::<u32>(a));
        for _ in 0..SMOOTHINGS {
            std::ptr::copy(curve, copy, CURVE_LEN as usize);
            for n in 1..0xFFE {
                let at = |m: usize| copy.add(m).read_unaligned();
                curve.add(n).write_unaligned(x87::weighted3_to_int(
                    at(n - 1),
                    at(n),
                    at(n + 1),
                    weights[0],
                    weights[1],
                    weights[2],
                ));
            }
        }
        x87::ramp_down(
            copy,
            0xFFE,
            copy.add(0xFFF).read_unaligned(),
            copy.read_unaligned(),
            game_read(RAMP_BY_RVA),
        );
        let (spread_scale, spread_base): (u32, u32) =
            (game_read(SPREAD_SCALE_RVA), game_read(SPREAD_BASE_RVA));
        for n in (0..RANDOMS as usize).rev() {
            let seed = next_seed();
            for (k, turn) in [0u32, 10, 20].into_iter().enumerate() {
                game_write(
                    RANDOMS_RVA + 0x2000 * k + 4 * n,
                    seed.rotate_right(turn) & 0xF_FFFF,
                );
            }
            let three = (seed.rotate_right(30) & 7) as i32;
            game_write(
                RANDOMS_RVA + 0xA000 + 4 * n,
                x87::int_times_plus(three, spread_scale, spread_base),
            );
        }
        for n in (0..RANDOMS as usize).rev() {
            for (k, d) in divisors.into_iter().enumerate() {
                let seed = next_seed();
                game_write(
                    RANDOMS_RVA + 0x6000 + 0x2000 * k + 4 * n,
                    (seed as i64 % d as i64) as u32,
                );
            }
        }
        game_write(CURVE_FLAG_RVA, 0u32);
        let [quarter_v, a, b, c, d] = RECORD_VALUE_RVAS.map(|r| game_read::<u32>(r));
        for n in 0..CURVE_RECORDS as usize {
            let record = CURVE_RECORDS_RVA + 0x80 * n;
            for q in 0..4 {
                game_write(record + 0x20 * q + 0xC, [quarter_v, u32::MAX, 0]);
            }
            for (o, v) in [
                (0x18, a),
                (0x38, c),
                (0x58, c),
                (0x78, a),
                (0x1C, b),
                (0x3C, b),
                (0x5C, d),
                (0x7C, d),
            ] {
                game_write(record + o, v);
            }
        }
        let first = x87::loaded(game_read(QUAD_VALUE_RVAS[0]));
        game_write(QUADS_RVA, [first; 4]);
        game_write(QUADS_RVA + 0x20, [game_read::<u32>(QUAD_VALUE_RVAS[1]); 4]);
        g.eax = game_read(QUAD_VALUE_RVAS[2]);
        game_write(QUADS_RVA + 0x30, [g.eax; 4]);
        (g.ebx, g.ecx, g.edx) = (b, c, d);
        g.esi = 0;
        g.edi = game_addr(CURVE_RECORDS_RVA) as u32 + 0x80 * CURVE_RECORDS;
    }
    Ok(g)
}

/// What `make_noise_curve` writes.
pub fn noise_curve_regions() -> Vec<crate::service::harness::Region> {
    let a = |r: usize| game_addr(r) as u32;
    vec![
        (a(CURVE_VARS_RVA), CURVE_VARS_LEN),
        crate::service::harness::at(SEED_RVA),
        (a(CURVE_RVA), 8 * CURVE_LEN as usize),
        (a(RANDOMS_RVA), 0xC000),
        crate::service::harness::at(CURVE_FLAG_RVA),
        (a(CURVE_RECORDS_RVA), 0x80 * CURVE_RECORDS as usize),
        (a(QUADS_RVA), 0x10),
        (a(QUADS_RVA + 0x20), 0x20),
    ]
}
