//! `physics::move_car` (0x43CCB0): the work car's place (`dword_71A10C` .. `114`, whole
//! numbers) moved by its motion (`dword_71A170` .. `178`, floats) times 65536 and the step
//! `flt_61388C` (`fistp` through `dword_62400C`), x, z, then y.
//!
//! In:  nothing
//! Out: eax y's move; the rest kept; no flags read
//!
//! Callers: `sub_43B635` (0x43B827, 0x43B8A0), every physics step.
//! Self-test: `physics/copy_hooks_test.rs` (a copy on test variables), 8 fillings.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, game_addr, game_read, hook_asm, rva};
use crate::math::x87_run::mul2_to_int;
use crate::physics::copy_hooks_test::{lot, test_moves};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::move_car", rva(0x43CCB0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The motion's three, the place's three, the scratch.
const VARIABLES: [usize; 7] = [
    0x71A170, 0x71A174, 0x71A178, 0x71A10C, 0x71A110, 0x71A114, 0x62400C,
];

unsafe fn get(v: &[u32], k: usize) -> u32 {
    unsafe { (v[k] as *const u32).read_unaligned() }
}

unsafe fn put(v: &[u32], k: usize, x: u32) {
    unsafe { (v[k] as *mut u32).write_unaligned(x) }
}

/// The Rust core on the variables at `v`: the last move.
pub unsafe fn move_at(v: &[u32]) -> u32 {
    unsafe {
        let k65536: u32 = game_read(rva(0x5C5D98));
        let dt: u32 = game_read(rva(0x61388C));
        let mut last = 0;
        for (motion, place) in [(0, 3), (2, 5), (1, 4)] {
            last = mul2_to_int(get(v, motion), k65536, dt) as u32;
            put(v, 6, last);
            put(v, place, get(v, place).wrapping_add(last));
        }
        last
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn move_car() -> u32 {
    unsafe { move_at(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { move_car() }))
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    dispatch_state(&HOOK, r, call_original, &[(v[3], 12), (v[6], 4)], &rust);
}

fn fill(seed: u32, out: &mut [u32]) {
    lot(seed, out);
    for (k, o) in out.iter_mut().take(3).enumerate() {
        *o = (seed as f32 * 2.5 - 9.0 + k as f32).to_bits();
    }
}

fn self_test() -> (usize, usize) {
    test_moves(
        &HOOK,
        0x43CCB0,
        0x6A,
        &[0xD9, 0x05, 0x70, 0xA1, 0x71, 0x00],
        &VARIABLES,
        12,
        8,
        &fill,
        &|v, _| Left::nothing().with(EAX, unsafe { move_at(v) }),
    )
}
