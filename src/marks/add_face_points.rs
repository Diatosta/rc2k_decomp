//! `marks::add_face_points` (0x4B9E29): face edi's points added to record ebp's, each once,
//! up to 0x80.
//!
//! In:  edi the face, ebp the spline record
//! Out: eax the record's count, ecx where the last search stopped, edx the last point read
//!
//! See `marks/splines.rs`. One caller, `sub_4B9D5F`. Refuses a record of more than 0x80
//! points. Self-test: the original and the Rust version on a record and a face of the
//! test's (points repeated, the record filling up), the refusal.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::splines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::add_face_points", rva(0x4B9E29), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { add_face_points(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [(r.ebp, SPLINE_LEN as usize)];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

const FACE_POINTS: u32 = 12;

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let spline = TestSpline::new();
    let face = vec![0u8; (0x10 + 0x10 * FACE_POINTS) as usize];
    let f = face.as_ptr() as u32;
    for points in [0u32, 1, 0x40, 0x7A, MOST_POINTS] {
        for (seed, count) in [(1u32, 0u32), (2, 4), (3, FACE_POINTS)] {
            let prepare = || unsafe {
                spline.fill(seed * 0x10D + points, points);
                let mut s = seed;
                for n in 0..face.len() as u32 / 4 {
                    s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
                    ((f + 4 * n) as *mut u32).write_unaligned(s >> 4);
                }
                (f as *mut u32).write_unaligned(count);
                for k in 0..FACE_POINTS {
                    // Some already in the record (ids < 0x60), some repeated in the face.
                    ((f + 0x10 + 0x10 * k) as *mut u32).write_unaligned(0x50 + k % 0x18);
                }
            };
            check_state(
                &mut t,
                &format!("{points:#x} points, a face of {count}"),
                call_original,
                Regs {
                    edi: f,
                    ebp: spline.addr(),
                    ..Regs::SENTINEL
                },
                &prepare,
                &[spline.region(), (f, face.len())],
                &rust,
            );
        }
    }
    spline.fill(3, MOST_POINTS + 1);
    t.check(
        unsafe {
            add_face_points(&Regs {
                edi: f,
                ebp: spline.addr(),
                ..Regs::SENTINEL
            })
        }
        .is_err(),
        || "0x81 points: the Rust version went on".into(),
    );
    t.finish()
}
