//! `physics::sum_forces` (0x43BBCB): five sums over gravity (`flt_613FC8`), the first
//! three also over the step `flt_61388C`: `flt_614C2C` = `flt_614250`, `flt_614C30` =
//! `flt_614254`, `flt_614C34` = `flt_614258` + `flt_61425C`, `flt_614C38` = the four at
//! 0x614834 + 0x20 k, `flt_614C3C` = the five at 0x614774 + 0x20 k (the records
//! `physics::clear_forces` clears). By the look of it the step's forces in g (a reading).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B995), every physics step.
//! Self-test: `physics/copy_hooks_test.rs` (a copy on test variables), 8 fillings under
//! three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, game_read, hook_asm, rva};
use crate::math::x87_run::{TEST_CONTROL_WORDS, sum_over, with_control_word};
use crate::physics::copy_hooks_test::test_moves;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::sum_forces", rva(0x43BBCB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The thirteen read, the five written.
const VARIABLES: [usize; 18] = [
    0x614250, 0x614254, 0x614258, 0x61425C, 0x614834, 0x614854, 0x614874, 0x614894, 0x614774,
    0x614794, 0x6147B4, 0x6147D4, 0x6147F4, 0x614C2C, 0x614C30, 0x614C34, 0x614C38, 0x614C3C,
];

/// The Rust core on the variables at `v`.
pub unsafe fn sum_at(v: &[u32]) {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
        let g: u32 = game_read(rva(0x613FC8));
        let dt: u32 = game_read(rva(0x61388C));
        put(13, sum_over(&[get(0)], &[g, dt]));
        put(14, sum_over(&[get(1)], &[g, dt]));
        put(15, sum_over(&[get(2), get(3)], &[g, dt]));
        put(16, sum_over(&[get(4), get(5), get(6), get(7)], &[g]));
        put(
            17,
            sum_over(&[get(8), get(9), get(10), get(11), get(12)], &[g]),
        );
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn sum_forces() {
    unsafe { sum_at(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { sum_forces() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    dispatch_state(&HOOK, r, call_original, &[(v[13], 0x14)], &rust);
}

fn fill(seed: u32, out: &mut [u32]) {
    for (k, o) in out.iter_mut().enumerate() {
        *o = (seed * 31 + k as u32 * 17) % 400;
        *o = (*o as f32 / 7.0 - 25.0).to_bits();
    }
}

fn self_test() -> (usize, usize) {
    let mut total = (0, 0);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            let (c, f) = test_moves(
                &HOOK,
                0x43BBCB,
                0x9D,
                &[0xD9, 0x05, 0x50, 0x42, 0x61, 0x00],
                &VARIABLES,
                18,
                8,
                &fill,
                &|v, _| {
                    unsafe { sum_at(v) };
                    Left::nothing()
                },
            );
            total.0 += c;
            total.1 += f;
        });
    }
    total
}
