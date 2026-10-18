//! `race::draw_car_entry` (0x455105): car ebx & 7's place turned by the view; drawn
//! (`race::draw_car_shined`) unless behind or aside, each counted (see
//! `race/car_draw.rs`).
//!
//! In:  ebx = the car (low three bits)
//! Out: ebx the car, esi its block, eax as it leaves it, then as the drawing leaves them
//!      (its caller `race::draw_list_entry` keeps all)
//!
//! Refuses (before anything) a car without its block. Shadow mode runs the original alone
//! (it draws). Self-test: a copy whose drawing call goes to a stand-in, against the Rust
//! version with the same: an identity view, cars placed ahead, behind and aside; all put
//! back (skipped with a null block).

use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::race::car_draw::*;
use crate::race::face_stage::{SEEN_RVA, compare_plain, seen_handler};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_car_entry", rva(0x455105), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { draw_car_entry(r, &mut entry_car_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let blocks: Vec<u32> = (0..8)
        .map(|n| unsafe { game_read::<u32>(rva(0x71B3B8) + 4 * n) })
        .collect();
    if blocks.contains(&0) {
        return t.skip("a car block is null (no race loaded)");
    }
    SEEN_RVA.store(rva(0x60627C), Ordering::SeqCst);
    crate::install::stand_ins::set_handler(seen_handler);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x455105,
            len: 0x84,
            entry: &[0x83, 0xE3, 0x07, 0x89, 0x1D, 0x7C, 0x62, 0x60],
            calls: &[0x34, 0x77],
            jumps_out: &[],
        },
        &|c| (c == rva(0x461669)).then(|| crate::install::stand_ins::stand_in(1)),
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
        g(0x64159C, 0xC),
        at(rva(0x641594)),
        at(rva(0x641598)),
    ];
    let mut kept = regions.clone();
    kept.push(g(0x5EA2CC, 0x40));
    for &b in &blocks {
        kept.push((b + 0x134, 0xC));
    }
    let saved = capture(&kept);
    for (ebx, place) in [
        (0u32, [0i32, 0, 0x10_0000]),
        (9, [0x20_0000, 0x1000, 0x10_0000]),
        (3, [-0x5_0000, 0, -0x5_0000]),
        (7, [-0x8_0000, 0, 0x1_0000]),
        (5, [0x30_0000, -0x2000, 0x40_0000]),
    ] {
        let block = blocks[(ebx & 7) as usize];
        let line = compare_plain(
            &format!("car {ebx} place {place:x?}"),
            &|| unsafe {
                restore(&kept, &saved);
                let m: [f32; 16] = [
                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
                ];
                game_write(rva(0x5EA2CC), m.map(f32::to_bits));
                for (k, p) in place.iter().enumerate() {
                    ((block + 0x134 + 4 * k as u32) as *mut i32).write_unaligned(*p);
                }
            },
            &regions,
            copy.addr(),
            Regs {
                ebx,
                ..Regs::SENTINEL
            },
            &mut |r| unsafe {
                draw_car_entry(r, &mut |p, r| {
                    seen_handler(p, r);
                    Ok(())
                })
            },
        );
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    restore(&kept, &saved);
    SEEN_RVA.store(0, Ordering::SeqCst);
    t.finish()
}
