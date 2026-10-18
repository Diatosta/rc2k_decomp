//! `race::fetch_view_value` (0x45550E): `dword_6415D8` = the drawn view's block (`off_63DBB0[dword_63DB9C]`) +0xFBC.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! See `race/frame_draw.rs`. Callers: `sub_45429B` (five) and `sub_455540`. What the value is is not read.
//! Refuses (before anything) a view index past the two (`off_63DBB0` has two). Shadow
//! mode: by state. Self-test: the game's view blocks, views 0 and 1, values; all put back
//! (skipped without view blocks); index 2 refused.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::race::frame::VIEW_BLOCKS;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::fetch_view_value", rva(0x45550E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { fetch_view_value() }
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions: Vec<Region> = vec![at(VIEW_OUT_RVA)];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let blocks: Vec<u32> = VIEW_BLOCKS
        .iter()
        .map(|&a| unsafe { game_read::<u32>(a) })
        .collect();
    if blocks.contains(&0) {
        return t.skip("no view blocks (no race loaded)");
    }
    let mut kept = vec![at(VIEW_RVA), at(VIEW_IN_RVA), at(VIEW_OUT_RVA)];
    kept.extend(blocks.iter().map(|&b| (b + 0xFBC, 4)));
    let saved = capture(&kept);
    for (view, value) in [(0u32, 0x1234_5678u32), (1, 0), (1, 0xFFFF_FFFF)] {
        let regions: Vec<Region> = vec![at(VIEW_OUT_RVA)];
        check_state(
            &mut t,
            &format!("view {view} value {value:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(VIEW_RVA, view);
                game_write(VIEW_IN_RVA, value);
                game_write(VIEW_OUT_RVA, 0xAAAA_AAAAu32);
                for &b in &blocks {
                    ((b + 0xFBC) as *mut u32).write_unaligned(value ^ 0x5A5A);
                }
                ((blocks[view as usize] + 0xFBC) as *mut u32).write_unaligned(value);
            },
            &regions,
            &rust,
        );
    }
    let refused = unsafe {
        game_write(VIEW_RVA, 2u32);
        fetch_view_value().is_err()
    };
    restore(&kept, &saved);
    t.check(refused, || "view 2 not refused".into());
    t.finish()
}
