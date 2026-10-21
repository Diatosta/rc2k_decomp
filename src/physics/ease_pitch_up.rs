//! `physics::ease_pitch_up` (0x438CBA): half the way from `g_car` +0x6B8 (`unk_71A690`)
//! up to `dword_608C20` (unsigned halving, `shr`), kept in `dword_608C24` and added to
//! +0x6B8. `physics::ease_pitch_down` (0x438CD3) is its twin towards `dword_608C1C`.
//!
//! In:  nothing
//! Out: eax the step; the rest kept; no flags read (the caller compares `dword_608C24`)
//!
//! Caller: `sub_43907F` (0x4392CD).
//! Self-test: `physics/copy_hooks_test.rs`, 8 fillings.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::cells::{get, put};
use crate::physics::copy_hooks_test::{lot, test_moves};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use rc2k_core::physics::easing::ease_up;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::ease_pitch_up", rva(0x438CBA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The variables (IDA addresses).
const VARIABLES: [usize; 3] = [0x608C20, 0x71A690, 0x608C24];

/// The Rust version on the variables' addresses.
// Converted: `rc2k_core::physics::easing::ease_up`.
unsafe fn core(v: &[u32]) -> Left {
    unsafe {
        let mut value = get(v[1]);
        let step = ease_up(&mut value, get(v[0]));
        put(v[2], step);
        put(v[1], value);
        Left::nothing().with(EAX, step)
    }
}

/// The Rust version.
pub unsafe fn ease_pitch_up() -> Left {
    unsafe { core(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { ease_pitch_up() })
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    dispatch_state(&HOOK, r, call_original, &[(v[1], 4), (v[2], 4)], &rust);
}

fn self_test() -> (usize, usize) {
    test_moves(
        &HOOK,
        0x438CBA,
        0x19,
        &[0xA1, 0x20, 0x8C, 0x60, 0x00],
        &VARIABLES,
        4,
        8,
        &|seed, out| {
            lot(seed, out);
            let _ = out;
        },
        &|v, _| unsafe { core(v) },
    )
}
