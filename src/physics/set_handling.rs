//! `physics::set_handling` (0x43EB5D): `physics::grade_or_reset`, then nine handling
//! figures from the car's setup (whole numbers in the work copy, 0 .. 256 by the look of
//! it): each lo + n / 256 (hi - lo) between two constants of 0x613F1C .. 0x613F88:
//! `flt_614C0C`, `flt_614C08` (by `dword_71A014`), `flt_614BF0` (`dword_719FF8`),
//! `flt_614BF4`, `flt_614BFC` (`dword_71A008`; the second from the pair for
//! `g_carDriveKind`, then times `dword_614C18`), `flt_614BEC`, `flt_614BF8`, `flt_614C00`,
//! `flt_614C04` (`dword_71A00C`). What each is, is a reading.
//!
//! In:  nothing
//! Out: ebx `g_carDriveKind`; the rest as `grade_or_reset` leaves them; no flags read
//!
//! Callers: `sub_43B635` (every physics step) and `race::settle_car`.
//! Refuses (before anything) a drive kind above 1 (the cars' records have 0 and 1; the
//! pair tables hold two).
//! Self-test: a copy of the original (its call to copies of `grade_or_reset` and below)
//! on a `physics::Mirror` of the work copy, 0x75B340 .. and 0x613E00 .. 0x614D00, settings
//! by lot, both drive kinds and both branches, against the converted version: exact at
//! double precision, the floats within `FLOAT` at extended; the refusal.

use crate::errors::{BadInput, Refused};
use crate::framework::{EBX, ESI, RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::math::x87;
use crate::math::x87_run::int_lerp;
use crate::physics::grade_or_reset::{MODE, copies, grade_or_reset_at};
use crate::physics::mirror::{Mirror, Spec};
use crate::physics::rate_damage::{OnMirror, fill, rate_damage_plain_at};
use crate::physics::reset_car_grades::reset_grades_at;
use crate::screens::harness::Left;
use crate::service::floats::{
    Agreement, FLOAT, Floats, Tolerance, at_test_precisions, compare_memory, dispatch_state_floats,
    floats_entry,
};
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::set_handling", rva(0x43EB5D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const RANGES: [(u32, u32); 3] = [
    (0x719FD8, 0x71AFB8),
    (0x75B340, 0x75B6AC),
    (0x613E00, 0x614D00),
];
pub const OWN_SPEC: Spec = (0x43EB5D, 0x177, &[0xE8, 0x2C, 0xCF, 0xFF, 0xFF], &[0], 49);
const KIND: u32 = 0x71A018; // g_carDriveKind
const K256: u32 = 0x5C5E2C; // flt_5C5E2C, 256.0

/// (out, setting, hi, lo) in the original's order, but the drive kind's.
const BEFORE: [(u32, u32, u32, u32); 4] = [
    (0x614C0C, 0x71A014, 0x613F28, 0x613F24),
    (0x614C08, 0x71A014, 0x613F30, 0x613F2C),
    (0x614BF0, 0x719FF8, 0x613F20, 0x613F1C),
    (0x614BF4, 0x71A008, 0x613F48, 0x613F44),
];
const AFTER: [(u32, u32, u32, u32); 4] = [
    (0x614BEC, 0x71A00C, 0x613F88, 0x613F84),
    (0x614BF8, 0x71A00C, 0x613F70, 0x613F6C),
    (0x614C00, 0x71A00C, 0x613F78, 0x613F74),
    (0x614C04, 0x71A00C, 0x613F80, 0x613F7C),
];

/// The converted version (`rc2k_core::physics::handling`, plain floats) on the variables
/// `m` reaches: the hook's. `physics::grade_or_reset` as there, its rating converted.
// Converted: `rc2k_core::physics::handling::set_handling`.
pub fn set_handling_plain_at(m: &Mirror) -> Result<Left, Refused> {
    let grade = |_: &mut OnMirror| unsafe {
        if m.get(MODE) == 2 {
            let car = m.a(0x719FD8);
            reset_grades_at(car, m.a(0x71A00C), m.a(0x71A008)).with(ESI, car)
        } else {
            rate_damage_plain_at(m)
        }
    };
    let (left, kind) = rc2k_core::physics::handling::set_handling(&mut OnMirror(m), grade)?;
    Ok(left.with(EBX, kind))
}

