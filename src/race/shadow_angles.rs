//! `race::shadow_angles` (0x4581F3): the shadow's points ordered round the farthest one
//! (see `race/car_shadow.rs`): it swapped into the first record, each angle made relative
//! to its (plus 2 pi below 0) and times 65536, sorted by `sub_4582AC` (the game's quicksort
//! of 0x20-byte records, covered here; its recursion a list of returns).
//!
//! In:  nothing
//! Out: the registers as it leaves them (a `call` follows in `sub_4575C1`)
//!
//! Refuses (before anything) no points or more than 0x40. Shadow mode: by state.
//! Self-test: point records of the test's (angles round, repeated, below the first), the
//! farthest one at each place; all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::car_shadow::*;
use crate::race::frame_draw::all_regs;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::shadow_angles", rva(0x4581F3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { shadow_angles(r)? }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn regions() -> Vec<Region> {
    vec![
        (game_addr(RECORDS_RVA) as u32, 0x820),
        at(FIRST_ANGLE_RVA),
        at(rva(0x62400C)),
    ]
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut kept = regions.clone();
    kept.extend([at(COUNT_RVA), at(FARTHEST_RVA)]);
    let saved = capture(&kept);
    let base = game_addr(RECORDS_RVA) as u32;
    for (count, far, seed) in [
        (8u32, 0u32, 1u32),
        (8, 5, 7),
        (3, 2, 9),
        (1, 0, 3),
        (12, 11, 0x55),
    ] {
        check_state(
            &mut t,
            &format!("count {count} farthest {far} seed {seed:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(COUNT_RVA, count);
                game_write(FARTHEST_RVA, base + 0x20 * far);
                for k in 0..count {
                    let r = base + 0x20 * k;
                    let angle = ((k * 37 + seed * 11) % 13) as f32 * 0.5 - 3.0;
                    for (o, v) in [
                        (0u32, 0x1000 * k),
                        (4, 7),
                        (8, 0x2000 + k),
                        (0xC, angle.to_bits()),
                        (0x10, 0x55),
                        (0x14, 0x66),
                        (0x18, k),
                        (0x1C, seed),
                    ] {
                        ((r + o) as *mut u32).write_unaligned(v);
                    }
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
