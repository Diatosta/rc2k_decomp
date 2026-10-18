//! `race::view_matrices` (0x4386A4): the view's matrices for the frame (see
//! `race/frame_draw.rs`): the turn (kept with `dword_63993C`; the car's undone for its
//! views 0 .. 3; else the identity turned twice by the view's six angles), views 1 and 2
//! rolled with the car (eased into `g_car` +0x6A0 by its speed), then the place:
//! `g_placedMatrix` (kept at 0x5EA1CC), `g_viewMatrix`, `g_viewBackMatrix`. With
//! `scene::matrix` / `scene::view_matrix`'s Rust versions; x87 as the original.
//!
//! In:  nothing
//! Out: eax and ebx the place's x and y negated, ecx 0, edx 0, esi `g_viewTurnMatrix`, edi
//!      `g_viewBackMatrix`, ebp `g_viewMatrix` (a `retn` follows)
//!
//! Caller: `race::camera_view` (0x438467). Shadow mode: by state (the matrices, their
//! work dwords, the inversion's work, `g_car` +0x6A0, `dword_62400C`). Self-test: the
//! angles, place, car's turn, speed and roll set by pattern, each view kind, the kept
//! turn; all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::scene::matrix::{self, matrix as whole, test_floats};
use crate::scene::view_matrix::view_regions;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::view_matrices", rva(0x4386A4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn matrices_regions() -> Vec<Region> {
    let mut v = view_regions();
    v.extend([
        whole(matrix::TURNED_RVA),
        at(rva(0x71A678)),
        at(rva(0x62400C)),
    ]);
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { view_matrices(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &matrices_regions(), &rust);
}

/// The view's inputs a test sets: the angles, the place, the kept turn, the car's turn,
/// its view, speed, roll target and roll.
pub fn view_inputs() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![
        g(0x606298, 0x38),
        g(0x5EA18C, 0x40),
        g(0x71A08C, 0x40),
        at(rva(0x71A65C)),
        at(rva(0x71A184)),
        at(rva(0x71A620)),
        at(rva(0x63993C)),
    ]
}

/// Sets the view's inputs for a case.
pub fn set_view_inputs(seed: &mut u32, view: u32, kept: u32) {
    unsafe {
        let angles = [
            0x1234_5678u32,
            0x0F00_0000,
            0xFFF0_0000,
            0x4000_0000,
            0x0000_1000,
            0x8000_0000,
        ];
        for (k, a) in angles.iter().enumerate() {
            let at = [
                0x6062B8usize,
                0x6062BC,
                0x6062C0,
                0x6062C4,
                0x6062C8,
                0x6062CC,
            ][k];
            game_write(rva(at), a.rotate_left(*seed % 7));
        }
        game_write(rva(0x606298), 0x10_0000u32 ^ *seed);
        game_write(rva(0x60629C), 0x2_0000u32);
        game_write(rva(0x6062A0), 0x18_0000u32.wrapping_sub(*seed));
        game_write(rva(0x5EA18C), test_floats(seed));
        game_write(rva(0x71A08C), test_floats(seed));
        game_write(rva(0x71A65C), view);
        game_write(rva(0x71A184), ((*seed % 300) as f32 * 0.37).to_bits());
        game_write(rva(0x71A620), *seed % 0x40000);
        game_write(
            rva(0x71A678),
            ((*seed % 50) as f32 / 100.0 - 0.25).to_bits(),
        );
        game_write(rva(0x63993C), kept);
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = matrices_regions();
    let mut kept = regions.clone();
    kept.extend(view_inputs());
    let saved = capture(&kept);
    let mut seed = 0x0DD0_0001u32;
    for (view, kept_turn) in [(0u32, 0u32), (1, 0), (2, 0), (3, 0), (4, 0), (7, 0), (2, 1)] {
        let s = seed;
        seed = seed.wrapping_mul(0x9E37_79B9).wrapping_add(1);
        check_state(
            &mut t,
            &format!("view {view} kept {kept_turn} seed {s:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| {
                restore(&kept, &saved);
                let mut s2 = s;
                set_view_inputs(&mut s2, view, kept_turn);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
