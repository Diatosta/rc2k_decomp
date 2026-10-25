//! `physics::rate_damage` (0x4AF1EC): the current car's damage figures (the work copy's
//! 0x71A278 .. 0x71A328) made into its working grades and failures (IDA, 2026-10-17;
//! what each figure is, is a reading at best). In the original's order:
//! - `dword_71A3CC` = 1 - `dword_71A290` k k694 - `flt_71A294` k k698 (k = `flt_75B388`;
//!   80 bits), times `flt_75B5A0` when `flt_75B3E4`'s bits are not 0 and `flt_71A298` is
//!   not below `flt_75B34C`; held to `flt_75B350` .. `flt_75B348`;
//! - `flt_71A3D0`, `flt_71A3D4`: 1 less four products each (`flt_71A284` / `flt_71A288`,
//!   `flt_71A320` / `flt_71A324`, `flt_71A278`, `dword_75B3F0`), held the same way;
//! - 23 ramps (`math::x87_run::ramp`, tables of (x0, y0, x1, y1) at 0x75B4E8 .. 0x75B568)
//!   into 0x71A3D8 .. 0x71A3E0 and 0x75B404 .. 0x75B49C, two of them times
//!   `flt_75B43C` k k, one squared;
//! - failures by chance: a figure times `flt_75B340` (`fistp` into `flt_75B36C`) against
//!   the random `dword_75B4A0` turned right by 4 .. 0x19, low word (the sign of the
//!   difference): `dword_71A400` (set once), `dword_71A3E4` .. `dword_71A3FC` (set from
//!   0x75B404 .., cleared again from 0x75B420 ..), `dword_71A404`, `dword_71A408`;
//! - `flt_71A2AC` = 0, and `g_carGearBits` = the gears (9 figures from 0x71A2A8) equal to
//!   `flt_75B34C` (C3: also no number);
//! - `dword_71A274` = 0 when five figures are all below `flt_75B34C`, else 1, and then
//!   `dword_71A3CC` = 0 unless `flt_71A2E8` is below it.
//!
//! In:  nothing
//! Out: eax `g_carGearBits`, ecx 9, esi 0x75B568 (the last table); the rest kept; no
//!      flags read (`sub_4AB5D0` returns, `sub_43BA8E` returns, `sub_43EB5D` goes on with
//!      `fild`)
//!
//! Caller: `physics::rate_current_car` (0x4AB5D0), its only one.

use crate::math::x87;
use crate::math::x87_run::{compare_full, ext_times, mul3, one_less_products, ramp, square};
use crate::physics::mirror::Mirror;
use crate::physics::store::DAMAGE;
use crate::scene::matrix3::rounded;
use crate::session::Float;

/// The ramps in order: (destination, figure, table).
const RAMPS_A: [(u32, u32, &[Float; 4]); 2] = [
    (0x71A3D8, 0x71A27C, &DAMAGE.lerp_tables[4]),
    (0x71A3DC, 0x71A280, &DAMAGE.lerp_tables[4]),
];
/// More ramps: (destination in the model, the car's figure, table).
const RAMPS_C: [(&Float, u32, &[Float; 4]); 18] = [
    (&DAMAGE.flt_75b490, 0x71A2A0, &DAMAGE.lerp_tables[7]),
    (&DAMAGE.flt_75b494, 0x71A2CC, &DAMAGE.lerp_tables[8]),
    (&DAMAGE.flt_75b498, 0x71A2CC, &DAMAGE.lerp_tables[9]),
    (&DAMAGE.flt_75b49c, 0x71A328, &DAMAGE.lerp_tables[10]),
    (&DAMAGE.flt_75b404, 0x71A2E8, &DAMAGE.lerp_tables[11]),
    (&DAMAGE.flt_75b408, 0x71A2EC, &DAMAGE.lerp_tables[11]),
    (&DAMAGE.flt_75b40c, 0x71A2F0, &DAMAGE.lerp_tables[11]),
    (&DAMAGE.flt_75b410, 0x71A2F4, &DAMAGE.lerp_tables[11]),
    (&DAMAGE.flt_75b414, 0x71A2F8, &DAMAGE.lerp_tables[11]),
    (&DAMAGE.flt_75b418, 0x71A2FC, &DAMAGE.lerp_tables[11]),
    (&DAMAGE.flt_75b41c, 0x71A300, &DAMAGE.lerp_tables[11]),
    (&DAMAGE.flt_75b420, 0x71A2E8, &DAMAGE.lerp_tables[11]),
    (&DAMAGE.flt_75b424, 0x71A2EC, &DAMAGE.lerp_tables[12]),
    (&DAMAGE.flt_75b428, 0x71A2F0, &DAMAGE.lerp_tables[12]),
    (&DAMAGE.flt_75b42c, 0x71A2F4, &DAMAGE.lerp_tables[12]),
    (&DAMAGE.flt_75b430, 0x71A2F8, &DAMAGE.lerp_tables[12]),
    (&DAMAGE.flt_75b434, 0x71A2FC, &DAMAGE.lerp_tables[12]),
    (&DAMAGE.flt_75b438, 0x71A300, &DAMAGE.lerp_tables[12]),
];
/// The failures that come and go: (flag, figure while set, figure while clear, turn).
const FAILURES: [(u32, &Float, &Float, u32); 7] = [
    (0x71A3E4, &DAMAGE.flt_75b420, &DAMAGE.flt_75b404, 0x4),
    (0x71A3E8, &DAMAGE.flt_75b424, &DAMAGE.flt_75b408, 0x5),
    (0x71A3EC, &DAMAGE.flt_75b428, &DAMAGE.flt_75b40c, 0x6),
    (0x71A3F0, &DAMAGE.flt_75b42c, &DAMAGE.flt_75b410, 0x7),
    (0x71A3F4, &DAMAGE.flt_75b430, &DAMAGE.flt_75b414, 0x8),
    (0x71A3F8, &DAMAGE.flt_75b434, &DAMAGE.flt_75b418, 0x9),
    (0x71A3FC, &DAMAGE.flt_75b438, &DAMAGE.flt_75b41c, 0xA),
];

