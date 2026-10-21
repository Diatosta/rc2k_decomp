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

use crate::errors::Refused;
use crate::framework::{
    ESI, RawHook, Regs, SelfTest, game_addr, game_ptr, game_read, game_write, hook_asm, rva,
};
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::matrix::float_places;
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::reflect_vector", rva(0x451A9D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

// ---- hook ----

fn regions() -> [Region; 2] {
    [(game_addr(VECTOR_RVA) as u32, 12), at(CHECKED_RVA)]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { reflect_vector_plain() };
    Ok(Left::nothing().with(ESI, game_addr(MATRIX_RVA) as u32))
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = float_places(&regions());
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(), &floats, &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let inputs = [
        (game_addr(NORMAL_RVA) as u32, 12),
        (game_addr(MATRIX_RVA) as u32, 0x40),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let matrices: [[f32; 16]; 3] = [
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        [
            0.36, 0.48, -0.8, 0.0, -0.8, 0.6, 0.0, 0.0, 0.48, 0.64, 0.6, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        [
            0.1234567, 3.3333333, -0.7171068, 9.0, 2.7282818, -0.5, 0.3333333, 9.0, -1.4242135,
            0.0009765, 1.7320508, 9.0, -33.333332, 0.1, 6.7, 9.0,
        ],
    ];
    let vectors: [([f32; 3], [f32; 3]); 8] = [
        ([0.0, 0.0, 1.0], [0.0, 0.0, 1.0]),
        ([0.6, -0.8, 0.0], [0.0, 1.0, 0.0]),
        ([0.577, 0.577, 0.577], [-0.267, 0.534, 0.802]),
        ([0.33333334, 0.6666667, -0.6666667], [0.1, 0.2, 0.3]),
        ([1e-20, 1e20, 3.0], [1e20, 1e-20, 0.33333334]),
        ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
        ([-0.123, 0.456, -0.789], [0.987, -0.654, 0.321]),
        ([f32::NAN, 1.0, 1.0], [1.0, 1.0, 1.0]),
    ];
    let tolerant_places = float_places(&regions);
    let tolerant = Floats {
        memory: &tolerant_places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for (m, matrix) in matrices.into_iter().enumerate() {
            for (v, n) in vectors {
                check_state_floats(
                    &mut t,
                    "scene::reflect_vector",
                    &format!("control word {word:#x}, matrix {m}, vector {v:?}, normal {n:?}"),
                    call_original,
                    Regs::SENTINEL,
                    &|| unsafe {
                        game_write(CHECKED_RVA, 0xAAAA_AAAAu32);
                        game_write(VECTOR_RVA, v.map(f32::to_bits));
                        game_write(NORMAL_RVA, n.map(f32::to_bits));
                        game_write(MATRIX_RVA, matrix.map(f32::to_bits));
                    },
                    &regions,
                    held,
                    &rust,
                );
            }
        }
    });
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
