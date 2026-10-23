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
//! Self-test: a copy of the original on a `physics::Mirror` of the work copy and
//! 0x75B340 .. 0x75B6AC (filled from the game's, then the figures, switches and random by
//! lot), against the converted version: exact at double precision, the grades within
//! `FLOAT` at extended.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, ESI, Regs};
use crate::math::x87;
use crate::math::x87_run::{compare_full, ext_times, mul3, one_less_products, ramp, square};
use crate::physics::mirror::{Mirror, Spec};
use crate::scene::matrix3::rounded;
use crate::screens::harness::Left;
use crate::service::floats::{FLOAT, Tolerance, floats_entry};
use crate::service::harness::Region;
use rc2k_core::physics::handling::{At, Figures};

/// The mirrored ranges of the tests: the work copy and the grades' block.
pub const RANGES: [(u32, u32); 2] = [(0x719FD8, 0x71AFB8), (0x75B340, 0x75B6AC)];
pub const SPEC: Spec = (
    0x4AF1EC,
    0x1055,
    &[0xD9, 0xE8, 0xD9, 0x05, 0x90, 0xA2, 0x71, 0x00],
    &[],
    349,
);

const K: u32 = 0x75B388;
const LOW: u32 = 0x75B350;
const HIGH: u32 = 0x75B348;
const ZERO: u32 = 0x75B34C;
const CHANCE: u32 = 0x75B340;
const RANDOM: u32 = 0x75B4A0;
const SCRATCH: u32 = 0x75B36C;

/// The ramps in order: (destination, figure, table).
const RAMPS_A: [(u32, u32, u32); 2] = [
    (0x71A3D8, 0x71A27C, 0x75B4E8),
    (0x71A3DC, 0x71A280, 0x75B4E8),
];
const RAMPS_C: [(u32, u32, u32); 18] = [
    (0x75B490, 0x71A2A0, 0x75B518),
    (0x75B494, 0x71A2CC, 0x75B528),
    (0x75B498, 0x71A2CC, 0x75B538),
    (0x75B49C, 0x71A328, 0x75B548),
    (0x75B404, 0x71A2E8, 0x75B558),
    (0x75B408, 0x71A2EC, 0x75B558),
    (0x75B40C, 0x71A2F0, 0x75B558),
    (0x75B410, 0x71A2F4, 0x75B558),
    (0x75B414, 0x71A2F8, 0x75B558),
    (0x75B418, 0x71A2FC, 0x75B558),
    (0x75B41C, 0x71A300, 0x75B558),
    (0x75B420, 0x71A2E8, 0x75B558),
    (0x75B424, 0x71A2EC, 0x75B568),
    (0x75B428, 0x71A2F0, 0x75B568),
    (0x75B42C, 0x71A2F4, 0x75B568),
    (0x75B430, 0x71A2F8, 0x75B568),
    (0x75B434, 0x71A2FC, 0x75B568),
    (0x75B438, 0x71A300, 0x75B568),
];
/// The failures that come and go: (flag, figure while set, figure while clear, turn).
const FAILURES: [(u32, u32, u32, u32); 7] = [
    (0x71A3E4, 0x75B420, 0x75B404, 4),
    (0x71A3E8, 0x75B424, 0x75B408, 5),
    (0x71A3EC, 0x75B428, 0x75B40C, 6),
    (0x71A3F0, 0x75B42C, 0x75B410, 7),
    (0x71A3F4, 0x75B430, 0x75B414, 8),
    (0x71A3F8, 0x75B434, 0x75B418, 9),
    (0x71A3FC, 0x75B438, 0x75B41C, 0xA),
];

