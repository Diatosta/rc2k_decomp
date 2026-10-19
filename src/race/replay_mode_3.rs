//! `race::replay_mode_3` (0x41CD89): with a replay there (`dword_5BEC84` not 0), the replay
//! viewer `sub_41CF0C` with eax 3 (see `race/replay_mode.rs`).
//!
//! In:  nothing
//! Out: eax 3 and the registers as the viewer leaves them, with a replay; else all kept
//!
//! Callers: the review menus' replay item (`menu::replay_item_b`). The viewer through `part`
//! (in the game through the game: not ported). Shadow mode runs the original alone (the
//! viewer plays the replay). Self-test: `replay_mode::test_mode`.

use crate::framework::{RawHook, Regs, hook_asm, rva, shadow};
use crate::race::replay_mode::{game_part, replay_mode, test_mode};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::replay_mode_3", rva(0x41CD89), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { replay_mode(r, 3, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    test_mode(&HOOK, 0x41CD89, 3)
}
