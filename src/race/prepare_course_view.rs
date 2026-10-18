//! `race::prepare_course_view` (0x4548A8): `dword_63F17C` 0x10, `race::face_hide_mask`,
//! `race::course_grid`; with `dword_621480` set, `dword_63F17C` 0x10 again,
//! `race::checkpoint_colours` (with two checkpoints or more, `g_checkpointCount`) and
//! `race::mark_far_cells`.
//!
//! In:  nothing
//! Out: the registers as its parts leave them (a `call` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_4557AB` (0x455812), each view drawn. Refuses
//! (before anything) no course records, or what `probe_ground` refuses (it runs first in
//! the Rust version: nothing it reads is written before it). Shadow mode: by state (the
//! probe's regions, the mask, the grid and past it, the checkpoint colours, the far cells'
//! dwords). Self-test: on the game's state, `dword_621480` as it is and flipped; all put
//! back (skipped without a stage).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::race::face_hide_mask::mask_regions;
use crate::race::frame_draw::*;
use crate::race::gather_record_faces::stage_there;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::prepare_course_view", rva(0x4548A8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const HIDING: usize = rva(0x621480); // dword_621480

fn regions() -> Vec<Region> {
    let mut v = mask_regions();
    v.extend(far_regions());
    v.extend(
        [
            0x63E15Cusize,
            0x63E160,
            0x63E174,
            0x63E178,
            0x5CCBF0,
            0x5CCBF4,
            0x614D08,
            0x614D0C,
            0x614D10,
        ]
        .iter()
        .map(|&a| at(rva(a))),
    );
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { prepare_course_view(r)? }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !stage_there() || unsafe { game_read::<u32>(rva(0x5CCD18)) } == 0 {
        return t.skip("no stage loaded");
    }
    let regions = regions();
    let mut kept = regions.clone();
    kept.push(at(HIDING));
    let saved = capture(&kept);
    let hiding: u32 = unsafe { game_read(HIDING) };
    for h in [hiding, u32::from(hiding == 0)] {
        check_state(
            &mut t,
            &format!("hiding {h}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(HIDING, h);
                // The grid's corner forgotten, so it is made.
                game_write(rva(0x63E174), 0x7FFF_FFFFu32);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
