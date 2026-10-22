//! `race::draw_face` (0x452C6F): the face's textures (switches, coordinates, the mode),
//! its corners' lights, then the depth test off, the polygons
//! (`race::draw_face_polygons`), the depth test on (see `race/course_faces.rs`).
//!
//! In:  esi = the face (kept)
//! Out: esi kept; the rest as the parts leave them (callers return)
//!
//! It begins with a call: private copy (0x60, its seven calls relocated), manual patch.
//! Callers: `sub_452887`, `sub_452B58`. Refuses (before anything) no race cars when lit.
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose depth-test and
//! polygon calls go to stand-ins (switches, coordinates and lights real), against the
//! Rust version on the same: the face stage, textures animated or not, a second texture,
//! lamps on and off; skipped without a race.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::course_faces::*;
use crate::race::face_stage::{FaceCase, FaceStage, STAND_IN_CARRY, compare, handler, noted};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::draw_face", rva(0x452C6F), thunk, &ORIGINAL)
    .private_copy(0x60)
    .relocate_calls(&[0x01, 0x0F, 0x45, 0x4A, 0x4F, 0x54, 0x59])
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
    if let Err(e) = unsafe { draw_face(r, &mut face_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_face(r, &mut face_part) }?;
    Ok(())
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
            address: 0x452C6F,
            len: 0x60,
            entry: &[0x56, 0xE8, 0xB7, 0x05, 0x00, 0x00, 0x83, 0x3D],
            calls: &[0x01, 0x0F, 0x45, 0x4A, 0x4F, 0x54, 0x59],
            jumps_out: &[],
        },
        &|c| {
            [(0x452358, 1), (0x452CCF, 2), (0x45237A, 3)]
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
    for (k, (textures, lights)) in [
        ((0u32, 0u32), quiet),
        ((3, 0), lit),
        ((3, 5), lit),
        ((0, 7), quiet),
        ((2, 2), lit),
    ]
    .into_iter()
    .enumerate()
    {
        let mut c = FaceCase::new([0, 1, 2, 3], lights);
        (c.light, c.textures, c.mode) = (k, textures, 4);
        cases.push((c, false));
    }
    for (c, carry) in cases {
        STAND_IN_CARRY.store(carry, Ordering::SeqCst);
        let line = compare(&mut stage, &c, copy.addr(), false, &mut |r| {
            unsafe { draw_face(r, &mut noted) }.map(|_| false)
        });
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    let _ = (quiet, lit);
    STAND_IN_CARRY.store(false, Ordering::SeqCst);
    stage.end();
    t.finish()
}
