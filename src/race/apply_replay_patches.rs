//! `race::apply_replay_patches` (0x41F2C9): see `race/replay_patches.rs`: the physics
//! patches the viewed replay was recorded with put in force.
//!
//! In:  nothing
//! Out: every register kept; no flags read (its caller `test`s eax after it)
//!
//! Caller: `race::open_viewed_replay`. Shadow mode compares. Self-test:
//! `replay_patches::test_patches`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::race::replay_patches::{apply_replay_patches, game_part, regions, test_patches};
use crate::race::viewer::{dispatch_regs, regs_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::apply_replay_patches",
    rva(0x41F2C9),
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
    // The patches' own variables are written by `apply_patches` on either side, the same.
    dispatch_regs(&HOOK, r, call_original, &regions(), 0, &|r| {
        apply_replay_patches(r, &mut game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| apply_replay_patches(r, &mut game_part))
}

fn self_test() -> (usize, usize) {
    test_patches(&HOOK, 0x41F2C9, 0x54, 0x4D, &apply_replay_patches)
}
