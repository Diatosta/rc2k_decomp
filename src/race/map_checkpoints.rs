//! `race::map_checkpoints` (0x445FB4): the best line placed on the map, anchor to anchor.
//!
//! In:  nothing
//! Out: with two checkpoints or more, eax the last anchor, ecx the count, ebx and esi as
//!      `race::next_map_anchor` leaves them; the rest preserved
//!
//! See `race/best_line.rs`. Callers: `sub_445772` ("map") and `sub_459BCA`. Refuses more
//! checkpoints than the lists hold. Self-test: faces and checkpoints of the test's, the
//! game's put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::best_line::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::map_checkpoints", rva(0x445FB4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    vec![
        (game_addr(MAP_WORK_RVA) as u32, MAP_WORK_LEN),
        (game_addr(MAP_X_RVA) as u32, 4 * MOST_CHECKS as usize),
        (game_addr(MAP_Y_RVA) as u32, 4 * MOST_CHECKS as usize),
        at(SCRATCH_RVA),
        at(ANCHORS_RVA),
        crate::math::turn_scratch_xz::scratch(),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        map_checkpoints(r)
    }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let kept = regions();
    let saved = capture(&kept);
    for checks in [20u32, 2, 1] {
        let line = TestLine::new(40, checks, &|k| k * 2, &crate::race::map_segment::anchors);
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
