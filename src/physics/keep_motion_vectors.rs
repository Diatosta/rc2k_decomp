//! `physics::keep_motion_vectors` (0x43BD41): unless `dword_614148` is set, the three
//! dwords at `dword_71A170` (the vector `physics::store_motion_length` measures) into
//! `dword_614BCC` .. `D4`, and those at `dword_71A15C` into `dword_614BDC` .. `E4` (eax
//! pushed around).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B89B), every physics step.
//! Self-test: `physics/copy_hooks_test.rs`, 8 fillings, the switch on and off.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::copy_hooks_test::{lot, test_moves};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::keep_motion_vectors",
    rva(0x43BD41),
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

/// The switch, six from, six to (IDA addresses).
const VARIABLES: [usize; 13] = [
    0x614148, 0x71A170, 0x71A174, 0x71A178, 0x71A15C, 0x71A160, 0x71A164, 0x614BCC, 0x614BD0,
    0x614BD4, 0x614BDC, 0x614BE0, 0x614BE4,
];

unsafe fn core(v: &[u32]) {
    unsafe {
        if (v[0] as *const u32).read_unaligned() != 0 {
            return;
        }
        for k in 0..6 {
            (v[k + 7] as *mut u32).write_unaligned((v[k + 1] as *const u32).read_unaligned());
        }
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn keep_motion_vectors() {
    unsafe { core(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { keep_motion_vectors() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    dispatch_state(&HOOK, r, call_original, &[(v[7], 12), (v[10], 12)], &rust);
}

fn self_test() -> (usize, usize) {
    test_moves(
        &HOOK,
        0x43BD41,
        0x4A,
        &[0x83, 0x3D, 0x48, 0x41, 0x61, 0x00, 0x00],
        &VARIABLES,
        13,
        8,
        &|seed, out| {
            lot(seed, out);
            out[0] = seed % 2;
        },
        &|v, _| {
            unsafe { core(v) };
            Left::nothing()
        },
    )
}
