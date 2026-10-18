//! `race::face_materials` (0x45265C): face esi's two triangles' textures, animated by car
//! 0's clock for objects with a speed (`byte_609C4C`), and their kinds' bits (see
//! `race/course_faces.rs`).
//!
//! In:  esi = the face
//! Out: eax, ebx, ecx as it leaves them, CF clear (callers go on with `mov` / calls)
//!
//! Callers: `sub_452393` (three places). Shadow mode: by state. Self-test: test faces
//! (objects with and without a speed, kinds' bits), the clock set; all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::course_faces::*;
use crate::race::frame_draw::all_regs;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::face_materials", rva(0x45265C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let out = unsafe { face_materials(r, r.esi) };
    let mut left = all_regs(&out);
    left.carry = Some(false);
    Ok(left)
}

fn regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![at(rva(0x63DA08)), g(0x63DA2C, 0x18), at(rva(0x62400C))]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut kept = regions.clone();
    kept.push(at(rva(0x70F3F4)));
    let saved = capture(&kept);
    let speeds: Vec<u32> = (0..0x100u32)
        .filter(|&k| unsafe { ((game_addr(rva(0x609C4C)) as u32 + k) as *const u8).read() } != 0)
        .take(2)
        .collect();
    let animated = speeds.first().copied().unwrap_or(1);
    let mut face = vec![0u8; 0x40];
    let f = face.as_mut_ptr() as u32;
    for (clock, a, b, kind) in [
        (0u32, animated, 0u32, 0x0Fu8),
        (12345, animated, animated, 0x3A),
        (0x7FFF_FFF0, 0, animated, 0x80),
    ] {
        check_state(
            &mut t,
            &format!("clock {clock:#x} objects {a:#x} {b:#x} kind {kind:#x}"),
            call_original,
            Regs {
                esi: f,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x70F3F4), clock);
                (f as *mut u8).write(kind);
                ((f + 1) as *mut u8).write(a as u8);
                ((f + 0x20) as *mut u8).write(kind ^ 0x0C);
                ((f + 0x21) as *mut u8).write(b as u8);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    drop(face);
    t.finish()
}
