//! `race::count_checks` (0x445AAB): for each checkpoint, how many up to it are checks
//! (their face's byte +0x26 neither 0 nor 1).
//!
//! In:  nothing
//! Out: with two checkpoints or more, ebp the last count, ebx the checkpoint count, esi
//!      the last face; the rest preserved
//!
//! See `race/best_line.rs`. One caller, `sub_445772` ("checks"). Refuses more checkpoints
//! than the lists hold. Self-test: faces and checkpoints of the test's, the game's put
//! back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva};
use crate::race::best_line::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::count_checks", rva(0x445AAB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn counts(n: u32) -> Region {
    (
        game_addr(CHECKS_RVA) as u32,
        4 * n.min(MOST_CHECKS) as usize,
    )
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        count_checks(r)
    }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let n = unsafe { game_read::<u32>(CHECK_COUNT_RVA) };
    dispatch_state(&HOOK, r, call_original, &[counts(n)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let kept = [counts(40)];
    let saved = capture(&kept);
    for checks in [0u32, 1, 2, 9, 40] {
        let line = TestLine::new(48, checks, &|k| (k * 5 + 1) % 48, &|n, f| {
            f.fill(0x33);
            f[0x26] = (n % 4) as u8;
        });
        let regions = [line.faces.region(), counts(40)];
        check_state(
            &mut t,
            &format!("{checks} checkpoints"),
            call_original,
            Regs::SENTINEL,
            &|| restore(&kept, &saved),
            &regions,
            &rust,
        );
        drop(line);
    }
    restore(&kept, &saved);
    t.finish()
}
