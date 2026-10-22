//! `race::draw_course_face` (0x45232A): face esi: its corners (copied to new points for
//! flag 0x200), and unless none is in sight its facings, the depth test off, its sides
//! (`race::draw_face_sides`), the depth test on; see `race/course_faces.rs`.
//!
//! In:  esi = the face
//! Out: the registers as it leaves them (`sub_4537E3` returns)
//!
//! It begins with its call: private copy (0x2E, its seven calls relocated), manual patch.
//! Refuses (before anything) no point records or points. Shadow mode runs the original
//! alone (it draws). Self-test: a copy whose depth-test and sides calls go to stand-ins
//! (corners, sight and facings real), against the Rust version on the same: the face
//! stage's world, test faces with and without flag 0x200, in sight and not; skipped
//! without a race.

use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva, shadow};
use crate::race::course_faces::*;
use crate::race::face_stage::{FaceCase, FaceStage, handler, noted};
use crate::race::level_init::take_called;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::draw_course_face", rva(0x45232A), thunk, &ORIGINAL)
    .private_copy(0x2E)
    .relocate_calls(&[0x00, 0x0D, 0x12, 0x19, 0x1E, 0x23, 0x28])
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

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_course_face(r, &mut course_face_part) }?;
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
            address: 0x45232A,
            len: 0x2E,
            entry: &[0xE8, 0x15, 0x01, 0x00, 0x00, 0x66, 0xF7, 0x46],
            calls: &[0x00, 0x0D, 0x12, 0x19, 0x1E, 0x23, 0x28],
            jumps_out: &[],
        },
        &|c| {
            [(0x452358usize, 1u32), (0x452393, 2), (0x45237A, 3)]
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
    let mut face = vec![0u32; 0x10];
    let f = face.as_mut_ptr() as u32;
    for (view, corners, flags, normal) in [
        (0usize, [0u32, 1, 2, 3], 0u16, 1.0f32),
        (0, [0, 1, 2, 2], 0x200, -1.0),
        (1, [4, 7, 1, 0], 0x204, 0.5),
        (2, [3, 2, 1, 0], 0x200, 1.0),
    ] {
        let c = FaceCase {
            view,
            ..FaceCase::new(corners, [0; 4])
        };
        let set = || unsafe {
            for (k, v) in corners.iter().enumerate() {
                ((f + 4 + 4 * k as u32) as *mut u32).write_unaligned(*v);
            }
            ((f + 2) as *mut u16).write_unaligned(flags);
            for (o, v) in [
                (0x18u32, normal),
                (0x30, 0.5),
                (0x38, -normal),
                (0x1C, -normal),
                (0x34, 1.0),
                (0x3C, normal),
            ] {
                ((f + o) as *mut u32).write_unaligned(v.to_bits());
            }
        };
        stage.prepare(&c);
        set();
        let mut ro = Regs {
            esi: f,
            ..Regs::SENTINEL
        };
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (
            take_called().iter().map(|c| c.0).collect::<Vec<_>>(),
            stage.state(),
        );
        stage.prepare(&c);
        set();
        let mut rr = Regs {
            esi: f,
            ..Regs::SENTINEL
        };
        let done = unsafe { draw_course_face(&mut rr, &mut noted) };
        let mine = (
            take_called().iter().map(|c| c.0).collect::<Vec<_>>(),
            stage.state(),
        );
        t.check(done.is_ok() && theirs == mine, || {
            let memory = theirs.1.0.iter().zip(&mine.1.0).position(|(a, b)| a != b);
            format!(
                "view {view} corners {corners:x?} flags {flags:#x}: {} calls {:?} / {:?}; memory first differs at {memory:x?}, records equal {}",
                if done.is_ok() { "done" } else { "refused" },
                theirs.0,
                mine.0,
                theirs.1.1 == mine.1.1
            )
        });
    }
    stage.end();
    drop(face);
    t.finish()
}
