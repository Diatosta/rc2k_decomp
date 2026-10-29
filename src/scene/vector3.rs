//! The game's three-float vector helpers at 0x4654EA .. 0x4659A1 (IDA, 2026-10-16), used by the
//! marks.
//!
//! Plain Rust since 2026-10-24 (the x87's arithmetic in `X`, `math/x87_float.rs`); the
//! original's instructions are `asm_reference` in the tests.

use crate::effects::work_store::MARKS_WORK;
use crate::math::cells::{Cells, load};
use crate::math::x87_float::{ONE, compare, ld, st};
use crate::scene::x87_mem::{fld, fnstsw, fstp80};

/// `scene::normalise_vector3` (0x4654EA): the three floats at esi each times 1 over their
/// length (`fsqrt` of the squares, the last two added first; `fld1; fdivrp`). Every
/// register kept.
pub fn normalise(v: &(impl Cells + ?Sized)) {
    let f = |k| ld(v.bits(k));
    let (x, y, z) = (f(0), f(1), f(2));
    let length = (x * x + (y * y + z * z)).sqrt();
    let t10 = ONE / length;
    let (x, y, z) = (f(0) * t10, f(1) * t10, f(2) * t10);
    v.set_bits(2, st(z));
    v.set_bits(1, st(y));
    v.set_bits(0, st(x));
}

/// `scene::dot_product3` (0x46553C): esi . edi, x x' + (y y' + z z'), unrounded. Every
/// register kept.
pub fn dot_product3(a: [u32; 3], b: [u32; 3]) -> [u8; 10] {
    let mut out = [0u8; 10];
    dot(a.as_ptr().cast(), b.as_ptr().cast(), out.as_mut_ptr());
    out
}

fn dot(a: *const u8, b: *const u8, o: *mut u8) {
    let t0 = fld(a, 0x0);
    let t1 = fld(b, 0x0);
    let t2 = t0 * t1;
    let t3 = fld(a, 0x4);
    let t4 = fld(b, 0x4);
    let t5 = t3 * t4;
    let t6 = fld(a, 0x8);
    let t7 = fld(b, 0x8);
    let t8 = t6 * t7;
    let t9 = t5 + t8;
    let t10 = t2 + t9;
    fstp80(o, 0x0, t10);
}

/// `n` floats at `at` (the game's memory, unaligned).
pub fn read_floats<const N: usize>(at: u32) -> [f32; N] {
    std::array::from_fn(|n| {
        crate::pointers::read_or_end::<f32>(at + 4 * n as u32, "scene::vector3's operand")
    })
}

/// The floats written at `at`.
pub fn write_floats(at: u32, v: &[f32]) {
    for (n, x) in v.iter().enumerate() {
        crate::pointers::write_or_end::<f32>(at + 4 * n as u32, *x, "scene::vector3's operand");
    }
}

/// `fld a; fcos; fcomp b; fstsw`: the status word's C0 .. C2 (0x700).
fn cosine_against(a: u32, b: u32) -> u32 {
    let mut status = 0u16;
    cosine(
        (&raw const a).cast(),
        (&raw const b).cast(),
        (&raw mut status).cast(),
    );
    status as u32 & 0x700
}

fn cosine(a: *const u8, b: *const u8, s: *mut u8) {
    let t0 = fld(a, 0x0);
    let t1 = t0.cos();
    let t2 = fld(b, 0x0);
    let t3 = compare(t1, t2);
    fnstsw(s, 0x0, t3); // status word C3 C2 C0 (TOP not modelled)
}

