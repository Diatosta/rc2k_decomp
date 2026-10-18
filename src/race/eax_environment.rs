//! `race::eax_environment` (0x45C7EE): the EAX sound environment's sum: `g_eaxEnvironment`
//! 0.0 and `dword_6D1AF0` 0, then every face of the course records in the 3 x 3 grid cells
//! around the middle (from `race::hold_to_limit(dword_60A87C)` - 1 on each axis; numbers
//! below 0x0FFFFFFF) weighed by `race::eax_face_weight` (`sub_45C84C` per cell, covered
//! here: its only caller is this function).
//!
//! In:  nothing
//! Out: eax the half, ecx the last cell, ebx and esi as the last cell left them (a `call`
//!      follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_4557AB` (0x45581C), each view drawn. Shadow mode:
//! by state. Self-test: test records, faces and points (pointers set), the grid's middle
//! cells set (marked, empty, one face, several), halves 4 and 16; all put back (skipped
//! while the root tables are empty).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::eax_face_weight::{WeightStage, weight_regions};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::eax_environment", rva(0x45C7EE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { eax_environment(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &weight_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let grid = game_addr(GRID_RVA) as u32;
    let extra: Vec<Region> = vec![
        (grid, 0x1000),
        at(rva(0x5CCCB8)),
        at(rva(0x60A87C)),
        at(rva(0x71C6D4)),
    ];
    let Some(stage) = WeightStage::new(&extra) else {
        return t.skip("the root tables are empty");
    };
    // Records: +8 first face, +0xC last.
    let mut records = vec![0u32; 8 * 6];
    for (k, (first, last)) in [(0u32, 11u32), (u32::MAX, 3), (4, 4), (7, 2), (2, 9), (5, 6)]
        .into_iter()
        .enumerate()
    {
        records[8 * k + 2] = first;
        records[8 * k + 3] = last;
    }
    let regions = weight_regions();
    for half in [4u32, 16] {
        check_state(
            &mut t,
            &format!("half {half}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                stage.prepare((0x100, 0x200));
                game_write(rva(0x5CCCB8), records.as_ptr() as u32);
                game_write(rva(0x60A87C), half);
                game_write(rva(0x71C6D4), 16u32);
                game_write(rva(0x6D1DD4), 7.0f32.to_bits());
                for k in 0..0x400u32 {
                    ((grid + 4 * k) as *mut u32).write_unaligned(0x0FFF_FFFF);
                }
                let middle = (half - 1) * 0x21;
                for (n, step) in [0u32, 1, 2, 32, 33, 34, 64, 65, 66].into_iter().enumerate() {
                    let v = if n == 4 { 0x8000_0001 } else { n as u32 % 6 };
                    ((grid + 4 * (middle + step)) as *mut u32).write_unaligned(v);
                }
            },
            &regions,
            &rust,
        );
    }
    stage.end();
    drop(records);
    t.finish()
}
