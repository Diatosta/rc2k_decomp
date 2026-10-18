//! `race::face_corners` (0x452444): face esi's four points into `dword_63D9EC` ..
//! `dword_63D9F8`, the fourth -1 when it is the third (see `race/course_faces.rs`).
//!
//! In:  esi = the face
//! Out: eax the fourth (a `call` follows)
//!
//! Caller: `sub_45232A`. Shadow mode: by state. Self-test: a test face, quads and
//! triangles.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::course_faces::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::face_corners", rva(0x452444), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(crate::framework::EAX, unsafe { face_corners(r.esi) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[(game_addr(rva(0x63D9EC)) as u32, 0x10)],
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions: [Region; 1] = [(game_addr(rva(0x63D9EC)) as u32, 0x10)];
    let saved = capture(&regions);
    let mut face = vec![0u32; 0x10];
    let f = face.as_mut_ptr() as u32;
    for (c, d) in [(7u32, 9u32), (7, 7), (0xFFFF_FFFF, 0xFFFF_FFFF)] {
        check_state(
            &mut t,
            &format!("third {c:#x} fourth {d:#x}"),
            call_original,
            Regs {
                esi: f,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                for (k, v) in [1u32, 5, c, d].iter().enumerate() {
                    ((f + 4 + 4 * k as u32) as *mut u32).write_unaligned(*v);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    drop(face);
    t.finish()
}
