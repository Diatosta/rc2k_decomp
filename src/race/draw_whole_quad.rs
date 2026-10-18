//! `race::draw_whole_quad` (0x452F3D): the face as one quad (entries 1, 2, 3, 0 from
//! corners 0, 1, 3, 2) drawn by `race::draw_marked_face` when no corner has lamp light
//! and all vertices are inside the depths and the screen; carry asks for triangles (see
//! `race/course_faces.rs`).
//!
//! In:  nothing (the face's working values)
//! Out: CF; the other registers as it leaves them (its caller tests the carry)
//!
//! Caller: `sub_452CCF`. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose drawing call goes to a stand-in (the projection real), against the Rust
//! version on the same: the face stage, quiet and lit corners, views (inside, partly
//! outside), the carry compared; skipped without a race.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::course_faces::*;
use crate::race::face_stage::{FaceCase, FaceStage, STAND_IN_CARRY, compare, handler, noted};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_whole_quad", rva(0x452F3D), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) =
        unsafe { draw_whole_quad(r, &mut whole_quad_part).map(|carry| r.set_carry(carry)) }
    {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let Some(mut stage) = FaceStage::new(&[]) else {
        return t.skip("a car block is null (no race loaded)");
    };
    crate::install::stand_ins::set_handler(handler);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x452F3D,
            len: 0x2EF,
            entry: &[0x8B, 0x0D, 0xEC, 0xD9, 0x63, 0x00, 0x89, 0x0D],
            calls: &[0x1A2, 0x2E6],
            jumps_out: &[],
        },
        &|c| {
            [(0x44CD90, 1)]
                .iter()
                .find(|(a, _)| rva(*a) == c)
                .map(|(_, p)| crate::install::stand_ins::stand_in(*p))
        },
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => {
            stage.end();
            return t.skip(&format!("copy: {e}"));
        }
    };
    let quiet = [0u32; 4];
    let lit = [0x0040_1020u32, 0, 0x0001_0000, 0x00FF_FFFF];
    let mut cases = Vec::new();
    for (k, (view, flat, lights, corners)) in [
        (0usize, false, quiet, [0u32, 1, 2, 3]),
        (0, false, lit, [0, 1, 2, 3]),
        (1, false, quiet, [0, 1, 2, 4]),
        (2, false, quiet, [7, 1, 2, 3]),
        (3, true, quiet, [0, 2, 4, 7]),
    ]
    .into_iter()
    .enumerate()
    {
        let mut c = FaceCase::new(corners, lights);
        (c.light, c.view, c.flat) = (k, view, flat);
        cases.push((c, false));
    }
    for (c, carry) in cases {
        STAND_IN_CARRY.store(carry, Ordering::SeqCst);
        let line = compare(&mut stage, &c, copy.addr(), true, &mut |r| unsafe {
            draw_whole_quad(r, &mut noted)
        });
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    let _ = (quiet, lit);
    STAND_IN_CARRY.store(false, Ordering::SeqCst);
    stage.end();
    t.finish()
}
