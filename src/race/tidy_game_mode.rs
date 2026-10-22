//! `race::tidy_game_mode` (0x436B69): the game mode's tidying ("gamemodestuff tidy"):
//! nothing but, for `dword_62389C` 2, a call of `nullsub_2`.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/race_end.rs`. One caller, `sub_444AC7` (`pushf` follows). Shadow mode: the
//! original and the Rust version, registers compared (nothing written). Self-test: modes
//! 0 .. 4, the game's mode put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::tidy_game_mode", rva(0x436B69), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
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
    let kept = [at(rva(0x62389C))];
    let saved = capture(&kept);
    for mode in 0..5u32 {
        check_state(
            &mut t,
            &format!("mode {mode}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(rva(0x62389C), mode) },
            &[],
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
