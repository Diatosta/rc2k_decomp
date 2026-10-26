//! `race::face_corners_to_new_points` (0x452722): the face's corners copied to the four
//! scene points after the last (their records marked unmade), the corners then naming the
//! copies; the count left (see `race/course_faces.rs`).
//!
//! In:  nothing (`pusha` .. `popa`)
//! Out: every register preserved
//!
//! Caller: `sub_45232A`. Refuses (before anything) no point records or points. Shadow
//! mode: by state. Self-test: test scene points (`g_scenePoints` pointed at them), quads,
//! triangles and a corner that is itself one of the new slots; all put back (skipped with
//! no point records).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::course_faces::face_corners_to_new_points;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::face_corners_to_new_points",
    rva(0x452722),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { face_corners_to_new_points()? };
    Ok(Left::nothing())
}

fn regions() -> Vec<Region> {
    let mut v: Vec<Region> = vec![(game_addr(rva(0x63D9EC)) as u32, 0x10)];
    unsafe {
        let (records, points, count): (u32, u32, u32) = (
            game_read(rva(0x5C6FB8)),
            game_read(rva(0x5CCCC0)),
            game_read(rva(0x5CCD20)),
        );
        if records != 0 && points != 0 {
            for k in 0..4u32 {
                v.push((records + ((count.wrapping_add(k) & 0xFFF) << 5), 4));
                v.push((
                    count.wrapping_add(k).wrapping_mul(12).wrapping_add(points),
                    12,
                ));
            }
        }
    }
    v
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
    let records: u32 = unsafe { game_read(rva(0x5C6FB8)) };
    if records == 0 {
        return t.skip("no point records");
    }
    let mut points = vec![0u32; 3 * 16];
    for (k, v) in points.iter_mut().enumerate() {
        *v = (k as u32).wrapping_mul(0x0101_0101) ^ 0x5A00_0000;
    }
    let p = points.as_ptr() as u32;
    let mut kept: Vec<Region> = vec![
        (game_addr(rva(0x63D9EC)) as u32, 0x10),
        at(rva(0x5CCCC0)),
        at(rva(0x5CCD20)),
    ];
    for k in 0..4u32 {
        kept.push((records + ((10 + k) << 5), 4));
    }
    let saved = capture(&kept);
    for corners in [
        [1u32, 2, 3, 4],
        [5, 6, 7, u32::MAX],
        [11, 10, 2, 12],
        [0, 0, 0, 0],
    ] {
        let prepare = || unsafe {
            restore(&kept, &saved);
            game_write(rva(0x5CCCC0), p);
            game_write(rva(0x5CCD20), 10u32);
            for (n, &c) in corners.iter().enumerate() {
                game_write(rva(0x63D9EC + 4 * n), c);
            }
        };
        prepare();
        let regions = regions();
        check_state(
            &mut t,
            &format!("corners {corners:x?}"),
            call_original,
            Regs::SENTINEL,
            &prepare,
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    drop(points);
    t.finish()
}
