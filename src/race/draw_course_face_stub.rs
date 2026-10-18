//! `race::draw_course_face_stub` (0x4537E3): `call sub_45232A; retn`, that is
//! `race::draw_course_face`; see `race/course_faces.rs`.
//!
//! In:  esi = the face
//! Out: as `draw_course_face` leaves them
//!
//! It is its call: private copy (6, the call relocated), manual patch. Caller:
//! `sub_44F9F3`. Shadow mode runs the original alone (it draws). Self-test: a copy whose
//! call goes to a stand-in, against the Rust side's one call.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::course_faces::*;
use crate::race::face_stage::compare_plain;
use crate::race::level_init::note;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::draw_course_face_stub",
    rva(0x4537E3),
    thunk,
    &ORIGINAL,
)
.private_copy(0x6)
.relocate_calls(&[0x00])
.manual_patch()
.self_test(self_test);

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
    if let Err(e) = unsafe { draw_course_face(r, &mut course_face_part) } {
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
            address: 0x4537E3,
            len: 0x6,
            entry: &[0xE8, 0x42, 0xEB, 0xFF, 0xFF, 0xC3],
            calls: &[0x00],
            jumps_out: &[],
        },
        &|c| {
            [(0x45232A, 1)]
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
    let line = compare_plain(
        "one call",
        &|| {},
        &[],
        copy.addr(),
        Regs::SENTINEL,
        &mut |r| {
            note(1, r);
            Ok(())
        },
    );
    t.check(line.is_none(), || line.unwrap_or_default());
    t.finish()
}
