//! `race::panel_marks` (0x45AEFE): the eight dwords at 0x6D14A2 .. 0x6D14C2 (not
//! 0x6D14B2) 0; for each view block, `dword_6D14B2` / `dword_6D14C6` = 400 + (+0xFA0),
//! and its first +0xF98 (at most 4) values at +0xF78, each + 1, at 0x6D14A2 / 0x6D14B6.
//!
//! In:  nothing
//! Out: esi the second block, ebp its count (held to 4), ecx the last count a loop ran
//!      with, eax the last value written (a `call` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_45AB36` (0x45AB4A), each frame drawn. The values
//! (marks on the panel's bar: a reading) are not settled. Shadow mode: by state. Self-test:
//! the game's view blocks, counts 0, 1, 4, 5, 0xFFFFFFFF; all put back (skipped without
//! view blocks).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::race::frame::VIEW_BLOCKS;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::panel_marks", rva(0x45AEFE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn view_blocks() -> Vec<u32> {
    VIEW_BLOCKS
        .iter()
        .map(|&a| unsafe { game_read::<u32>(a) })
        .collect()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { panel_marks() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &panel_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let views = view_blocks();
    if views.contains(&0) {
        return t.skip("no view blocks (no race loaded)");
    }
    let regions = panel_regions();
    let mut kept = regions.clone();
    kept.extend(views.iter().map(|&b| (b + 0xF78, 0x2C)));
    let saved = capture(&kept);
    for (first, second) in [(0u32, 0u32), (1, 4), (5, 0), (0xFFFF_FFFF, 2), (3, 3)] {
        check_state(
            &mut t,
            &format!("counts {first:#x} {second:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                let w = |a: u32, v: u32| (a as *mut u32).write_unaligned(v);
                for &(a, len) in &regions {
                    std::ptr::write_bytes(a as *mut u8, 0xAA, len);
                }
                for (n, (&b, count)) in views.iter().zip([first, second]).enumerate() {
                    for k in 0..8u32 {
                        w(b + 0xF78 + 4 * k, 0x10 * n as u32 + k);
                    }
                    w(b + 0xF98, count);
                    w(b + 0xFA0, 0x77 + n as u32);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
