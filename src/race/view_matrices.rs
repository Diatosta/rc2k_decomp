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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_write, rva};
use crate::race::frame_draw::*;
use crate::scene::matrix::{self, matrix as whole, test_floats};
use crate::scene::view_matrix::view_regions;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, state_entry};

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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
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
