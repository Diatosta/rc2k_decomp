//! `physics::try_last_face` (0x441E0C): ebx = `dword_5CCC78` (the face found last); when
//! it is not -1 and below `g_courseFaceCount`, `physics::face_under_point` on it inside
//! `pusha` / `popa`. CF set when the point is still on it.
//!
//! In:  nothing
//! Out: ebx = `dword_5CCC78`; CF; the rest kept
//!
//! See `physics/ground.rs`. Caller: `sub_441E2F`, first (`jb`), every search.
//! Self-test: a copy of the original (its call to a copy of `face_under_point`) on the
//! test's ground: last faces -1, every face, the count and past it, against the test's
//! points.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::ground::*;
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::try_last_face", rva(0x441E0C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: `r` as the original leaves it, and the carry.
pub unsafe fn try_last_face(r: &mut Regs) -> Result<bool, Refused> {
    unsafe { try_last_face_at(&Ground::game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    let carry = unsafe { try_last_face(&mut g) }?;
    Ok(left_of(&g, carry))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &Ground::game().regions(), &rust);
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
    let mut test = TestGround::new(20, 25, 1);
    let copies = match test.copies() {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    test.lay_out(3);
    let g = test.ground();
    let mut found = 0;
    for (n, &(x, z)) in test_points().iter().enumerate() {
        for last in (0..20u32).chain([u32::MAX, 20, 0x8000_0000]) {
            let prepare = || unsafe {
                test.clear_work();
                test.set_point(x, z, 0, n % 2 == 0);
                g.put(0x5CCC78, last);
            };
            if check_case(
                &mut t,
                &format!("last face {last:#x}, point ({x:#x}, {z:#x})"),
                copies.try_last_face.addr(),
                &test,
                Regs::SENTINEL,
                &prepare,
                &|g, r| unsafe { try_last_face_at(g, r) },
                true,
            ) {
                found += 1;
            }
        }
    }
    t.check(found > 0, || "no case found the point".into());
    drop(copies);
    t.finish()
}
