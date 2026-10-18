//! `race::draw_lit_triangle` (0x4537E9): the triangle in entries 0 .. 2, as it is
//! (`scene::draw_clipped_polygon`) without lamp light at its corners, else cut in four at
//! its sides' middles (new points lit by `race::light_colour_at`), the colours plus the
//! lights, and each clipped and drawn (`race::clip_and_draw_face`); see
//! `race/course_faces.rs`.
//!
//! In:  nothing (entries 0 .. 2, `g_polygonVertexCount` 3)
//! Out: the registers as the parts leave them (callers return or reload)
//!
//! Callers: `sub_452CF3` (two). Refuses (before anything) no points, or no race cars.
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose two drawing
//! calls go to stand-ins (projection, scaling, fade and light real), against the Rust
//! version on the same: the face stage (`race/face_stage.rs`), quiet and lit corners,
//! modes 2 and 5, views and flat projection; skipped without a race.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::course_faces::*;
use crate::race::face_stage::{FaceCase, FaceStage, STAND_IN_CARRY, compare, handler, noted};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_lit_triangle", rva(0x4537E9), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { draw_lit_triangle(r, &mut lit_triangle_part) } {
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
            address: 0x4537E9,
            len: 0x99E,
            entry: &[0xA0, 0x2E, 0x55, 0x97, 0x00, 0x0A, 0x05, 0x72],
            calls: &[
                0x13, 0x56, 0x6C, 0x71, 0x24C, 0x41C, 0x5EC, 0x8D4, 0x915, 0x956, 0x997,
            ],
            jumps_out: &[],
        },
        &|c| {
            [(0x44CCEF, 1), (0x454187, 2)]
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
    for (k, (view, flat, mode, lights)) in [
        (0usize, false, 2u32, quiet),
        (0, false, 2, lit),
        (1, false, 5, lit),
        (3, true, 5, lit),
        (2, false, 2, [0, 0x0010_0000, 0, 0]),
    ]
    .into_iter()
    .enumerate()
    {
        let mut c = FaceCase::new([0, 1, 2, 3], lights);
        (c.light, c.view, c.flat, c.mode) = (k, view, flat, mode);
        cases.push((c, false));
    }
    for (c, carry) in cases {
        STAND_IN_CARRY.store(carry, Ordering::SeqCst);
        let line = compare(&mut stage, &c, copy.addr(), false, &mut |r| {
            unsafe { draw_lit_triangle(r, &mut noted) }.map(|_| false)
        });
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    let _ = (quiet, lit);
    STAND_IN_CARRY.store(false, Ordering::SeqCst);
    stage.end();
    t.finish()
}
