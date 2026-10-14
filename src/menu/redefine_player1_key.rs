//! `menu::redefine_player1_key` (0x47D3D8): one frame of redefining a key of player 1.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `menu/redefine_keys.rs`; the player is the first of `g_players`, whatever
//! `dword_75598C` says. In the draw list of player 1's redefine keys page (0x758DE2); IDA had
//! not made it a function. `sub_47D43E` (where the key goes, the same as `sub_47D4DE`) is
//! inlined.
//! Refuses, before the key is taken: no player 1, a place of 12 or more, a reserved
//! list without its end.
//! Compared in shadow mode (the key replayed). Self-test: two settings of `dword_75598C`, 48 settings each.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::redefine_keys::{check_redefine, regions, rust};
use crate::service::harness::dispatch_keyed_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::redefine_player1_key",
    rva(0x47D3D8),
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

/// Player 1.
fn player() -> u32 {
    crate::menu::player::player_at(0)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_keyed_state(&HOOK, r, call_original, &regions(player()), &rust(player));
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    check_redefine(&mut t, call_original, &[0, 2], player);
    t.finish()
}
