//! `physics::car_angles` (0x43BEF4): `physics::matrix_angles` of `g_crashMatrix`, then
//! each angle times 65536 (`flt_5C5D98`), 4096 (`flt_5C5E28`) and 1 / 2pi (`flt_5C5DD4`),
//! `fistp`, shifted left 4: `flt_5C70D4` into `dword_71A99C`, `D8` into `dword_71A998`,
//! `DC` into `dword_71A9A0` (`g_car` +0x9C4, +0x9C0, +0x9C8): the car's angles in the
//! game's whole-number units (a reading).
//!
//! In:  nothing
//! Out: esi = `g_crashMatrix`; the rest kept; no flags read
//!
//! Caller: `physics::update_car_angles`.
//! Self-test: a copy of the original (the matrix, angles and ints the test's, its call to
//! `matrix_angles`' test copy) on matrices by lot, under three precisions.

use crate::errors::Refused;
use crate::framework::{
    ESI, RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva,
};
use crate::math::x87_run::{TEST_CONTROL_WORDS, mul3_to_int, with_control_word};
use crate::physics::matrix_angles::{AnglePlaces, AngleTest, matrix_angles_at};
use crate::race::stage_prefs::CRASH_MATRIX_RVA;
use crate::redirect::RedirectedCopy;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::car_angles", rva(0x43BEF4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const INTS_RVA: usize = rva(0x71A998); // dword_71A998 (from D8), 99C (from D4), 9A0 (from DC)

/// The Rust core: the matrix at `m`, `matrix_angles`' variables at `p`, the three ints at
/// `ints` (+4 from the first angle, +0 from the second, +8 from the third).
pub unsafe fn car_angles_at(m: u32, p: &AnglePlaces, ints: u32) {
    unsafe {
        matrix_angles_at(m, p);
        let scale = |a: u32| {
            mul3_to_int(
                a,
                game_read::<u32>(rva(0x5C5D98)),
                game_read::<u32>(rva(0x5C5E28)),
                game_read::<u32>(rva(0x5C5DD4)),
            )
            .wrapping_shl(4)
        };
        let angle = |k: u32| (p.angles as *const u32).add(k as usize).read_unaligned();
        for (k, place) in [(0u32, 4u32), (1, 0), (2, 8)] {
            let v = scale(angle(k));
            ((ints + place) as *mut i32).write_unaligned(v);
        }
    }
}

/// The Rust version.
pub unsafe fn car_angles() -> Left {
    let m = game_addr(CRASH_MATRIX_RVA) as u32;
    unsafe { car_angles_at(m, &AnglePlaces::game(), game_addr(INTS_RVA) as u32) };
    Left::nothing().with(ESI, m)
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { car_angles() })
}

/// What the function writes.
pub fn regions() -> Vec<Region> {
    let mut r = AnglePlaces::game().regions();
    r.push((game_addr(INTS_RVA) as u32, 12));
    r
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// A copy of the original on `test` (its call to `test.entry`).
pub fn test_copy(test: &AngleTest) -> Result<RedirectedCopy, String> {
    let target = test.entry;
    let a = test.places.angles;
    crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43BEF4,
            len: 0x7B,
            entry: &[0x8D, 0x35, 0x8C, 0xA0, 0x71, 0x00],
            calls: &[6],
            jumps_out: &[],
        },
        &|r| (r == rva(0x4212C3)).then_some(target),
        &[
            (0x71A08C, test.m),
            (0x5C70D4, a),
            (0x5C70D8, a + 4),
            (0x5C70DC, a + 8),
            (0x71A998, test.ints),
            (0x71A99C, test.ints + 4),
            (0x71A9A0, test.ints + 8),
        ],
        10,
    )
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let test = match AngleTest::new() {
        Ok(x) => x,
        Err(why) => return t.skip(&why),
    };
    let copy = match test_copy(&test) {
        Ok(x) => x,
        Err(why) => return t.skip(&why),
    };
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..12u32 {
                test.fill(seed);
                let mut ro = Regs::SENTINEL;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = capture(&test.all());
                test.fill(seed);
                unsafe { car_angles_at(test.m, &test.places, test.ints) };
                let mine = capture(&test.all());
                let regs_ok = Left::nothing()
                    .with(ESI, test.m)
                    .matches(&Regs::SENTINEL, &ro);
                t.check(regs_ok && theirs == mine, || {
                    let first = theirs.iter().zip(&mine).position(|(a, b)| a != b);
                    format!("control word {word:#x}, seed {seed}: regs_ok={regs_ok}, first difference at byte {first:?}")
                });
            }
        });
    }
    drop((copy, test));
    t.finish()
}
