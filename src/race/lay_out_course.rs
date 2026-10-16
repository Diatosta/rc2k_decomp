//! `race::lay_out_course` (0x421BE3): the course memory's five pointers.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/norms.rs`. Nine callers (`race::store_norms` and the stage's loading).
//! Self-test: counts of the test's, the game's variables put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::norms::*;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::lay_out_course", rva(0x421BE3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { lay_out_course() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &course_inputs(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let inputs = course_inputs();
    let saved = capture(&inputs);
    for (faces, points) in [(0u32, 0u32), (20, 30), (0x1234, 0x5678), (0xFFFF_FFFF, 7)] {
        let course = TestCourse::new(1, 1, false, 3);
        unsafe {
            crate::framework::game_write(FACE_COUNT_RVA, faces);
            crate::framework::game_write(POINT_COUNT_RVA, points);
        }
        let set = capture(&inputs);
        check_state(
            &mut t,
            &format!("faces {faces:#x}, points {points:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| restore(&inputs, &set),
            &inputs,
            &rust,
        );
        drop(course);
    }
    restore(&inputs, &saved);
    t.finish()
}
