//! `race::draw_face_triangles` (0x452CF3): the face's two triangles (corners 0, 1, 2
//! and 3, 2, 1) into entries 0 .. 2 and each drawn (`race::draw_lit_triangle`) unless
//! facing away (see `race/course_faces.rs`).
//!
//! In:  nothing (the face's working values)
//! Out: the registers as the parts leave them (its caller returns)
//!
//! Caller: `sub_452CCF`. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose two drawing calls go to a stand-in, against the Rust version on the same:
//! the face stage, each facing, triangles and quads; skipped without a race.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::course_faces::*;
use crate::race::face_stage::{FaceCase, FaceStage, STAND_IN_CARRY, compare, handler, noted};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_face_triangles", rva(0x452CF3), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { draw_face_triangles(r, &mut triangles_part) } {
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
            address: 0x452CF3,
            len: 0x24A,
            entry: &[0x83, 0x3D, 0xE4, 0xD9, 0x63, 0x00, 0x00, 0x0F],
            calls: &[0x119, 0x244],
            jumps_out: &[],
        },
        &|c| {
            [(0x4537E9, 1)]
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
    for (away, fourth) in [
        ((0u32, 0u32), 3u32),
        ((1, 0), 3),
        ((0, 1), 3),
        ((1, 1), 3),
        ((0, 0), u32::MAX),
    ] {
        let mut c = FaceCase::new([0, 1, 2, fourth], lit);
        c.away = away;
        cases.push((c, false));
    }
    for (c, carry) in cases {
        STAND_IN_CARRY.store(carry, Ordering::SeqCst);
        let line = compare(&mut stage, &c, copy.addr(), false, &mut |r| {
            unsafe { draw_face_triangles(r, &mut noted) }.map(|_| false)
        });
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    let _ = (quiet, lit);
    STAND_IN_CARRY.store(false, Ordering::SeqCst);
    stage.end();
    t.finish()
}
