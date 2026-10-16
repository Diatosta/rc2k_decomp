//! `race::face_normal` (0x422144): the normal of three of the course's points.
//!
//! In:  eax, ebx, ecx = the points
//! Out: eax = the third's z less the first's, ebx = the third's y, ecx = its z; the rest
//!      preserved
//!
//! See `race/norms.rs`. Two callers, both in `race::store_face_normal` (inside `pusha`).
//! Self-test: points of the test's, the game's work and vector put back afterwards.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::norms::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::face_normal", rva(0x422144), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { face_normal(r) };
    Ok(Left::nothing()
        .with(EAX, g.eax)
        .with(EBX, g.ebx)
        .with(ECX, g.ecx))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &norm_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = norm_regions();
    let mut all = regions.clone();
    all.push(at(POINTS_RVA));
    let saved = capture(&all);
    let points: Vec<i32> = vec![
        0,
        0,
        0,
        100,
        0,
        0,
        0,
        100,
        0,
        0,
        0,
        100,
        -5000,
        7000,
        12,
        65535,
        -65535,
        3,
        7,
        7,
        7,
        0x7FFF_0000,
        -0x7FFF_0000,
        1,
    ];
    let base = points.as_ptr() as u32;
    for (a, b, c) in [
        (0u32, 1, 2),
        (0, 2, 1),
        (1, 2, 3),
        (4, 5, 6),
        (0, 0, 0),
        (6, 6, 1),
        (7, 4, 0),
        (3, 7, 5),
    ] {
        check_state(
            &mut t,
            &format!("points {a}, {b}, {c}"),
            call_original,
            Regs {
                eax: a,
                ebx: b,
                ecx: c,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&all, &saved);
                game_write(POINTS_RVA, base);
            },
            &regions,
            &rust,
        );
    }
    restore(&all, &saved);
    drop(points);
    t.finish()
}
