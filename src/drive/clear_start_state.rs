//! `drive::clear_start_state` (0x4376E3): six of the work copy's start values 0:
//! `dword_71A234` (the grades' mode, `physics::grade_or_reset`), `dword_71A070`,
//! `dword_71AA88`, `flt_71A650`, `flt_71A658`, `dword_71A01C`.
//!
//! In:  nothing
//! Out: every register kept; no flags read (`drive::store_car` next)
//!
//! Caller: `sub_43703E` (0x4371BC, 0x43729B).
//! Self-test: `physics/copy_hooks_test.rs`, 4 fillings.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::copy_hooks_test::{lot, test_moves};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::clear_start_state", rva(0x4376E3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const VARIABLES: [usize; 6] = [0x71A234, 0x71A070, 0x71AA88, 0x71A650, 0x71A658, 0x71A01C];

unsafe fn core(v: &[u32]) {
    for &a in v {
        unsafe { (a as *mut u32).write_unaligned(0) };
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn clear_start_state() {
    unsafe { core(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { clear_start_state() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions: Vec<_> = game().iter().map(|&a| (a, 4)).collect();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    test_moves(
        &HOOK,
        0x4376E3,
        0x3D,
        &[0xC7, 0x05, 0x34, 0xA2, 0x71, 0x00, 0x00, 0x00, 0x00, 0x00],
        &VARIABLES,
        6,
        4,
        &lot,
        &|v, _| {
            unsafe { core(v) };
            Left::nothing()
        },
    )
}
