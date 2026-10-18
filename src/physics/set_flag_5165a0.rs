//! `physics::set_flag_5165a0` (0x45D16D): `dword_5165A0` 1 when the current car's entry in
//! `dword_641680` is not -1. See `physics/listed_car_flags.rs` (the core, the test).
//!
//! In:  nothing
//! Out: eax the entry; the rest kept
//!
//! Caller: `sub_436AFB`.
//! Refuses (before anything) a current car past 7.
//! Self-test: `listed_car_flags::test_twin`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::listed_car_flags::{set_flag, test_twin};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::set_flag_5165a0", rva(0x45D16D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FLAG: usize = 0x5165A0; // dword_5165A0

/// The Rust version.
pub unsafe fn set_flag_5165a0() -> Result<Left, Refused> {
    unsafe { set_flag(FLAG) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { set_flag_5165a0() }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[(game_addr(rva(FLAG)) as u32, 4)],
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    test_twin(&HOOK, 0x45D16D, &[0xA1, 0xF8, 0xB3, 0x71, 0x00], FLAG)
}
