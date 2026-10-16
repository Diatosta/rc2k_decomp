//! `race::checkpoint_distances` (0x445AF1): the distance along the best line at each
//! checkpoint, and the whole.
//!
//! In:  ecx (only with fewer than two checkpoints: where the 0 goes)
//! Out: edi the length, ecx where it was put; with two checkpoints or more, eax, ebx, esi
//!      the last point's z, number and address; the rest preserved
//!
//! See `race/best_line.rs`. Callers: `sub_445772` ("dists") and `sub_459BCA`. Refuses a
//! last checkpoint of 0 or past the lists, and with fewer than two checkpoints an ecx past
//! the list. Self-test: checkpoints of the test's, the game's put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::best_line::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::checkpoint_distances",
    rva(0x445AF1),
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

fn regions() -> Vec<Region> {
    vec![
        (game_addr(DISTANCES_RVA) as u32, 4 * MOST_CHECKS as usize),
        (game_addr(rva(0x623658)) as u32, 8),
        at(SCRATCH_RVA),
        at(LINE_LENGTH_RVA),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        checkpoint_distances(r)
    }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut kept = regions();
    kept.push(at(LAST_CHECK_RVA));
    let saved = capture(&kept);
    for (checks, ecx) in [(12u32, 0u32), (2, 0), (1, 5), (0, 0x40)] {
        let line = TestLine::new(30, checks, &|k| (k * 7 + 2) % 30, &|_, f| f.fill(0x44));
        check_state(
            &mut t,
            &format!("{checks} checkpoints, ecx {ecx:#x}"),
            call_original,
            Regs {
                ecx,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&kept, &saved);
                game_write(LAST_CHECK_RVA, checks.wrapping_sub(1));
            },
            &regions(),
            &rust,
        );
        drop(line);
    }
    restore(&kept, &saved);
    t.finish()
}
