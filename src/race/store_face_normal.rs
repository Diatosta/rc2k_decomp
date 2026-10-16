//! `race::store_face_normal` (0x421F72): a face's repeated-points flag and its two
//! normals.
//!
//! In:  ebx = the face
//! Out: every register preserved
//!
//! See `race/norms.rs`. One caller, `race::store_norms`. Self-test: a course of the test's
//! (`norms::TestCourse`, three and four points, points out of range and repeated), every
//! face, the game's variables put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::norms::*;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::store_face_normal", rva(0x421F72), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { store_face_normal(r.ebx, &mut game_normal) };
    Ok(Left::nothing())
}

fn regions(face: u32) -> Vec<crate::service::harness::Region> {
    let faces: u32 = unsafe { crate::framework::game_read(FACES_RVA) };
    let mut r = norm_regions();
    r.push((faces.wrapping_add(face << 6), 0x40));
    r
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(r.ebx), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let work = norm_regions();
    let saved = capture(&work);
    for (quad, seed) in [(false, 1u32), (true, 2)] {
        let course = TestCourse::new(20, 30, quad, seed);
        let start = capture(&[course.region()]);
        for face in 0..20 {
            check_state(
                &mut t,
                &format!("quad {quad}, face {face}"),
                call_original,
                Regs {
                    ebx: face,
                    ..Regs::SENTINEL
                },
                &|| {
                    restore(&[course.region()], &start);
                    restore(&work, &saved);
                },
                &[&regions(face)[..], &[course.region()]].concat(),
                &rust,
            );
        }
        drop(course);
    }
    restore(&work, &saved);
    t.finish()
}
