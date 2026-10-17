//! `physics::keep_motion` (0x440B7C): `dword_71A17C`, `dword_71A180`, `g_carSpeed`
//! (0x71A184) into `dword_61409C`, `dword_6140A0`, `dword_6140A4` (eax pushed around).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B87E), every physics step.
//! Self-test: `physics/copy_hooks_test.rs`, 8 fillings.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::copy_hooks_test::{lot, test_moves};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::keep_motion", rva(0x440B7C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// From, to, three times (IDA addresses).
const VARIABLES: [usize; 6] = [0x71A17C, 0x71A180, 0x71A184, 0x61409C, 0x6140A0, 0x6140A4];

unsafe fn core(v: &[u32]) {
    for k in 0..3 {
        unsafe { (v[k + 3] as *mut u32).write_unaligned((v[k] as *const u32).read_unaligned()) };
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn keep_motion() {
    unsafe { core(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { keep_motion() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    dispatch_state(&HOOK, r, call_original, &[(v[3], 12)], &rust);
}

fn self_test() -> (usize, usize) {
    test_moves(
        &HOOK,
        0x440B7C,
        0x21,
        &[0x50, 0xA1, 0x7C, 0xA1, 0x71, 0x00],
        &VARIABLES,
        6,
        8,
        &lot,
        &|v, _| {
            unsafe { core(v) };
            Left::nothing()
        },
    )
}
