//! `race::place_object_matrix` (0x42739A): `g_madeObjectMatrix` for an object at
//! `dword_63501C` .. (see `race/frame_draw.rs`): turned by its three angles (with
//! `dword_6365D0` 0), facing the view (materials 1, 2), turned with the view (5) or by its
//! own angles and tilts (others), scaled by `dword_635038` / 16, placed and moved by the
//! view. On `scene::matrix` / `scene::view_matrix`'s Rust versions.
//!
//! In:  nothing
//! Out: eax and ebx the place's x and y, ecx 0, edx 0, esi `g_turnedMatrix`, edi
//!      `g_viewTurnMatrix`, ebp `g_madeObjectMatrix` (none changed with `dword_6365C8` 4)
//!
//! It begins with its call: private copy (0x478, its 41 calls relocated), manual patch.
//! Callers: `race::draw_flare`, `race::draw_lamp_flare`, `scene::draw_described_object`
//! (0x451DDF). Shadow mode: by state. Self-test: the angles, place, size, view matrices
//! and the object's material set, each way; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_write, rva};
use crate::race::frame_draw::*;
use crate::scene::matrix::{self, matrix as whole, test_floats};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, state_entry};

pub fn object_matrix_regions() -> Vec<Region> {
    use crate::scene::view_matrix::{TURN_Z_RVA, TURN_Z_WORK_RVA};
    let g = |a: usize, len: usize| (game_addr(a) as u32, len);
    let mut v: Vec<Region> = [
        matrix::TURN_X_RVA,
        matrix::TURN_Y_RVA,
        TURN_Z_RVA,
        matrix::TURNED_RVA,
        matrix::LIT_RVA,
        matrix::PLACED_RVA,
        matrix::OBJECT_RVA,
        rva(0x5EA444),
        rva(0x5EA484),
    ]
    .map(whole)
    .to_vec();
    v.extend([
        g(matrix::TURN_X_WORK_RVA, 8),
        g(matrix::TURN_Y_WORK_RVA, 8),
        g(TURN_Z_WORK_RVA, 8),
        g(matrix::LIGHT_DIRECTION_RVA, 0xC),
        at(rva(0x62400C)),
    ]);
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { place_object_matrix(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// The inputs a placed-object test sets.
pub fn object_inputs() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![
        g(0x63501C, 0xC),
        g(0x635038, 4),
        g(0x63503C, 0xC),
        g(0x63505C, 0xC),
        at(rva(0x6365C8)),
        at(rva(0x6365D0)),
        at(rva(0x6214FC)),
        g(0x5CD27C, 8),
        g(0x6062B8, 0x10),
        g(0x635094, 8),
        g(0x5EA1CC, 0x40),
        g(0x5EA20C, 0x40),
        g(0x5EA24C, 0x40),
    ]
}

/// Sets a placed object's inputs: `way` 0 the plain turn, 1 material 1, 5 material 5,
/// 3 own angles without tilts, 4 with both tilts.
pub fn set_object_inputs(seed: &mut u32, way: u32, material_at: u32) {
    unsafe {
        let s = *seed;
        game_write(rva(0x63501C), 0x10_0000u32 ^ (s % 0x1000));
        game_write(rva(0x635020), 0x2_0000u32);
        game_write(rva(0x635024), 0x18_0000u32);
        game_write(rva(0x635038), 0x100 + s % 0x400);
        game_write(rva(0x63503C), s.rotate_left(3) & 0xFFFF);
        game_write(rva(0x635040), if way == 4 { 0x1234 } else { 0 });
        game_write(rva(0x635044), s.rotate_left(9) & 0xFFFF);
        game_write(rva(0x63505C), 0x2000u32);
        game_write(rva(0x635060), if way == 4 { 0x0800 } else { 0 });
        game_write(rva(0x635064), if way == 4 { 0x4000 } else { 0 });
        game_write(rva(0x6365C8), 2u32);
        game_write(rva(0x6365D0), u32::from(way != 0));
        game_write(rva(0x6214FC), 1u32);
        game_write(rva(0x5CD280), 0u32);
        let material = match way {
            1 => 1u8,
            5 => 5,
            _ => 3,
        };
        (material_at as *mut u8).write(material);
        game_write(rva(0x6062B8), s.wrapping_mul(0x9E37_79B9));
        game_write(rva(0x6062C4), s.wrapping_mul(0x7F4A_7C15));
        game_write(rva(0x5EA1CC), test_floats(seed));
        game_write(rva(0x5EA20C), test_floats(seed));
        game_write(rva(0x5EA24C), test_floats(seed));
    }
}
