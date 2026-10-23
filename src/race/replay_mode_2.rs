//! `race::replay_mode_2` (0x41CD9D): the replay viewer with eax 2 (a saved replay chosen
//! first), with no check of a replay there (unlike `replay_mode_1` / `_3`).
//!
//! In:  nothing
//! Out: eax 2, the rest as the viewer leaves them (kept)
//!
//! Caller: `sub_470529` (a menu item). Its call through `part` (0 the viewer; in the game
//! its Rust version). Ungated (the viewer runs for long and gates its own frames). Shadow
//! mode runs the original alone. Self-test: a copy whose call goes to a stand-in, against
//! the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::viewer::{Part, dispatch_parts, regs_entry, run_both};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::replay_mode_2", rva(0x41CD9D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

/// The Rust version.
pub fn replay_mode_2(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = Regs { eax: 2, ..*entry };
    part(1, &mut r)?;
    Ok(r)
}

/// In the game: the viewer's Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::replay_viewer::{game_part, replay_viewer};
    *r = replay_viewer(r, &mut game_part, true)?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        replay_mode_2(r, &mut game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| replay_mode_2(r, &mut game_part))
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov eax, 2
    let copy = match crate::race::viewer::copy(
        0x41CD9D,
        0xB,
        &[0xB8, 0x02, 0x00, 0x00, 0x00],
        &[0x5],
        &[(0x41CF0C, 1)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::race::viewer::quiet();
    run_both(
        &mut t,
        "the viewer",
        copy.addr(),
        Regs::SENTINEL,
        &[],
        0,
        &|| {},
        &|r| replay_mode_2(r, &mut crate::race::viewer::test_part),
    );
    t.finish()
}
