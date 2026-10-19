//! `race::restore_replay_patches` (0x41F31D): see `race/replay_patches.rs`: the game's
//! own physics patches back after a replay's.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `race::close_viewed_replay`. Shadow mode compares. Self-test:
//! `replay_patches::test_patches`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::race::replay_patches::{game_part, regions, restore_replay_patches, test_patches};
use crate::race::viewer::dispatch_regs;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::restore_replay_patches",
    rva(0x41F31D),
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

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_regs(&HOOK, r, call_original, &regions(), 0, &|r| {
        restore_replay_patches(r, &mut game_part)
    });
}

fn self_test() -> (usize, usize) {
    test_patches(&HOOK, 0x41F31D, 0x41, 0x3A, &restore_replay_patches)
}
