//! `physics::turn_by_matrix_at` (0x45FC73): the three floats at esi turned and placed by
//! the matrix of the car block at edi (+0xB4; for the work car `g_crashMatrix`) with
//! `scene::rotate_point`, and stored back (rounded to floats). `physics::turn_by_car_matrix`
//! with the block given.
//!
//! In:  esi the point, edi the car block
//! Out: the point turned; every register kept
//!
//! Caller: `physics::apply_torque` (back into the world).
//! Self-test: `physics::turn_by_inverse_at`'s, on this function and offset.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::physics::turn_by_car_matrix::{turn_point_by, turn_point_by_plain};
use crate::physics::turn_by_inverse_at::test_turn_at;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::turn_by_matrix_at",
    rva(0x45FC73),
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

/// Where a car block's matrix is.
pub const MATRIX_OFFSET: u32 = 0xB4;

/// The Rust version.
pub unsafe fn turn_by_matrix_at(v: u32, car: u32) {
    unsafe { turn_point_by(car + MATRIX_OFFSET, v) }
}

/// The converted version (`turn_point_by_plain`): the hook's.
pub unsafe fn turn_by_matrix_at_plain(v: u32, car: u32) {
    unsafe { turn_point_by_plain(car + MATRIX_OFFSET, v) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { turn_by_matrix_at_plain(r.esi, r.edi) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(r.esi, 12)], &rust);
}

fn self_test() -> (usize, usize) {
    test_turn_at(&HOOK, call_original, MATRIX_OFFSET)
}
