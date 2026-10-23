//! `physics::move_car` (0x43CCB0): the work car's place (`dword_71A10C` .. `114`, whole
//! numbers) moved by its motion (`dword_71A170` .. `178`, floats) times 65536 and the step
//! `flt_61388C` (`fistp` through `dword_62400C`), x, z, then y.
//!
//! In:  nothing
//! Out: eax y's move; the rest kept; no flags read
//!
//! Callers: `sub_43B635` (0x43B827, 0x43B8A0), every physics step.
//! Self-test: `physics/copy_hooks_test.rs` (`test_moves_floats`: a copy on test
//! variables against the converted version), 8 fillings, exact at double precision.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs, game_addr, game_read, rva};
use crate::math::x87_run::mul2_to_int;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;
use rc2k_core::physics::motion::move_place;

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

/// The converted version (`rc2k_core::physics::motion`, plain floats) on the variables
/// at `v`: the last move.
// Converted: `rc2k_core::physics::motion::move_place`.
pub unsafe fn move_at_plain(v: &[u32]) -> u32 {
    unsafe {
        let k = |x: usize| f32::from_bits(game_read::<u32>(rva(x)));
        let motion = [0, 1, 2].map(|n| f32::from_bits(get(v, n)));
        let mut place = [get(v, 3), get(v, 4), get(v, 5)];
        let last = move_place(motion, &mut place, k(0x5C5D98), k(0x61388C));
        put(v, 6, last as u32);
        for (n, p) in place.iter().enumerate() {
            put(v, 3 + n, *p);
        }
        last as u32
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (the exact one, for `physics::car_physics_step` and
/// `physics::apply_wheel_torques`).
pub unsafe fn move_car() -> u32 {
    unsafe { move_at(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { move_at_plain(&game()) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
