//! `race::draw_own_car` (0x455AD3): the drawn view's car seen from inside drawn (depth cleared first when asked), then the marks' overlays (see `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (its caller goes on with calls)
//!
//! Refuses (before anything) the car without its block. Callers: `sub_4559A6`. Shadow mode runs the original alone (it draws). Self-test: a copy
//! whose calls go to stand-ins, against the Rust version with the same: a test block, inside or not, the depth switches.

use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::race::face_stage::compare_plain;
use crate::race::frame_views::{draw_own_car, own_car_part};
use crate::race::level_init::note;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_own_car", rva(0x455AD3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { draw_own_car(r, &mut own_car_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(rva(0x621480)) } == 0 {
        return t.skip("no race loaded");
    }
    let mut block = vec![0u8; 0x1000];
    let b = block.as_mut_ptr() as u32;
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x455AD3,
            len: 0x9D,
            entry: &[0x83, 0x3D, 0x80, 0x14, 0x62, 0x00, 0x00, 0x0F],
            calls: &[0x5A, 0x60, 0x65, 0x79, 0x88, 0x97],
            jumps_out: &[],
        },
        &|c| {
            [
                (0x44E9A0usize, 1u32),
                (0x44E41F, 2),
                (0x461669, 3),
                (0x44E77E, 4),
                (0x44EB72, 5),
                (0x45632B, 6),
            ]
            .iter()
            .find(|(a, _)| rva(*a) == c)
            .map(|(_, p)| crate::install::stand_ins::stand_in(*p))
        },
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let regions: Vec<Region> = vec![
        at(rva(0x60627C)),
        at(rva(0x62DAD4)),
        at(rva(0x606280)),
        at(rva(0x6240F8)),
    ];
    let mut kept = regions.clone();
    kept.extend([
        at(rva(0x63DB9C)),
        at(rva(0x63DBA8)),
        g(0x71B3B8, 0x20),
        at(rva(0x635014)),
    ]);
    let saved = capture(&kept);
    for (n, case) in [(0x10u32, 0u32, 0u32), (0x10, 1, 0), (0x10, 0, 1), (0, 1, 1)]
        .into_iter()
        .enumerate()
    {
        let line = compare_plain(
            &format!("case {n}: {case:x?}"),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x63DB9C), 0u32);
                game_write(rva(0x63DBA8), 2u32);
                game_write(rva(0x71B3B8 + 8), b);
                ((b + 0x6FC) as *mut u32).write_unaligned(case.0);
                ((b + 0x900) as *mut u32).write_unaligned(case.1);
                ((b + 0x240) as *mut u32).write_unaligned(0x77);
                game_write(rva(0x635014), case.2);
            },
            &regions,
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                draw_own_car(r, &mut |p, r| {
                    note(p, r);
                    Ok(())
                })
            },
        );
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    restore(&kept, &saved);
    drop(block);
    t.finish()
}
