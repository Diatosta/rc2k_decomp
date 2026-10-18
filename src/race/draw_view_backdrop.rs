//! `race::draw_view_backdrop` (0x4651E6): a quad over the view window (one pixel wider),
//! texture `dword_5D1DE8` at one place, drawn by the device's unclipped fan with the glow's
//! render states; the depth buffer on after (see `race/frame_flush.rs`).
//!
//! In:  nothing
//! Out: as the depth buffer's call leaves them
//!
//! The render states (`sub_44E44D`) and the fan (`d3d::draw_fan_unclipped`) are the
//! platform's (section 15). Caller: `sub_454806`. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose four calls go to stand-ins, against the Rust version with
//! the same: view windows.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::face_stage::compare_plain;
use crate::race::frame_flush::{backdrop_part, draw_view_backdrop};
use crate::race::level_init::note;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_view_backdrop", rva(0x4651E6), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { draw_view_backdrop(r, &mut backdrop_part) } {
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
            address: 0x4651E6,
            len: 0x19B,
            entry: &[0xC7, 0x05, 0xF0, 0x3D, 0x62, 0x00, 0x00, 0x00],
            calls: &[0x181, 0x18B, 0x190, 0x195],
            jumps_out: &[],
        },
        &|c| {
            [
                (0x44E44Dusize, 1u32),
                (0x44E77E, 2),
                (0x41AD9C, 3),
                (0x44E41F, 4),
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
    let regions: Vec<Region> = vec![g(0x9605A0, 0x80), g(0x623DF0, 0x38), at(rva(0x6240F8))];
    let mut kept = regions.clone();
    kept.extend([g(0x623FB0, 8), g(0x62408C, 8), at(rva(0x5D1DE8))]);
    let saved = capture(&kept);
    for (place, size) in [
        ((0u32, 0u32), (640u32, 480u32)),
        ((120, 34), (179, 64)),
        ((u32::MAX, 5), (0x7FFF_FFFF, 1)),
    ] {
        let line = compare_plain(
            &format!("place {place:?} size {size:?}"),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x623FB0), [place.0, place.1]);
                game_write(rva(0x62408C), [size.0, size.1]);
            },
            &regions,
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                draw_view_backdrop(r, &mut |p, r| {
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
