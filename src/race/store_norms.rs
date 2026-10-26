//! `race::store_norms` (0x421F02): every face's normals (`SHAUNINITEVERYTHING : store
//! norms`).
//!
//! In:  nothing
//! Out: ebx = the face count; every other register preserved
//!
//! See `race/norms.rs`. One caller, `race::loadlevel_init`. Refuses no faces or 0x100000 or
//! more. Shadow mode compares the course memory's faces, the layout and the work. Self-test:
//! courses of the test's, the game's variables put back afterwards.

use crate::errors::Refused;
use crate::framework::{EBX, RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::race::norms::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

// It begins with its call: the jump is written by hand and the original runs from a
// private copy, its two calls relocated.
pub static HOOK: RawHook = RawHook::new("race::store_norms", rva(0x421F02), thunk, &ORIGINAL)
    .private_copy(0x19)
    .relocate_calls(&[0x00, 0x0A])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { store_norms(r, &mut game_normal) }?;
    Ok(Left::nothing().with(EBX, g.ebx))
}

/// The faces as the layout will put them, the layout and the work.
fn regions() -> Vec<Region> {
    let mut r = norm_regions();
    r.extend(course_inputs());
    unsafe {
        let memory: u32 = game_read(rva(0x5C6F94));
        let first: u32 = game_read(rva(0x5CCD18));
        let faces: u32 = game_read(FACE_COUNT_RVA);
        if memory != 0 && faces < 0x10_0000 {
            r.push((
                memory + 0x100 + first.wrapping_mul(0x20),
                (faces * 0x40) as usize,
            ));
        }
    }
    r
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let work = norm_regions();
    let saved = capture(&work);
    for (faces, points, quad, seed) in [
        (1u32, 4u32, false, 5u32),
        (25, 40, true, 6),
        (40, 25, false, 7),
    ] {
        let course = TestCourse::new(faces, points, quad, seed);
        let start = capture(&[course.region()]);
        let all = [&work[..], &course_inputs(), &[course.region()]].concat();
        check_state(
            &mut t,
            &format!("{faces} faces, {points} points, quad {quad}"),
            call_original,
            Regs::SENTINEL,
            &|| {
                restore(&[course.region()], &start);
                restore(&work, &saved);
            },
            &all,
            &rust,
        );
        drop(course);
    }
    restore(&work, &saved);
    t.finish()
}
