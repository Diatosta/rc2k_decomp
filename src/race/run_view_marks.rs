//! `race::run_view_marks` (0x456336): the drawn view's marks: which show, the layout, the car's turn, the cars shared, the records' first words, the screen centre for the layout around `marks::frame_marks` and `marks::run_frame` (see `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (its caller goes on with calls)
//!
//! Refuses (before anything) a null view block or car record, or no marks settings. Callers: `sub_4559A6`. Shadow mode runs the original alone (it runs the marks). Self-test: a copy
//! whose calls go to stand-ins, against the Rust version with the same: views, replay and the other-view switch.

use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::race::face_stage::compare_plain;
use crate::race::frame_views::{run_view_marks, view_marks_part};
use crate::race::level_init::note;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::run_view_marks", rva(0x456336), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { run_view_marks(r, &mut view_marks_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(rva(0x621890)) } == 0
        || unsafe { game_read::<u32>(crate::marks::setup::SETTINGS_RVA) } == 0
    {
        return t.skip("no marks on");
    }
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x456336,
            len: 0x14E,
            entry: &[0x83, 0x3D, 0x90, 0x18, 0x62, 0x00, 0x00, 0x0F],
            calls: &[0x92, 0x12E, 0x13C],
            jumps_out: &[],
        },
        &|c| {
            [(0x456484usize, 1u32), (0x4D14B4, 2), (0x4D1549, 3)]
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
    let mut regions: Vec<Region> = vec![g(0x5C2F8C, 0xC), g(0x5EA3CC, 0x40), g(0x62409C, 8)];
    for k in 0..8 {
        let record: u32 = unsafe { game_read(rva(0x71B3D8) + 4 * k) };
        if record == 0 {
            return t.skip("a car record is null");
        }
        regions.push((record, 4));
    }
    let mut kept = regions.clone();
    kept.extend([
        at(rva(0x63DB9C)),
        at(rva(0x6235E8)),
        at(rva(0x601024)),
        at(rva(0x6416B8)),
    ]);
    let saved = capture(&kept);
    for (n, case) in [(0u32, 0u32, 0u32, 1u32), (1, 0, 1, 0), (0, 1, 0, 1)]
        .into_iter()
        .enumerate()
    {
        let line = compare_plain(
            &format!("case {n}: {case:x?}"),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x63DB9C), case.0);
                game_write(rva(0x6235E8), case.1);
                game_write(rva(0x601024), case.2);
                game_write(rva(0x6416B8), case.3);
            },
            &regions,
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                run_view_marks(r, &mut |p, r| {
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
