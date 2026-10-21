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
//! Self-test: `physics/copy_hooks_test.rs` (`test_moves_floats`: a copy on test
//! variables against the converted version), 8 fillings, exact at double precision.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, game_read, hook_asm, rva};
use crate::math::x87_run::sum_over;
use crate::physics::copy_hooks_test::test_moves_floats;
use crate::screens::harness::Left;
use crate::service::floats::{FLOAT, Floats, dispatch_state_floats};
use rc2k_core::physics::motion::sum_forces as sum_forces_core;
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

/// The converted version (`rc2k_core::physics::motion`, plain floats) on the variables
/// at `v`.
// Converted: `rc2k_core::physics::motion::sum_forces`.
pub unsafe fn sum_at_plain(v: &[u32]) {
    unsafe {
        let get = |k: usize| f32::from_bits((v[k] as *const u32).read_unaligned());
        let forces: [f32; 13] = std::array::from_fn(get);
        let k = |x: usize| f32::from_bits(game_read(rva(x)));
        let sums = sum_forces_core(&forces, k(0x613FC8), k(0x61388C));
        for (n, s) in sums.iter().enumerate() {
            (v[13 + n] as *mut u32).write_unaligned(s.to_bits());
        }
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (the exact x87 one, for `physics::car_physics_step`).
pub unsafe fn sum_forces() {
    unsafe { sum_at(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { sum_at_plain(&game()) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    let floats = Floats {
        memory: &[((v[13], 0x14), FLOAT)],
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &[(v[13], 0x14)], &floats, &rust);
}

fn fill(seed: u32, out: &mut [u32]) {
    for (k, o) in out.iter_mut().enumerate() {
        *o = (seed * 31 + k as u32 * 17) % 400;
        *o = (*o as f32 / 7.0 - 25.0).to_bits();
    }
}

fn self_test() -> (usize, usize) {
    test_moves_floats(
        &HOOK,
        0x43BBCB,
        0x9D,
        &[0xD9, 0x05, 0x50, 0x42, 0x61, 0x00],
        &VARIABLES,
        18,
        8,
        &fill,
        &|v, _| {
            unsafe { sum_at_plain(v) };
            Left::nothing()
        },
        &[
            (13, FLOAT),
            (14, FLOAT),
            (15, FLOAT),
            (16, FLOAT),
            (17, FLOAT),
        ],
        &[],
    )
}
