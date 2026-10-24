//! The game's three-float vector helpers at 0x4654EA .. 0x4659A1 (IDA, 2026-10-16), used by the
//! marks.

use crate::framework::{game_read, game_write, rva};
use crate::math::x87::x87;

/// `scene::normalise_vector3` (0x4654EA): the three floats at esi each times 1 over their
/// length (`fsqrt` of the squares, the last two added first; `fld1; fdivrp`). Every
/// register kept.
pub unsafe fn normalise_vector3(v: u32) {
    x87!(
        "fld dword ptr [{v}]", ".byte 0xD8, 0xC8",
        "fld dword ptr [{v} + 4]", ".byte 0xD8, 0xC8",
        "fld dword ptr [{v} + 8]", ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
        "fsqrt", "fld1", ".byte 0xDE, 0xF1",
        "fld dword ptr [{v}]", ".byte 0xD8, 0xC9",
        "fld dword ptr [{v} + 4]", ".byte 0xD8, 0xCA",
        "fld dword ptr [{v} + 8]", ".byte 0xD8, 0xCB",
        "fstp dword ptr [{v} + 8]", "fstp dword ptr [{v} + 4]", "fstp dword ptr [{v}]",
        "fstp st(0)";
        v = v
    );
}

/// `scene::dot_product3` (0x46553C): esi . edi, x x' + (y y' + z z'), unrounded. Every
/// register kept.
pub unsafe fn dot_product3(a: u32, b: u32) -> [u8; 10] {
    let mut out = [0u8; 10];
    x87!(
        "fld dword ptr [{a}]", "fmul dword ptr [{b}]",
        "fld dword ptr [{a} + 4]", "fmul dword ptr [{b} + 4]",
        "fld dword ptr [{a} + 8]", "fmul dword ptr [{b} + 8]",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
        "fstp tbyte ptr [{o}]";
        a = a, b = b, o = out.as_mut_ptr()
    );
    out
}

/// `n` floats at `at` (the game's memory, unaligned).
pub unsafe fn read_floats<const N: usize>(at: u32) -> [f32; N] {
    std::array::from_fn(|n| unsafe { ((at + 4 * n as u32) as *const f32).read_unaligned() })
}

/// The floats written at `at`.
pub unsafe fn write_floats(at: u32, v: &[f32]) {
    for (n, x) in v.iter().enumerate() {
        unsafe { ((at + 4 * n as u32) as *mut f32).write_unaligned(*x) };
    }
}

const START_RVA: usize = rva(0x71CA36); // flt_71CA36, pi / 2
const STEP_RVA: usize = rva(0x71CA3A); // flt_71CA3A, pi / 4
const HALF_RVA: usize = rva(0x71CA3E); // flt_71CA3E, 0.5
pub const STATUS_RVA: usize = rva(0x71C814); // dword_71C814: the last compare's C0 .. C2

/// `fld a; fcos; fcomp b; fstsw`: the status word's C0 .. C2 (0x700).
fn cosine_against(a: u32, b: u32) -> u32 {
    let mut status: u16 = 0;
    x87!(
        "fld dword ptr [{a}]", "fcos", "fcomp dword ptr [{b}]", "fstsw word ptr [{s}]";
        a = &a as *const u32, b = &b as *const u32, s = &mut status as *mut u16
    );
    status as u32 & 0x700
}

/// `scene::angle_between_vectors` (0x4658CF): the vectors at edi and esi normalised in
/// place (that order), their dot product kept as a float, and the angle whose cosine it
/// is found by 16 halvings from pi / 2 by pi / 4 (floats each step; out when the compare
/// is unordered); the last compare's C0 .. C2 in `dword_71C814`. Every register kept.
/// Returns the angle (a float, loaded).
pub unsafe fn angle_between_vectors(a: u32, b: u32) -> [u8; 10] {
    unsafe {
        let mut angle = crate::math::x87::loaded(game_read(START_RVA));
        let mut step = crate::math::x87::loaded(game_read(STEP_RVA));
        normalise_vector3(b);
        normalise_vector3(a);
        let dot = crate::scene::matrix3::rounded(dot_product3(a, b));
        let half: u32 = game_read(HALF_RVA);
        for _ in 0..16 {
            let status = cosine_against(angle, dot);
            game_write(STATUS_RVA, status);
            match status {
                0x400 => break,
                0x100 => angle = crate::math::x87::sub(angle, step),
                _ => angle = crate::math::x87::add(angle, step),
            }
            step = crate::math::x87::mul(step, half);
        }
        crate::scene::matrix3::extended(angle)
    }
}
