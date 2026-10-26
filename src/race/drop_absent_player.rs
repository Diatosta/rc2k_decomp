//! `race::drop_absent_player` (0x436402): in a two-player race (`dword_63DB94` 2, unless
//! `dword_62389C` is 2 or 4), the first player whose block's +0xB0 is not 0 is out of the
//! race: +0x25C 0, +0xFA8 1.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/race_cars.rs`. Callers: `sub_43603C` and its four set-ups. Shadow mode: the
//! Rust version, the blocks put back, the original's, compared. Self-test: the game's
//! blocks filled with a pattern, every mode 0 .. 5, one and two players, each player
//! absent or not, both versions, put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::race_cars::*;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::drop_absent_player", rva(0x436402), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { drop_absent_player() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &player_regions(), &rust);
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
    let kept = saved_regions();
    let saved = capture(&kept);
    let regions = player_regions();
    for mode in 0..6u32 {
        for players in [1u32, 2] {
            for absent in [[0u32, 0u32], [1, 0], [0, 0x8000_0000], [5, 6]] {
                check_state(
                    &mut t,
                    &format!("mode {mode}, {players} players, absent {absent:x?}"),
                    call_original,
                    Regs::SENTINEL,
                    &|| prepare(0x2468_ACE0, mode, players, 2, absent),
                    &regions,
                    &rust,
                );
            }
        }
    }
    restore(&kept, &saved);
    t.finish()
}
