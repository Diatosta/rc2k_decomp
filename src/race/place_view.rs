//! `race::place_view` (0x4389A8): the view's place and angles from the camera (see
//! `race/frame_draw.rs`): flat (`dword_60932C`) as it is, with `g_car` +0x6A8 moved by
//! `dword_5CCD80`; else moved back by `dword_5CCD80` turned by the camera's two angles
//! (`math::turn_point_xz_ecx`, `math::turn_point_xz`), and kept also as whole numbers and
//! floats (`dword_6062B0`, `dword_6062B4`, `flt_6062A4` ..).
//!
//! In:  nothing
//! Out: eax, ebx, ecx (and edx when turned) as it leaves them (its caller `sub_438325`
//!      jumps on to a `call`)
//!
//! Shadow mode: by state (the place, the angles' dwords, `g_car` +0x6A8, the scratch
//! point). Self-test: the camera's dwords set (places, distances, angles), flat and not;
//! all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::place_view", rva(0x4389A8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![
        g(0x606298, 0xC),
        g(0x6062A4, 0x2C),
        g(0x71A680, 4),
        g(0x5C5F4C, 0xC),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { place_view(r) }))
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
    for a in [
        0x5CCDA0usize,
        0x5CCDC0,
        0x5CCDE0,
        0x5CCD40,
        0x5CCD60,
        0x5CCD80,
        0x60932C,
    ] {
        kept.push(at(rva(a)));
    }
    let saved = capture(&kept);
    for (flat, place, back, angles) in [
        (
            0u32,
            [0x10_0000u32, 0x2_0000, 0x18_0000],
            0x4_0000u32,
            [0x1000u32, 0x3000],
        ),
        (
            0,
            [0xFFF0_0000, 0, 0x7FFF_0000],
            0x10_0000,
            [0xC000, 0x8000],
        ),
        (1, [0x12_3456, 0x6_5432, 0x1_0000], 0x800, [0x4000, 0]),
        (0, [5, 6, 7], 0, [0, 0xFFFF]),
    ] {
        check_state(
            &mut t,
            &format!("flat {flat} place {place:x?} back {back:#x} angles {angles:x?}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x60932C), flat);
                game_write(rva(0x5CCDA0), place[0]);
                game_write(rva(0x5CCDC0), place[1]);
                game_write(rva(0x5CCDE0), place[2]);
                game_write(rva(0x5CCD80), back);
                game_write(rva(0x5CCD40), angles[0]);
                game_write(rva(0x5CCD60), angles[1]);
                game_write(rva(0x71A680), 0x1234u32);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
