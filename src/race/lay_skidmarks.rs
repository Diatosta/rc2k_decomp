//! `race::lay_skidmarks` (0x4588DE): each race car's skidding wheels, their places turned
//! by the car's matrix, given skid points (`race::add_skid_point`) or their slots cleared;
//! see `race/skid_laying.rs`.
//!
//! In:  nothing
//! Out: eax the last car's bit, ecx the count (the rest kept)
//!
//! Refuses (before anything) a car without its block or more than eight. Caller:
//! `race::lay_skidmarks_if_on`. Shadow mode runs the original alone. Self-test: a copy whose point calls go to a stand-in noting the
//! place (the turns real), against the Rust version with the same: the game's cars with
//! their wheels' skids, grips and switches varied; all put back (skipped without a race).

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva, shadow};
use crate::race::face_stage::{SEEN_RVA, compare_plain, seen_handler};
use crate::race::skid_laying::{lay_skidmarks, skid_point_part};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::lay_skidmarks", rva(0x4588DE), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { lay_skidmarks(r, &mut skid_point_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let count: u32 = unsafe { game_read(rva(0x63DB90)) };
    let blocks: Vec<u32> = (0..count.min(8))
        .map(|n| unsafe { game_read::<u32>(rva(0x71B3B8) + 4 * n as usize) })
        .collect();
    if count == 0 || count > 8 || blocks.contains(&0) {
        return t.skip("no race loaded");
    }
    SEEN_RVA.store(rva(0x643AD8), Ordering::SeqCst);
    crate::install::stand_ins::set_handler(seen_handler);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4588DE,
            len: 0x325,
            entry: &[0x83, 0x3D, 0x90, 0xDB, 0x63, 0x00, 0x00, 0x0F],
            calls: &[0x9B, 0xD4, 0x157, 0x190, 0x213, 0x24C, 0x2CF, 0x308],
            jumps_out: &[],
        },
        &|c| (c == rva(0x45860C)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let mut regions: Vec<Region> = vec![g(0x643AD8, 0xC), at(rva(0x62400C)), at(rva(0x645B14))];
    for &b in &blocks {
        regions.push((b + 0xF48, 0x10));
    }
    let mut kept = regions.clone();
    for &b in &blocks {
        kept.extend([(b + 0x298, 4), (b + 0x600, 0x10), (b + 0xF40, 8)]);
    }
    let saved = capture(&kept);
    for seed in [0u32, 1, 2] {
        let line = compare_plain(
            &format!("seed {seed}"),
            &|| unsafe {
                restore(&kept, &saved);
                for (n, &b) in blocks.iter().enumerate() {
                    let s = seed.wrapping_mul(7).wrapping_add(n as u32);
                    ((b + 0x298) as *mut u32).write_unaligned(0x1E0 ^ (s << 5) & 0x1E0);
                    for wheel in 0..4u32 {
                        ((b + 0x600 + 4 * wheel) as *mut u32)
                            .write_unaligned(u32::from((s + wheel) % 3 != 0));
                    }
                    ((b + 0xF40) as *mut f32).write_unaligned(if s % 2 == 0 {
                        100.0
                    } else {
                        -1.0
                    });
                    ((b + 0xF44) as *mut f32).write_unaligned(50.0 * s as f32);
                }
            },
            &regions,
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                lay_skidmarks(r, &mut |p, r| {
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
