//! `race::camera_view` (0x438325): the view for the frame (see `race/frame_draw.rs`):
//! `dword_606274`, `dword_60626C`, `dword_606270` from the work car; without
//! `dword_621480` `race::place_view`; else the place and angles from the car's camera
//! (or, with `dword_63993C`, from the crash: the kept turn made, the place the car's raised
//! by 0x8000), as whole numbers and floats (also `g_car` +0x734 ..) and
//! `scene::set_view_window`; then `race::view_matrices`.
//!
//! In:  nothing
//! Out: as `view_matrices` leaves them (a `call` follows)
//!
//! Caller: `sub_4557AB` (0x4557E5), each view drawn. Shadow mode: by state (all its parts'
//! writes). Self-test: the car's camera and the switches set by pattern (`dword_621480` on
//! and off, the crash), the view window's inputs as the game has them; all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::race::view_matrices::{matrices_regions, set_view_inputs, view_inputs};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::camera_view", rva(0x438325), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let mut v = matrices_regions();
    v.extend(crate::scene::set_view_window::regions());
    v.extend([
        g(0x60626C, 0xC),
        g(0x606294, 0x3C),
        g(0x71A6A4, 0xC),
        g(0x71A70C, 0xC),
        g(0x71A680, 4),
        g(0x5C5F4C, 0xC),
        g(0x5EA18C, 0x40),
    ]);
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { camera_view(r) }))
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
    kept.extend(view_inputs());
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    kept.extend([
        at(rva(0x621480)),
        g(0x71A6C8, 4),
        g(0x71A674, 4),
        g(0x71A67C, 0x4C),
        g(0x71A10C, 0xC),
        at(rva(0x606278)),
        g(0x5CCD40, 0xA4),
        at(rva(0x60932C)),
    ]);
    let saved = capture(&kept);
    let mut seed = 0xCA3E_0001u32;
    for (in_race, view, crash) in [
        (0u32, 4u32, 0u32),
        (1, 4, 0),
        (1, 1, 0),
        (1, 2, 1),
        (1, 0, 0),
    ] {
        let s = seed;
        seed = seed.wrapping_mul(0x9E37_79B9).wrapping_add(1);
        check_state(
            &mut t,
            &format!("in race {in_race} view {view} crash {crash} seed {s:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                let mut s2 = s;
                set_view_inputs(&mut s2, view, crash);
                game_write(rva(0x621480), in_race);
                let car = game_addr(rva(0x71A67C)) as u32;
                for k in 0..0x13u32 {
                    ((car + 4 * k) as *mut u32).write_unaligned(s.rotate_left(k) % 0x20_0000);
                }
                game_write(rva(0x71A10C), 0x11_0000u32);
                game_write(rva(0x71A110), 0x2_2000u32);
                game_write(rva(0x71A114), 0x17_7000u32);
                game_write(rva(0x606278), 0x1234u32);
                game_write(rva(0x60932C), 0u32);
                for (k, a) in [0x5CCDA0usize, 0x5CCDC0, 0x5CCDE0].iter().enumerate() {
                    game_write(rva(*a), 0x10_0000u32 + k as u32 * 0x3_0000);
                }
                game_write(rva(0x5CCD80), 0x4_0000u32);
                game_write(rva(0x5CCD40), 0x2000u32);
                game_write(rva(0x5CCD60), 0x6000u32);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
