//! `race::clear_view_depth` (0x44E9A0): with the device on, the clip rectangle the whole screen and the depth cleared there (`sub_44E9EE`, the platform's) (see `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (its caller goes on with calls)
//!
//! Callers: `race::draw_own_car`. Shadow mode runs the original alone (it clears the depth buffer). Self-test: a copy
//! whose calls go to stand-ins, against the Rust version with the same: the device on and off.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::face_stage::compare_plain;
use crate::race::frame_views::{clear_view_depth, own_car_clear_part};
use crate::race::level_init::note;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::clear_view_depth", rva(0x44E9A0), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { clear_view_depth(r, &mut own_car_clear_part) } {
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
            address: 0x44E9A0,
            len: 0x39,
            entry: &[0x83, 0x3D, 0x8C, 0xDA, 0x5C, 0x00, 0x00, 0x74],
            calls: &[0x33],
            jumps_out: &[],
        },
        &|c| {
            [(0x44E9EEusize, 1u32)]
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
    let regions: Vec<Region> = vec![g(0x624138, 0x10)];
    let mut kept = regions.clone();
    kept.extend([at(rva(0x5CDA8C))]);
    let saved = capture(&kept);
    for (n, case) in [0u32, 1].into_iter().enumerate() {
        let line = compare_plain(
            &format!("case {n}: {case:x?}"),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x5CDA8C), case);
            },
            &regions,
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                clear_view_depth(r, &mut |p, r| {
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
