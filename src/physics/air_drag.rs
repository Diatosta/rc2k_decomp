//! `physics::air_drag` (0x43F2DA): for the motion's three (`dword_71A17C`,
//! `dword_71A180`, `g_carSpeed`): its square times `flt_613FCC` over `flt_5C5E34` into
//! `flt_614810` .. `flt_614818`, made against the motion (|it| when the motion is below 0
//! or no number, else -|it|). By the look of it the air's drag (a reading).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B80E), every physics step.
//! Self-test: `physics/copy_hooks_test.rs` (a copy on test variables), 8 fillings (signs,
//! zeros, NaN) under three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, game_read, hook_asm, rva};
use crate::math::x87;
use crate::math::x87_run::{
    TEST_CONTROL_WORDS, abs_of, compare_full, square_times_over, with_control_word,
};
use crate::physics::copy_hooks_test::test_moves;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::air_drag", rva(0x43F2DA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const VARIABLES: [usize; 6] = [0x71A17C, 0x71A180, 0x71A184, 0x614810, 0x614814, 0x614818];

/// The Rust core on the variables at `v`.
pub unsafe fn drag_at(v: &[u32]) {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
        let (k, d, zero) = (
            game_read::<u32>(rva(0x613FCC)),
            game_read::<u32>(rva(0x5C5E34)),
            game_read::<u32>(rva(0x5C5D90)),
        );
        for i in 0..3 {
            put(3 + i, square_times_over(get(i), k, d));
            let a = abs_of(get(3 + i));
            let below = compare_full(get(i), zero) & 0x100 != 0;
            put(3 + i, if below { a } else { x87::negated(a) });
        }
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn air_drag() {
    unsafe { drag_at(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { air_drag() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    dispatch_state(&HOOK, r, call_original, &[(v[3], 12)], &rust);
}

fn fill(seed: u32, out: &mut [u32]) {
    let motion = [
        [3.0f32, -4.0, 25.0],
        [0.0, -0.0, 1.0],
        [-17.5, 2.25, f32::NAN],
        [100.0, -100.0, 0.5],
    ][(seed % 4) as usize];
    for k in 0..3 {
        out[k] = (motion[k] * (1.0 + seed as f32 / 10.0)).to_bits();
        out[3 + k] = 0xAAAA_AAAA;
    }
}

fn self_test() -> (usize, usize) {
    let mut total = (0, 0);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            let (c, f) = test_moves(
                &HOOK,
                0x43F2DA,
                0xBB,
                &[0xD9, 0x05, 0x7C, 0xA1, 0x71, 0x00],
                &VARIABLES,
                15,
                8,
                &fill,
                &|v, _| {
                    unsafe { drag_at(v) };
                    Left::nothing()
                },
            );
            total.0 += c;
            total.1 += f;
        });
    }
    total
}
