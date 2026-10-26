//! `race::replay_cameras_2` (0x445CF1): each checkpoint's nearest replay camera of the set
//! "replaycam2" (faces of kind 0x38).
//!
//! In:  nothing
//! Out: see `race::best_line::replay_cameras`
//!
//! See `race/best_line.rs`. Callers: `sub_445772` ("replaycam2") and `sub_459BCA`. Refuses
//! more checkpoints than the lists hold, no faces or 0x100000 or more. Self-test: faces and
//! checkpoints of the test's, the game's put back.

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
    RawHook::new("race::replay_cameras_2", rva(0x445CF1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SET: Cameras = CAMERAS[1];

fn regions() -> Vec<Region> {
    vec![
        (game_addr(SET.list) as u32, 0x1000),
        at(SET.count),
        (game_addr(SET.nearest) as u32, 4 * MOST_CHECKS as usize),
        (game_addr(rva(0x623658)) as u32, 8),
        (game_addr(rva(0x623668)) as u32, 8),
        at(SCRATCH_RVA),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        replay_cameras(r, &SET)
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
    for (checks, every) in [(10u32, 3usize), (2, 7), (10, 1000), (1, 3)] {
        let line = TestLine::new(40, checks, &|k| (k * 3 + 1) % 40, &|n, f| {
            f.fill(0x55);
            if n % every == 2 {
                f[0] = 0xFF;
                f[1] = SET.kind;
            }
        });
        let mut regions = kept.clone();
        regions.push(line.faces.region());
        check_state(
            &mut t,
            &format!("{checks} checkpoints, a camera in {every}"),
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
