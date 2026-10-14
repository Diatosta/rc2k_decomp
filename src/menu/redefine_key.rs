//! `menu::redefine_key` (0x47D472): one frame of redefining a key of the current player.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `menu/redefine_keys.rs`; the player is `dword_75598C`'s. In the draw lists of the
//! redefine keys pages of players 2, 3 and 4 (0x758DEE, 0x758DFA, 0x758E06). The function
//! `sub_47D4DE` (where the key goes) is inlined.
//! Refuses, before the key is taken: no current player, a place of 12 or more, a reserved
//! list without its end.
//! Compared in shadow mode (the key replayed). Self-test: the four players, 48 settings.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::redefine_keys::{check_redefine, regions, rust};
use crate::service::harness::dispatch_keyed_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::redefine_key", rva(0x47D472), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The current player.
fn player() -> u32 {
    crate::menu::player_settings::current().map_or(0, |p| p as u32)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_keyed_state(&HOOK, r, call_original, &regions(player()), &rust(player));
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    check_redefine(&mut t, call_original, &[0, 1, 2, 3], player);
    t.finish()
}
