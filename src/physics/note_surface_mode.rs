//! `physics::note_surface_mode` (0x43F180): `dword_71A6C8` = 1 when `dword_71A65C` is 2
//! or 3, else 0. What the mode is is not known.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `physics::feel_ground` (0x43F1A8), first.
//! Self-test: `physics/copy_hooks_test.rs` (a copy on test variables), modes 0 .. 5.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::copy_hooks_test::test_moves;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::note_surface_mode",
    rva(0x43F180),
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

const VARIABLES: [usize; 2] = [0x71A65C, 0x71A6C8];

/// The Rust core on the variables at `v`.
pub unsafe fn note_at(v: &[u32]) {
    unsafe {
        let mode = (v[0] as *const u32).read_unaligned();
        (v[1] as *mut u32).write_unaligned(u32::from(mode == 2 || mode == 3));
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn note_surface_mode() {
    unsafe { note_at(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { note_surface_mode() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(game()[1], 4)], &rust);
}

fn self_test() -> (usize, usize) {
    test_moves(
        &HOOK,
        0x43F180,
        0x28,
        &[0x83, 0x3D, 0x5C, 0xA6, 0x71, 0x00, 0x02],
        &VARIABLES,
        4,
        6,
        &|seed, out| {
            out[0] = seed;
            out[1] = 0xAAAA_AAAA;
        },
        &|v, _| {
            unsafe { note_at(v) };
            Left::nothing()
        },
    )
}
