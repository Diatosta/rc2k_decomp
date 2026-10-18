//! `race::scroll_maps` (0x45B4E6): `race::scroll_map` on each view's car, loaded into and
//! stored from the work copy; see `race/map_scroll.rs`.
//!
//! In:  nothing
//! Out: as the last store leaves them
//!
//! Refuses (before anything) a view car without its block or no course faces or points.
//! Caller: `sub_45429B` (0x45442E). Shadow mode runs the original alone (the redraws write
//! pictures). Self-test: a copy whose scrolls go to a stand-in (the loads and stores real),
//! against the Rust version with the same: one view, two, the third view car; skipped with
//! a null block.

use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::race::face_stage::compare_plain;
use crate::race::level_init::note;
use crate::race::map_scroll::{scroll_maps, scroll_part};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::scroll_maps", rva(0x45B4E6), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { scroll_maps(r, &mut scroll_part) } {
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
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x45B4E6,
            len: 0x41,
            entry: &[0x83, 0x3D, 0x10, 0xA9, 0x52, 0x00, 0x00, 0x74],
            calls: &[0x09, 0x0E, 0x13, 0x19, 0x1E, 0x23, 0x31, 0x36, 0x3B],
            jumps_out: &[],
        },
        &|c| (c == rva(0x45B527)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let mut regions: Vec<Region> =
        vec![(game_addr(rva(0x719FD8)) as u32, 0xFE0), at(rva(0x71B3F8))];
    for &b in &blocks {
        regions.push((b, 0xFE0));
    }
    let mut kept = regions.clone();
    kept.extend([
        at(rva(0x52A910)),
        at(rva(0x63DB98)),
        at(rva(0x63DBA8)),
        at(rva(0x63DBAC)),
        at(rva(0x63DBA0)),
    ]);
    let saved = capture(&kept);
    for (one, players, cars) in [
        (0u32, 1u32, (2u32, 5u32, 7u32)),
        (0, 2, (1, 6, 3)),
        (1, 2, (0, 4, 5)),
    ] {
        let line = compare_plain(
            &format!("one {one} players {players} cars {cars:?}"),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x52A910), one);
                game_write(rva(0x63DB98), players);
                game_write(rva(0x63DBA8), cars.0);
                game_write(rva(0x63DBAC), cars.1);
                game_write(rva(0x63DBA0), cars.2);
            },
            &regions,
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                scroll_maps(r, &mut |p, r| {
                    note(p, r);
                    Ok(())
                })
            },
        );
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    restore(&kept, &saved);
    t.finish()
}
