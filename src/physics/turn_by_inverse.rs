//! `physics::turn_by_inverse` (0x43F118): the three floats at esi turned and placed by
//! `g_carInverseMatrix` (0x71A9A4, the work copy's +0x9CC, made from `g_crashMatrix` by
//! `sub_43F16C`) with `scene::rotate_point`, and stored back (rounded to floats).
//!
//! In:  esi the point
//! Out: the point turned; every register kept
//!
//! See `physics/turn_by_car_matrix.rs` (the core and the test). Callers (13): the cars'
//! physics under `sub_4454D2`.
//! Self-test: as `physics::turn_by_car_matrix`'s, on this function's copy.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::turn_by_car_matrix::{test_turn, turn_point_by, turn_point_by_plain};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::turn_by_inverse", rva(0x43F118), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const INVERSE_RVA: usize = rva(0x71A9A4); // g_carInverseMatrix

/// The Rust version.
pub unsafe fn turn_by_inverse(v: u32) {
    unsafe { turn_point_by(game_addr(INVERSE_RVA) as u32, v) }
}

/// The converted version (`turn_point_by_plain`): the hook's.
pub unsafe fn turn_by_inverse_plain(v: u32) {
    unsafe { turn_point_by_plain(game_addr(INVERSE_RVA) as u32, v) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { turn_by_inverse_plain(r.esi) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(r.esi, 12)], &rust);
}

fn self_test() -> (usize, usize) {
    test_turn(
        &HOOK,
        0x43F118,
        &[0xD9, 0x46, 0x08, 0xD9, 0x46, 0x04],
        0x71A9A4,
    )
}
