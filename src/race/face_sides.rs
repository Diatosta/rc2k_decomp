//! `race::face_sides` (0x4525BE): which of face esi's triangles face the view (see
//! `race/course_faces.rs`; `sub_4525F8`, the facing test, covered here, its x87 run the
//! original's instructions): `dword_63D9E4` the first's, `dword_63D9E8` 1 or the second's.
//!
//! In:  esi = the face
//! Out: eax the last answer, ebx the last triangle's point (0 / 4 outside a race), ebp the
//!      face (a `mov` follows, not of them)
//!
//! Caller: `sub_45232A`. Shadow mode: by state. Self-test: test faces and scene points
//! (`g_scenePoints` pointed at them), normals towards and away from the view, triangles
//! and quads, in a race and not.

use crate::errors::Refused;
use crate::framework::{
    EAX, EBP, EBX, ESI, RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva,
};
use crate::race::course_faces::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::face_sides", rva(0x4525BE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let out = unsafe { face_sides(r.esi) };
    Ok(Left::nothing()
        .with(EAX, out.eax)
        .with(EBX, out.ebx)
        .with(ESI, out.esi)
        .with(EBP, out.ebp))
}

fn regions() -> [Region; 2] {
    [at(rva(0x63D9E4)), at(rva(0x63D9E8))]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let kept = [
        regions[0],
        regions[1],
        at(rva(0x5CCCC0)),
        at(rva(0x621480)),
        at(rva(0x63D9F8)),
        (game_addr(rva(0x6062A4)) as u32, 0xC),
    ];
    let saved = capture(&kept);
    let points: Vec<u32> = vec![0x1000, 0x200, 0x3000, 0x2000, 0x100, 0x5000, 0, 0, 0];
    let mut face = vec![0u32; 0x10];
    let f = face.as_mut_ptr() as u32;
    for (normal, second, race, triangle) in [
        (1.0f32, -1.0f32, 1u32, false),
        (-1.0, 1.0, 1, false),
        (1.0, 1.0, 1, true),
        (0.0, -0.5, 1, false),
        (1.0, 1.0, 0, false),
    ] {
        check_state(
            &mut t,
            &format!("normals {normal} {second} race {race} triangle {triangle}"),
            call_original,
            Regs {
                esi: f,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x5CCCC0), points.as_ptr() as u32);
                game_write(rva(0x621480), race);
                game_write(rva(0x63D9F8), if triangle { u32::MAX } else { 2 });
                game_write(rva(0x6062A4), 0.0f32.to_bits());
                game_write(rva(0x6062A8), 0.0f32.to_bits());
                game_write(rva(0x6062AC), 0.0f32.to_bits());
                let w = |o: u32, v: u32| ((f + o) as *mut u32).write_unaligned(v);
                w(8, 0);
                w(0xC, 1);
                for (o, v) in [
                    (0x18u32, normal),
                    (0x30, normal * 0.5),
                    (0x38, normal),
                    (0x1C, second),
                    (0x34, second),
                    (0x3C, -second),
                ] {
                    w(o, v.to_bits());
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    drop(face);
    t.finish()
}
