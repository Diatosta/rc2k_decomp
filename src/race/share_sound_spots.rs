//! `race::share_sound_spots` (0x45C7B9): `dword_5165EC` = `dword_6D1DC4`, and that many
//! 0x14-byte spots copied from 0x6D1AF4 to 0x5165F0, where the sound side
//! (`sub_40E642`) reads them.
//!
//! In:  nothing
//! Out: ecx 0, esi past the spots (0x6D1AF4 with none), edi past the copy (a `call`
//!      follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_45429B` (0x454742), each frame drawn. What a spot
//! is (made by `sub_45C6ED`) is a reading. Refuses (before anything) more than 36 (both
//! tables hold 36: 0x6D1AF4 runs to the count, 0x5165F0 to `unk_5168C0`). Shadow mode: by
//! state. Self-test: the game's tables with a pattern, counts 0, 1, 17, 36; all put back;
//! 37 refused.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::share_sound_spots", rva(0x45C7B9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    vec![
        at(SHARED_COUNT_RVA),
        (
            game_addr(SHARED_SPOTS_RVA) as u32,
            0x14 * SPOTS_MAX as usize,
        ),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { share_sound_spots() }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut kept = regions.clone();
    kept.extend([
        at(SPOT_COUNT_RVA),
        (game_addr(SPOTS_RVA) as u32, 0x14 * SPOTS_MAX as usize),
    ]);
    let saved = capture(&kept);
    for n in [0u32, 1, 17, 36] {
        check_state(
            &mut t,
            &format!("{n} spots"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(SPOT_COUNT_RVA, n);
                let from = game_addr(SPOTS_RVA) as *mut u8;
                for k in 0..0x14 * SPOTS_MAX as usize {
                    from.add(k).write(k as u8 ^ 0x5A);
                }
                std::ptr::write_bytes(
                    game_addr(SHARED_SPOTS_RVA) as *mut u8,
                    0xAA,
                    0x14 * SPOTS_MAX as usize,
                );
            },
            &regions,
            &rust,
        );
    }
    let refused = unsafe {
        game_write(SPOT_COUNT_RVA, 37u32);
        share_sound_spots().is_err()
    };
    restore(&kept, &saved);
    t.check(refused, || "37 spots not refused".into());
    t.finish()
}
