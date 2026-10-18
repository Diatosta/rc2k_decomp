//! `race::draw_screen_tint` (0x455A11): a tinted quad over the view window: half-through blue-grey under `dword_71C3E0`, else the sun's glare by `dword_6365F4` (see `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (its caller goes on with calls)
//!
//! Callers: `sub_4559A6`. Shadow mode runs the original alone (it draws). Self-test: a copy
//! whose calls go to stand-ins, against the Rust version with the same: the switch, glare 0, in range and past it.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::face_stage::compare_plain;
use crate::race::frame_views::{draw_screen_tint, tinted_quad_part};
use crate::race::level_init::note;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_screen_tint", rva(0x455A11), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { draw_screen_tint(r, &mut tinted_quad_part) } {
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
            address: 0x455A11,
            len: 0xC2,
            entry: &[0x83, 0x3D, 0xE0, 0xC3, 0x71, 0x00, 0x00, 0x75],
            calls: &[0x74, 0xBC],
            jumps_out: &[],
        },
        &|c| {
            [(0x44B538usize, 1u32)]
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
    let regions: Vec<Region> = vec![g(0x623DE8, 8), g(0x623E10, 0x18), at(rva(0x6415C4))];
    let mut kept = regions.clone();
    kept.extend([
        at(rva(0x71C3E0)),
        at(rva(0x6365F4)),
        g(0x60A570, 0xC),
        g(0x62408C, 8),
    ]);
    let saved = capture(&kept);
    for (n, case) in [
        (1u32, 0u32),
        (0, 0),
        (0, 0x80),
        (0, 0x1234),
        (0, 0xFFFF_FFF0),
    ]
    .into_iter()
    .enumerate()
    {
        let line = compare_plain(
            &format!("case {n}: {case:x?}"),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x71C3E0), case.0);
                game_write(rva(0x6365F4), case.1);
                game_write(rva(0x60A570), [0x40u32, 0x1180, 0x22C0]);
                game_write(rva(0x6415C4), 0x55u32);
            },
            &regions,
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                draw_screen_tint(r, &mut |p, r| {
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
