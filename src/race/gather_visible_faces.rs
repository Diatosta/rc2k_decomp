//! `race::gather_visible_faces` (0x454B71): `race::gather_record_faces` of each course
//! record in `g_courseGrid` (numbers below 0x0FFFFFFF: the far cells' marked ones are left
//! out), then of the last record once more.
//!
//! In:  nothing
//! Out: ecx 0x400, ebp the last record's number; the rest kept (each gathering is inside
//!      `pusha`; a `call` follows)
//!
//! Its test for the last record having been gathered sets ebp inside the `pusha`, so it is
//! lost and the last record is always gathered again (`notes/bugs.md`). See
//! `race/frame_draw.rs`. Caller: `sub_4557AB` (0x455817), each view drawn. Refuses (before
//! anything) what `gather_record_faces` would for any of the records. Shadow mode: by
//! state. Self-test: the grid set to a few of the game's records (and marked cells), the
//! lists' counts set; all put back (skipped without a stage).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::race::gather_record_faces::stage_there;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::gather_visible_faces",
    rva(0x454B71),
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

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { gather_visible_faces(r)? }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &gather_regions(&visible_records()),
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !stage_there() || unsafe { game_read::<u32>(rva(0x5CCD18)) } < 8 {
        return t.skip("no stage loaded");
    }
    let grid = game_addr(GRID_RVA) as u32;
    let grid_region: [Region; 1] = [(grid, 0x1000)];
    let grid_saved = capture(&grid_region);
    for (picks, counts) in [
        (&[0u32, 3, 5][..], 0x10u32),
        (&[1, 0x8000_0002, 7][..], 0),
        (&[][..], 0),
    ] {
        unsafe {
            for k in 0..0x400u32 {
                ((grid + 4 * k) as *mut u32).write_unaligned(0x0FFF_FFFF);
            }
            for (k, &p) in picks.iter().enumerate() {
                ((grid + 4 * (k as u32 * 37 % 0x400)) as *mut u32).write_unaligned(p);
            }
        }
        let regions = gather_regions(&visible_records());
        let saved = capture(&regions);
        check_state(
            &mut t,
            &format!("records {picks:x?} counts {counts:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &saved);
                game_write(SORT_COUNT_RVA, counts);
                game_write(DEPTH_COUNT_RVA, counts);
            },
            &regions,
            &rust,
        );
        restore(&regions, &saved);
    }
    restore(&grid_region, &grid_saved);
    t.finish()
}
