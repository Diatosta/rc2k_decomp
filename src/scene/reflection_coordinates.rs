//! `scene::reflection_coordinates` (0x45198C): where on the reflection's picture a
//! vertex of a shiny surface looks.
//!
//! In:  nothing; `dword_6399A8` = the number of the vertex's normal, `dword_6399AC` =
//!      the number of its point
//! Out: `dword_6399B0`, `dword_6399B4` = the two texture coordinates as whole numbers;
//!      eax, edx and esi as its parts leave them (eax of the second
//!      `math::normalise_vector`, edx = the point's record, esi = the matrix of
//!      `scene::reflect_vector`); everything else preserved
//!
//! The normal's place in the view (record 0x1000 + (number & 0xFFF), +0x10 .. +0x18)
//! goes to `dword_5EA428` ..; the point's, negated, to the work vector, which is made
//! one long and copied to `qword_5EA438` ..: the direction from the point to the eye.
//! `scene::reflect_vector`; the result, with `flt_5C5D38` (1.0) added to its second
//! part, is made one long; then each of its first and third parts, times
//! `flt_5C5E40` (0.5) plus the same, times `flt_5C5D98` (65536) times `flt_5C5E2C`
//! (256), rounded (`fistp`), is a coordinate.
//! Callers: `scene::cached_reflection_coordinates`, by a call and by a jump.
//! Refuses when there are no records and what `math::normalise_vector` refuses.

use crate::errors::BadInput;
use crate::framework::{game_read, game_write, rva};
use crate::math::normalise_vector::{VECTOR_RVA, normalise_vector};
use crate::math::x87;
use crate::race::numbers_store::NUMBERS;
use crate::scene::project_point::RECORDS_RVA;
use crate::scene::reflect_vector::{NORMAL_RVA, VECTOR_RVA as MIRRORED_RVA, reflect_vector};
use crate::scene::x87_mem::{fistp, fld, fstp};

// ---- game data ----

pub const NORMAL_NUMBER_RVA: usize = rva(0x6399A8); // dword_6399A8
pub const POINT_NUMBER_RVA: usize = rva(0x6399AC); // dword_6399AC
pub const COORDINATES_RVA: usize = rva(0x6399B0); // dword_6399B0, dword_6399B4

// ---- Rust version ----

/// `fld a; fmul half; fadd half; fmul scale_a; fmul scale_b; fistp`.
fn coordinate(a: u32, half: u32, scales: [u32; 2]) -> i32 {
    let (v, mut out) = ([a, half, scales[0], scales[1]], 0i32);
    unsafe { coordinate_at(v.as_ptr().cast(), (&raw mut out).cast()) };
    out
}

unsafe fn coordinate_at(v: *const u8, o: *mut u8) {
    unsafe {
        let t0 = fld(v, 0x0);
        let t1 = fld(v, 0x4);
        let t2 = t0 * t1;
        let t3 = fld(v, 0x4);
        let t4 = t2 + t3;
        let t5 = fld(v, 0x8);
        let t6 = t4 * t5;
        let t7 = fld(v, 0xC);
        let t8 = t6 * t7;
        fistp(o, 0x0, t8);
    }
}

/// `fld a; fchs; fstp`.
fn negated(a: u32) -> u32 {
    let mut out = 0u32;
    unsafe { negated_at((&raw const a).cast(), (&raw mut out).cast()) };
    out
}

unsafe fn negated_at(a: *const u8, o: *mut u8) {
    unsafe {
        let t0 = fld(a, 0x0);
        let t1 = -t0;
        fstp(o, 0x0, t1);
    }
}

pub unsafe fn reflection_coordinates() -> Result<(), BadInput> {
    unsafe {
        let records: u32 = game_read(RECORDS_RVA);
        if records == 0 {
            return Err(BadInput("no records for the points"));
        }
        let record = |number: u32| records.wrapping_add(((number & 0xFFF) + 0x1000) << 5);
        let place = |record: u32| ((record as usize + 0x10) as *const [u32; 3]).read_unaligned();
        let normal = record(game_read(NORMAL_NUMBER_RVA));
        game_write(MIRRORED_RVA, place(normal));
        let point = record(game_read::<u32>(POINT_NUMBER_RVA).wrapping_neg());
        game_write(VECTOR_RVA, place(point).map(negated));
        normalise_vector()?;
        let to_eye: [u32; 3] = game_read(VECTOR_RVA);
        game_write(NORMAL_RVA, to_eye);
        reflect_vector();
        let mirrored: [u32; 3] = game_read(MIRRORED_RVA);
        game_write(
            VECTOR_RVA,
            [
                x87::loaded(mirrored[0]),
                x87::add(mirrored[1], NUMBERS.one.bits()),
                x87::loaded(mirrored[2]),
            ],
        );
        normalise_vector()?;
        let looks: [u32; 3] = game_read(VECTOR_RVA);
        let half: u32 = NUMBERS.half.bits();
        let scales = [NUMBERS.k65536.bits(), NUMBERS.k256.bits()];
        game_write(
            COORDINATES_RVA,
            [
                coordinate(looks[0], half, scales),
                coordinate(looks[2], half, scales),
            ],
        );
        Ok(())
    }
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Kind, differential};

    /// The original's instructions for `coordinate`.
    unsafe fn coordinate_at_ref(v: *mut u8, o: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{v}]",
                "fmul dword ptr [{v} + 4]",
                "fadd dword ptr [{v} + 4]",
                "fmul dword ptr [{v} + 8]",
                "fmul dword ptr [{v} + 12]",
                "fistp dword ptr [{o}]",
                v = in(reg) v,
                o = in(reg) o,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    /// The original's instructions for `negated`.
    unsafe fn negated_at_ref(a: *mut u8, o: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{a}]",
                "fchs",
                "fstp dword ptr [{o}]",
                a = in(reg) a,
                o = in(reg) o,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    #[test]
    fn coordinate_matches_the_assembly() {
        // dwords 0 .. 4 the four floats, 4 the whole number
        differential::<5>(
            "reflection_coordinates coordinate",
            8000,
            20,
            [Kind::Exact; 5],
            |rng, m| {
                m[0] = rng.unit().to_bits();
                m[1] = if rng.below(2) == 0 {
                    0.5f32.to_bits()
                } else {
                    rng.fl()
                };
                m[2] = if rng.below(2) == 0 {
                    65536.0f32.to_bits()
                } else {
                    rng.moderate()
                };
                m[3] = if rng.below(2) == 0 {
                    256.0f32.to_bits()
                } else {
                    rng.moderate()
                };
                m[4] = 0xAAAA_AAAA;
            },
            |base| unsafe { coordinate_at_ref(base, base.add(16)) },
            |base| unsafe { coordinate_at(base, base.add(16)) },
        );
    }

    #[test]
    fn negated_matches_the_assembly() {
        differential::<2>(
            "reflection_coordinates negated",
            8000,
            21,
            [Kind::Exact; 2],
            |rng, m| {
                m[0] = rng.fl();
                m[1] = 0xAAAA_AAAA;
            },
            |base| unsafe { negated_at_ref(base, base.add(4)) },
            |base| unsafe { negated_at(base, base.add(4)) },
        );
    }
}
