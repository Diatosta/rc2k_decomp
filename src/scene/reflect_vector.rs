//! `scene::reflect_vector` (0x451A9D): a direction mirrored at a surface and turned by
//! a matrix.
//!
//! In:  nothing
//! Out: esi = the matrix `flt_5EA20C`; everything else preserved
//!
//! v = the three floats at `dword_5EA428`, n = the three at `qword_5EA438`:
//! d = v . n (the first and second products added first, then the third); v = v * d, each rounded to a
//! float; `dword_62400C` = d; v = v + v - n; then v through the matrix `flt_5EA20C`
//! (`scene::rotate_point`, inlined) and back to its place. x87, with the original's
//! instructions; d stays in the x87's own width between its uses.
//! One caller, `sub_45198C` (the reflection's texture coordinates).
//!
//! Converted 2026-10-21: the hook's Rust version is `reflect_vector_plain`, through
//! `rc2k_core::scene::reflection::reflect` (plain floats; the x87 version stays for the
//! Rust callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{ESI, Regs, game_addr, game_ptr, game_read, game_write, rva};
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::screens::harness::Left;
use crate::service::floats::*;

// ---- game data ----

pub const VECTOR_RVA: usize = rva(0x5EA428); // dword_5EA428, dword_5EA42C, dword_5EA430
pub const NORMAL_RVA: usize = rva(0x5EA438); // qword_5EA438, dword_5EA440
pub const MATRIX_RVA: usize = rva(0x5EA20C); // flt_5EA20C

// ---- Rust version ----

pub unsafe fn reflect_vector() {
    unsafe {
        let (v, n): ([u32; 3], [u32; 3]) = (game_read(VECTOR_RVA), game_read(NORMAL_RVA));
        // The vector, the normal, d.
        let mut w = [v[0], v[1], v[2], n[0], n[1], n[2], 0];
        core::arch::asm!(
            "fld dword ptr [{w} + 8]",
            "fmul dword ptr [{w} + 20]",
            "fld dword ptr [{w} + 4]",
            "fmul dword ptr [{w} + 16]",
            "fld dword ptr [{w}]",
            "fmul dword ptr [{w} + 12]",
            ".byte 0xDE, 0xC1", // faddp st(1), st
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fld dword ptr [{w} + 8]",
            ".byte 0xD8, 0xC9", // fmul st, st(1)
            "fstp dword ptr [{w} + 8]",
            "fld dword ptr [{w} + 4]",
            ".byte 0xD8, 0xC9", // fmul st, st(1)
            "fstp dword ptr [{w} + 4]",
            "fld dword ptr [{w}]",
            ".byte 0xD8, 0xC9", // fmul st, st(1)
            "fstp dword ptr [{w}]",
            "fstp dword ptr [{w} + 24]",
            "fld dword ptr [{w}]",
            ".byte 0xD8, 0xC0", // fadd st, st
            "fsub dword ptr [{w} + 12]",
            "fstp dword ptr [{w}]",
            "fld dword ptr [{w} + 4]",
            ".byte 0xD8, 0xC0", // fadd st, st
            "fsub dword ptr [{w} + 16]",
            "fstp dword ptr [{w} + 4]",
            "fld dword ptr [{w} + 8]",
            ".byte 0xD8, 0xC0", // fadd st, st
            "fsub dword ptr [{w} + 20]",
            "fstp dword ptr [{w} + 8]",
            "fld dword ptr [{w} + 8]",
            "fld dword ptr [{w} + 4]",
            "fld dword ptr [{w}]",
            // `scene::rotate_point` (0x44D110), as in `project_point.rs`.
            "fld dword ptr [{m}]",
            "fmul st(0), st(1)",
            "fld dword ptr [{m} + 0x10]",
            "fmul st(0), st(3)",
            "fld dword ptr [{m} + 0x20]",
            "fmul st(0), st(5)",
            "faddp st(1), st(0)",
            "faddp st(1), st(0)",
            "fadd dword ptr [{m} + 0x30]",
            "fxch st(3)",
            "fld dword ptr [{m} + 4]",
            "fmul st(0), st(2)",
            "fld dword ptr [{m} + 0x14]",
            "fmul st(0), st(4)",
            "fld dword ptr [{m} + 0x24]",
            "fmul st(0), st(3)",
            "faddp st(1), st(0)",
            "faddp st(1), st(0)",
            "fadd dword ptr [{m} + 0x34]",
            "fxch st(3)",
            "fld dword ptr [{m} + 8]",
            "fmulp st(3), st(0)",
            "fld dword ptr [{m} + 0x18]",
            "fmulp st(1), st(0)",
            "fld dword ptr [{m} + 0x28]",
            "fmulp st(2), st(0)",
            "faddp st(1), st(0)",
            "faddp st(1), st(0)",
            "fadd dword ptr [{m} + 0x38]",
            "fxch st(2)",
            "fstp dword ptr [{w}]",
            "fstp dword ptr [{w} + 4]",
            "fstp dword ptr [{w} + 8]",
            w = in(reg) w.as_mut_ptr(),
            m = in(reg) game_ptr::<u32>(MATRIX_RVA),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
        game_write(VECTOR_RVA, [w[0], w[1], w[2]]);
        game_write(CHECKED_RVA, w[6]);
    }
}

/// The converted version (plain floats): the hook's.
// Converted: `rc2k_core::scene::reflection::reflect`
pub unsafe fn reflect_vector_plain() {
    unsafe {
        let (v, n): ([u32; 3], [u32; 3]) = (game_read(VECTOR_RVA), game_read(NORMAL_RVA));
        let m = game_addr(MATRIX_RVA) as u32;
        // The fourth float of each row is not read.
        let matrix: [f32; 15] = std::array::from_fn(|i| {
            if i % 4 == 3 {
                0.0
            } else {
                f32::from_bits(((m + 4 * i as u32) as *const u32).read_unaligned())
            }
        });
        let (r, d) = rc2k_core::scene::reflection::reflect(
            v.map(f32::from_bits),
            n.map(f32::from_bits),
            &matrix,
        );
        game_write(VECTOR_RVA, r.map(f32::to_bits));
        game_write(CHECKED_RVA, d.to_bits());
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { reflect_vector_plain() };
    Ok(Left::nothing().with(ESI, game_addr(MATRIX_RVA) as u32))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
