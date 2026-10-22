//! `drive::play_inputs_if` (0x4600A9): `drive::play_inputs` while playing a replay
//! (`dword_5C5CE0` 2).
//!
//! In:  nothing
//! Out: as `play_inputs` leaves the registers when called; no flags read
//!
//! Caller: `drive::replay_step`.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): the mode and cars.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::play_inputs_if", rva(0x4600A9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::plays_inputs`.
pub unsafe fn play_inputs_if(entry: &Regs) -> Left {
    if rc2k_core::drive::replay_frames::plays_inputs(g(0x5C5CE0)) {
        return unsafe { crate::drive::play_inputs::play_inputs(entry) };
    }
    Left::nothing()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { play_inputs_if(r) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(a(0x6138C4), 16)], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all: Vec<Region> = vec![
        (a(0x6138C4), 16),
        (a(0x71B3F8), 4),
        (a(0x6EF7E4), 4),
        (a(0x7194BC), 0x80),
        (a(0x5C5CE0), 4),
    ];
    let saved = capture(&all);
    for case in 0..9u32 {
        let set = || {
            p(0x5C5CE0, case % 3);
            p(0x71B3F8, case / 3);
            p(0x6EF7E4, 2);
        };
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
