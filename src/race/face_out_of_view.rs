//! `race::face_out_of_view` (0x45246F): carry when none of the face's corners is in sight
//! (see `race/course_faces.rs`): each corner's scene record (made when not there), its
//! depth not negative and |x| below `flt_63DA00` or |x| x `flt_63D9FC` below the depth.
//!
//! In:  nothing (the corners `dword_63D9EC` ..)
//! Out: CF; eax the last corner looked at, edx its record (its caller tests the carry)
//!
//! Caller: `sub_45232A`. Refuses (before anything) no point records or points. Shadow
//! mode: by state and the carry. Self-test: corners on scene points of the test's
//! (`g_scenePoints` pointed at them, their records unmade), in sight and not; all put back
//! (skipped without a race).

use crate::errors::Refused;
use crate::framework::{
    EAX, EDX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::race::course_faces::face_out_of_view;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::face_out_of_view", rva(0x45246F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (out, carry) = unsafe { face_out_of_view(r)? };
    Ok(Left::carry(carry)
        .with(EAX, out.eax)
        .with(EDX, out.edx)
        .with(crate::framework::ECX, out.ecx))
}

fn regions() -> Vec<Region> {
    let records: u32 = unsafe { game_read(rva(0x5C6FB8)) };
    vec![(records, 0x20 * 0x10), at(rva(0x62400C))]
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions: Vec<Region> = vec![at(rva(0x62400C))];
    let records: u32 = unsafe { game_read(rva(0x5C6FB8)) };
    if records != 0 {
        for k in 0..4 {
            let p: u32 = unsafe { game_read(rva(0x63D9EC + 4 * k)) };
            regions.extend(crate::scene::project_point::regions(p));
        }
    }
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(rva(0x621480)) } == 0
        || unsafe { game_read::<u32>(rva(0x5C6FB8)) } == 0
    {
        return t.skip("no race loaded");
    }
    let regions = regions();
    let mut kept = regions.clone();
    kept.extend([
        at(rva(0x5CCCC0)),
        (game_addr(rva(0x63D9EC)) as u32, 0x10),
        at(rva(0x63DA00)),
        at(rva(0x63D9FC)),
        at(rva(0x5CCD20)),
    ]);
    let saved = capture(&kept);
    let view = game_addr(rva(0x606298)) as u32;
    let (vx, vy, vz) = unsafe {
        (
            (view as *const u32).read_unaligned(),
            ((view + 4) as *const u32).read_unaligned(),
            ((view + 8) as *const u32).read_unaligned(),
        )
    };
    let mut points = vec![0u32; 3 * 16];
    for k in 0..16u32 {
        points[3 * k as usize] = vx
            .wrapping_add(k.wrapping_mul(0x9000))
            .wrapping_sub(0x40000);
        points[3 * k as usize + 1] = vy;
        points[3 * k as usize + 2] = vz.wrapping_add((k as i32 - 8).wrapping_mul(0x20000) as u32);
    }
    let p = points.as_ptr() as u32;
    for (corners, near) in [
        ([0u32, 1, 2, 3], 1000.0f32),
        ([4, 5, 6, u32::MAX], 0.5),
        ([12, 13, 14, 15], 1e9),
        ([0, 1, 2, 3], 0.0),
    ] {
        check_state(
            &mut t,
            &format!("corners {corners:x?} near {near}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x5CCCC0), p);
                game_write(rva(0x5CCD20), 16u32);
                game_write(rva(0x63DA00), near.to_bits());
                game_write(rva(0x63D9FC), 1.5f32.to_bits());
                let records: u32 = game_read(rva(0x5C6FB8));
                for k in 0..16u32 {
                    ((records + 0x20 * k) as *mut u32).write_unaligned(u32::MAX);
                }
                for (n, &c) in corners.iter().enumerate() {
                    game_write(rva(0x63D9EC + 4 * n), c);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    drop(points);
    t.finish()
}
