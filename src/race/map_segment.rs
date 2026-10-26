//! `race::map_segment` (0x446036): a stretch of the best line, from one map anchor to the
//! next, placed on the map.
//!
//! In:  nothing (the anchors in `dword_6236C4`, `dword_6236C0`)
//! Out: see `race::best_line::map_segment`
//!
//! See `race/best_line.rs`. Callers: `race::map_checkpoints` (inside `pusha`) and the code
//! at 0x44601B (the whole line as one stretch). Refuses anchors past the lists. Self-test:
//! faces and checkpoints of the test's, the game's put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::best_line::*;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::map_segment", rva(0x446036), thunk, &ORIGINAL).self_test(self_test);

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
        crate::math::turn_scratch_xz::scratch(),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        map_segment(r)
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

pub fn anchors(n: usize, f: &mut [u8]) {
    f.fill(0x66);
    let place = if n.is_multiple_of(4) {
        [(n * 3) as u16, (n * 5 + 1) as u16]
    } else {
        [0, 0]
    };
    f[0x2C..0x2E].copy_from_slice(&place[0].to_le_bytes());
    f[0x2E..0x30].copy_from_slice(&place[1].to_le_bytes());
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let kept = regions();
    let saved = capture(&kept);
    let line = TestLine::new(40, 20, &|k| k * 2, &anchors);
    for (a, b) in [(0u32, 2u32), (2, 6), (0, 19), (7, 3)] {
        check_state(
            &mut t,
            &format!("from {a} to {b}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(MAP_WORK_RVA + 0xC, a);
                game_write(MAP_WORK_RVA + 8, b);
            },
            &kept,
            &rust,
        );
    }
    drop(line);
    restore(&kept, &saved);
    t.finish()
}
