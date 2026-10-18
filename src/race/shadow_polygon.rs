//! `race::shadow_polygon` (0x45787F): the shadow's hull as scene points (after the scene's
//! own, from `g_scenePointCount` + 0x10) and as the polygon's work entries with a circle's
//! texture places (see `race/car_shadow.rs`; x87 the original's instructions).
//!
//! In:  nothing
//! Out: the registers as it leaves them (a `mov` follows, not of them)
//!
//! Refuses (before anything) a hull of no points or more than 0x40. Shadow mode: by state
//! (the work entries, the new scene points). Self-test: hulls of 3, 5 and 8 records of
//! the test's, scene points of the test's (`g_scenePoints` pointed at them); all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::car_shadow::*;
use crate::race::frame_draw::all_regs;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::shadow_polygon", rva(0x45787F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { shadow_polygon(r)? }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn regions() -> Vec<Region> {
    let points: u32 = unsafe { crate::framework::game_read(rva(0x5CCCC0)) };
    let scene: u32 = unsafe { crate::framework::game_read(rva(0x5CCD20)) };
    vec![
        (game_addr(rva(0x975510)) as u32, 0x44 * 0x40),
        (
            points.wrapping_add(scene.wrapping_mul(12)) + 0xC0,
            12 * 0x40,
        ),
        at(rva(0x62400C)),
    ]
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut points = vec![0u32; 3 * 0x80];
    let base = game_addr(RECORDS_RVA) as u32;
    let mut kept = vec![
        at(rva(0x5CCCC0)),
        at(rva(0x5CCD20)),
        at(HULL_COUNT_RVA),
        at(GROUND_RVA),
        (base, 0x20 * 8),
        (game_addr(rva(0x975510)) as u32, 0x44 * 0x40),
        at(rva(0x62400C)),
        at(rva(0x641994)),
        at(rva(0x641998)),
    ];
    kept.push(at(rva(0x609328)));
    let saved = capture(&kept);
    for n in [3u32, 5, 8] {
        let p = points.as_mut_ptr() as u32;
        let prepare = || unsafe {
            restore(&kept, &saved);
            game_write(rva(0x5CCCC0), p);
            game_write(rva(0x5CCD20), 4u32);
            game_write(HULL_COUNT_RVA, n);
            game_write(GROUND_RVA, 0x1234u32);
            game_write(rva(0x641994), 127.5f32.to_bits());
            game_write(rva(0x641998), 256.0f32.to_bits());
            game_write(rva(0x609328), 0x1FFu32);
            for k in 0..n {
                let r = base + 0x20 * k;
                ((r) as *mut u32).write_unaligned(0x1000 * k);
                ((r + 8) as *mut u32).write_unaligned(0x3000 - 0x100 * k);
            }
        };
        prepare();
        let regions = regions();
        check_state(
            &mut t,
            &format!("hull {n}"),
            call_original,
            Regs::SENTINEL,
            &prepare,
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    drop(points);
    t.finish()
}
