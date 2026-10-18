//! `race::face_depth_test_on` (0x45237A): `scene::depth_test_on` unless `dword_63D9E0` is
//! set, then `dword_63D9E0` 0 (see `race/course_faces.rs`; the test looks the wrong way
//! round, `notes/bugs.md`).
//!
//! In:  nothing
//! Out: as the depth test's call leaves them (eax 1, ebx 0xE), or all kept
//!
//! Callers: `sub_452C6F`, `sub_45232A`. Shadow mode runs the original alone (it tells the
//! device). Self-test: as `face_depth_test_off`'s.

use crate::framework::{RawHook, Regs, hook_asm, rva, shadow};
use crate::race::course_faces::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::face_depth_test_on", rva(0x45237A), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { face_depth_test_on(r, &mut depth_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    crate::race::face_depth_test_off::depth_test(
        &HOOK,
        0x45237A,
        0x19,
        &[0x83, 0x3D, 0xE0, 0xD9, 0x63, 0x00, 0x00, 0x75],
        &[0x09],
        0x44E3E1,
        &|r, p| unsafe {
            // The stand-in's place is 1; the Rust version calls part 2.
            face_depth_test_on(r, &mut |_, r| p(1, r))
        },
    )
}
