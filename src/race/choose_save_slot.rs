//! `race::choose_save_slot` (0x41E6DC): the viewer's stage tidied away
//! (`race::viewer_tidy_stage`), the saving screen of the replay slots
//! (`menu::replay_slot_loop`, eax as given: the caller's 1), the stage loaded again
//! (`race::viewer_load_stage`).
//!
//! In:  eax 1 (the saving screen)
//! Out: eax the slot chosen or -1 (the screen's; the loading keeps it); the rest as the
//!      screen leaves them
//!
//! Caller: `race::viewer_action` (0x16). Its calls through `part`: 1 `viewer_tidy_stage`,
//! 2 `replay_slot_loop`, 3 `viewer_load_stage`; in the game their Rust versions. Ungated (a
//! screen). Shadow mode runs the original alone. Self-test: a copy whose calls go to
//! stand-ins, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::viewer::{Part, answer, dispatch_parts, regs_entry, run_both, test_part};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::choose_save_slot", rva(0x41E6DC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

/// The Rust version: the registers it leaves.
pub fn choose_save_slot(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    for place in 1..=3 {
        part(place, &mut r)?;
    }
    Ok(r)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::{viewer_load_stage as load, viewer_tidy_stage as tidy};
    *r = match place {
        1 => tidy::viewer_tidy_stage(r, &mut tidy::game_part)?,
        2 => crate::menu::replay_slot_loop::replay_slot_loop(
            r,
            &mut crate::menu::replay_slot_loop::game_part,
        )?,
        _ => load::viewer_load_stage(r, &mut load::game_part)?,
    };
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        choose_save_slot(r, &mut game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| choose_save_slot(r, &mut game_part))
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // call race::viewer_tidy_stage
    let copy = match crate::race::viewer::copy(
        0x41E6DC,
        0x10,
        &[0xE8, 0xD3, 0xE7, 0xFF, 0xFF],
        &[0x0, 0x5, 0xA],
        &[(0x41CEB4, 1), (0x46F786, 2), (0x41CDC2, 3)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    run_both(
        &mut t,
        "a slot chosen",
        copy.addr(),
        Regs {
            eax: 1,
            ..Regs::SENTINEL
        },
        &[],
        0,
        &|| {
            answer(Box::new(|place, r| {
                if place == 2 {
                    r.eax = 4;
                }
            }))
        },
        &|r| choose_save_slot(r, &mut test_part),
    );
    crate::race::viewer::quiet();
    t.finish()
}
