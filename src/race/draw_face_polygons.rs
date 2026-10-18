//! `race::draw_face_polygons` (0x452CCF): in a race with both triangles facing the view
//! the face as one quad (`race::draw_whole_quad`), as triangles
//! (`race::draw_face_triangles`) otherwise or when the quad's carry asks (see
//! `race/course_faces.rs`).
//!
//! In:  nothing
//! Out: the registers as the parts leave them (its caller goes on with a call)
//!
//! Caller: `sub_452C6F`. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose two calls go to stand-ins (the quad's giving each carry), against the Rust
//! version on the same; each facing; skipped without a race.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::course_faces::*;
use crate::race::face_stage::{FaceCase, FaceStage, STAND_IN_CARRY, compare, handler, noted};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_face_polygons", rva(0x452CCF), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { draw_face_polygons_in_game(r) } {
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
            address: 0x452CCF,
            len: 0x24,
            entry: &[0x83, 0x3D, 0x80, 0x14, 0x62, 0x00, 0x00, 0x74],
            calls: &[0x16, 0x1C],
            jumps_out: &[],
        },
        &|c| {
            [(0x452F3D, 1), (0x452CF3, 2)]
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
    for (away, carry) in [
        ((0u32, 0u32), false),
        ((0, 0), true),
        ((1, 0), false),
        ((0, 1), true),
        ((1, 1), false),
    ] {
        let mut c = FaceCase::new([0, 1, 2, 3], quiet);
        c.away = away;
        cases.push((c, carry));
    }
    for (c, carry) in cases {
        STAND_IN_CARRY.store(carry, Ordering::SeqCst);
        let line = compare(&mut stage, &c, copy.addr(), false, &mut |r| {
            let carry = STAND_IN_CARRY.load(Ordering::SeqCst);
            unsafe {
                draw_face_polygons(
                    r,
                    &mut |r| {
                        noted(1, r)?;
                        Ok(carry)
                    },
                    &mut noted,
                )
            }
            .map(|_| false)
        });
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    let _ = (quiet, lit);
    STAND_IN_CARRY.store(false, Ordering::SeqCst);
    stage.end();
    t.finish()
}
