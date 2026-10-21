//! `physics::ease_low_speed_forces` (0x440CF3): `flt_61475C` 1.0; when |`g_carSpeed`| is
//! below `flt_613E64` (20; an unordered compare counts as below), `flt_61475C` =
//! (|speed| / `flt_613E64` + `flt_613E68`) x `flt_613E6C` and the two forces
//! `flt_614248`, `flt_61424C` times it.
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller's next is an `fld`)
//!
//! Caller: `sub_43DFB1` (0x43E523).
//! Self-test: a copy of the original (its four variables the test's) against the converted
//! version, speeds by lot about the limit, exact at double precision.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::math::x87;
use crate::math::x87_run::{TEST_CONTROL_WORDS, abs_compare, abs_div_add_mul, with_control_word};
use crate::physics::copy_hooks_test::check_floats_at;
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::floats::{FLOAT, Floats, dispatch_state_floats};
use crate::service::harness::capture;
use rc2k_core::physics::drive_forces::{eased, low_speed_factor};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::ease_low_speed_forces",
    rva(0x440CF3),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

/// The core: the factor, the speed and the two forces at these places.
pub unsafe fn ease_low_speed_forces_at(factor: u32, speed: u32, forces: [u32; 2]) {
    unsafe {
        put(factor, 0x3F80_0000);
        let limit = k(0x613E64);
        if abs_compare(at(speed), limit) & 0x100 == 0 {
            return;
        }
        put(
            factor,
            abs_div_add_mul(at(speed), limit, k(0x613E68), k(0x613E6C)),
        );
        for f in forces {
            put(f, x87::mul(at(f), at(factor)));
        }
    }
}

/// The converted version (`rc2k_core::physics::drive_forces`, plain floats) on the
/// factor, the speed and the two forces at these places.
// Converted: `rc2k_core::physics::drive_forces::{low_speed_factor, eased}`.
pub unsafe fn ease_low_speed_forces_plain_at(factor: u32, speed: u32, forces: [u32; 2]) {
    unsafe {
        let f = |a: u32| f32::from_bits(at(a));
        let kf = |a: usize| f32::from_bits(k(a));
        put(factor, 0x3F80_0000);
        let Some(by) = low_speed_factor(f(speed), kf(0x613E64), kf(0x613E68), kf(0x613E6C)) else {
            return;
        };
        put(factor, by.to_bits());
        let eased = eased([f(forces[0]), f(forces[1])], by);
        put(forces[0], eased[0].to_bits());
        put(forces[1], eased[1].to_bits());
    }
}

fn game() -> (u32, u32, [u32; 2]) {
    let g = |a: usize| game_addr(rva(a)) as u32;
    (g(0x61475C), g(0x71A184), [g(0x614248), g(0x61424C)])
}

/// The Rust version (the exact one, for `physics::drive_wheels_step`).
pub unsafe fn ease_low_speed_forces() {
    let (factor, speed, forces) = game();
    unsafe { ease_low_speed_forces_at(factor, speed, forces) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    let (factor, speed, forces) = game();
    unsafe { ease_low_speed_forces_plain_at(factor, speed, forces) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let (factor, _, forces) = game();
    let floats = Floats {
        memory: &[((factor, 4), FLOAT), ((forces[0], 8), FLOAT)],
        regs: &[],
    };
    dispatch_state_floats(
        &HOOK,
        r,
        call_original,
        &[(factor, 4), (forces[0], 8)],
        &floats,
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 4];
    let base = cells.as_mut_ptr() as u32;
    let (factor, speed, forces) = (base, base + 4, [base + 8, base + 12]);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x440CF3,
            len: 0x63,
            entry: &[0xD9, 0xE8, 0xD9, 0x1D, 0x5C, 0x47, 0x61, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x61475C, factor),
            (0x71A184, speed),
            (0x614248, forces[0]),
            (0x61424C, forces[1]),
        ],
        10,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 16)];
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..10u32 {
                let fill = || {
                    let mut f = [0u32; 3];
                    test_floats(seed * 3 + 1, &mut f);
                    let s = match seed {
                        0 => 0x7FC0_0000,
                        1 => 20.0f32.to_bits(),
                        2 => (-19.5f32).to_bits(),
                        _ => (f32::from_bits(f[0]) % 40.0).to_bits(),
                    };
                    unsafe { (base as *mut [u32; 4]).write([0xAAAA_AAAA, s, f[1], f[2]]) };
                };
                fill();
                let mut ro = Regs::SENTINEL;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = capture(&all);
                fill();
                unsafe { ease_low_speed_forces_plain_at(factor, speed, forces) };
                let mine = capture(&all);
                let regs_ok = ro.preserved_except(&Regs::SENTINEL, 0);
                let floats = [((factor, 4), FLOAT), ((forces[0], 8), FLOAT)];
                check_floats_at(
                    &mut t,
                    "physics::ease_low_speed_forces",
                    word,
                    &all,
                    &theirs,
                    &mine,
                    &floats,
                    regs_ok,
                    || {
                        format!(
                            "control word {word:#x}, seed {seed}: regs_ok={regs_ok} original {theirs:x?}; rust {mine:x?}"
                        )
                    },
                );
            }
        });
    }
    drop((copy, cells));
    t.finish()
}
