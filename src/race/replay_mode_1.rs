//! `race::replay_mode_1` (0x41CD75): with a replay there (`dword_5BEC84` not 0), the replay
//! viewer `race::replay_viewer` with eax 1 (see `race/replay_mode.rs`).
//!
//! In:  nothing
//! Out: eax 1 and the registers as the viewer leaves them, with a replay; else all kept
//!
//! Callers: the review menus' replay items (`menu::replay_item_a`) and 0x4430EF. The viewer through `part`
//! (in the game its Rust version). Ungated (the viewer runs for long and gates its own
//! frames). Shadow mode runs the original alone (the viewer plays the replay). Self-test: `replay_mode::test_mode`.

use crate::framework::{RawHook, Regs, hook_asm, rva, shadow};
use crate::race::replay_mode::{game_part, replay_mode, test_mode};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::replay_mode_1", rva(0x41CD75), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { replay_mode(r, 1, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { replay_mode(r, 1, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    test_mode(&HOOK, 0x41CD75, 1)
}