/// What it writes.
pub fn regions(m: &Mirror) -> Vec<Region> {
    let mut r: Vec<Region> = [
        0x71A274, 0x71A2AC, 0x71A3CC, 0x71A3D0, 0x71A3D4, 0x71A3D8, 0x71A3DC, 0x71A3E0, 0x71A400,
        0x71A404, 0x71A408, 0x71A40C, SCRATCH, 0x75B43C,
    ]
    .iter()
    .map(|&v| (m.a(v), 4))
    .collect();
    r.push((m.a(0x71A3E4), 0x1C));
    r.push((m.a(0x75B404), 0x38));
    r.push((m.a(0x75B490), 0x10));
    r
}

/// The variables of `rc2k_core::physics::handling` where `m` has them.
pub struct OnMirror<'a>(pub &'a Mirror);

/// The IDA address of a variable.
pub fn ida(at: At) -> u32 {
    match at {
        At::Car(o) => 0x719FD8 + o,
        At::Grades(o) => 0x75B340 + o,
        At::Handling(o) => 0x613E00 + o,
        At::Scale => 0x5C5E2C,
    }
}

impl Figures for OnMirror<'_> {
    fn get(&self, at: At) -> u32 {
        unsafe { self.0.get(ida(at)) }
    }
    fn put(&mut self, at: At, v: u32) {
        unsafe { self.0.put(ida(at), v) }
    }
}

/// The float places it writes, held to `FLOAT` at extended precision.
pub fn floats(m: &Mirror) -> Vec<(Region, Tolerance)> {
    [
        (0x71A3CC, 0x18),
        (0x75B43C, 4),
        (0x75B404, 0x38),
        (0x75B490, 0x10),
    ]
    .iter()
    .map(|&(v, len)| ((m.a(v), len), FLOAT))
    .collect()
}

/// The converted version (`rc2k_core::physics::handling`, plain floats) on the variables
/// `m` reaches: the hook's.
// Converted: `rc2k_core::physics::handling::rate_damage`.
pub fn rate_damage_plain_at(m: &Mirror) -> Left {
    let bits = rc2k_core::physics::handling::rate_damage(&mut OnMirror(m));
    Left::nothing()
        .with(EAX, bits)
        .with(ECX, rc2k_core::physics::handling::GEARS)
        .with(ESI, m.a(ida(rc2k_core::physics::handling::LAST_TABLE)))
}

