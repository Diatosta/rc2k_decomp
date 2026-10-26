//! `physics::copy_car_word_8` (0x440C66): `g_car` +8 (`dword_719FE0`) into
//! `dword_6140F0`, and `dword_6140EC` 0 (in the gearbox: by the look of it the moment a
//! gear change began; a reading).
//!
//! In:  nothing
//! Out: eax the word; the rest kept; no flags read
//!
//! Caller: `sub_43C8AC`.
//! Self-test: `physics/copy_hooks_test.rs`, 6 fillings.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::copy_hooks_test::{lot, test_moves};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::copy_car_word_8", rva(0x440C66), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The word, its copy, the cleared one (IDA addresses).
const VARIABLES: [usize; 3] = [0x719FE0, 0x6140F0, 0x6140EC];

unsafe fn core(v: &[u32]) -> Left {
    unsafe {
        let w = (v[0] as *const u32).read_unaligned();
        (v[1] as *mut u32).write_unaligned(w);
        (v[2] as *mut u32).write_unaligned(0);
        Left::nothing().with(EAX, w)
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn copy_car_word_8() -> Left {
    unsafe { core(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { copy_car_word_8() })
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    dispatch_state(&HOOK, r, call_original, &[(v[1], 4), (v[2], 4)], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    test_moves(
        &HOOK,
        0x440C66,
        0x15,
        &[0xA1, 0xE0, 0x9F, 0x71, 0x00],
        &VARIABLES,
        3,
        6,
        &lot,
        &|v, _| unsafe { core(v) },
    )
}