/// `scene::angle_between_vectors` (0x4658CF): the vectors at edi and esi normalised in
/// place (that order), their dot product kept as a float, and the angle whose cosine it
/// is found by 16 halvings from pi / 2 by pi / 4 (floats each step; out when the compare
/// is unordered); the last compare's C0 .. C2 in `dword_71C814`. Every register kept.
/// Returns the angle (a float, loaded).
pub fn angle_between_vectors(a: &(impl Cells + ?Sized), b: &(impl Cells + ?Sized)) -> [u8; 10] {
    let mut angle = crate::math::x87::loaded(MARKS_WORK.cosine_start.get());
    let mut step = crate::math::x87::loaded(MARKS_WORK.cosine_step.get());
    normalise(b);
    normalise(a);
    let dot = crate::scene::matrix3::rounded(dot_product3(load(a), load(b)));
    let half: u32 = MARKS_WORK.cosine_half.get();
    for _ in 0..16 {
        let status = cosine_against(angle, dot);
        MARKS_WORK.fpu_status.set(status);
        match status {
            0x400 => break,
            0x100 => angle = crate::math::x87::sub(angle, step),
            _ => angle = crate::math::x87::add(angle, step),
        }
        step = crate::math::x87::mul(step, half);
    }
    crate::scene::matrix3::extended(angle)
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Kind, differential};

    /// The original's instructions for `normalise_vector3`.
    unsafe fn normalise_ref(v: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{v}]", ".byte 0xD8, 0xC8",
                "fld dword ptr [{v} + 4]", ".byte 0xD8, 0xC8",
                "fld dword ptr [{v} + 8]", ".byte 0xD8, 0xC8",
                ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
                "fsqrt", "fld1", ".byte 0xDE, 0xF1",
                "fld dword ptr [{v}]", ".byte 0xD8, 0xC9",
                "fld dword ptr [{v} + 4]", ".byte 0xD8, 0xCA",
                "fld dword ptr [{v} + 8]", ".byte 0xD8, 0xCB",
                "fstp dword ptr [{v} + 8]", "fstp dword ptr [{v} + 4]", "fstp dword ptr [{v}]",
                "fstp st(0)",
                v = in(reg) v,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    /// The original's instructions for `dot_product3`.
    unsafe fn dot_ref(a: *mut u8, b: *mut u8, o: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{a}]", "fmul dword ptr [{b}]",
                "fld dword ptr [{a} + 4]", "fmul dword ptr [{b} + 4]",
                "fld dword ptr [{a} + 8]", "fmul dword ptr [{b} + 8]",
                ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1",
                "fstp tbyte ptr [{o}]",
                a = in(reg) a,
                b = in(reg) b,
                o = in(reg) o,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    /// The original's instructions for `cosine_against` (the status word is stored whole).
    unsafe fn cosine_ref(a: *mut u8, b: *mut u8, s: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{a}]", "fcos", "fcomp dword ptr [{b}]", "fstsw word ptr [{s}]",
                a = in(reg) a,
                b = in(reg) b,
                s = in(reg) s,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    #[test]
    fn normalise_matches_the_assembly() {
        differential::<3>(
            "normalise_vector3",
            8000,
            4,
            [Kind::Float; 3],
            |rng, m| {
                let moderate = rng.below(2) == 0;
                for d in m.iter_mut() {
                    *d = if moderate { rng.moderate() } else { rng.fl() };
                }
            },
            |base| unsafe { normalise_ref(base) },
            |base| {
                use crate::scene::x87_mem::HostCell;
                normalise(&[0, 4, 8].map(|k| HostCell(base as usize + k)))
            },
        );
    }

    #[test]
    fn dot_matches_the_assembly() {
        // a, b, the 80-bit result (3 dwords)
        differential::<9>(
            "dot_product3",
            8000,
            5,
            [Kind::Exact; 9],
            |rng, m| {
                let moderate = rng.below(2) == 0;
                for d in m.iter_mut().take(6) {
                    *d = if moderate { rng.moderate() } else { rng.fl() };
                }
                m[6..].fill(0xAAAA_AAAA);
            },
            |base| unsafe { dot_ref(base, base.add(12), base.add(24)) },
            |base| unsafe { dot(base, base.add(12), base.add(24)) },
        );
    }

    #[test]
    fn cosine_matches_the_assembly() {
        // the angle, the float it is compared with (often the cosine itself), the status
        differential::<3>(
            "cosine_against",
            8000,
            6,
            [Kind::Exact; 3],
            |rng, m| {
                m[0] = rng.angle();
                m[1] = match rng.below(3) {
                    0 => (f32::from_bits(m[0]) as f64).cos() as f32,
                    1 => rng.unit(),
                    _ => f32::from_bits(rng.fl()),
                }
                .to_bits();
                m[2] = 0;
            },
            |base| unsafe {
                cosine_ref(base, base.add(4), base.add(8));
                let s = base.add(8).cast::<u16>().read_unaligned() & 0x700;
                base.add(8).cast::<u32>().write_unaligned(s as u32);
            },
            |base| unsafe {
                cosine(base, base.add(4), base.add(8));
                let s = base.add(8).cast::<u16>().read_unaligned() & 0x700;
                base.add(8).cast::<u32>().write_unaligned(s as u32);
            },
        );
    }
}
