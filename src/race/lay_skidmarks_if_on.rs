//! `race::lay_skidmarks_if_on` (0x4585F3): `race::lay_skidmarks` unless `dword_6235E8` or
//! `dword_6416BC` is set (see `race/skid_laying.rs`).
//!
//! In:  nothing
//! Out: as `lay_skidmarks` leaves them, or all kept
//!
//! Caller: `sub_45429B` (0x454795). Shadow mode runs the original alone. Self-test: a copy
//! whose call goes to a stand-in, the switches each way.

use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva, shadow};
use crate::race::face_stage::compare_plain;
use crate::race::level_init::note;
use crate::race::skid_laying::{lay_part, lay_skidmarks_if_on};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::lay_skidmarks_if_on", rva(0x4585F3), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { lay_skidmarks_if_on(r, &mut lay_part) } {
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
            address: 0x4585F3,
            len: 0x19,
            entry: &[0x83, 0x3D, 0xE8, 0x35, 0x62, 0x00, 0x00, 0x75],
            calls: &[0x13],
            jumps_out: &[],
        },
        &|c| (c == rva(0x4588DE)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let kept: Vec<Region> = vec![at(rva(0x6235E8)), at(rva(0x6416BC))];
    let saved = capture(&kept);
    for (a, b) in [(0u32, 0u32), (1, 0), (0, 1), (1, 1)] {
        let line = compare_plain(
            &format!("switches {a} {b}"),
            &|| unsafe {
                game_write(rva(0x6235E8), a);
                game_write(rva(0x6416BC), b);
            },
            &[],
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                lay_skidmarks_if_on(r, &mut |p, r| {
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
