//! `physics::spin_wheels` (0x43CD1A): the driven wheels' turns (`dword_71A1D0`,
//! `dword_71A1D4`) on by `g_carDriveSpeed` / `g_carDriveSpeed2` over `flt_613EBC`, times
//! the step `flt_61388C`, 65536 and `flt_5C5DC8` (`fistp` through `dword_62400C`). By the
//! look of it the wheels' angles for the drawing (a reading).
//!
//! In:  nothing
//! Out: eax the second's turn; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B90A), every physics step.
//! Self-test: `physics/copy_hooks_test.rs` (`test_moves_floats`: a copy on test
//! variables against the converted version), 8 fillings, exact at double precision.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, game_addr, game_read, hook_asm, rva};
use crate::math::x87_run::div_mul3_to_int;
use crate::physics::copy_hooks_test::{lot, test_moves_floats};
use crate::screens::harness::Left;
use crate::service::floats::{Floats, ROUNDED, dispatch_state_floats, floats_entry};
use rc2k_core::physics::motion::spin;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::spin_wheels", rva(0x43CD1A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// Speed 1, speed 2, turn 1, turn 2, scratch.
const VARIABLES: [usize; 5] = [0x71A190, 0x71A194, 0x71A1D0, 0x71A1D4, 0x62400C];

unsafe fn get(v: &[u32], k: usize) -> u32 {
    unsafe { (v[k] as *const u32).read_unaligned() }
}

unsafe fn put(v: &[u32], k: usize, x: u32) {
    unsafe { (v[k] as *mut u32).write_unaligned(x) }
}

/// The Rust core on the variables at `v`: the last turn added.
pub unsafe fn spin_at(v: &[u32]) -> u32 {
    unsafe {
        let k = |x: usize| game_read::<u32>(rva(x));
        let (wheel, dt, k65536, scale) = (k(0x613EBC), k(0x61388C), k(0x5C5D98), k(0x5C5DC8));
        let mut last = 0;
        for (speed, turn) in [(0, 2), (1, 3)] {
            last = div_mul3_to_int(get(v, speed), wheel, dt, k65536, scale) as u32;
            put(v, 4, last);
            put(v, turn, get(v, turn).wrapping_add(last));
        }
        last
    }
}

/// The converted version (`rc2k_core::physics::motion`, plain floats) on the variables
/// at `v`: the last turn added.
// Converted: `rc2k_core::physics::motion::spin`.
pub unsafe fn spin_at_plain(v: &[u32]) -> u32 {
    unsafe {
        let k = |x: usize| f32::from_bits(game_read::<u32>(rva(x)));
        let speeds = [0, 1].map(|n| f32::from_bits(get(v, n)));
        let mut turns = [get(v, 2), get(v, 3)];
        let last = spin(
            speeds,
            &mut turns,
            k(0x613EBC),
            k(0x61388C),
            k(0x5C5D98),
            k(0x5C5DC8),
        );
        put(v, 4, last as u32);
        put(v, 2, turns[0]);
        put(v, 3, turns[1]);
        last as u32
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub unsafe fn spin_wheels() -> u32 {
    unsafe { spin_at(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { spin_at_plain(&game()) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    let floats = Floats {
        memory: &[((v[2], 8), ROUNDED), ((v[4], 4), ROUNDED)],
        regs: &[(EAX, ROUNDED)],
    };
    dispatch_state_floats(
        &HOOK,
        r,
        call_original,
        &[(v[2], 8), (v[4], 4)],
        &floats,
        &rust,
    );
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}

fn fill(seed: u32, out: &mut [u32]) {
    lot(seed, out);
    out[0] = (seed as f32 * 7.5 - 20.0).to_bits();
    out[1] = (seed as f32 * -3.25 + 9.0).to_bits();
}

fn self_test() -> (usize, usize) {
    test_moves_floats(
        &HOOK,
        0x43CD1A,
        0x5F,
        &[0xD9, 0x05, 0x90, 0xA1, 0x71, 0x00],
        &VARIABLES,
        8,
        8,
        &fill,
        &|v, _| Left::nothing().with(EAX, unsafe { spin_at_plain(v) }),
        &[(2, ROUNDED), (3, ROUNDED), (4, ROUNDED)],
        &[(EAX, ROUNDED)],
    )
}