/// The float places it writes, held to `FLOAT` at extended precision.
fn floats(m: &Mirror) -> Vec<(Region, Tolerance)> {
    let mut f = crate::physics::rate_damage::floats(m);
    f.push(((m.a(0x614BEC), 0x24), FLOAT));
    f
}

/// The Rust core on the variables `m` reaches (the exact one, for `race::settle_car` and
/// `physics::car_physics_step`).
pub unsafe fn set_handling_at(m: &Mirror) -> Result<Left, Refused> {
    unsafe {
        let kind = m.get(KIND);
        if kind > 1 {
            return Err(BadInput("a drive kind above 1 (the pair tables hold two)").into());
        }
        let left = grade_or_reset_at(m);
        let k256 = m.get(K256);
        let lerp = |(out, n, hi, lo): (u32, u32, u32, u32)| {
            m.put(out, int_lerp(m.get(n) as i32, k256, m.get(hi), m.get(lo)));
        };
        for v in BEFORE {
            lerp(v);
        }
        lerp((0x614BFC, 0x71A008, 0x613F54 + 4 * kind, 0x613F4C + 4 * kind));
        m.put(0x614BFC, x87::mul(m.get(0x614BFC), m.get(0x614C18)));
        for v in AFTER {
            lerp(v);
        }
        Ok(left.with(EBX, kind))
    }
}

/// The Rust version (the exact one).
pub unsafe fn set_handling() -> Result<Left, Refused> {
    unsafe { set_handling_at(&Mirror::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    set_handling_plain_at(&Mirror::game())
}

fn regions(m: &Mirror) -> Vec<Region> {
    let mut r = crate::physics::grade_or_reset::regions(m);
    r.push((m.a(0x614BEC), 0x24));
    r
}

extern "C" fn dispatch(r: &mut Regs) {
    let m = Mirror::game();
    let memory = floats(&m);
    let floats = Floats {
        memory: &memory,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(&m), &floats, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let m = Mirror::of(&RANGES);
    let (inner, keep) = match copies(&m) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let copy = match m.copy(OWN_SPEC, &|a| (a == 0x43BA8E).then_some(inner)) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = m.regions();
    let memory = floats(&m);
    let tolerant = Floats {
        memory: &memory,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, fl| {
        for seed in 1..=8u32 {
            let saved = capture(&all);
            fill(&m, seed);
            unsafe {
                m.put(MODE, if seed % 4 == 1 { 2 } else { 0 });
                m.put(KIND, seed % 2);
                for (k, v) in [0x719FF8u32, 0x71A008, 0x71A00C, 0x71A014]
                    .into_iter()
                    .enumerate()
                {
                    m.put(v, (seed * 37 + k as u32 * 71) % 300);
                }
                m.put(0x614C18, (0.5 + seed as f32 / 7.0).to_bits());
            }
            let input = capture(&all);
            let mut ro = Regs::SENTINEL;
            unsafe { call_game(&mut ro, copy.addr()) };
            let theirs = capture(&all);
            restore(&all, &input);
            let mine_left = set_handling_plain_at(&m);
            let mine = capture(&all);
            let regs_ok = mine_left
                .as_ref()
                .is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro));
            let agreement = compare_memory(&all, &theirs, &mine, fl);
            t.check(regs_ok && agreement != Agreement::Differ, || {
                let first = theirs.iter().zip(&mine).position(|(a, b)| a != b);
                format!(
                    "control word {word:#x}, seed {seed}: regs_ok={regs_ok}, first difference at byte {first:?}"
                )
            });
            restore(&all, &saved);
        }
    });
    let saved = capture(&all);
    unsafe { m.put(KIND, 2) };
    let before = capture(&all);
    let refused = set_handling_plain_at(&m).is_err();
    t.check(refused && capture(&all) == before, || {
        "drive kind 2: not refused, or something written".into()
    });
    restore(&all, &saved);
    drop((copy, keep));
    t.finish()
}
