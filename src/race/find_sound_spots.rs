//! `race::find_sound_spots` (0x45C605): `g_soundSpotCount` 0 and the 5 x 5 cells around the view's grid place by `race::cell_sound_spots` (see `race/sound_spots.rs`).
//!
//! In:  nothing
//! Out: ecx the last cell, ebx / esi as the last cell leaves them
//!
//! Caller: `sub_4547AE` (0x45473D). Refuses (before anything) a cell past the grid, or no records, faces or points when a cell has a record. Shadow mode: by state. Self-test: a test course
//! (`SpotStage`: faces, records, points, the grid's cells), reaches 4; all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::frame_draw::all_regs;
use crate::race::sound_spots::*;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::find_sound_spots", rva(0x45C605), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { find_sound_spots(r)? }))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = SpotStage::live_regions();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let stage = SpotStage::new();
    let regions = stage.regions();
    for (n, count) in [(0u32, 0u32), (0, 7)] {
        check_state(
            &mut t,
            &format!("run {n:#x} count {count}"),
            call_original,
            Regs {
                eax: n,
                ..Regs::SENTINEL
            },
            &|| stage.prepare(4, count),
            &regions,
            &rust,
        );
    }
    stage.end();
    t.finish()
}
