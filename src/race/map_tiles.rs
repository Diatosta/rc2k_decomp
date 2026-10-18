//! `race::map_tiles` (0x45B663): the map picture's slot and first tile (its corner >> 4)
//! for the tiles' blit (`race::map_tiles_stub`); see `race/map_scroll.rs`.
//!
//! In:  nothing (the work copy `g_car`)
//! Out: eax the slot, ecx / edx the tile, then as the blit leaves them
//!
//! Caller: `race::scroll_map`. Shadow mode runs the original alone (the blit writes a
//! picture). Self-test: a copy whose call goes to a stand-in, against the Rust version with
//! the same: corners, `dword_52A910` each way.

use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva, shadow};
use crate::race::face_stage::compare_plain;
use crate::race::level_init::note;
use crate::race::map_scroll::{map_tiles, tiles_part};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::map_tiles", rva(0x45B663), thunk, &ORIGINAL).self_test(self_test);

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
    let _ = unsafe { map_tiles(r, &mut tiles_part) };
}

/// Notes the place and the tile row (edx) in the note's eax.
fn handler(place: u32, r: &mut Regs) {
    note(place, r);
    let mut y = Regs { eax: r.edx, ..*r };
    note(9, &mut y);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(handler);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x45B663,
            len: 0x2C,
            entry: &[0x8B, 0x0D, 0xAC, 0xAA, 0x71, 0x00, 0xC1, 0xF9],
            calls: &[0x26],
            jumps_out: &[],
        },
        &|c| (c == rva(0x45B68F)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let kept: Vec<Region> = vec![
        at(rva(0x71AAAC)),
        at(rva(0x71AAB0)),
        at(rva(0x52A910)),
        at(rva(0x5D0E00)),
        at(rva(0x71B3F8)),
    ];
    let saved = capture(&kept);
    for (x, y, one) in [
        (0x40u32, 0x80u32, 0u32),
        (0xFFFF_FFC0, 0x10, 1),
        (0x123, 0x8000_0000, 0),
    ] {
        let line = compare_plain(
            &format!("corner {x:#x} {y:#x} one {one}"),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x71AAAC), x);
                game_write(rva(0x71AAB0), y);
                game_write(rva(0x52A910), one);
                game_write(rva(0x5D0E00), 0x40u32);
                game_write(rva(0x71B3F8), 3u32);
            },
            &[],
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                map_tiles(r, &mut |p, r| {
                    handler(p, r);
                    Ok(())
                })
            },
        );
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    restore(&kept, &saved);
    t.finish()
}