/// The Rust core on the variables `m` reaches; what it leaves in the registers (the
/// exact one, for `physics::grade_or_reset` and `physics::rate_current_car`).
pub unsafe fn rate_damage_at(m: &Mirror) -> Left {
    unsafe {
        let g = |v: u32| m.get(v);
        let less = |a: u32, b: u32| compare_full(a, b) & 0x100 != 0;
        let hold = |v: u32| {
            if less(g(v), g(LOW)) {
                m.put(v, x87::loaded(g(LOW)));
            }
            if !less(g(v), g(HIGH)) {
                m.put(v, x87::loaded(g(HIGH)));
            }
        };
        let k = g(K);
        let mut e =
            one_less_products(&[[g(0x71A290), k, g(0x75B694)], [g(0x71A294), k, g(0x75B698)]]);
        if g(0x75B3E4) != 0 && !less(g(0x71A298), g(ZERO)) {
            e = ext_times(e, g(0x75B5A0));
        }
        m.put(0x71A3CC, rounded(e));
        hold(0x71A3CC);
        for (out, a, b) in [
            (0x71A3D0, 0x71A284, 0x71A320),
            (0x71A3D4, 0x71A288, 0x71A324),
        ] {
            let e = one_less_products(&[
                [g(a), k, g(0x75B6A0)],
                [g(b), k, g(0x75B69C)],
                [g(0x71A278), k, g(0x75B6A4)],
                [g(0x75B3F0), k, g(0x75B6A8)],
            ]);
            m.put(out, rounded(e));
            hold(out);
        }
        let ramp_into = |out: u32, x: u32, t: u32| {
            let table = [g(t), g(t + 4), g(t + 8), g(t + 12)];
            m.put(out, ramp(g(x), table, g(LOW)));
        };
        for (out, x, t) in RAMPS_A {
            ramp_into(out, x, t);
        }
        for (out, x) in [(0x71A3D8, 0x75B440), (0x71A3DC, 0x75B444)] {
            ramp_into(0x75B43C, x, 0x75B4F8);
            m.put(out, mul3(g(0x75B43C), k, k, g(out)));
        }
        ramp_into(0x71A3E0, 0x71A308, 0x75B508);
        m.put(0x71A3E0, square(g(0x71A3E0)));
        for (out, x, t) in RAMPS_C {
            ramp_into(out, x, t);
        }
        // Chance: the figure times `flt_75B340` against the random's low word turned.
        let chance = |figure: u32, turn: u32| {
            let t = x87::mul_to_int(g(figure), g(CHANCE)) as u32;
            m.put(SCRATCH, t);
            let r = g(RANDOM).rotate_right(turn) & 0xFFFF;
            (r.wrapping_sub(t) as i32) < 0
        };
        if g(0x71A400) == 0 && chance(0x75B49C, 0xD) {
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
        if chance(0x75B490, 0xB) {
            m.put(0x71A404, 1);
        }
        if chance(0x75B494, 0x19) {
            m.put(0x71A404, 1);
        }
        m.put(0x71A408, 0);
        if chance(0x75B498, 0x12) {
            m.put(0x71A408, 1);
        }
        m.put(0x71A2AC, 0);
        let mut bits = 0u32;
        for n in 0..9 {
            if compare_full(g(0x71A2A8 + 4 * n), g(ZERO)) & 0x4000 != 0 {
                bits |= 1 << n;
            }
        }
        m.put(0x71A40C, bits);
        let all_below = [0x71A290, 0x71A2A0, 0x71A2A4, 0x71A2CC, 0x71A2E8]
            .iter()
            .all(|&v| less(g(v), g(ZERO)));
        if all_below {
            m.put(0x71A274, 0);
        } else {
            m.put(0x71A274, 1);
            if !less(g(0x71A2E8), g(ZERO)) {
                m.put(0x71A3CC, 0);
            }
        }
        Left::nothing()
            .with(EAX, bits)
            .with(ECX, 9)
            .with(ESI, m.a(0x75B568))
    }
}

/// The Rust version (the exact one, for `physics::rate_current_car`).
pub unsafe fn rate_damage() -> Left {
    unsafe { rate_damage_at(&Mirror::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(rate_damage_plain_at(&Mirror::game()))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}

/// A case's inputs in the mirror: the figures 0x71A278 .. 0x71A32C (and 0x75B440,
/// 0x75B444, 0x75B3F0) between -0.5 and 2 with some exact grades, the flags, the switch
/// and the random by `seed`; the tables kept (the game's), but for one case with a flat
/// table.
pub fn fill(m: &Mirror, seed: u32) {
    let mut s = seed.wrapping_mul(0x9E37_79B9) | 1;
    let mut next = move || {
        s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
        s >> 8
    };
    unsafe {
        let zero = m.get(ZERO);
        for v in (0x71A278..0x71A32C)
            .step_by(4)
            .chain([0x75B440, 0x75B444, 0x75B3F0])
        {
            let r = next();
            let f = match r % 7 {
                0 => zero,
                1 => 1.0f32.to_bits(),
                _ => (((r >> 3) % 2500) as f32 / 1000.0 - 0.5).to_bits(),
            };
            m.put(v, f);
        }
        for v in (0x71A3E4..0x71A40C).step_by(4) {
            m.put(v, next() % 2);
        }
        m.put(
            0x75B3E4,
            if seed.is_multiple_of(3) {
                0
            } else {
                1.0f32.to_bits()
            },
        );
        m.put(RANDOM, next().wrapping_mul(0x01F3_A9C5));
        if seed % 5 == 4 {
            m.put(0x75B558 + 8, m.get(0x75B558));
        }
    }
}