/// The Rust core on the variables `m` reaches (the
/// exact one, for `physics::grade_or_reset` and `physics::rate_current_car`).
pub unsafe fn rate_damage_at(m: &Mirror) {
    unsafe {
        let d = &DAMAGE;
        let g = |v: u32| m.get(v);
        let less = |a: u32, b: u32| compare_full(a, b) & 0x100 != 0;
        let hold = |v: u32| {
            if less(g(v), d.gauge_low.bits()) {
                m.put(v, x87::loaded(d.gauge_low.bits()));
            }
            if !less(g(v), d.grade_high.bits()) {
                m.put(v, x87::loaded(d.grade_high.bits()));
            }
        };
        let k = d.grade_k.bits();
        let mut e = one_less_products(&[
            [g(0x71A290), k, d.figures[7].bits()],
            [g(0x71A294), k, d.figures[8].bits()],
        ]);
        if d.flt_75b3e4.bits() != 0 && !less(g(0x71A298), d.gauge_top.bits()) {
            e = ext_times(e, d.wear_rates[10].bits());
        }
        m.put(0x71A3CC, rounded(e));
        hold(0x71A3CC);
        for (out, a, b) in [
            (0x71A3D0, 0x71A284, 0x71A320),
            (0x71A3D4, 0x71A288, 0x71A324),
        ] {
            let e = one_less_products(&[
                [g(a), k, d.figures[10].bits()],
                [g(b), k, d.figures[9].bits()],
                [g(0x71A278), k, d.figures[11].bits()],
                [d.dword_75b3f0.bits(), k, d.figures[12].bits()],
            ]);
            m.put(out, rounded(e));
            hold(out);
        }
        let table_of = |t: &[Float; 4]| [t[0].bits(), t[1].bits(), t[2].bits(), t[3].bits()];
        for (out, x, t) in RAMPS_A {
            m.put(out, ramp(g(x), table_of(t), d.gauge_low.bits()));
        }
        for (out, x) in [(0x71A3D8, &d.flt_75b440), (0x71A3DC, &d.flt_75b444)] {
            d.flt_75b43c.set_bits(ramp(
                x.bits(),
                table_of(&d.lerp_tables[5]),
                d.gauge_low.bits(),
            ));
            m.put(out, mul3(d.flt_75b43c.bits(), k, k, g(out)));
        }
        m.put(
            0x71A3E0,
            ramp(g(0x71A308), table_of(&d.lerp_tables[6]), d.gauge_low.bits()),
        );
        m.put(0x71A3E0, square(g(0x71A3E0)));
        for (out, x, t) in RAMPS_C {
            out.set_bits(ramp(g(x), table_of(t), d.gauge_low.bits()));
        }
        // Chance: the figure times `flt_75B340` against the random's low word turned.
        let chance = |figure: &Float, turn: u32| {
            let t = x87::mul_to_int(figure.bits(), d.chance_scale.bits()) as u32;
            d.scratch.set_bits(t);
            let r = d.clock.get().rotate_right(turn) & 0xFFFF;
            (r.wrapping_sub(t) as i32) < 0
        };
        if g(0x71A400) == 0 && chance(&d.flt_75b49c, 0xD) {
            m.put(0x71A400, 1);
        }
        for (flag, set, clear, turn) in FAILURES {
            if g(flag) != 0 {
                if chance(set, turn) {
                    m.put(flag, 0);
                }
            } else if chance(clear, turn) {
                m.put(flag, 1);
            }
        }
        m.put(0x71A404, 0);
        if chance(&d.flt_75b490, 0xB) {
            m.put(0x71A404, 1);
        }
        if chance(&d.flt_75b494, 0x19) {
            m.put(0x71A404, 1);
        }
        m.put(0x71A408, 0);
        if chance(&d.flt_75b498, 0x12) {
            m.put(0x71A408, 1);
        }
        m.put(0x71A2AC, 0);
        let mut bits = 0u32;
        for n in 0..9 {
            if compare_full(g(0x71A2A8 + 4 * n), d.gauge_top.bits()) & 0x4000 != 0 {
                bits |= 1 << n;
            }
        }
        m.put(0x71A40C, bits);
        let all_below = [0x71A290, 0x71A2A0, 0x71A2A4, 0x71A2CC, 0x71A2E8]
            .iter()
            .all(|&v| less(g(v), d.gauge_top.bits()));
        if all_below {
            m.put(0x71A274, 0);
        } else {
            m.put(0x71A274, 1);
            if !less(g(0x71A2E8), d.gauge_top.bits()) {
                m.put(0x71A3CC, 0);
            }
        }
    }
}
