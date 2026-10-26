//! `race::checkpoint_angles` (0x4459DF): each link of the best line's angle.
//!
//! In:  nothing
//! Out: eax the angle at 2; with two checkpoints or more, ebx the count; the rest
//!      preserved
//!
//! See `race/best_line.rs`. Callers: `sub_445772` ("angles") and `sub_459BCA`. Refuses more
//! checkpoints than the lists hold. Self-test: checkpoints of the test's, the game's put
//! back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::best_line::*;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::checkpoint_angles", rva(0x4459DF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    vec![
        (game_addr(ANGLES_RVA) as u32, 4 * (MOST_CHECKS as usize + 2)),
        at(SCRATCH_RVA),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        checkpoint_angles(r)
    }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
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
    let kept = regions();
    let saved = capture(&kept);
    for checks in [12u32, 2, 1] {
        let line = TestLine::new(30, checks, &|k| (k * 11 + 4) % 30, &|_, f| f.fill(0x22));
        check_state(
            &mut t,
            &format!("{checks} checkpoints"),
            call_original,
            Regs::SENTINEL,
            &|| restore(&kept, &saved),
            &kept,
            &rust,
        );
        drop(line);
    }
    restore(&kept, &saved);
    t.finish()
}
