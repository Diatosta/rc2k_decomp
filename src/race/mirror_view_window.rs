//! `race::mirror_view_window` (0x455F17): the rear-view mirror's window (place by view and
//! table, 0xB3 x 0x40 on 800 x 600 scaled to the screen), `scene::set_view_window`,
//! `race::set_up_fog` (see `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: as `set_up_fog` leaves them (its caller in `sub_4547AE` goes on with calls)
//!
//! Refuses (before anything) a place whose scaling would fault (the original's `div`).
//! Shadow mode runs the original alone (the fog tells the device). Self-test: a copy whose
//! fog call goes to a stand-in (the window real), against the Rust version with the same:
//! each view, the switch `dword_6D1502`, screen sizes; all put back.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::face_stage::compare_plain;
use crate::race::frame_views::{mirror_part, mirror_view_window};
use crate::race::level_init::note;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::mirror_view_window", rva(0x455F17), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { mirror_view_window(r, &mut mirror_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x455F17,
            len: 0xED,
            entry: &[0x8B, 0x0D, 0xA4, 0x38, 0x62, 0x00, 0x03, 0x0D],
            calls: &[0xCE, 0xD3],
            jumps_out: &[],
        },
        &|c| (c == rva(0x44FBD9)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let mut regions = crate::scene::set_view_window::regions().to_vec();
    for a in [
        0x62408Cusize,
        0x624090,
        0x623FB0,
        0x623FB4,
        0x624080,
        0x6365C4,
        0x63993C,
        0x624138,
        0x62413C,
        0x624140,
        0x624144,
    ] {
        regions.push(at(rva(a)));
    }
    let mut kept: Vec<Region> = regions.clone();
    kept.extend([
        at(rva(0x4F494C)),
        at(rva(0x4F4950)),
        at(rva(0x6238A4)),
        at(rva(0x63DB9C)),
        (game_addr(rva(0x6D1502)) as u32, 4),
    ]);
    kept.extend(crate::scene::set_view_window::inputs());
    let saved = capture(&kept);
    for (view, switch, size) in [
        (0u32, 1u32, (800u32, 600u32)),
        (1, 0, (640, 480)),
        (2, 1, (1024, 768)),
        (0, 0, (1280, 1024)),
    ] {
        let line = compare_plain(
            &format!("view {view} switch {switch} size {size:?}"),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x63DB9C), view);
                game_write(rva(0x6238A4), 0u32);
                ((game_addr(rva(0x6D1502))) as *mut u32).write_unaligned(switch);
                game_write(rva(0x4F494C), size.0);
                game_write(rva(0x4F4950), size.1);
            },
            &regions,
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                mirror_view_window(r, &mut |p, r| {
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
