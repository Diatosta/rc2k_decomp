//! `race::play_from_here` (0x41E1E2): see `race/play_marked.rs`: the viewer plays from the
//! block and frame shown to the end.
//!
//! In:  nothing
//! Out: eax the block played from, ebx and ecx its frame; no flags read
//!
//! Callers: `race::viewer_action` (6, and 0x22, 0x23 after it). Ungated (it plays a
//! replay). Shadow mode runs the original alone. Self-test: `play_marked::test_play` from
//! the shared tail.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::race::play_marked::{game_part, play_from_here, test_play};
use crate::race::viewer::dispatch_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::play_from_here", rva(0x41E1E2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        play_from_here(r, &mut game_part)
    });
}

fn self_test() -> (usize, usize) {
    test_play(&HOOK, true)
}
