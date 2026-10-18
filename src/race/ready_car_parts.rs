//! `race::ready_car_parts` (0x460D5C): each race car with its parts shown: its parts'
//! matrices made (`race::car_part_matrices`) and kept in its block (see
//! `race/car_parts.rs`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (its caller goes on with calls)
//!
//! Refuses (before anything) no race cars (the original's loop would run away), more than
//! eight, or one without its block. Caller: `sub_45429B` (0x454429). Shadow mode runs the
//! original alone. Self-test: a copy whose call goes to a stand-in, against the Rust version
//! with the same: the game's cars, parts shown or not; all put back (skipped without a
//! race).

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva, shadow};
use crate::race::car_parts::{parts_part, ready_car_parts};
use crate::race::face_stage::compare_plain;
use crate::race::level_init::note;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::ready_car_parts", rva(0x460D5C), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { ready_car_parts(r, &mut parts_part) } {
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
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x460D5C,
            len: 0x120,
            entry: &[0xB9, 0x00, 0x00, 0x00, 0x00, 0x8B, 0x34, 0x8D],
            calls: &[0x1A, 0xED],
            jumps_out: &[],
        },
        &|c| (c == rva(0x460E7C)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let mut regions: Vec<Region> = vec![g(0x5EA3CC, 0x40), g(0x5E9F8C, 0x538)];
    for &b in &blocks {
        regions.push((b + 0x25C, 4));
        regions.push((b + 0x740, 0x1C0));
    }
    let saved = capture(&regions);
    for pattern in [0u32, 0xFF, 0x5A] {
        let line = compare_plain(
            &format!("shown {pattern:#x}"),
            &|| unsafe {
                restore(&regions, &saved);
                for (n, &b) in blocks.iter().enumerate() {
                    ((b + 0x25C) as *mut u32).write_unaligned((pattern >> n) & 1);
                }
            },
            &regions,
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                ready_car_parts(r, &mut |p, r| {
                    note(p, r);
                    Ok(())
                })
            },
        );
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    restore(&regions, &saved);
    t.finish()
}
