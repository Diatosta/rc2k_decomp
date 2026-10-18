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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::scene::matrix::{self, matrix as whole, test_floats};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

const CALLS: [usize; 41] = [
    0x0, 0x57, 0x80, 0x97, 0xA4, 0xBB, 0xC6, 0xDD, 0x100, 0x117, 0x124, 0x13B, 0x148, 0x15F, 0x16A,
    0x181, 0x1D8, 0x1EF, 0x206, 0x21D, 0x22B, 0x238, 0x24F, 0x25C, 0x273, 0x289, 0x2A0, 0x2B7,
    0x2CE, 0x339, 0x350, 0x367, 0x37E, 0x39D, 0x3B4, 0x3BF, 0x3D6, 0x42D, 0x444, 0x45B, 0x472,
];

pub static HOOK: RawHook =
    RawHook::new("race::place_object_matrix", rva(0x42739A), thunk, &ORIGINAL)
        .private_copy(0x478)
        .relocate_calls(&CALLS)
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &object_matrix_regions(), &rust);
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

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = object_matrix_regions();
    let material_at = game_addr(rva(0x97DF30)) as u32 + 0x20;
    let mut kept = regions.clone();
    kept.extend(object_inputs());
    kept.push((material_at, 1));
    let saved = capture(&kept);
    let mut seed = 0x0B1E_C001u32;
    for way in [0u32, 1, 5, 3, 4, 6] {
        let s = seed;
        seed = seed.wrapping_mul(0x9E37_79B9).wrapping_add(1);
        check_state(
            &mut t,
            &format!("way {way} seed {s:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| {
                restore(&kept, &saved);
                let mut s2 = s;
                set_object_inputs(&mut s2, way.min(5), material_at);
                if way == 6 {
                    // Nothing after the light's direction.
                    unsafe { game_write(rva(0x6365C8), 4u32) };
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
